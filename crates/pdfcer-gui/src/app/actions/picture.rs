//! `app::actions::picture` — the apply half of Insert image: a raster picture
//! through `EditSession::add_image`, an SVG or EMF drawing through
//! `add_svg_on_layer` / `add_emf_on_layer`, each on the current layer, and the
//! placed object selected so the next press resizes it.
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
    let label = match picture {
        Picture::Raster(image) => {
            raster(doc, page, rect, fit, image);
            select_newest(doc, page);
            return;
        }
        #[cfg(feature = "svg-import")]
        Picture::Svg(_) => "add-svg",
        Picture::Emf(_) => "add-emf",
    };
    let Ok(layer) = super::drawlayer::for_add(doc, label) else {
        return;
    };
    let on = super::drawlayer::id(layer.as_ref());
    let receipt = layer.map(|l| l.receipt);
    vector_edit(doc, label, page, 1, |session| {
        let notes = match picture {
            Picture::Raster(_) => Vec::new(),
            #[cfg(feature = "svg-import")]
            Picture::Svg(svg) => {
                let p = session.add_svg_on_layer(page, rect, svg, on)?;
                drawing_notes(p.distorted, &p.notes.summary())
            }
            Picture::Emf(emf) => {
                let p = session.add_emf_on_layer(page, rect, emf, on)?;
                drawing_notes(p.distorted, &p.notes.summary())
            }
        };
        Ok::<_, pdfcer_core::edit::EditError>(notes.into_iter().chain(receipt.clone()).collect())
    });
    select_newest(doc, page);
}

/// Place `picture` as a `/Stamp` comment filling `rect` on `page`, pre-turned
/// by the page's `/Rotate` so it reads upright (`super::customstamp` carries
/// the argument), signed by `author`, dated and given the pen's `opacity`.
pub(super) fn stamp(
    doc: &mut OpenDoc,
    page: usize,
    rect: Rect,
    picture: &Picture,
    author: &str,
    opacity: Option<f64>,
) {
    let rotate = doc.pages.get(page).map_or(0, |p| p.rotate);
    let kind = picture.kind();
    let Ok(layer) = super::drawlayer::for_add(doc, "stamp-image") else {
        return;
    };
    let options = pdfcer_core::edit::MarkupOptions {
        note: Some(super::annots::signed_note("", Some(author))),
        opacity,
        layer: super::drawlayer::id(layer.as_ref()),
        ..Default::default()
    };
    let label = match picture {
        Picture::Raster(_) => "stamp-image",
        #[cfg(feature = "svg-import")]
        Picture::Svg(_) => "stamp-svg",
        Picture::Emf(_) => "stamp-emf",
    };
    vector_edit(doc, label, page, 1, |session| {
        let (id, notes) = match picture {
            Picture::Raster(image) => (
                Some(session.add_image_stamp(page, rect, image, &options)?),
                Vec::new(),
            ),
            #[cfg(feature = "svg-import")]
            Picture::Svg(svg) => {
                let p = session.add_svg_stamp(page, rect, svg, &options)?;
                (p.annot_id, drawing_notes(p.distorted, &p.notes.summary()))
            }
            Picture::Emf(emf) => {
                let p = session.add_emf_stamp(page, rect, emf, &options)?;
                (p.annot_id, drawing_notes(p.distorted, &p.notes.summary()))
            }
        };
        let turned = match id.filter(|_| rotate != 0) {
            Some(id) => {
                let pivot = ((rect.llx + rect.urx) / 2.0, (rect.lly + rect.ury) / 2.0);
                session
                    .set_annotation_rotation(id, pivot, f64::from(rotate))?
                    .degrees
            }
            None => 0.0,
        };
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!(
                "picture-stamp-placed kind={kind} id={} page={page} rotate={rotate} \
                 turned={turned} signed={}",
                id.map_or(0, |id| id.num),
                !author.is_empty()
            )
        });
        Ok::<_, pdfcer_core::edit::EditError>(notes)
    });
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
    let Ok(layer) = super::drawlayer::for_add(doc, "add-image") else {
        return;
    };
    vector_edit(doc, "add-image", page, 1, |session| {
        // The constructor, because `NewImage` is `#[non_exhaustive]`.
        let spec = pdfcer_core::edit::NewImage::new(page, rect, image);
        let spec = match &layer {
            Some(l) => spec.on_layer(l.id),
            None => spec,
        };
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
            notes.extend(layer.map(|l| l.receipt));
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
pub(super) fn select_newest(doc: &mut OpenDoc, page: usize) {
    let count = doc
        .page_objects()
        .map(|provider| provider.page_objects().objects.len());
    if let Some(last) = count.and_then(|n| n.checked_sub(1)) {
        doc.selection
            .select_placed(page, crate::canvas::target::TargetId::Object(last as u64));
    }
}
