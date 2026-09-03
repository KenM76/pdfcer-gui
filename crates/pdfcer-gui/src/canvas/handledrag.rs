//! # `canvas::handledrag` — **dragging a Bézier handle**, the last of Phase 1
//!
//! ## What this closes, and the row that was wrong about it
//!
//! `pdfcer`'s `gui` column ticked *"edit a Bézier handle"* `[x]`. Their sweep of
//! 2026-08-19 corrected it to `⬜ nothing`: one of six rows that were true of
//! the **old** in-repo shell and became false, untouched, when the column's
//! referent moved to this build.
//!
//! ★ **Nothing was blocking it.** `EditSession::move_handle` has existed since
//! Pass 30.1, with a `Handle` enum, a planner, a `v`/`y` re-spelling path and a
//! disclosure contract — the whole capability, documented, waiting. What was
//! missing was a way to *see* a handle and a way to *grab* one, and both are
//! this shell's.
//!
//! ## ★★ Why this is a distinct verb from moving a node
//!
//! Because the two change different things and the engine says so in the type.
//! `move_node` moves a point the curve passes **through**; `move_handle` moves
//! a point that governs the curve's **shape** and that the curve never touches.
//! A single "move a point" verb would have to infer which the operator meant
//! from what they grabbed — exactly the inference `pdfcer_core::vector::Handle`
//! exists to remove.
//!
//! ## ★ The gesture priority, and the rule behind it
//!
//! A handle sits **inside** the selection's bounding box, so `handles::grip_at`
//! answers `Grip::Move` for every press on one. Left alone, that makes handles
//! undraggable — the identical collision that made the corner *anchors*
//! undraggable until the eight scale grips were confined to the Object rung.
//!
//! Both are the same rule: **the most specific thing under the pointer wins**,
//! and specificity is depth down the selection ladder. So the press is tested
//! against handles first, then anchors, then the box.
//!
//! ## ★★ The disclosure this owes, and it is invisible by construction
//!
//! `move_handle` returns a list of sentences that is **empty unless a `v`/`y`
//! segment had to be re-spelled as `c`**. ISO 32000-1 §8.5.2.1 Table 59 gives a
//! cubic three spellings and two omit a control point by making it equal to a
//! point the segment already carries; a handle that must hold its own value
//! cannot be expressed in those, so the drag rewrites the operator.
//!
//! **The curve draws identically.** Nothing on the page changes. What changes
//! is that the original bytes are gone and dragging back does not restore them.
//! That is precisely rule 4's surviving half — *an inference the operator
//! cannot see still owes an off-canvas report* — and it is why
//! `VectorAction::MoveHandle.into()`'s apply arm forwards those sentences to the disclosure
//! channel rather than discarding them as "no error".
//!
//! ## What is deliberately NOT here
//!
//! - **Turning a line into a curve.** `move_handle` refuses with
//!   `NoHandleHere` when the neighbouring segment is straight, and the engine's
//!   own comment says why: *"turning a line into a curve is a different
//!   operation and is not inferred from a drag"*. This shell agrees by not
//!   drawing a handle there, so the refusal is unreachable from the pointer.
//! - **Symmetric / smooth handle constraints.** Dragging one handle and having
//!   its opposite mirror is a *modelling* convention (Inkscape's node types),
//!   not a PDF one — the format has no notion of a smooth node, so the shell
//!   would have to invent and store one. It would also make one gesture two
//!   engine calls and two undo entries.
//! - **A closed subpath's first-anchor incoming handle.** The closing segment
//!   of an `h`-terminated subpath has no operands in the content stream, so
//!   there is nothing for `move_handle` to rewrite. `ObjectModelProvider::
//!   node_handles` does not return it, so no control is offered for it.
//!
//! ## conventions: drag-moves
//!
//! Corpus: `ui-conventions/drag-moves.md`.
//!
//! - D1 live-preview: the dragged handle follows the pointer, and the
//!   decomposition still holds its old position — so without this the handle
//!   would sit still while the operator dragged it.
//! - D2 derived-from-commit: the preview is the position the release writes.
//! - D3 escape-cancels: the gesture machine drops it before anything is written.
//! - D4 one-undo-entry: `move_handle` is one engine command.
//! - D5 modifiers-constrain: **Shift locks the handle to its ANCHOR's axis** —
//!   not to the press's, which is what makes a clean horizontal or vertical
//!   tangent and is why [`anchor`] exists. Applied by `canvas::interact`
//!   through [`crate::canvas::constrain::reposition`]. Alt to break or restore
//!   a smooth node's symmetry — the Bézier-specific half of this row — is still
//!   absent and is recorded as a gap rather than waived.
//! - D6 snapping: **GAP** — a handle does not snap.
//! - D7 no-op-is-not-an-edit: **GAP** — a zero-travel release is not checked.
//! - D8 grab-point: this variant carries the pointer's POSITION rather than a
//!   delta, and that is deliberate rather than an oversight of D8: a Bézier
//!   handle is a small dot the operator grabs at its centre, so "the handle goes
//!   where the pointer is" and "the handle moves by the delta" differ by at most
//!   the grab slack. Stated so the difference from `dimdrag`'s vertex drag —
//!   which DOES preserve the grab, because a vertex handle sits on a shape the
//!   operator is aiming at — is a decision rather than an inconsistency.
//! - D9 disclosure: WAIVED — moving a control point changes no measured value
//!   pdfcer authored.

