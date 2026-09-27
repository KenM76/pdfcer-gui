//! # `app::actions::annot` — the verbs whose subject is a whole annotation
//!
//! Move it, resize it, remove it, write the note on it, **answer it, and
//! record whether its window opens**. Held here rather than in
//! [`super::action`] so that the action enum stays inside R2's 1,500-line
//! ceiling.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/annot.md`.

pub use pdfcer_gui_base::annotaction::AnnotAction;
