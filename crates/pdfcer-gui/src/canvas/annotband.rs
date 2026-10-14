//! # `canvas::annotband` — a select marquee over markups, links and ce dimensions
//!
//! Runs after the content marquee on the same release. Page content and
//! annotations cannot be selected together (`SelectionState` holds one or
//! the other), so a band that caught content keeps the content, as it always
//! has, and the status line says which annotations were left out and how to
//! band them alone. A band that caught no content selects the annotations it
//! caught, combined with Shift (add) and Ctrl (subtract) as the content band
//! is.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/annotband.md`.

use crate::app::modes::Capabilities;
use crate::app::state::OpenDoc;
use crate::canvas::marquee::Combine;
use crate::canvas::pick::PickFilter;
use crate::canvas::selection::SelectionState;

/// Combine a completed select band's annotations into `selection`.
///
/// `band` is the band in canvas space and whether it is a crossing band.
pub fn band_select(
    doc: &OpenDoc,
    caps: &Capabilities,
    pick: &PickFilter,
    page: usize,
    band: (egui::Rect, bool),
    combine: Combine,
    selection: &mut SelectionState,
) {
    if !caps.author_markup || !doc.annotations_visible() {
        return;
    }
    let (rect, crossing) = band;
    let hits: Vec<_> = crate::canvas::selection::annot::in_band(doc, page, rect, crossing)
        .into_iter()
        .filter(|h| crate::canvas::selection::annot::pickable(pick, caps, &h.target))
        .collect();
    let caught = hits.len();
    let kept = if !selection.entries().is_empty() {
        if caught > 0 {
            crate::app::actions::record_note(
                doc.edit_epoch,
                crate::text::pick::annotations_left_out(caught),
            );
        }
        "content"
    } else if hits.is_empty() && combine != Combine::Replace {
        return;
    } else {
        selection.band_annots(hits, crate::canvas::marquee::merge(combine));
        "annots"
    };
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            "annot-band page={page} caught={caught} merge={} crossing={crossing} kept={kept} \
             selected={}",
            combine.label(),
            selection.annots().len()
        )
    });
}
