//! # `text` — the operator-facing copy that still reaches into the app
//!
//! Everything in [`pdfcer_gui_base::text`] is re-exported here, so
//! `crate::text::…` names one catalog. The modules declared below name a
//! type from `app`, `canvas` or `panels` and so cannot live in the base.

pub use pdfcer_gui_base::text::*;

pub mod menus;
pub mod panels;
pub mod settings;

/// The label and tooltip of every ribbon command. Consumed by
/// `crate::shell::commands`.
/// The four sentences the object clipboard can say when it cannot act.
pub mod clipboard;
/// Every string the Forms panel shows. Consumed by `crate::panels::forms`.
pub mod formfield;
/// Font-glyph coverage: *can the stack actually draw this character?*
#[cfg(test)]
pub mod glyphs;
/// Every word the two Security controls that **write** protection into a file
/// say — O119. The WRITE side; [`security`] is the READ side, and the two are
/// separate modules because they make opposite kinds of claim. Nothing is
/// duplicated across the seam: this module calls `security`'s wording verbatim
/// wherever one fact serves both.
pub mod protect;
pub mod redact;
pub mod textedit;
/// Every sentence the text-EDITING tool shows: the three refusals a caret can
/// meet, and the rule-4 disclosure the engine does not write for a pinned tail.
/// Consumed by `crate::canvas::textedit` and by the `CommitTextEdit` apply arm.
/// Copy for the three markup kinds that carry words. Its header carries the
/// one distinction every string in it has to preserve: a text box prints and a
/// sticky note does not.
/// Every word the TOOLS say, wherever they are said — the one-line status
/// strip, the Properties panel's armed-tool section, and the canvas refusals.
pub mod tool;

#[cfg(test)]
mod about_tests;
/// The three sentences a dragged-and-dropped file can answer with.
/// **The document tab strip, and the page drag between documents.** What a tab
/// says, and what a drag says it is about to do.
pub mod doctabs;
