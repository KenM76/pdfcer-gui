//! `app::actions::buttonicon` — the apply half of a push button's *Choose
//! picture…*: ask for a file, import it as a raster, and make it the button's
//! icon through `forms::edit_widget`, one undo entry.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/buttonicon.md`.

use pdfcer_core::edit::WidgetEdit;
use pdfcer_gui_base::picture::Picture;

use crate::app::files::{Picked, pick_image_source};
use crate::app::state::OpenDoc;
use crate::text::panels::buttonicon as t;

/// Ask for a picture and set it as `field`'s icon on placement `widget`. A
/// cancelled picker changes nothing; an unreadable file or a drawing changes
/// nothing and says why.
pub(super) fn pick(doc: &mut OpenDoc, field: &str, widget: usize) {
    let Picked::Path(source) = pick_image_source() else {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!("button-icon-cancelled field={field} widget={widget}")
        });
        return;
    };
    let picture = std::fs::read(&source)
        .map_err(|e| e.to_string())
        .and_then(|bytes| Picture::import(&source, &bytes));
    let refusal = match &picture {
        Ok(Picture::Raster(image)) => {
            // As `panels::properties::buttonicon::push_edit`: foreign artwork
            // would hide the picture.
            let edit = WidgetEdit::new()
                .with_button_icon(image)
                .with_replace_foreign_appearance(true);
            super::forms::edit_widget(doc, field, widget, &edit, t::touched_icon());
            return;
        }
        Ok(drawing) => (drawing.kind(), t::icon_not_a_drawing(drawing.kind())),
        Err(detail) => ("unreadable", crate::text::images::import_failed(detail)),
    };
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        format!(
            "button-icon-declined field={field} widget={widget} reason={}",
            refusal.0
        )
    });
    super::record_note(doc.edit_epoch, refusal.1);
}
