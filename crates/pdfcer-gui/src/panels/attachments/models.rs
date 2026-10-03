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
/// The region the first PRC model's Save as mesh button publishes.
#[cfg(feature = "3d")]
pub const REGION_MESH: &str = "models.mesh"; // ui-text-exempt: trace region name, never displayed
/// The region the first PRC model's View button publishes.
#[cfg(feature = "3d")]
pub const REGION_VIEW: &str = "models.view"; // ui-text-exempt: trace region name, never displayed
/// The first *Picture…* button, on a model with a page picture to replace.
pub const REGION_POSTER: &str = "models.poster"; // ui-text-exempt: trace region name, never displayed

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
    #[cfg(feature = "3d")]
    let mut first_mesh = true;
    let mut first_poster = true;
    for (ordinal, artwork) in listed.iter().enumerate() {
        ui.label(t::row(artwork));
        if let Some(said) = t::source(artwork) {
            ui.label(egui::RichText::new(said).small().weak());
        }
        ui.horizontal(|ui| {
            let save = ui.button(t::save_button()).on_hover_text(t::save_tooltip());
            if ordinal == 0 {
                crate::diag::ui_rect_visible(REGION_SAVE, save.rect, ui.clip_rect());
            }
            if save.clicked() {
                actions.push(Action::Attachment(AttachmentAction::SaveModel {
                    artwork: artwork.clone(),
                }));
            }
            // Only PRC is decoded; a U3D row offers its bytes alone.
            #[cfg(feature = "3d")]
            if artwork.declared == Some(pdfcer_core::threed::ThreeDFormat::Prc) {
                let view = ui.button(t::view_button()).on_hover_text(t::view_tooltip());
                let mesh = ui.button(t::mesh_button()).on_hover_text(t::mesh_tooltip());
                if first_mesh {
                    crate::diag::ui_rect_visible(REGION_VIEW, view.rect, ui.clip_rect());
                    crate::diag::ui_rect_visible(REGION_MESH, mesh.rect, ui.clip_rect());
                    first_mesh = false;
                }
                if view.clicked() {
                    actions.push(Action::Attachment(AttachmentAction::ViewModel {
                        artwork: artwork.clone(),
                    }));
                }
                if mesh.clicked() {
                    actions.push(Action::Attachment(AttachmentAction::SaveMesh {
                        artwork: artwork.clone(),
                    }));
                }
            }
            if has_own_poster(artwork) {
                let poster = ui
                    .button(t::poster_button())
                    .on_hover_text(t::poster_tooltip());
                if std::mem::take(&mut first_poster) {
                    crate::diag::ui_rect_visible(REGION_POSTER, poster.rect, ui.clip_rect());
                }
                if poster.clicked() {
                    actions.push(Action::Attachment(AttachmentAction::PickModelPoster {
                        artwork: artwork.clone(),
                    }));
                }
            }
        });
    }
    ui.separator();
}

/// A `/3D` annotation of its own, whose page picture `set_3d_poster` can
/// replace; a RichMedia asset has none.
#[must_use]
pub fn has_own_poster(artwork: &pdfcer_core::threed::ThreeDArtwork) -> bool {
    artwork.annot_id.is_some()
        && matches!(
            artwork.source,
            pdfcer_core::threed::ThreeDSource::Stream { .. }
        )
}
