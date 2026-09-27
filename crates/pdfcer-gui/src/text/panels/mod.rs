//! # `text::panels` — panel copy that names a panel type
//!
//! Re-exports [`pdfcer_gui_base::text::panels`]; the tests that reach into
//! `crate::panels` stay here.

pub use pdfcer_gui_base::text::panels::*;

#[cfg(test)]
mod attachments_tests;
