//! # `app::prefs` — the shell's preferences
//!
//! The type and its file format are `pdfcer_gui_base::prefs`; this module adds
//! the one piece that needs the application's document state.

pub use pdfcer_gui_base::prefs::*;

pub mod offpage;

#[cfg(test)]
mod ocrlayer_tests;
