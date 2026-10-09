//! # `shell::commands::catalog::arrange` — the Markup and Edit tabs'
//! **Arrange** groups: which mark, or which page object, is drawn on top
//!
//! Design and rationale: `docs/modules/pdfcer-gui/shell/commands/catalog/arrange.md`.

use egui_shell::Command;

use super::command;
use crate::text::commands as t;

/// This group's commands, in ribbon order.
pub(super) fn band() -> Vec<Command> {
    vec![
        command("markup.bring_to_front", t::markup_bring_to_front(), 560)
            .enabled_when("selection.markup_restylable"),
        command("markup.bring_forward", t::markup_bring_forward(), 561)
            .enabled_when("selection.markup_restylable"),
        command("markup.send_backward", t::markup_send_backward(), 562)
            .enabled_when("selection.markup_restylable"),
        command("markup.send_to_back", t::markup_send_to_back(), 563)
            .enabled_when("selection.markup_restylable"),
    ]
}

/// The Edit tab's four, on page objects; `app::dispatch::arrange` routes both
/// sets.
pub(super) fn edit_band() -> Vec<Command> {
    use crate::shell::menus::OBJECTS_RESTACKABLE as RESTACKABLE;
    vec![
        command("edit.bring_to_front", t::edit_bring_to_front(), 480).enabled_when(RESTACKABLE),
        command("edit.bring_forward", t::edit_bring_forward(), 481).enabled_when(RESTACKABLE),
        command("edit.send_backward", t::edit_send_backward(), 482).enabled_when(RESTACKABLE),
        command("edit.send_to_back", t::edit_send_to_back(), 483).enabled_when(RESTACKABLE),
    ]
}
