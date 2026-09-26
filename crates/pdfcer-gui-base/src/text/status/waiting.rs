//! # `text::status::waiting` — **sentences about the program being busy**
//!
//! One function today. It is its own module rather than a paragraph in
//! [`super`] because it is a different **species** of sentence from everything
//! that catalog holds, and the distinction is the one `app::status`'s own header
//! spends forty lines making.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/status/waiting.md`.

/// **The page is still being redrawn** — `OPERATOR_REQUESTS.md` O63.
#[must_use]
pub const fn page_catching_up() -> &'static str {
    "Your change is made \u{2014} the picture of it is still being drawn."
}

/// **Line weights are off, so this is not what will print** —
/// `OPERATOR_REQUESTS.md` **O137**.
#[must_use]
pub const fn line_weights_off() -> &'static str {
    "Line weights are off \u{2014} every line is drawn one pixel wide. Printing and exporting \
     still use the real widths."
}

/// **The line-weights mode is on and it changed NOTHING in view** —
/// `OPERATOR_REQUESTS.md` **O137**, and the half this shell asked the engine
/// for by name.
#[must_use]
pub const fn line_weights_no_effect() -> &'static str {
    "Line weights are off and nothing in view was thick enough to thin — this part of the \
     drawing looks the same either way."
}
