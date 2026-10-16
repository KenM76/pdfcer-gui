//! # `app::paneltoggles` — show or hide any panel from a dock right-click
//!
//! The `dock.tab` and `dock.blank` menus carry a `panel_toggles` custom item;
//! this draws it as one checkbox per panel this build registers (R8: an
//! unregistered panel command lists nothing) and whose command is enabled now.
//! A tick means the panel has a tab somewhere in the dock, or floats. The
//! choice is applied after the dock is drawn, by `PdfcerApp::toggle_tab`.

use crate::panels::Panel;
use egui_shell::dock::{DockState, PanelId};
use egui_shell::{CommandRegistry, ConditionSet, HandlerToken};

/// The menu item kind that draws the checkboxes.
pub const KIND: &str = "panel_toggles"; // ui-text-exempt: manifest kind, never displayed

/// The panels a dock menu offers, and the one the operator picked.
#[derive(Debug, Default)]
pub struct PanelToggles {
    rows: Vec<(Panel, String, bool)>,
    /// The panel whose checkbox was clicked this frame.
    pub picked: Option<Panel>,
}

impl PanelToggles {
    /// Every registered, enabled panel, with its label and whether it has a tab.
    #[must_use]
    pub fn census(registry: &CommandRegistry, conditions: &ConditionSet, dock: &DockState) -> Self {
        let rows = Panel::ALL
            .into_iter()
            .filter_map(|panel| {
                let command = registry.get(panel.command_id())?;
                command.is_enabled(conditions).then(|| {
                    let shown = dock.layout().contains(&PanelId::new(panel.command_id()));
                    (panel, command.label.clone(), shown)
                })
            })
            .collect();
        Self { rows, picked: None }
    }

    /// Whether no panel can be listed.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Draw the checkboxes for a `panel_toggles` row; other kinds draw nothing.
    pub fn draw(&mut self, ui: &mut egui::Ui, kind: &str) -> Option<HandlerToken> {
        if kind != KIND {
            return None;
        }
        for (panel, label, shown) in &self.rows {
            let mut ticked = *shown;
            let response = ui.checkbox(&mut ticked, label.as_str());
            crate::diag::ui_rect(
                &format!("dock.menu.panel.{}", panel.command_id()), // ui-text-exempt: trace region
                response.rect,
            );
            if response.clicked() {
                self.picked = Some(*panel);
                ui.close();
            }
        }
        None
    }
}
