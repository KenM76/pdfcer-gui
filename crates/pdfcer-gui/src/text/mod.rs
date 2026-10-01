//! # `text` — the operator-facing copy that still reaches into the app
//!
//! Everything in [`pdfcer_gui_base::text`] is re-exported here, so
//! `crate::text::…` names one catalog. The modules declared below name a
//! type from this crate and so cannot live in the base.

pub use pdfcer_gui_base::text::*;

pub mod menus;
pub mod settings;

/// Font-glyph coverage: *can the stack actually draw this character?*
#[cfg(test)]
pub mod glyphs;

#[cfg(test)]
mod about_tests;
