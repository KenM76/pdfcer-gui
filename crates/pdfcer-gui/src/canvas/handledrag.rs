//! # `canvas::handledrag` — **dragging a Bézier handle**, the last of Phase 1
//!
//! ## What this closes, and the row that was wrong about it
//!
//! `pdfcer`'s `gui` column ticked *"edit a Bézier handle"* `[x]`. Their sweep of
//! 2026-08-19 corrected it to `⬜ nothing`: one of six rows that were true of
//! the **old** in-repo shell and became false, untouched, when the column's
//! referent moved to this build.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/handledrag.md`.
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
pub const GRAB_PX: f32 = 8.0;

/// The handles of every selected anchor, in canvas space.
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
        format!(
            "handle-commit node={} side={:?} in_form={} to=[{:.2} {:.2}]",
            frame.node,
            frame.handle,
            entered.object.is_leaf(),
            to.x,
            to.y
        )
    });
    // Two verbs, one gesture — `OPERATOR_REQUESTS.md` O70. The address
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

    /// **The nearest handle wins, not the first.**
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

#[cfg(test)]
mod o69_tolerance_tests {
    use super::*;

    /// **An anchor is easier to hit than an object, and exactly as easy
    /// as its own control point** — `OPERATOR_REQUESTS.md` O69.
    #[test]
    fn an_anchor_is_caught_more_easily_than_an_object_and_as_easily_as_a_handle() {
        // Bound through locals rather than compared as literals, so clippy
        // reads them as values rather than as a constant assertion. The
        // property is about the RELATIONSHIP between two constants, which is
        // exactly what a `const` block would hide from a reader looking for
        // why one of them was changed.
        let node = crate::canvas::mapping::NODE_SCREEN_TOLERANCE_PX;
        let object = crate::canvas::mapping::SELECT_SCREEN_TOLERANCE_PX;
        let handle = GRAB_PX;
        assert!(
            node > object,
            "an anchor is a small target and must be more forgiving than a whole object"
        );
        assert!(
            (node - handle).abs() < f32::EPSILON,
            "an anchor must be no harder to hit than the control point hanging off it — \
             they were 6 and 8"
        );
    }
}
