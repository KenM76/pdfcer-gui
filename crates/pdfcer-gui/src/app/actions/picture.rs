//! `app::actions::picture` — the apply half of Insert image: a raster picture
//! through `EditSession::add_image`, an SVG or EMF drawing through `add_svg` /
//! `add_emf`, and the placed object selected so the next press resizes it.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/picture.md`.

use pdfcer_core::edit::ImageFit;
use pdfcer_core::page_tree::Rect;
use pdfcer_gui_base::picture::Picture;

use super::apply::vector_edit;
use crate::app::state::OpenDoc;
use crate::text::images;

/// Place `picture` in `rect` on `page`, then select it. `fit` applies to a
/// raster only: the engine always stretches a drawing to the box, and says so
/// through `distorted`.
pub(super) fn insert(doc: &mut OpenDoc, page: usize, rect: Rect, fit: ImageFit, picture: &Picture) {
    match picture {
        Picture::Raster(image) => raster(doc, page, rect, fit, image),
        #[cfg(feature = "svg-import")]
        Picture::Svg(svg) => vector_edit(doc, "add-svg", page, 1, |session| {
            session
                .add_svg(page, rect, svg)
                .map(|p| drawing_notes(p.distorted, &p.notes.summary()))
        }),
        Picture::Emf(emf) => vector_edit(doc, "add-emf", page, 1, |session| {
            session
                .add_emf(page, rect, emf)
                .map(|p| drawing_notes(p.distorted, &p.notes.summary()))
        }),
    }
    select_newest(doc, page);
}

fn drawing_notes(distorted: bool, summary: &str) -> Vec<String> {
    images::drawing_disclosures(distorted, (!summary.is_empty()).then_some(summary))
}

/// Every placement says the effective resolution, whether the picture kept
/// its shape, and whether it was re-encoded: a picture at 12 dpi and one at
/// 300 look the same on screen and differ on paper.
fn raster(
    doc: &mut OpenDoc,
    page: usize,
    rect: Rect,
    fit: ImageFit,
    image: &pdfcer_core::image_import::ImportedImage,
) {
    vector_edit(doc, "add-image", page, 1, |session| {
        // The constructor, because `NewImage` is `#[non_exhaustive]`.
        let spec = pdfcer_core::edit::NewImage::new(page, rect, image);
        let spec = match fit {
            ImageFit::Stretch => spec.stretching(),
            // Contain is the default; a fit mode the engine adds lands here,
            // which never distorts a picture the operator did not ask to.
            _ => spec,
        };
        session.add_image(&spec).map(|outcome| {
            let d = &outcome.disclosures;
            let mut notes = images::placement_disclosures(
                d.effective_dpi,
                d.below_screen_resolution,
                d.letterboxed,
                d.aspect_distorted,
                d.recompressed,
                d.source_bytes,
                d.stored_bytes,
            );
            notes.extend(images::source_decoding_notes(d));
            notes
        })
    });
}

/// Select the page's last object in paint order, which is what every
/// placement verb appends. Unselected, the operator's first press on it would
/// be a marquee rather than a resize. The count comes from the model rebuilt
/// after the edit, and its borrow ends before the selection is touched. A page
/// that no longer decomposes leaves the selection alone rather than naming an
/// index that may be some other object.
fn select_newest(doc: &mut OpenDoc, page: usize) {
    let count = doc
        .page_objects()
        .map(|provider| provider.page_objects().objects.len());
    if let Some(last) = count.and_then(|n| n.checked_sub(1)) {
        doc.selection
            .select_placed(page, crate::canvas::target::TargetId::Object(last as u64));
    }
}