use egui::{Pos2, Vec2};
use pdfcer_core::vector::{Handle, Point};

use crate::app::actions::{Action, VectorAction};
use crate::canvas::gesture::Phase;
use crate::canvas::mapping::PageMapping;
use crate::canvas::selection::SelectionState;
use crate::panels::objects::provider::ObjectModelProvider;

/// How close, in screen pixels, a press must be to a handle's centre to grab
/// it.
///
/// Eight — larger than the six-pixel mark it is grabbing, matching the two
/// points of slack `handles::GRIP_GRAB_SLACK_PX` gives a resize grip and for
/// the same reason: a target that requires hitting its exact pixels is a target
/// an operator misses, and the miss here is worse than for a grip because it
/// falls through to a *move of the whole object*.
pub const GRAB_PX: f32 = 8.0;

/// The handles of every selected anchor, in canvas space.
///
/// Returned as a flat list rather than grouped per anchor, because both
/// consumers — the hit test and the painter — want to walk all of them, and the
/// anchor each belongs to travels on the tuple.
///
/// Empty at the Object and Part rungs: a handle is a property of a *selected
/// anchor*, and there is no selected anchor above the Node rung.
#[must_use]
pub fn visible(
    selection: &SelectionState,
    provider: &ObjectModelProvider,
    page: &pdfcer_core::page_tree::Page,
    page_index: usize,
) -> Vec<(usize, Handle, Pos2)> {
    let Some(entered) = selection.entered_object() else {
        return Vec::new();
    };
    if entered.page != page_index {
        return Vec::new();
    }
    let Some(subpath) = entered.subpath else {
        return Vec::new();
    };
    // ★★ **Either index space, as of 2026-09-01** — `OPERATOR_REQUESTS.md` O70.
    //
    // This resolved a page paint-order index and returned empty for anything
    // inside a form XObject, with the right reason at the time: the handles are
    // grab targets for a verb that writes the PAGE's content stream, and
    // offering one for a gesture that must then refuse is the placeholder R9
    // forbids.
    //
    // `pdfcer-core` Pass 188.0 shipped `move_handle_in_form`, and
    // `provider::geometry` answers where a leaf's controls are — so the grab
    // target now leads somewhere.
    let mut out = Vec::new();
    for node in selection.selected_nodes_on(page_index, entered.object) {
        for (side, point) in provider.node_handles_of(entered.object, subpath, node) {
            if let Some(canvas) =
                crate::viewer::pdf_space_to_canvas(egui::pos2(point.x as f32, point.y as f32), page)
            {
                out.push((node, side, canvas));
            }
        }
    }
    out
}

/// The handle under a screen-space press, if any.
///
/// # ★ Why the press is in SCREEN space and the handles are in canvas space
///
/// Because the grab radius is a **screen** distance — eight pixels is eight
/// pixels at any zoom, which is what makes the target feel the same size on an
/// A1 sheet at 0.38× and on a letter page at 1×. Comparing in canvas space
/// would make the radius shrink with the zoom, so a handle on a drawing would
/// be ungrabbable at exactly the zoom an operator uses to see the whole sheet.
#[must_use]
pub fn at(
    handles: &[(usize, Handle, Pos2)],
    map: &PageMapping,
    press: Pos2,
) -> Option<(usize, Handle)> {
    handles
        .iter()
        .map(|(node, side, canvas)| (*node, *side, map.to_screen(*canvas).distance(press)))
        .filter(|(_, _, d)| *d <= GRAB_PX)
        // The NEAREST, not the first. The incoming and outgoing handles of one
        // anchor can be within a few pixels of each other on a shallow curve,
        // and "whichever came first in the list" would make which one the
        // operator got depend on the decomposition order — a coin toss they
        // cannot see and cannot learn.
        .min_by(|a, b| a.2.total_cmp(&b.2))
        .map(|(node, side, _)| (node, side))
}

