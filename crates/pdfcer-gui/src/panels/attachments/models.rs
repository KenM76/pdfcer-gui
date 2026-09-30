//! The 3D models a document carries — U3D, PRC or STEP data inside `/3D` and
//! `/RichMedia` annotations — one row each, with a Save button.

use egui::Ui;
use pdfcer_core::threed::{ThreeDNotes, list_3d_with_notes};

use crate::app::actions::Action;
use crate::app::actions::attachments::AttachmentAction;
use crate::app::state::OpenDoc;
use crate::text::panels::models as t;

/// The region the first model's Save button publishes.
pub const REGION_SAVE: &str = "models.save"; // ui-text-exempt: trace region name, never displayed

/// Draw the section; a document with no 3D content draws nothing.
pub fn section(ui: &mut Ui, doc: &OpenDoc, actions: &mut Vec<Action>) {
    let (listed, notes) = list_3d_with_notes(&*doc.session);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        format!(
            "models-section count={} without_stream={} truncated={}",
            listed.len(),
            notes.annotations_without_stream,
            notes.truncated
        )
    });
    if listed.is_empty() && notes == ThreeDNotes::default() {
        return;
    }
    // No `.strong()` — DEFECTS.md D11: pale text on pale ground.
    ui.label(t::heading());
    ui.label(t::count(listed.len()));
    for said in t::listing_notes(&notes) {
        ui.label(egui::RichText::new(said).small().weak());
    }
    for (ordinal, artwork) in listed.iter().enumerate() {
        ui.label(t::row(artwork));
        if let Some(said) = t::source(artwork) {
            ui.label(egui::RichText::new(said).small().weak());
        }
        let save = ui.button(t::save_button()).on_hover_text(t::save_tooltip());
        if ordinal == 0 {
            crate::diag::ui_rect_visible(REGION_SAVE, save.rect, ui.clip_rect());
        }
        if save.clicked() {
            actions.push(Action::Attachment(AttachmentAction::SaveModel {
                artwork: artwork.clone(),
            }));
        }
    }
    ui.separator();
}
