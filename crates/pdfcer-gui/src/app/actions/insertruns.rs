//! Inserting a page list that may name a source page more than once.
//!
//! `EditSession::insert_pages` copies its whole list through one import
//! mapping, so a repeated source index resolves to the page object already
//! copied and the target's `/Kids` ends up holding one object twice — a page
//! tree pdfcer's own reader rejects as a cycle. The list is therefore cut into
//! runs with no repeat inside any of them, each run inserted by its own call
//! (a fresh mapping, so fresh copies), and the calls folded into one undo
//! step. A list with no repeat is one run and one call.
//! The engine side is filed as G185 in the request channel.

use pdfcer_core::edit::{CommandKind, EditError, EditSession, InsertOutcome};
use pdfcer_core::pageops::InsertPosition;
use pdfcer_core::view::DocumentView;

/// `pages` cut into consecutive runs, a new run starting wherever an index
/// already appears in the current one. Order is preserved; the runs
/// concatenate back to `pages`.
pub(super) fn distinct_runs(pages: &[usize]) -> Vec<&[usize]> {
    let mut runs = Vec::new();
    let mut start = 0;
    let mut seen = std::collections::HashSet::new();
    for (i, page) in pages.iter().enumerate() {
        if !seen.insert(*page) {
            runs.push(&pages[start..i]);
            start = i;
            seen.clear();
            seen.insert(*page);
        }
    }
    if start < pages.len() {
        runs.push(&pages[start..]);
    }
    runs
}

/// What [`insert`] did: the summed outcome of every call that landed, and
/// the two things a single call cannot report.
pub(super) struct Inserted {
    pub outcome: InsertOutcome,
    /// `false` when the calls landed but could not be folded into one undo
    /// step (`coalesce_last` returned `false`).
    pub folded: bool,
    /// How many engine commands landed — the undo steps when not folded.
    pub calls: usize,
    /// A later run's refusal after earlier runs had already landed. Those
    /// pages stay; the caller discloses how many arrived.
    pub stopped: Option<EditError>,
}

/// Insert `pages` of `source` at `position` into a target of `page_count`
/// pages, as one undo step whatever the repeats.
///
/// # Errors
///
/// The first call's refusal, when nothing landed. A refusal after something
/// landed is `Ok` with [`Inserted::stopped`] set, because the document has
/// changed and the funnel must resync it.
pub(super) fn insert(
    session: &mut EditSession,
    source: &DocumentView<'_>,
    pages: &[usize],
    position: InsertPosition,
    page_count: usize,
) -> Result<Inserted, EditError> {
    let first_slot = position.slot(page_count);
    let mut total = InsertOutcome::default();
    let mut calls = 0;
    let mut stopped = None;
    for run in distinct_runs(pages) {
        let at = if calls == 0 {
            position
        } else {
            let slot = first_slot + total.pages_inserted;
            if slot >= page_count + total.pages_inserted {
                InsertPosition::End
            } else {
                InsertPosition::Before(slot)
            }
        };
        match session.insert_pages(source, run, at) {
            Ok(outcome) => {
                calls += 1;
                add(&mut total, &outcome);
            }
            Err(e) if calls == 0 => return Err(e),
            Err(e) => {
                stopped = Some(e);
                break;
            }
        }
    }
    let folded = calls < 2
        || session.coalesce_last(
            calls,
            CommandKind::InsertPages {
                count: total.pages_inserted,
            },
        );
    Ok(Inserted {
        outcome: total,
        folded,
        calls,
        stopped,
    })
}

/// Insert `count` blank sheets of `size_pt` (width, height) at `position`.
///
/// The engine has no blank-page verb, so the sheets are copies of a one-page
/// blank document of that size, made here and inserted `count` times through
/// [`insert`] — one undo step, a fresh page object per sheet.
pub(super) fn insert_blank(
    doc: &mut crate::app::state::OpenDoc,
    size_pt: (f64, f64),
    count: usize,
    position: InsertPosition,
) {
    let rect = pdfcer_core::page_tree::Rect::from_corners(0.0, 0.0, size_pt.0, size_pt.1);
    let blank = match pdfcer_gui_base::blank::document_sized(rect) {
        Ok((blank, _)) => blank,
        Err(detail) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!("insert-blank-refused reason={detail}")
            });
            super::record_note(doc.edit_epoch, crate::text::pages::insert_failed(&detail));
            return;
        }
    };
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "insert-blank-requested n={count} w_pt={:.2} h_pt={:.2} position={position:?}",
            size_pt.0, size_pt.1
        )
    });
    super::pages::insert_from_view(doc, &blank.view(), &vec![0; count], position);
}

/// Sum `more` into `total`; the flags are true when any call's was.
fn add(total: &mut InsertOutcome, more: &InsertOutcome) {
    total.pages_inserted += more.pages_inserted;
    total.orphaned_widgets += more.orphaned_widgets;
    total.orphaned_widgets_unrecoverable += more.orphaned_widgets_unrecoverable;
    total.source_outline_dropped |= more.source_outline_dropped;
    total.source_page_labels_dropped |= more.source_page_labels_dropped;
    total.page_labels_stale |= more.page_labels_stale;
    total.page_label_ranges += more.page_label_ranges;
}

#[cfg(test)]
mod tests {
    use super::distinct_runs;

    #[test]
    fn a_list_without_repeats_is_one_run() {
        assert_eq!(distinct_runs(&[2, 0, 1]), vec![&[2, 0, 1][..]]);
    }

    #[test]
    fn a_repeat_starts_a_new_run() {
        let runs = distinct_runs(&[0, 0, 0]);
        assert_eq!(runs, vec![&[0][..], &[0][..], &[0][..]]);
        let runs = distinct_runs(&[0, 1, 2, 1, 3]);
        assert_eq!(runs, vec![&[0, 1, 2][..], &[1, 3][..]]);
    }

    #[test]
    fn the_runs_concatenate_back_to_the_list() {
        let pages = [4, 1, 4, 4, 2, 1, 0];
        assert_eq!(distinct_runs(&pages).concat(), pages);
        assert!(distinct_runs(&[]).is_empty());
    }
}
