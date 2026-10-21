//! # `dialogs::shrink_images` — Format ▸ Make pictures smaller, on the
//! pictures selected in the open document
//!
//! Contract: opened by `app::dispatch::shrinkimages` with the image XObjects
//! the selection names, each once. It offers the compact window's picture
//! rows (`dialogs::compact_images`), says that a picture drawn in more than
//! one place changes everywhere it is drawn, and on Apply pushes
//! [`VectorAction::ShrinkImages`] for the document it was opened on; the
//! edit and its report are `app::actions::vector::shrinkimages`. A refused
//! pairing is greyed by the shared rows, so Apply never carries one.

use pdfcer_core::object::ObjId;

use crate::app::actions::{Action, VectorAction};
use crate::dialogs::compact_images::{self, Pictures};
use crate::text::compact_images as t;

/// The window body.
pub const REGION_BODY: &str = "shrink-images.body"; // ui-text-exempt: trace region name, never displayed
/// The button that applies.
pub const REGION_COMMIT: &str = "shrink-images.commit"; // ui-text-exempt: trace region name, never displayed

/// The window.
#[derive(Debug)]
pub struct ShrinkImagesDialog {
    /// The document it was opened on (`OpenDoc::serial`).
    doc: u64,
    /// The page the first picture was selected on; the edit's trace page.
    page: usize,
    /// The pictures, each once.
    ids: Vec<ObjId>,
    /// The choices; always on here, since opening the window is the asking.
    pictures: Pictures,
    commit_requested: bool,
    close_requested: bool,
}

impl ShrinkImagesDialog {
    /// For the pictures `ids`, first selected on `page`, in document `doc`.
    #[must_use]
    pub fn new(doc: u64, page: usize, ids: Vec<ObjId>) -> Self {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!("shrink-images-opened page={page} pictures={}", ids.len())
        });
        Self {
            doc,
            page,
            ids,
            pictures: Pictures {
                on: true,
                ..Pictures::OFF
            },
            commit_requested: false,
            close_requested: false,
        }
    }

    /// Draw it. Returns whether it stays open; it closes itself when another
    /// document comes on screen, since its pictures are not that document's.
    pub fn show(&mut self, ctx: &egui::Context, active: u64, actions: &mut Vec<Action>) -> bool {
        if active != self.doc {
            return false;
        }
        let (frame, ()) = crate::dialogs::host::Host::new(
            "shrink-images", // ui-text-exempt: a viewport key, never displayed.
            t::shrink_title(),
            egui::vec2(440.0, 320.0),
            egui::vec2(340.0, 240.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            self.body(ui);
        });
        if std::mem::take(&mut self.commit_requested) {
            let Some(patch) = self.pictures.patch() else {
                return false;
            };
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    "shrink-images-requested pictures={} ppi={}",
                    self.ids.len(),
                    self.pictures.ppi.unwrap_or(0)
                )
            });
            actions.push(Action::Vector(VectorAction::ShrinkImages {
                page: self.page,
                ids: self.ids.clone(),
                patch,
            }));
            return false;
        }
        !frame.closed && !std::mem::take(&mut self.close_requested)
    }

    fn body(&mut self, ui: &mut egui::Ui) {
        ui.label(t::shrink_intro(self.ids.len()));
        ui.add_space(6.0);
        egui::Grid::new("shrink-images-grid") // ui-text-exempt: an id, never displayed
            .show(ui, |ui| compact_images::rows(ui, &mut self.pictures));
        ui.add_space(6.0);
        ui.small(t::shrink_everywhere());
        ui.add_space(8.0);
        ui.separator();
        ui.horizontal(|ui| {
            let apply = ui.button(t::shrink_button());
            crate::diag::ui_rect_visible(REGION_COMMIT, apply.rect, ui.clip_rect());
            if apply.clicked() {
                self.commit_requested = true;
            }
            if ui.button(t::shrink_cancel()).clicked() {
                self.close_requested = true;
            }
        });
    }
}
