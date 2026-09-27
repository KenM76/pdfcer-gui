//! # `editmodel` — the text editor's window-free arithmetic
//!
//! The caret, the multi-line draft, the follower disposition and the new-text
//! pen: pure functions of a string, an index or `egui::Memory`, with no page
//! mapping or open document in them. `pdfcer_gui::canvas::textedit`
//! re-exports them and holds everything that needs the window.

/// Which of the two text verbs is armed.
pub mod kind;

/// The caret's own arithmetic — insert, delete, and the four movements. Pure
/// functions of a `&str` and an index, with no window in them; its header says
/// why that is a seam and not a cut.
pub mod caret;
/// Which way the rest of the line moves when an edit changes its width.
pub mod disposition;
/// The caret's arithmetic inside a draft that holds more than one line.
pub mod lines;
/// The face, size and colour new page text is written in, kept in
/// `egui::Memory`; its header says why there and not with the markup pen.
pub mod pen;
// The byte-level proof that the untouched tail did not move, with an
// `EditOptions::default()` run beside it as the falsifier. `#[cfg(test)]`
// inside; it compiles to nothing in a release build.
mod proof;
// The per-keystroke re-measure measurement `DEFECTS.md` D4b's fix would need,
// and the reason it is not wired. `#[ignore]`d; run it and read the numbers.
mod cost;
