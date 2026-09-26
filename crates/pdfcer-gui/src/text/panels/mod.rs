//! # `text::panels` — panel copy that names a panel type
//!
//! Re-exports [`pdfcer_gui_base::text::panels`] and adds the modules that
//! reach into `crate::panels`.

pub use pdfcer_gui_base::text::panels::*;

/// **Every sentence pdfcer says about which layer a selection is on** — the
/// panel's long form and the status bar's short clause, generated from one
/// [`crate::panels::layers::highlight::Membership`] so the two surfaces cannot
/// drift. Its own module under R2 and under `DEFECTS.md` D5; the module header
/// argues the seam.
pub mod layers;
/// The Objects panel, and the wording of every object fact.
pub mod objects;
pub use layers::layer_selection_unlayered;

#[cfg(test)]
mod attachments_tests;
