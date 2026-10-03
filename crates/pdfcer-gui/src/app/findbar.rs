//! # `app::findbar` — drawing the Find overlay from the application
//!
//! Contract: pins the bar inside the canvas viewport (or `content_rect`
//! before the canvas has drawn, which excludes an OS status bar or notch) and
//! offers Replace only where the mode may edit content.

use crate::app::PdfcerApp;
use crate::app::actions::Action;

impl PdfcerApp {
    /// Draw the Find bar for this frame.
    pub(super) fn show_find_bar(&mut self, ui: &mut egui::Ui, actions: &mut Vec<Action>) {
        let host = crate::canvas::zoom::last_frame(ui.ctx())
            .map_or_else(|| ui.ctx().content_rect(), |f| f.viewport_rect);
        let offered = self.capabilities().edit_content;
        crate::find::bar::show(ui, &mut self.find, &self.status, (host, offered), actions);
    }
}
