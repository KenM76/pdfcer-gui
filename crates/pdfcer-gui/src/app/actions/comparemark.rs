//! # `app::actions::comparemark` — a comparison's changes as comments
//!
//! `pdfcer_core::compare::mark_comparison` on the open document through the
//! edit funnel: the area marks, then the text marks and the summary page, as
//! ONE undo entry the engine folds while it commits. Area marks alone go
//! through `mark_regions`, which is one entry by itself.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/comparemark.md`.

use pdfcer_core::compare::{ResultError, ResultOptions, mark_comparison, mark_regions};
use pdfcer_core::edit::{CommandKind, EditSession};
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

/// Mark the comparison. The sentences returned reach the status line through
/// the funnel.
///
/// [`ResultError::Summary`] comes after the marks, which the engine keeps
/// as one undo entry; it is returned as `Ok` with the failure as a sentence,
/// because the funnel refreshes the pages only on `Ok`.
fn mark(
    session: &mut EditSession,
    prefs: &Prefs,
    marks: &ComparisonMarks,
) -> Result<Vec<String>, ResultError> {
    let (older, newer) = marks.labels.clone();
    let options = ResultOptions::default()
        .with_summary_page(marks.summary_page)
        .with_author(prefs.author_name.clone())
        .with_labels(older, newer);
    let regions = (!marks.regions.is_empty()).then_some(marks.regions.as_slice());
    let outcome = match &marks.report {
        Some(changes) => mark_comparison(session, regions, changes, &options).map(|done| {
            let (a, u) = done
                .regions
                .as_ref()
                .map_or((0, 0), |r| (r.annotations, r.unmarked.len()));
            (
                a + done.changes.annotations,
                u + done.changes.unmarked.len(),
            )
        }),
        None => mark_regions(session, &marks.regions, &options)
            .map(|r| (r.annotations, r.unmarked.len())),
    };
    let top = match session.undo_kinds().next() {
        Some(CommandKind::MarkComparison) => "mark-comparison", // ui-text-exempt: a trace token
        Some(_) => "other", // ui-text-exempt: a trace token
        None => "none",     // ui-text-exempt: a trace token
    };
    let (annotations, unmarked) = match outcome {
        Ok(counts) => counts,
        Err(e @ ResultError::Summary(_)) if !crate::app::unlock::Refusal::encrypted(&e) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!("compare-marked partial=1 top={top}")
            });
            return Ok(vec![t::mark_failed(&e.to_string())]);
        }
        Err(e) => return Err(e),
    };
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("compare-marked annotations={annotations} unmarked={unmarked} top={top}")
    });
    let mut notes = vec![t::marked(annotations)];
    if unmarked > 0 {
        notes.push(t::unmarked(unmarked));
    }
    Ok(notes)
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
