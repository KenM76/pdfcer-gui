//! The 3D viewer laid out for a phone: one scrolling column, its buttons
//! first so they are reachable, the picture the full width, then the parts
//! and the notes. `MOBILE_DESIGN.md` section 3 sets the phone width.

use egui::Ui;
use pdfcer_gui_base::icons::{self, Icon};

use super::ModelView;
use crate::text::panels::modelphone as t;

/// The picture's height as a share of the screen's, so the controls above
/// it and the start of the parts list stay in sight.
const PICTURE_SHARE: f32 = 0.55;

/// Room kept between the column and the screen's right edge for the window's
/// frame, in points.
const EDGE: f32 = 16.0;

/// A button showing `icon` alone on a phone, with `label` and `tip` in its
/// tooltip, so a row of them fits the width; `label` in words elsewhere.
pub(super) fn glyph_button(ui: &mut Ui, icon: Icon, label: &str, tip: &str) -> egui::Response {
    if egui_shell::ribbon::phone::is_phone(ui.ctx()) {
        ui.add(egui::Button::image(icons::image(ui, icon)))
            .on_hover_text(label)
            .on_hover_text(tip)
    } else {
        ui.button(label).on_hover_text(tip)
    }
}

impl ModelView {
    /// [`ModelView::body`] on a phone.
    pub(super) fn phone_body(&mut self, ui: &mut Ui) {
        // The column is held to the screen: a vertical scroll area otherwise
        // widens to its widest row, the window grows to match, and every row
        // after it is laid out past the screen's edge.
        let shown = (ui.ctx().content_rect().right() - ui.max_rect().left() - EDGE).max(64.0);
        let width = ui.available_width().min(shown);
        ui.set_max_width(width);
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.set_max_width(width - ui.spacing().scroll.allocated_width());
            ui.horizontal_wrapped(|ui| self.buttons(ui));
            self.controls(ui);
            ui.small(t::view_hint_touch());
            self.file_view_note(ui);
            let width = ui.available_width();
            let height = (ui.ctx().content_rect().height() * PICTURE_SHARE).min(width.max(64.0));
            self.picture(ui, egui::vec2(width, height.max(64.0)));
            self.notes(ui);
            self.parts_list(ui);
        });
    }
}
