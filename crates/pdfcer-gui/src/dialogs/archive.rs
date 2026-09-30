//! The Add archive time-stamp window: which server to ask, and what the stamp
//! will be on this document. The work happens in `app::actions::archive`.

use egui::Ui;

use crate::app::actions::Action;
use crate::app::state::{OpenDoc, Status};
use crate::text::archive as t;

/// The window body's rect, for `ui-verify`.
// ui-text-exempt: trace region name, never displayed
pub const REGION_BODY: &str = "archive.body";
/// The server field.
// ui-text-exempt: trace region name, never displayed
pub const REGION_SERVER: &str = "archive.server";
/// The button that goes on to the picker.
// ui-text-exempt: trace region name, never displayed
pub const REGION_SAVE: &str = "archive.commit";

/// The window's state.
pub struct ArchiveDialog {
    server: String,
    /// Signatures in the document when the window opened.
    signatures: usize,
    save_requested: bool,
    close_requested: bool,
}

impl ArchiveDialog {
    fn open(doc: &OpenDoc, remembered: Option<&str>) -> Self {
        Self {
            server: remembered.unwrap_or_default().to_owned(),
            signatures: doc.session.signature_census().signatures,
            save_requested: false,
            close_requested: false,
        }
    }

    /// Draw it. Returns whether it stays open. The server asked is remembered
    /// in `prefs`, shared with the Sign window.
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        actions: &mut Vec<Action>,
        prefs: &mut crate::app::prefs::Prefs,
    ) -> bool {
        let (frame, ()) = crate::dialogs::host::Host::new(
            "archive-timestamp", // ui-text-exempt: a viewport key, never displayed.
            t::window_title(),
            egui::vec2(520.0, 260.0),
            egui::vec2(380.0, 200.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            self.body(ui);
        });
        if std::mem::take(&mut self.save_requested) {
            let server = self.server.trim().to_owned();
            if prefs.sign_timestamp_server.as_deref() != Some(server.as_str()) {
                prefs.sign_timestamp_server = Some(server.clone());
            }
            crate::diag::trace(|| format!("archive-requested signatures={}", self.signatures)); // ui-text-exempt: diagnostic trace, never displayed
            actions.push(Action::ArchiveTimestamp { server });
            return false;
        }
        !frame.closed && !std::mem::take(&mut self.close_requested)
    }

    fn body(&mut self, ui: &mut Ui) {
        ui.label(t::expected(self.signatures));
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.label(t::server_label());
            // escape-disposition: dialog-cancels
            let field = ui
                .add(egui::TextEdit::singleline(&mut self.server).desired_width(f32::INFINITY))
                .on_hover_text(t::server_hover());
            crate::diag::ui_rect_visible(REGION_SERVER, field.rect, ui.clip_rect());
        });
        ui.add_space(12.0);
        ui.separator();
        let ready = crate::sign::timestamp::requested(&self.server).is_some();
        ui.horizontal(|ui| {
            let save = ui
                .add_enabled(ready, egui::Button::new(t::save_button()))
                .on_disabled_hover_text(t::needs_server());
            crate::diag::ui_rect_visible(REGION_SAVE, save.rect, ui.clip_rect());
            if save.clicked() {
                self.save_requested = true;
            }
            if ui.button(t::cancel_button()).clicked() {
                self.close_requested = true;
            }
        });
    }
}

/// The window for the current document, if one is open.
pub fn open_for(status: &Status, remembered: Option<&str>) -> Option<ArchiveDialog> {
    match status {
        Status::Open(doc) => Some(ArchiveDialog::open(doc, remembered)),
        _ => None,
    }
}
