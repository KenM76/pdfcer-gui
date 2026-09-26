//! # `shell::commands::catalog::arrange` — the Markup tab's **Arrange** group:
//! which mark is drawn on top
//!
//! Design and rationale: `docs/modules/pdfcer-gui/shell/commands/catalog/arrange.md`.

use egui_shell::Command;

use super::command;
use crate::text::commands as t;

/// This group's commands, in ribbon order.
///
/// ★ **Front first, back last** — the order every reference application uses,
/// and it is not arbitrary: read top to bottom the four are a single axis from
/// nearest to furthest, so the list itself teaches what the words mean. Sorting
/// them any other way (the two ends together, then the two steps) would group
/// them by *how far* rather than by *which way*, and an operator scanning for
/// "send it back" would have to read all four.
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
