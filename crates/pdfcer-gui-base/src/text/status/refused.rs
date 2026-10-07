//! # `text::status::refused` — **the one sentence for an edit the ENGINE
//! refused and could not be asked why**
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/status/refused.md`.

/// **An edit reached the engine, the engine refused it, and this shell cannot
/// say why** — `OPERATOR_REQUESTS.md` **O116**, 2026-09-04.
#[must_use]
pub const fn edit_declined_by_engine() -> &'static str {
    "That change was refused, and the document is unchanged."
}

/// An edit that found the document still held by background work (a
/// recognition, a preview render) after the funnel's bounded wait.
#[must_use]
pub const fn session_busy() -> &'static str {
    "Not changed: pdfcer was still finishing background work on this document. Try again \
     in a moment."
}
