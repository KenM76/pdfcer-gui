//! # `text::status::refused` — **the one sentence for an edit the ENGINE
//! refused and could not be asked why**
//!
//! Design and rationale: `docs/modules/pdfcer-gui/text/status/refused.md`.

/// **An edit reached the engine, the engine refused it, and this shell cannot
/// say why** — `OPERATOR_REQUESTS.md` **O116**, 2026-09-04.
#[must_use]
pub const fn edit_declined_by_engine() -> &'static str {
    "That change was refused, and the document is unchanged."
}
