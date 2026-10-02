//! # `app::actions::reface` — committing a text edit whose new characters the
//! run's font lacks, with those characters set in another face
//!
//! [`commit`] commits the tokenised line through the ordinary plan, then for
//! each segment, last first, moves its token to the face (`format_text`) and
//! writes the characters over it (`edit_text`). Every step lands in one undo
//! entry (`coalesce_last`); a step that fails undoes the ones before it, so the
//! page is either fully edited or as it was.

use pdfcer_core::edit::{CommandKind, EditSession};
use pdfcer_core::text_edit::{EditRequest, FontSelector, FormatOptions, FormatRequest};
use pdfcer_gui_base::editmodel::reface::{Reface, Tokens};
use pdfcer_gui_base::text::reface as t;

use super::funnel::vector_edit;
use crate::app::settings::SettingsExt;
use crate::app::state::OpenDoc;
use crate::canvas::textedit::Plan;

/// Commit `replacement` with `reface`'s characters in its face, when the run
/// still refuses any of them and the line can be tokenised. Answers whether
/// it took the commit; `false` leaves it to the ordinary path.
pub(super) fn try_commit(
    doc: &mut OpenDoc,
    page: usize,
    run: usize,
    original: &str,
    replacement: &str,
    reface: &Reface,
) -> bool {
    let chars = crate::canvas::textedit::repertoire::refused_now(doc, page, run, &reface.chars);
    if chars.is_empty() {
        return false;
    }
    let reface = Reface {
        chars,
        ..reface.clone()
    };
    let tokens = page_lines(doc, page).and_then(|lines| {
        let refs: Vec<&str> = lines.iter().map(String::as_str).collect();
        pdfcer_gui_base::editmodel::reface::tokenise_any(replacement, &reface, &refs)
    });
    let Some(tokens) = tokens else {
        // ui-text-exempt: diagnostic trace, never displayed.
        crate::diag::trace(|| format!("text-edit-reface-declined page={page} run={run}"));
        return false;
    };
    let plan = crate::canvas::textedit::plan(doc, page, run, original, &tokens.text);
    vector_edit(doc, "edit-text", page, 1, |session| {
        commit(session, &plan, &tokens, &reface, page)
    });
    let reads = page_lines(doc, page).is_some_and(|l| l.concat().contains(replacement));
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "text-edit-reface-readback page={page} run={run} reads={}",
            u8::from(reads)
        )
    });
    true
}

/// A re-faced commit that was not made, with the engine's reason.
pub struct Stopped {
    why: t::Stopped,
    face: String,
    detail: String,
}

impl std::fmt::Display for Stopped {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&t::stopped(self.why, &self.face, &self.detail))
    }
}

/// Run the whole gesture on `session`; answers the notes to show.
pub(super) fn commit(
    session: &mut EditSession,
    plan: &Plan,
    tokens: &Tokens,
    reface: &Reface,
    page: usize,
) -> Result<Vec<String>, Stopped> {
    let stop = |why, detail: String| Stopped {
        why,
        face: reface.label.clone(),
        detail,
    };
    let mut notes = match plan
        .attempt("commit", |r| session.edit_text(r, &plan.options))
        .0
    {
        Ok(report) => report.disclosures,
        Err(e) => return Err(stop(t::Stopped::Line, e.to_string())),
    };
    let mut steps = 1_usize;
    for (token, segment) in tokens.segments.iter().rev() {
        let format = FormatRequest::new(page, token).font(FontSelector::new(&reface.face));
        if let Err(e) = session.format_text(&format, &FormatOptions::default()) {
            unwind(session, steps);
            return Err(stop(t::Stopped::Face, e.to_string()));
        }
        steps += 1;
        let write = EditRequest::find_replace(page, token, segment);
        match session.edit_text(&write, &plan.options) {
            Ok(report) => notes.extend(report.disclosures),
            Err(e) => {
                unwind(session, steps);
                return Err(stop(t::Stopped::Chars, e.to_string()));
            }
        }
        steps += 1;
    }
    trace(page, tokens, steps);
    if !session.coalesce_last(steps, CommandKind::EditText) {
        notes.push(t::undo_split(steps));
    }
    notes.push(t::set_in(&reface.chars, &reface.label));
    let mut seen = std::collections::HashSet::new();
    notes.retain(|n| seen.insert(n.clone()));
    Ok(notes)
}

/// Undo the `steps` commands this gesture pushed. Each leaves a redo entry;
/// the engine has no way to drop them (G083).
fn unwind(session: &mut EditSession, steps: usize) {
    for _ in 0..steps {
        let _ = session.undo();
    }
}

fn trace(page: usize, tokens: &Tokens, steps: usize) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "text-edit-reface-committed page={page} segments={} steps={steps}",
            tokens.segments.len()
        )
    });
}

/// Every line of `page` as the session prints it now, for the token floor.
fn page_lines(doc: &OpenDoc, page: usize) -> Option<Vec<String>> {
    let view = doc.session.view();
    let pages = pdfcer_core::page_tree::pages_in(&view).ok()?;
    let text = pdfcer_core::text_extract::extract_page_view(
        &view,
        pages.get(page)?,
        page,
        &doc.settings.extract_options(),
    )
    .ok()?;
    Some(text.runs.into_iter().map(|r| r.text).collect())
}
