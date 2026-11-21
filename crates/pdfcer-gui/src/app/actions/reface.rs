//! # `app::actions::reface` — committing a text edit whose new characters the
//! run's font lacks, with those characters set in another face
//!
//! [`try_commit`] asks the engine to set the characters in the face itself
//! (`EditOptions::with_fallback`), as one undo entry, for a match in one text
//! object however many show operators it spans. A refusal is shown in the
//! engine's words.

use pdfcer_core::text_edit::{EditReport, FallbackSource};
use pdfcer_gui_base::editmodel::fallbackface;
use pdfcer_gui_base::editmodel::reface::Reface;
use pdfcer_gui_base::text::reface as t;

use super::funnel::vector_edit;
use crate::app::settings::SettingsExt;
use crate::app::state::OpenDoc;

/// Commit `replacement` with `reface`'s characters in its face, through the
/// engine's fallback (G078), when the run still refuses any of them. Answers
/// whether it took the commit; `false` leaves it to the ordinary path.
pub(super) fn try_commit(
    doc: &mut OpenDoc,
    page: usize,
    run: usize,
    original: &str,
    replacement: &str,
    reface: &Reface,
) -> bool {
    let (chars, two_ways) =
        crate::canvas::textedit::repertoire::refused_now(doc, page, run, &reface.chars);
    if chars.is_empty() {
        return false;
    }
    let reface = Reface {
        chars,
        ..reface.clone()
    };
    let mut engine = crate::canvas::textedit::plan(doc, page, run, original, replacement);
    engine.options = engine
        .options
        .with_fallback(fallbackface::named(&reface.face));
    vector_edit(doc, "edit-text", page, 1, |session| {
        let (result, tier) = engine.attempt("commit", |r| session.edit_text(r, &engine.options));
        match result {
            Ok(report) => Ok(engine_notes(page, run, report, &reface, &two_ways)),
            Err(error) if crate::app::unlock::Refusal::encrypted(&error) => {
                if let Some(cause) = session.encryption_refusal_cause() {
                    crate::app::unlock::record_cause(cause);
                }
                Err(error.to_string())
            }
            Err(error) => {
                crate::app::status::decline::record_edit_text_refusal(
                    page,
                    run,
                    engine.reached_one_operator(tier),
                    &error,
                );
                Err(error.to_string())
            }
        }
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

/// The disclosures of an edit the engine committed, with the fallback traced
/// and said in the operator's terms when it set any characters.
fn engine_notes(
    page: usize,
    run: usize,
    report: EditReport,
    reface: &Reface,
    two_ways: &[char],
) -> Vec<String> {
    let spanned = report.operators_spanned;
    let mut notes = report.disclosures;
    if let Some(used) = report.fallback {
        let named: Vec<String> = used
            .characters
            .iter()
            .map(|c| format!("U+{:04X}", u32::from(*c)))
            .collect();
        let source = fallbackface::source_token(used.source);
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!(
                "text-edit-fallback page={page} run={run} characters={} face={} source={source} \
                 operators={spanned}",
                named.join(","),
                used.base_font
            )
        });
        notes.push(t::set_in(&used.characters, two_ways, &reface.label));
        if used.source == FallbackSource::AddedStandard14 {
            notes.push(t::not_embedded(&reface.label));
        }
    }
    notes
}

/// Every line of `page` as the session prints it now, for the read-back.
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
