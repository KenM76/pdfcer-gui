//! # `canvas::vertexroute` — **which of TWO node-edit verb families one drag
//! reaches**
//!
//! Split out of [`super::interact`] under **R2** on 2026-09-05, when markup
//! node editing would have taken that file past 1,500 lines for the third time.
//! It is the same seam [`super::dragroute`] draws one gesture along, and the
//! header there states the rule this one inherits:
//!
//! > One gesture — press on the thing, drag it — reaches different engine verbs,
//! > and **which one is decided entirely by what is selected**.
//!
//! ## The two families
//!
//! | selection | verbs | identity | preflight |
//! |---|---|---|---|
//! | a **ce dimension** | `move_dimension_vertex`, `insert_dimension_vertex`, `remove_dimension_vertex` | a sidecar `DimensionId` | `vertex_edit_preview` (count edits only) |
//! | a **markup shape** | `reshape_annotation` and its three wrappers | a stable `ObjId` | `reshape_annotation_preview` (**every** edit) |
//!
//! ★ R8b rule 15 is enforced by the type here rather than by care: a **ce
//! dimension** is the thing pdfcer authors and measures with, a **markup
//! shape** is a comment somebody drew, and [`Subject`] makes the two a `match`
//! the compiler checks. **pdf dimensions** — CAD page content — are neither and
//! are nowhere near this module; a content path's anchors are
//! [`super::handledrag`]'s.
//!
//! ## ★★ Why they are two variants rather than one call with a flag
//!
//! Because the one thing that must never happen on this canvas is a gesture
//! aimed at the wrong verb, and a ce dimension arriving at
//! `reshape_annotation` is precisely that: it is a `/Line` with
//! `/IT /LineDimension`, so it passes every *"is this markup?"* test, and the
//! engine refuses it by name (`EditError::AnnotationIsCeDimension`) as the
//! **backstop** rather than as the mechanism. `canvas::selection::AnnotKind`
//! exists to make the routing decidable before the engine is asked, and this
//! module is where that decision is spent.
//!
//! ## ★ What is applied here, above both branches
//!
//! **Shift**, and **Alt**, once each:
//!
//! * `ui-conventions/drag-moves.md` D5 — Shift locks the node to one axis,
//!   through [`super::constrain::reposition`], which filters the displacement
//!   **from the press** so the grab point survives (D8). Two copies of *"what
//!   does Shift mean"* is how two node drags in one program come to disagree
//!   about it.
//! * D6 — Alt suspends the snap, read live, and asked of the same
//!   `snap_query_enabled` a measure pick asks. It is what makes a generous
//!   catch radius affordable: the offer is refusable, so it can afford to be
//!   eager.
//!
//! ## What this module does NOT decide
//!
//! * **Whether the press is a node drag at all.** `canvas::pressing` resolves
//!   which node, and `canvas::gesture::press_kind` decides what the press
//!   means.
//! * **Whether an edit is allowed.** Each branch asks its own engine preflight,
//!   and neither restates the engine's rules.
//! * **What the preview looks like.** Each branch derives it from the same
//!   geometry its release commits.

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
/// ★ Two polyline fields rather than one, matching the preview slots
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
    /// ★ Shared between the two subjects deliberately, unlike the polylines:
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
    // ★★ SHIFT LOCKS A NODE TO ONE AXIS, measured from the PRESS — so the grab
    // point survives (`drag-moves` D8). Applied once, above the fork, so both
    // subjects receive the same constrained position from one filter.
    let at = crate::canvas::constrain::reposition(ctx, shift, from, at);
    // ★★ ALT SUSPENDS THE SNAP, read live and asked of the same
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
            // ★ The candidate TRAVELS to the painter rather than being
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

    /// ★ **The two subjects are distinct values**, which is the whole content
    /// of this type.
    ///
    /// A tripwire rather than a tautology: the day somebody replaces [`Subject`]
    /// with a `bool` to save a line, this is what stops the replacement being
    /// invisible. `canvas::gesture::DragKind`'s own note explains what a shared
    /// variant with a discriminator inside it costs — a gesture aimed at the
    /// wrong verb, which never looks broken from a chair.
    #[test]
    fn a_ce_dimension_and_a_markup_are_not_the_same_subject() {
        assert_ne!(Subject::CeDimension, Subject::Markup);
    }

    /// **A frame that routes nowhere previews nothing.**
    ///
    /// [`Previews::default`] is what both branches return when the selection
    /// does not name a shape they own, and a default that carried a `Some`
    /// would put a stale polyline on the canvas for every drag that reached no
    /// verb.
    #[test]
    fn the_empty_route_draws_nothing() {
        let out = Previews::default();
        assert!(out.dimension.is_none());
        assert!(out.markup.is_none());
        assert!(out.snap.is_none());
    }
}