/// What a handle drag needs from the frame, gathered by the caller.
///
/// The same shape and the same reason as `canvas::resizing::Frame`: everything
/// below is a pure function of these, so the geometry is testable without a
/// document, a provider or an `egui::Context`.
pub struct Frame<'a> {
    /// The anchor whose handle is moving, object-scoped.
    pub node: usize,
    /// Arriving or leaving.
    pub handle: Handle,
    /// Where the pointer is, in canvas space.
    pub at: Pos2,
    /// Draw, or commit.
    pub phase: Phase,
    /// The page the drag is on.
    pub page_index: usize,
    /// The frame's mapping, for the canvas → screen half of the preview.
    pub map: Option<&'a PageMapping>,
    /// The page, for the canvas → PDF conversion.
    pub page: Option<&'a pdfcer_core::page_tree::Page>,
}

/// Run a handle drag, raising [`VectorAction::MoveHandle.into()`] on release.
///
/// Returns the preview to draw while the drag is in flight: the handle's
/// canvas-space position and the anchor it is tethered to, so the overlay can
/// draw the tether moving with the pointer.
///
/// # ★ Why the preview is the pointer position and not a ghost of the curve
///
/// Because drawing the curve the drag *would* produce means evaluating the
/// Bézier this shell does not own — and a preview curve that differed from what
/// the engine writes, by any amount, would be two rendering paths for one
/// shape. `BENCHMARK.md`'s standing rule about previews applies: **a preview
/// shows the cursor, the render shows the document.**
pub fn drag(
    frame: Frame<'_>,
    selection: &SelectionState,
    provider: Option<&ObjectModelProvider>,
    actions: &mut Vec<Action>,
) -> Option<Pos2> {
    if frame.phase != Phase::Complete {
        // Mid-drag: the handle follows the pointer and nothing is committed.
        return Some(frame.at);
    }
    let (Some(page), Some(_map)) = (frame.page, frame.map) else {
        return None;
    };
    let entered = selection.entered_object()?;
    // The provider is asked for only so a drag on a page whose model has gone
    // refuses rather than addressing a stale index — the same guard
    // `resizing::action` makes for the same reason.
    provider?;
    let to = crate::viewer::canvas_to_pdf_space(frame.at, page)?;
    let to = Point::new(f64::from(to.x), f64::from(to.y));
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        //
        // ★ `in_form=` since 2026-09-01 (O70): the same gesture now reaches two
        // verbs, and a trace that named only the node would leave a reader
        // unable to tell which — on a document where the two index spaces hold
        // 129,758 and 10,256 entries, that is the difference between reading a
        // correct drag and a wrong one.
        format!(
            "handle-commit node={} side={:?} in_form={} to=[{:.2} {:.2}]",
            frame.node,
            frame.handle,
            entered.object.is_leaf(),
            to.x,
            to.y
        )
    });
    // ★★ Two verbs, one gesture — `OPERATOR_REQUESTS.md` O70. The address
    // space decides which, and it is asked here rather than inside the action
    // because the two carry different index types and only this point knows
    // which one it is holding.
    let action = match (
        entered.object.leaf_index(),
        entered.object.page_object_index(),
    ) {
        (Some(leaf), _) => VectorAction::MoveHandleInForm {
            page: frame.page_index,
            leaf,
            node: frame.node,
            handle: frame.handle,
            to,
        },
        (None, Some(object)) => VectorAction::MoveHandle {
            page: frame.page_index,
            object,
            node: frame.node,
            handle: frame.handle,
            to,
        },
        // Structurally unreachable — a `TargetId` is one or the other — and
        // refused rather than defaulted, because either default would address
        // the wrong list.
        (None, None) => return None,
    };
    actions.push(action.into());
    None
}

