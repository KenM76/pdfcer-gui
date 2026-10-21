//! # `app::dispatch::shrinkimages` — Format ▸ Make pictures smaller: open
//! the picture options for every picture selected
//!
//! Contract: [`shrinkable_images`] is the one test behind the command's
//! `enabled_when`, its canvas and Objects-panel menu rows and this dispatch:
//! the Object rung, and at least one selected object an image drawn through
//! an image XObject. Inline images have no stream of their own to rewrite and
//! form XObjects are not pictures, so both are left out; a mixed selection
//! offers the command for the pictures in it. The window is
//! `dialogs::shrink_images`.

use pdfcer_core::object::ObjId;
use pdfcer_core::vector::{ImageSource, VectorObject};
use pdfcer_gui_base::canvastarget::CanvasTargetProvider;

use crate::app::PdfcerApp;
use crate::app::state::{OpenDoc, Status};
use crate::canvas::selection::{SelectionLevel, SelectionState};

/// The command id.
pub(crate) const ID: &str = "format.shrink_images";

/// The first page a picture is selected on, and the selected pictures' image
/// XObjects, each once. `None` when no selected object is such a picture.
#[must_use]
pub(crate) fn shrinkable_images(
    doc: &OpenDoc,
    selection: &SelectionState,
) -> Option<(usize, Vec<ObjId>)> {
    if selection.level() != SelectionLevel::Object {
        return None;
    }
    let provider = doc.page_objects()?;
    let mut pages: Vec<usize> = selection.entries().iter().map(|e| e.page).collect();
    pages.sort_unstable();
    pages.dedup();
    let mut first = None;
    let mut ids = Vec::new();
    for page in pages {
        let Some(model) = provider.page_objects_model(page) else {
            continue;
        };
        for object in selection.object_indices_on(page) {
            let Some(VectorObject::Image(img)) = model.objects.get(object) else {
                continue;
            };
            let Some(id) = img.xobject.filter(|_| img.source != ImageSource::Form) else {
                continue;
            };
            first.get_or_insert(page);
            if !ids.contains(&id) {
                ids.push(id);
            }
        }
    }
    Some((first?, ids))
}

/// Open the window on the selected pictures, or trace why not.
pub(super) fn dispatch(app: &mut PdfcerApp) {
    let target = match &app.status {
        Status::Open(doc) if app.capabilities().edit_content => {
            shrinkable_images(doc, &doc.selection).map(|(page, ids)| (doc.serial, page, ids))
        }
        _ => None,
    };
    let Some((serial, page, ids)) = target else {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("command-declined id={ID} reason=no-picture-selected")
        });
        return;
    };
    app.dialogs.open_shrink_images(serial, page, ids);
}
