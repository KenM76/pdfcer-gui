//! # `app::panels` — putting a panel on screen, and taking it off again
//!
//! Two methods, and the distinction between them is the whole subject of this
//! file: **not every command that shows a panel is a toggle.**
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/panels.md`.

use crate::app::PdfcerApp;

impl PdfcerApp {
    /// **Show `panel`, or close it if it is already on screen.**
    pub(super) fn toggle_panel(&mut self, panel: crate::panels::Panel) {
        let id = egui_shell::dock::PanelId::new(panel.command_id());
        if !self.dock.is_on_screen(&id) {
            self.show_panel(panel);
            return;
        }
        let closed = self.dock.layout_mut().close(&id);
        self.dock.normalize();
        self.modes
            .record_layout(self.dock.layout(), &mut self.layout);
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed.
                "panel-closed id={} closed={closed}",
                id.as_str()
            )
        });
    }

    /// **Put `panel` on screen, mounting it first if the operator's
    /// arrangement no longer holds it.**
    pub(super) fn show_panel(&mut self, panel: crate::panels::Panel) {
        let id = egui_shell::dock::PanelId::new(panel.command_id());

        if self.dock.activate(&id) {
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed.
                    "panel-shown id={} mounted=already",
                    id.as_str()
                )
            });
            return;
        }

        let home = crate::app::modes::layout_for("edit").find(&id);
        let (side, column, stack) = home.map_or((egui_shell::dock::DockSide::Right, 0, 0), |a| {
            (a.side, a.column, a.stack)
        });
        self.dock
            .layout_mut()
            .mount(side, column, stack, id.clone());
        self.dock.normalize();
        let shown = self.dock.activate(&id);
        self.modes
            .record_layout(self.dock.layout(), &mut self.layout);
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed.
                "panel-shown id={} mounted=now side={} shown={shown}",
                id.as_str(),
                side.key(),
            )
        });
    }
}
