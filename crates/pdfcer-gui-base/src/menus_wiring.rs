//! **The optional capabilities pdfcer hands to every context menu**, and
//! the account of why each one exists.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/menus_wiring.md`.

use egui_shell::menu::ContextMenu;
use egui_shell::{CommandRegistry, ConditionSet, HandlerToken};

/// Attach the menu for `context_id` to a widget's secondary click, with
/// pdfcer's two capabilities wired, and report the commands the operator
/// chose.
pub fn attach(
    shell: &egui_shell::manifest::Shell,
    registry: &CommandRegistry,
    response: &egui::Response,
    context_id: &str,
    conditions: &ConditionSet,
) -> Vec<HandlerToken> {
    let mut sink = |name: &str, rect: egui::Rect| crate::diag::ui_rect(name, rect);
    let mut icons = crate::icons::paint_ribbon_icon;
    ContextMenu::new()
        .reporting_rects_to(&mut sink)
        .with_icon_painter(&mut icons)
        .attach(response, shell, registry, context_id, conditions)
}
