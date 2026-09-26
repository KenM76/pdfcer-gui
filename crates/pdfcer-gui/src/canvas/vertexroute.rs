//! # `canvas::vertexroute` — **which of TWO node-edit verb families one drag
//! reaches**
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/vertexroute.md`.

use pdfcer_core::vector::Point;
use pdfcer_core::vector::snap::SnapCandidate;

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::canvas::gesture::Phase;
use crate::canvas::mapping::PageMapping;
use crate::canvas::selection::SelectionState;
use crate::canvas::target::CanvasTargetProvider;

/// **Whose node is being dragged.**
///
/// Two variants and not a boolean, for the reason `canvas::dimdrag`'s
/// `VertexIntent` gives about its three: the two reach different engine verb
/// families, and a `bool` named `is_markup` is a fact a caller may read
/// backwards while a variant is one the compiler makes them handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Subject {
    /// A **ce dimension**'s corner. Re-measures — this is the one gesture in
    /// the family that changes a printed number.
    CeDimension,
    /// A **markup shape**'s node — a `/Polygon`, `/PolyLine` or `/Line` the
    /// operator drew as a comment. Changes no number; a `/Measure` dictionary
    /// written by another program is left alone and disclosed.
    Markup,
}

/// What one frame of a node drag needs, gathered at the call site.
///
/// A struct rather than ten parameters, for `dimdrag::VertexFrame`'s reason:
/// three members are `Option`s of borrowed things and two are `Pos2`s in the
/// same space, both of which a positional list would let a caller transpose
/// silently.
pub struct Frame<'a> {
    /// The frame's context — the snap settings and the live modifiers.
    pub ctx: &'a egui::Context,
    /// Whose node. See [`Subject`].
    pub subject: Subject,
    /// Which node, sampled at the press.
    pub index: usize,
    /// Where the press landed, in canvas space — the grab point.
    pub from: egui::Pos2,
    /// Where the pointer is now, in canvas space, **before** the Shift filter.
    pub at: egui::Pos2,
    /// Draw, or commit.
    pub phase: Phase,
    /// The open document.
    pub doc: &'a OpenDoc,
    /// What is selected. **This is what names the shape.**
    pub selection: &'a SelectionState,
    /// The decomposition, for the snap query. `None` means no snapping this
    /// frame rather than an error.
    pub targets: Option<&'a dyn CanvasTargetProvider>,
    /// The frame's mapping, which owns the snap tolerance in page units.
    pub map: &'a PageMapping,
    /// Whether Shift is held **this frame**.
    pub shift: bool,
}

/// The previews one node-drag frame produced — at most one polyline is `Some`.
///
/// Two polyline fields rather than one, matching the preview slots
/// `canvas::previews` already carries and for their stated reason: the painter
/// reads each independently, and one `Vec` whose meaning depends on which
/// selection is live is a value the paint loop has to interrogate.
#[derive(Default)]
pub struct Previews {
    /// A ce dimension redrawn from the corner's new position, page space.
    pub dimension: Option<Vec<(Point, Point)>>,
    /// A markup shape redrawn from the node's new position, page space.
    pub markup: Option<Vec<(Point, Point)>>,
    /// What the node is snapping to, if anything.
    ///
    /// Shared between the two subjects deliberately, unlike the polylines:
    /// it is one screen-space glyph drawn by one painter from one candidate,
    /// and a snap marker means the same thing whichever kind of node produced
    /// it. Splitting it would give the operator two markers to learn for one
    /// inference.
    pub snap: Option<SnapCandidate>,
}

/// Route one frame of a node drag to the verb family the selection names.
///
/// See the module header for what is applied above the fork and why.
pub fn dragged(frame: Frame<'_>, actions: &mut Vec<Action>) -> Previews {
    let Frame {
        ctx,
        subject,
        index,
        from,
        at,
        phase,
        doc,
        selection,
        targets,
        map,
        shift,
    } = frame;
    // SHIFT LOCKS A NODE TO ONE AXIS, measured from the PRESS — so the grab
    // point survives (`drag-moves` D8). Applied once, above the fork, so both
    // subjects receive the same constrained position from one filter.
    let at = crate::canvas::constrain::reposition(ctx, shift, from, at);
    // ALT SUSPENDS THE SNAP, read live and asked of the same
    // `snap_query_enabled` a measure pick asks.
    let alt_held = ctx.input(|i| i.modifiers.alt);
    let mut out = Previews::default();
    match subject {
        Subject::CeDimension => {
            let dragged = crate::canvas::dimdrag::drag_vertex(
                crate::canvas::dimdrag::VertexFrame {
                    ctx,
                    index,
                    from,
                    at,
                    phase,
                    doc,
                    selection,
                    targets,
                    map,
                    alt_held,
                },
                actions,
            );
            out.dimension = dragged.segments;
            // The candidate TRAVELS to the painter rather than being
            // re-queried there, which is `measure::Resolved`'s whole reason for
            // existing: a marker resolved a second time is a second derivation,
            // and this project has already shipped one that sat away from the
            // point it described for four days.
            out.snap = dragged.snap;
        }
        Subject::Markup => {
            let dragged = crate::canvas::annotnodes::drag(
                crate::canvas::annotnodes::NodeFrame {
                    ctx,
                    index,
                    from,
                    at,
                    phase,
                    doc,
                    selection,
                    targets,
                    map,
                    alt_held,
                },
                actions,
            );
            out.markup = dragged.segments;
            out.snap = dragged.snap;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The two subjects are distinct values**, which is the whole content
    /// of this type.
    #[test]
    fn a_ce_dimension_and_a_markup_are_not_the_same_subject() {
        assert_ne!(Subject::CeDimension, Subject::Markup);
    }

    /// **A frame that routes nowhere previews nothing.**
    #[test]
    fn the_empty_route_draws_nothing() {
        let out = Previews::default();
        assert!(out.dimension.is_none());
        assert!(out.markup.is_none());
        assert!(out.snap.is_none());
    }
}
