//! # `find` — the Find bar
//!
//! The search, its state and the bar's widgets are `pdfcer_gui_base::find`,
//! re-exported here. This crate keeps the bar tests that need the application:
//! its command registry and its font set.

pub use pdfcer_gui_base::find::*;

#[cfg(test)]
mod tests;
