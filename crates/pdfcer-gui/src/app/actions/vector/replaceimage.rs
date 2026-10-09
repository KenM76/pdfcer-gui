//! Commit `VectorAction::ReplaceImage`: draw a new raster in place of one
//! image object, kept in its shape inside the old one's box, as one undo entry.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/vector/replaceimage.md`.

use pdfcer_core::edit::{EditError, ImageFit};
use pdfcer_core::image_import::ImportedImage;
use pdfcer_core::vector::VectorObject;

use crate::app::actions::apply::vector_edit_on_page;
use crate::app::state::OpenDoc;
use crate::text::images as words;

/// Replace the image at `object` on `page` with `image`.
pub(super) fn apply(doc: &mut OpenDoc, page: usize, object: usize, image: &ImportedImage) {
    let old = xobject_of(doc, object);
    let mut new = None;
    vector_edit_on_page(doc, "replace-image", page, 1, |session| {
        let outcome = session.replace_image(page, object, image, ImageFit::Contain)?;
        new = Some(outcome.image_id.num);
        Ok::<_, EditError>(notes(&outcome.disclosures))
    });
    let Some(new) = new else {
        return;
    };
    let model = xobject_of(doc, object);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "image-replaced page={page} object={object} old={} new={new} model={}",
            old.map_or(0, |id| id.num),
            model.map_or(0, |id| id.num)
        )
    });
}

/// The image XObject the model reads at `object`, if it is an image drawn
/// through one; an inline image has none.
fn xobject_of(doc: &OpenDoc, object: usize) -> Option<pdfcer_core::object::ObjId> {
    let provider = doc.page_objects()?;
    match provider.page_objects().objects.get(object)? {
        VectorObject::Image(img) => img.xobject,
        _ => None,
    }
}

/// The off-canvas report: the replacement's resolution, shape and encoding,
/// what decoding its file left out, and that the old image's data stays.
fn notes(d: &pdfcer_core::edit::ImageAuthorDisclosures) -> Vec<String> {
    let mut said = Vec::new();
    if d.letterboxed && !d.aspect_distorted {
        said.push(words::replace_letterboxed().to_owned());
    }
    // The placement sentences' shape clauses speak of "the box you gave it";
    // here the box is the old picture's, said above.
    said.extend(words::placement_disclosures(
        d.effective_dpi,
        d.below_screen_resolution,
        false,
        d.aspect_distorted,
        d.recompressed,
        d.source_bytes,
        d.stored_bytes,
    ));
    said.extend(words::source_decoding_notes(d));
    said.push(words::replace_old_data_kept().to_owned());
    said
}
