//! **Re-span a text markup** — `EditSession::respan_text_markup`, raised by
//! `canvas::respan` on the release of an end-grip drag.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/annots/respan.md`.

use super::*;

/// The funnel label, and the first token of the funnel's own trace line.
const LABEL: &str = "respan-text-markup"; // ui-text-exempt: diagnostic trace name

/// Replace `id`'s `/QuadPoints` with `quads` and re-bake its appearance, as
/// one undo entry. Stamps `/M`; discloses every property the re-bake dropped.
pub(super) fn respan(doc: &mut OpenDoc, id: ObjId, quads: &[pdfcer_core::annot_author::Quad]) {
    let modified = crate::app::clock::pdf_date_utc();
    super::super::apply::vector_edit(doc, LABEL, 0, 1, |session| {
        session
            .respan_text_markup(id, quads, modified.as_deref())
            .map(|outcome| {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    //
                    // `quads=` BEFORE->AFTER and `rect=` BEFORE->AFTER: a
                    // re-span writes the quads, the `/Rect` and the `/AP`, and
                    // a build that wrote only the quads changes the first pair
                    // and not the second.
                    let before = outcome
                        .rect_before
                        .map_or_else(|| "none".to_owned(), |r| format!("{r:?}"));
                    format!(
                        "{LABEL}-applied id={} subtype={} quads={}->{} rect={before}->{:?} \
                         dropped={} m={}",
                        id.num,
                        outcome.subtype,
                        outcome.quads_before,
                        outcome.quads_after,
                        outcome.rect_after,
                        outcome.dropped.len(),
                        outcome.mod_date_written
                    )
                });
                outcome
                    .dropped
                    .iter()
                    .map(|d| crate::text::panels::properties::markup_dropped(*d).to_owned())
                    .collect()
            })
    });
}
