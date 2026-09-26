//! # `annotkind` — which family an annotation belongs to
//!
//! Kept apart from the selection model so the pick filter can name it without
//! reaching the canvas. `pdfcer_gui::canvas::selection` re-exports it.

/// Which family an annotation belongs to, and therefore **which verb may
/// restyle it**.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AnnotKind {
    /// Ordinary markup — a shape, a note, a stamp, a text markup.
    /// `EditSession::set_markup_style` is its verb.
    Markup,
    /// A **ce dimension**: a `/Line` carrying `/IT /LineDimension` and a record
    /// in the document's `/PieceInfo` sidecar.
    ///
    /// `set_dimension_style` is its verb. Handing one to `set_markup_style`
    /// regenerates it as a bare line and loses its label and witness lines,
    /// which is why the engine refuses that by name.
    CeDimension,
}
