//! # `app::actions::split` — Pages ▸ Split…: one document written as several
//!
//! The window's preview and the write share [`plan`], which calls the engine's
//! `pageops::plan_split` — the same function `split_with_labels` calls — so the
//! list of files the window shows is the list that is written.
//!
//! Like extract, this changes no document: it reads `doc.session.view()`, so
//! the files carry unsaved edits, and writes bytes to disk. Every target is
//! checked against the open document's own path before anything is written.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/split.md`.

use std::path::Path;

use pdfcer_core::pageops::{
    DocumentView, ExtractedPageLabels, PageOpError, SeparationPolicy, SplitCriterion, SplitPart,
};

pub use pdfcer_gui_base::subactions::SplitRequest;

use crate::app::state::OpenDoc;
use crate::text::split_pages as t;

/// What the window shows before Split: the files, or why there are none.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Preview {
    /// The files that would be written, and how many already exist.
    Parts {
        /// The engine's plan, names filled in.
        parts: Vec<SplitPart>,
        /// How many of `parts` name a file already in the folder.
        existing: usize,
    },
    /// Nothing would be written; the sentence says why.
    Refused(String),
}

/// The name `{stem}` stands for: the file's own name, or a created
/// document's title.
#[must_use]
pub fn stem_of(doc: &OpenDoc) -> String {
    let path = doc.stored_under().unwrap_or(&doc.path);
    path.file_stem().map_or_else(
        // ui-text-exempt: a filename fallback for a path with no stem, not operator copy.
        || String::from("document"),
        |s| s.to_string_lossy().into_owned(),
    )
}

/// The preview for `request` against `view`. `source` is the open document's
/// own file, which no part may be written over.
#[must_use]
pub fn plan(
    view: &DocumentView<'_>,
    request: &SplitRequest,
    stem: &str,
    source: Option<&Path>,
) -> Preview {
    if let Some(refusal) = template_refusal(&request.template) {
        return Preview::Refused(refusal.to_owned());
    }
    if request.folder.as_os_str().is_empty() {
        return Preview::Refused(t::folder_missing().to_owned());
    }
    if !request.folder.is_dir() {
        return Preview::Refused(t::folder_not_found().to_owned());
    }
    let parts =
        match pdfcer_core::pageops::plan_split(view, &request.criterion, &request.template, stem) {
            Ok(parts) => parts,
            Err(error) => return Preview::Refused(refusal(&error)),
        };
    let mut existing = 0;
    for part in &parts {
        let target = request.folder.join(&part.name);
        if source.is_some_and(|s| crate::app::files::same_file(s, &target)) {
            return Preview::Refused(t::would_overwrite_source(&part.name));
        }
        existing += usize::from(target.exists());
    }
    Preview::Parts { parts, existing }
}

/// The sentence for a pattern the engine would accept and the file system
/// would not: a folder separator, a character Windows refuses, or no `.pdf`.
fn template_refusal(template: &str) -> Option<&'static str> {
    if template
        .chars()
        .any(|c| matches!(c, '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|'))
    {
        return Some(t::template_bad_character());
    }
    (!template.to_ascii_lowercase().ends_with(".pdf")).then_some(t::template_not_pdf())
}

/// The window's sentence for a planning refusal.
fn refusal(error: &PageOpError) -> String {
    match error {
        PageOpError::NoSplitPoints => t::no_split_points().to_owned(),
        PageOpError::AmbiguousNames { first, second } => t::ambiguous_names(*first, *second),
        other => t::plan_failed(&other.to_string()),
    }
}

/// **Write the split.** Re-plans against the document as it is now, so an edit
/// made while the window was open cannot make the files disagree with the
/// check against the source path. The status line gets the receipt or the
/// failure, naming how many files reached the disk first.
pub(super) fn split(doc: &OpenDoc, request: &SplitRequest, separations: SeparationPolicy) {
    let note = match write(doc, request, separations) {
        Ok(Written { files, kept_labels }) => {
            let mut note = t::wrote(files, &request.folder.display().to_string());
            match (kept_labels, doc.page_labels().is_some()) {
                (_, false) => {}
                (true, true) => {
                    note.push(' ');
                    note.push_str(t::labels_kept());
                }
                (false, true) => {
                    note.push(' ');
                    note.push_str(t::labels_dropped());
                }
            }
            note
        }
        Err((written, detail)) => t::failed(written, &detail),
    };
    super::record_note(doc.edit_epoch, note);
}

/// A finished split: how many files, and whether labels were carried.
struct Written {
    files: usize,
    kept_labels: bool,
}

/// Plan, assemble and write every part. `Err` carries how many files were
/// written before the failure and the sentence for it.
fn write(
    doc: &OpenDoc,
    request: &SplitRequest,
    separations: SeparationPolicy,
) -> Result<Written, (usize, String)> {
    let view = doc.session.view();
    let stem = stem_of(doc);
    if let Preview::Refused(why) = plan(&view, request, &stem, doc.stored_under()) {
        trace_failed(0, &why);
        return Err((0, why));
    }
    let parts = pdfcer_core::pageops::split_with_labels(
        &view,
        &request.criterion,
        &request.template,
        &stem,
        separations,
        request.labels,
    )
    .map_err(|error| {
        trace_failed(0, &error.to_string());
        (0, error.to_string())
    })?;
    let total = parts.len();
    for (index, (part, bytes, report)) in parts.iter().enumerate() {
        let target = request.folder.join(&part.name);
        if let Err(error) = std::fs::write(&target, bytes) {
            trace_failed(index, &error.to_string());
            return Err((index, error.to_string()));
        }
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed
                "split-part n={} of={total} first={} last={} pages={} name={:?} bytes={} labels_dropped={} label_ranges={}",
                index + 1,
                part.first_page,
                part.last_page,
                report.pages,
                part.name,
                bytes.len(),
                u8::from(report.page_labels_dropped),
                report.page_label_ranges,
            )
        });
    }
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed
            "split-written files={total} rule={} labels={} folder={:?}",
            rule_token(&request.criterion),
            labels_token(request.labels),
            request.folder,
        )
    });
    Ok(Written {
        files: total,
        kept_labels: request.labels == ExtractedPageLabels::Keep,
    })
}

fn trace_failed(written: usize, detail: &str) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        format!("split-failed written={written} detail={detail:?}")
    });
}

/// The trace token for a rule.
#[must_use]
pub fn rule_token(criterion: &SplitCriterion) -> String {
    // ui-text-exempt: trace tokens, never displayed
    match criterion {
        SplitCriterion::EveryN(n) => format!("every-{n}"),
        SplitCriterion::AfterPages(after) => format!("after-{}", after.len()),
        SplitCriterion::TopLevelBookmarks => "bookmarks".to_owned(),
        _ => "other".to_owned(),
    }
}

/// The trace token for a labels choice.
#[must_use]
pub const fn labels_token(labels: ExtractedPageLabels) -> &'static str {
    // ui-text-exempt: trace tokens, never displayed
    match labels {
        ExtractedPageLabels::Keep => "keep",
        ExtractedPageLabels::Drop => "drop",
        _ => "other",
    }
}

#[cfg(test)]
mod tests;
