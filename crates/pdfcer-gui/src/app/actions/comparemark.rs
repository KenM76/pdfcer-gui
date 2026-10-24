//! # `app::actions::comparemark` — a comparison's changes as comments
//!
//! `pdfcer_core::compare::mark_regions` then `mark_changes`, as the engine's
//! own `compare --output` runs them, on the open document through the edit
//! funnel. Each mark is its own engine command; the gesture is one, so the
//! commands it pushed are folded into one undo entry. A fold the stack cannot
//! make is said, not hidden.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/comparemark.md`.

use pdfcer_core::compare::{ResultError, ResultOptions, mark_changes, mark_regions};
use pdfcer_core::edit::{EditSession, MAX_UNDO_DEPTH};
use pdfcer_gui_base::newcomment::ComparisonMarks;

use crate::app::prefs::Prefs;
use crate::app::state::OpenDoc;
use crate::text::panels::compare as t;

/// Write `marks` to `doc` as comments.
pub(super) fn apply(doc: &mut OpenDoc, prefs: &Prefs, marks: &ComparisonMarks) {
    let page = doc.view.page_index;
    let count = marks.report.as_ref().map_or(0, |r| r.changes.len()) + marks.regions.len();
    // ui-text-exempt: the funnel's trace label, never displayed
    super::funnel::vector_edit(doc, "compare-mark", page, count, |session| {
        mark(session, prefs, marks)
    });
}

/// Run both marking verbs and fold what they pushed. The sentences returned
/// reach the status line through the funnel.
fn mark(
    session: &mut EditSession,
    prefs: &Prefs,
    marks: &ComparisonMarks,
) -> Result<Vec<String>, ResultError> {
    let before = session.undo_depth();
    let (older, newer) = marks.labels.clone();
    let options = ResultOptions::default()
        .with_summary_page(marks.summary_page)
        .with_author(prefs.author_name.clone())
        .with_labels(older, newer);
    let mut notes = Vec::new();
    let mut annotations = 0;
    let mut unmarked = 0;
    if !marks.regions.is_empty() {
        let report = mark_regions(session, &marks.regions, &options)?;
        annotations += report.annotations;
        unmarked += report.unmarked.len();
    }
    if let Some(changes) = &marks.report {
        match mark_changes(session, changes, &options) {
            Ok(report) => {
                annotations += report.annotations;
                unmarked += report.unmarked.len();
            }
            // The area marks are already applied; keep them and say why the
            // text marks are not there.
            Err(e) if annotations > 0 => notes.push(t::mark_failed(&e.to_string())),
            Err(e) => return Err(e),
        }
    }
    let folded = fold(session, before);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            "compare-marked annotations={annotations} unmarked={unmarked} folded={}",
            u8::from(folded)
        )
    });
    notes.push(t::marked(annotations));
    if unmarked > 0 {
        notes.push(t::unmarked(unmarked));
    }
    if !folded {
        notes.push(t::mark_many_undos().to_owned());
    }
    Ok(notes)
}

/// Fold every command pushed since the stack stood at `before` into one.
///
/// The count is read off the stack rather than off the reports, because the
/// summary page and a mark spanning lines push commands the reports do not
/// count one for one. A stack at its cap may have dropped the oldest entries,
/// which makes the difference a lie, so a full stack is never folded.
fn fold(session: &mut EditSession, before: usize) -> bool {
    let after = session.undo_depth();
    if after >= MAX_UNDO_DEPTH {
        return false;
    }
    let pushed = after.saturating_sub(before);
    if pushed < 2 {
        return true;
    }
    let Some(kind) = session.undo_kinds().nth(pushed - 1) else {
        return false;
    };
    session.coalesce_last(pushed, kind)
}

impl crate::app::unlock::Refusal for ResultError {
    fn encrypted(&self) -> bool {
        use pdfcer_core::edit::EditError;
        use pdfcer_core::text_edit::PlaceTextError;
        matches!(
            self,
            Self::Pages(EditError::DocumentEncrypted)
                | Self::Summary(PlaceTextError::Insert(EditError::DocumentEncrypted))
        )
    }
}
