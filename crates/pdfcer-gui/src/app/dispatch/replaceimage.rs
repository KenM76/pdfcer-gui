//! # `app::dispatch::replaceimage` — Format ▸ Replace image: choose a picture
//! file and draw it in place of the one selected image
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/dispatch/replaceimage.md`.

use pdfcer_core::vector::{ImageSource, VectorObject};
use pdfcer_gui_base::canvastarget::CanvasTargetProvider;

use crate::app::PdfcerApp;
use crate::app::actions::{Action, VectorAction};
use crate::app::state::{OpenDoc, Status};
use crate::canvas::selection::SelectionLevel;

/// The command id.
pub(crate) const ID: &str = "format.replace_image";

/// The page and page object a Replace acts on: the Object rung, exactly one
/// page object selected, and that object an image the engine can replace — an
/// image XObject or an inline image, never a form XObject.
#[must_use]
pub(crate) fn replaceable_image(doc: &OpenDoc) -> Option<(usize, usize)> {
    if doc.selection.level() != SelectionLevel::Object || doc.selection.entries().len() != 1 {
        return None;
    }
    let page = doc.selection.entries().first()?.page;
    let [object] = doc.selection.object_indices_on(page)[..] else {
        return None;
    };
    let provider = doc.page_objects()?;
    let model = provider.page_objects_model(page)?;
    match model.objects.get(object)? {
        VectorObject::Image(img) if img.source != ImageSource::Form => Some((page, object)),
        _ => None,
    }
}

/// Pick a file, import it, and queue the replacement; a drawing is refused
/// with a status sentence, a failed import with the importer's.
pub(super) fn dispatch(app: &PdfcerApp, actions: &mut Vec<Action>) {
    let target = match &app.status {
        Status::Open(doc) if app.capabilities().edit_content => replaceable_image(doc),
        _ => None,
    };
    let Some((page, object)) = target else {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("command-declined id={ID} reason=no-image-selected")
        });
        return;
    };
    let crate::app::files::Picked::Path(path) = crate::app::files::pick_image_source() else {
        return;
    };
    let Some(picture) = super::images::import(&app.status, &path) else {
        return;
    };
    let Some(image) = picture.raster() else {
        if let Status::Open(doc) = &app.status {
            crate::app::actions::record_note(
                doc.edit_epoch,
                crate::text::images::replace_needs_raster(picture.kind()),
            );
        }
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!(
                "command-declined id={ID} reason=not-a-raster kind={}",
                picture.kind()
            )
        });
        return;
    };
    actions.push(Action::Vector(VectorAction::ReplaceImage {
        page,
        object,
        image: std::sync::Arc::new(image.clone()),
    }));
}
