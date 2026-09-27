//! # `text` — the operator-facing copy that still reaches into the app
//!
//! Everything in [`pdfcer_gui_base::text`] is re-exported here, so
//! `crate::text::…` names one catalog. The modules declared below name a
//! type from `app`, `canvas` or `panels` and so cannot live in the base.

pub use pdfcer_gui_base::text::*;

pub mod menus;
pub mod panels;
pub mod settings;

/// The four sentences the object clipboard can say when it cannot act.
pub mod clipboard;
/// Font-glyph coverage: *can the stack actually draw this character?*
#[cfg(test)]
pub mod glyphs;
/// Every sentence the text-editing tool shows: the refusals a caret can meet,
/// and the disclosure the engine does not write for a pinned tail.
pub mod textedit;

#[cfg(test)]
mod about_tests;