/// **The on-curve anchor a handle belongs to, in canvas space.**
///
/// Only ever asked for by a *constrained* handle drag —
/// [`crate::canvas::constrain::toward`] needs a point to measure the
/// displacement from, and for a control point that point is its anchor rather
/// than the press. A handle's whole meaning is its direction and distance from
/// the on-curve point it serves, so locking it to the *press* row would lock a
/// quantity nobody thinks in.
///
/// ★ It is a separate call, made only when Shift is down, because
/// [`ObjectModelProvider::subpath_node_points`] allocates over every anchor of
/// the subpath. Folding it into the unconstrained path would put that
/// allocation on every frame of every handle drag for a value nothing reads —
/// the same cost `canvas::moving::drag` is at pains to avoid, where one
/// measured CAD export has 6,681 anchors.
///
/// Subpath-scoped rather than object-scoped for the same reason: the anchor is
/// known to be on the entered subpath, and asking the object costs every other
/// subpath's nodes as well.
#[must_use]
pub fn anchor(
    selection: &SelectionState,
    provider: &ObjectModelProvider,
    page: &pdfcer_core::page_tree::Page,
    node: usize,
) -> Option<Pos2> {
    let entered = selection.entered_object()?;
    let subpath = entered.subpath?;
    // `None` for a leaf — see the module's note on why the ladder stops.
    let object = entered.object.page_object_index()?;
    let point = provider
        .subpath_node_points(object, subpath)
        .into_iter()
        .find(|(index, _)| *index == node)
        .map(|(_, p)| p)?;
    crate::viewer::pdf_space_to_canvas(egui::pos2(point.x as f32, point.y as f32), page)
}

/// The tether from an anchor to one of its handles, as a screen-space pair.
///
/// A free function so the overlay can draw it without knowing how a handle is
/// found, and so this file owns the one statement of *what a handle looks
/// like*: a line from the on-curve point to the control point, with a mark at
/// the far end. That is the universal vector-editor idiom — Illustrator,
/// Inkscape, Figma and the old shell all draw it — and the reason it is
/// universal is that a control point with no tether is an unexplained dot.
#[must_use]
pub fn tether(anchor: Pos2, handle: Pos2) -> (Pos2, Vec2) {
    (anchor, handle - anchor)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map() -> PageMapping {
        PageMapping::new(
            egui::Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(600.0, 800.0)),
            (600.0, 800.0),
            1.0,
        )
    }

    #[test]
    fn a_press_on_a_handle_finds_it() {
        let hs = vec![(3, Handle::Outgoing, Pos2::new(100.0, 100.0))];
        let m = map();
        assert_eq!(
            at(&hs, &m, Pos2::new(102.0, 101.0)),
            Some((3, Handle::Outgoing))
        );
    }

    #[test]
    fn a_press_beyond_the_grab_radius_finds_nothing() {
        let hs = vec![(3, Handle::Outgoing, Pos2::new(100.0, 100.0))];
        let m = map();
        assert!(at(&hs, &m, Pos2::new(100.0 + GRAB_PX + 2.0, 100.0)).is_none());
    }

    /// ★★ **The nearest handle wins, not the first.**
    ///
    /// An anchor's two handles can be within a few pixels of each other on a
    /// shallow curve. "Whichever came first in the list" would make which one
    /// the operator got depend on the decomposition order — a coin toss they
    /// cannot see and cannot learn, and one that would show up as "sometimes it
    /// drags the wrong side".
    #[test]
    fn the_nearest_of_two_close_handles_wins() {
        let hs = vec![
            (3, Handle::Incoming, Pos2::new(100.0, 100.0)),
            (3, Handle::Outgoing, Pos2::new(104.0, 100.0)),
        ];
        let m = map();
        assert_eq!(
            at(&hs, &m, Pos2::new(105.0, 100.0)),
            Some((3, Handle::Outgoing)),
            "a press nearer the outgoing handle must not pick the incoming one \
             merely because it is listed first"
        );
        assert_eq!(
            at(&hs, &m, Pos2::new(99.0, 100.0)),
            Some((3, Handle::Incoming))
        );
    }

    /// Nothing selected, nothing to grab.
    #[test]
    fn an_empty_handle_list_grabs_nothing() {
        assert!(at(&[], &map(), Pos2::new(0.0, 0.0)).is_none());
    }

    /// A drag still in flight commits nothing and previews the pointer.
    #[test]
    fn a_drag_in_flight_commits_nothing() {
        let mut actions = Vec::new();
        let preview = drag(
            Frame {
                node: 1,
                handle: Handle::Outgoing,
                at: Pos2::new(50.0, 60.0),
                phase: Phase::InFlight,
                page_index: 0,
                map: None,
                page: None,
            },
            &SelectionState::default(),
            None,
            &mut actions,
        );
        assert_eq!(preview, Some(Pos2::new(50.0, 60.0)));
        assert!(actions.is_empty(), "nothing commits before the release");
    }

    /// The tether runs from the anchor to the handle, in that order.
    #[test]
    fn the_tether_runs_from_the_anchor() {
        let (from, v) = tether(Pos2::new(10.0, 10.0), Pos2::new(40.0, 30.0));
        assert_eq!(from, Pos2::new(10.0, 10.0));
        assert!((v.x - 30.0).abs() < 1e-6 && (v.y - 20.0).abs() < 1e-6);
    }
}
