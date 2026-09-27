//! # `canvas::selection` — the selection STATE, and the invariant it exists to hold
//!
//! ## The two halves of this module, and the seam between them
//!
//! [`identity`] holds what a selection **is**: [`Selection`],
//! [`SelectionLevel`], [`ClickHit`] and [`EscapeOutcome`] — four `Copy` types
//! which between them cannot name a place on the screen. That file carries the
//! *"selection is an identity, not a position"* argument in full, because it is
//! an argument about the shape of a **type** and is answered by reading four
//! field declarations.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/selection/mod.md`.

/// Clicking the things pdfcer itself put on the page — stamps, notes, shapes
/// and ce dimensions. A sibling of [`identity`], not a variant of it: an
/// annotation is addressed by a STABLE `ObjId` where page content is addressed
/// by a paint-order index, and the four ways the two differ are tabulated in
/// its header.
pub mod annot;
/// What a selection **is** — the four `Copy` types the state below accumulates,
/// none of which can hold a coordinate. The pure half; see its header for why
/// "identity, not position" is a claim about a type rather than about a method.
pub use pdfcer_gui_base::selectionidentity as identity;

pub use annot::{AnnotKind, AnnotSelection, AnnotTarget};
pub use identity::{ClickHit, EscapeOutcome, Selection, SelectionLevel};

#[cfg(test)]
use crate::canvas::target::TargetId;

pub use pdfcer_gui_base::selectionstate::SelectionState;

// The selection algebra's assertions; see its header for why the tests are a
// seam of their own.
#[cfg(test)]
mod tests;
