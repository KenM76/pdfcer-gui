//! # `text::handsign` — what the *Sign here* window says
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/handsign.md`.

/// The window's title bar.
#[must_use]
pub const fn window_title() -> &'static str {
    "Sign here"
}

/// The opening instruction.
#[must_use]
pub const fn intro() -> &'static str {
    "Draw your signature in the box below: hold the left mouse button down and write."
}

/// Shown faintly inside the empty drawing area.
#[must_use]
pub const fn pad_hint() -> &'static str {
    "Draw your signature here"
}

/// What the result is, and what it is not.
#[must_use]
pub const fn what_it_is() -> &'static str {
    "Your signature goes onto the page like ink on paper. It is not a digital (certificate) \
     signature."
}

/// Wipes the drawing area.
#[must_use]
pub const fn clear() -> &'static str {
    "Clear"
}

/// Takes back the last stroke drawn.
#[must_use]
pub const fn undo_stroke() -> &'static str {
    "Undo last stroke"
}

/// Puts the remembered or last-placed signature back in the drawing area.
#[must_use]
pub const fn use_last() -> &'static str {
    "Use my last signature"
}

/// The keep-a-copy option.
#[must_use]
pub const fn remember() -> &'static str {
    "Remember my signature on this computer"
}

/// Hover on [`remember`].
#[must_use]
pub const fn remember_hover() -> &'static str {
    "Keeps a copy beside pdfcer, never inside the document, so you can place it again next \
     time. Untick it and place a signature to delete the copy."
}

/// The commit button.
#[must_use]
pub const fn place() -> &'static str {
    "Place signature"
}

/// Hover on [`place`] while nothing usable is drawn.
#[must_use]
pub const fn place_needs_drawing() -> &'static str {
    "Draw your signature first."
}

/// Closes the window without signing.
#[must_use]
pub const fn cancel() -> &'static str {
    "Cancel"
}

/// The certificate route, offered only in builds that can sign with one.
#[must_use]
pub const fn digital_id() -> &'static str {
    "Use a digital ID (certificate) instead…"
}

/// Hover on [`digital_id`].
#[must_use]
pub const fn digital_id_hover() -> &'static str {
    "A certificate signature proves who signed and shows any later change. You need a digital \
     ID file (.pfx or .p12)."
}
