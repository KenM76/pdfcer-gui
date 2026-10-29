//! # The phone layout's bars (`MOBILE_DESIGN.md` §3)
//!
//! Drawn in place of the ribbon, the document tabs and the docks when the
//! window is narrower than [`egui_shell::ribbon::phone::PHONE_MAX_WIDTH`]. The
//! commands, conditions, glyphs and rect reports are the ribbon's own; only
//! the placement differs.

use egui_shell::ribbon::phone::PhoneText;

use crate::app::PdfcerApp;
use crate::app::actions::Action;
use crate::app::state::Status;
use crate::text::phone as t;

impl PdfcerApp {
    /// The app bar, bottom bar and tool sheet, dispatching what was invoked.
    pub(super) fn phone_bars(&mut self, ui: &mut egui::Ui, actions: &mut Vec<Action>) {
        let Some(shell) = self.shell.as_ref() else {
            return;
        };
        let conditions = self.conditions(ui.ctx());
        let mut report_rect = |name: &str, rect: egui::Rect| {
            crate::diag::ui_rect(name, rect);
        };
        let mut icons = crate::icons::paint_ribbon_icon;
        let title = self
            .active_path()
            .map(|path| crate::text::doctabs::tab_label(path, false))
            .unwrap_or_else(|| t::NO_DOCUMENT.to_owned());
        let subtitle = match &self.status {
            Status::Open(doc) if !doc.pages.is_empty() => {
                t::page_subtitle(doc.view.page_index + 1, doc.pages.len())
            }
            _ => String::new(),
        };
        let text = PhoneText {
            title: &title,
            subtitle: &subtitle,
            more: t::MORE,
            all_tools: t::ALL_TOOLS,
            all_tools_icon: Some(pdfcer_gui_base::icons::Icon::Grid.name()),
        };
        let tokens = egui_shell::ribbon::Ribbon::new()
            .with_conditions(&conditions)
            .reporting_rects_to(&mut report_rect)
            .with_icon_painter(&mut icons)
            .with_tab_label(self.prefs.ribbon_tab_face)
            .render_phone(
                ui,
                shell,
                &self.commands,
                &mut self.ribbon,
                &mut self.phone,
                text,
            );
        for token in tokens {
            self.dispatch_token(ui.ctx(), token, actions);
        }
    }
}
