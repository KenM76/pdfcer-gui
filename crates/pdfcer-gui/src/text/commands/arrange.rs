//! # `text::commands::arrange` — the four labels of **Markup ▸ Arrange**, the
//! controls that decide which mark is on top
//!
//! Design and rationale: `docs/modules/pdfcer-gui/text/commands/arrange.md`.

use super::CommandText;

/// `markup.bring_to_front`
#[must_use]
pub const fn markup_bring_to_front() -> CommandText {
    CommandText::new(
        "Bring to front",
        "Draw the selected mark over everything else on this page, all the way to the front.",
    )
}

/// `markup.bring_forward`
#[must_use]
pub const fn markup_bring_forward() -> CommandText {
    CommandText::new(
        "Bring forward",
        "Draw the selected mark over the next one up, moving it one place forward on this page.",
    )
}

/// `markup.send_backward`
#[must_use]
pub const fn markup_send_backward() -> CommandText {
    CommandText::new(
        "Send backward",
        "Draw the selected mark under the next one down, moving it one place back on this page.",
    )
}

/// `markup.send_to_back`
#[must_use]
pub const fn markup_send_to_back() -> CommandText {
    CommandText::new(
        "Send to back",
        "Draw the selected mark under everything else on this page, all the way to the back.",
    )
}
