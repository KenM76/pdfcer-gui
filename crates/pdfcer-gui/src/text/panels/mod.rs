//! # `text::panels` — panel copy that names a panel type
//!
//! Re-exports [`pdfcer_gui_base::text::panels`] and adds the modules that
//! reach into `crate::panels`.

pub use pdfcer_gui_base::text::panels::*;

/// The Objects panel, and the wording of every object fact.
pub mod objects;

#[cfg(test)]
mod attachments_tests;
