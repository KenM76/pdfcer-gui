//! # `canvas::annotnodes::stroke` — **whole strokes of a freehand mark**
//!
//! With the Points tool armed, an **Alt+drag** on any point of an `/Ink`
//! moves that point's whole stroke (`InkEdit::MoveStroke`), and the markup
//! menu's *Delete this stroke*, right-clicked on a stroke, removes it
//! (`InkEdit::RemoveStroke`). Each is one undo entry through
//! `inknodes::reshape_ink`, preflighted by `reshape_ink_preview` exactly as a
//! point edit is, so a refused edit is never previewed. The engine refuses the
//! mark's last stroke; the row is then absent and the mark's own Delete is the
//! route.

use pdfcer_core::edit::{EditError, EditSession, InkEdit};
use pdfcer_core::object::ObjId;
use pdfcer_core::vector::Point;
use pdfcer_core::vector::snap::SnapCandidate;

use super::menu::{NodePick, RowState};
use super::{Geometry, NodeDrag, Plan, TRACE_DECLINED, refusal_for};
use crate::app::actions::Action;
use crate::app::actions::annot::AnnotAction;
use crate::canvas::gesture::Phase;

/// `markup-stroke-move id=… stroke=… dx=… dy=… snap=…` — a whole-stroke move
/// was released and raised.
pub const TRACE_MOVE_STROKE: &str = "markup-stroke-move"; // ui-text-exempt: diagnostic trace name

/// Whether this frame's drag moves a whole stroke: an `/Ink`, the Points
/// tool, Alt held without Ctrl. Not Shift: Shift locks every point drag to an
/// axis (`constrain::reposition`). Alt also turns snapping off, so the stroke
/// moves by exactly the pointer's travel.
pub(super) fn whole(ctx: &egui::Context, shape: &Geometry) -> bool {
    shape.strokes.is_some()
        && crate::canvas::tool::active(ctx).is_node()
        && ctx.input(|i| i.modifiers.alt && !i.modifiers.command)
}

/// The stroke a right-click pick on an `/Ink` lands in.
pub(super) fn picked(shape: &Geometry, pick: NodePick) -> Option<usize> {
    let flat = match pick {
        NodePick::Node(index) => index,
        NodePick::Segment { after, .. } => after,
        NodePick::Elsewhere => return None,
    };
    Some(shape.strokes.as_ref()?.address(flat)?.0)
}

/// *Delete this stroke*'s state for `pick`: live when the engine would remove
/// it, absent otherwise — on a shape that is not ink, off the shape, and on a
/// mark's only stroke, which no waiting changes.
pub(super) fn removal(session: &EditSession, shape: &Geometry, pick: NodePick) -> RowState {
    let Some(stroke) = picked(shape, pick) else {
        return RowState::Absent;
    };
    match session.reshape_ink_preview(shape.id, &InkEdit::RemoveStroke { stroke }) {
        Ok(_) => RowState::Live,
        Err(_) => RowState::Absent,
    }
}

/// The action *Delete this stroke* raises for `pick`, re-asked of the engine.
pub(super) fn remove(session: &EditSession, shape: &Geometry, pick: NodePick) -> Option<Action> {
    let stroke = picked(shape, pick)?;
    let edit = InkEdit::RemoveStroke { stroke };
    session.reshape_ink_preview(shape.id, &edit).ok()?;
    Some(Action::Annot(Plan::Ink(edit).action(shape.id)))
}

/// Everything one frame of a whole-stroke move needs.
pub(super) struct Move<'a> {
    /// The read side of the document, for the preflight.
    pub session: &'a EditSession,
    /// The mark as it stands.
    pub shape: &'a Geometry,
    /// The grabbed point, flat index.
    pub index: usize,
    /// Where that point is now, page space.
    pub old: Point,
    /// Where it is dragged to, page space, after snapping.
    pub target: Point,
    /// Draw, or commit.
    pub phase: Phase,
    /// What the grabbed point is snapping to.
    pub snap: Option<SnapCandidate>,
    /// Where the release's one action goes.
    pub actions: &'a mut Vec<Action>,
}

/// One frame of a whole-stroke move: the grabbed point goes to `target` and
/// every other point of its stroke by the same delta.
pub(super) fn moved(m: Move<'_>) -> NodeDrag {
    let Some(table) = m.shape.strokes.as_ref() else {
        return NodeDrag::default();
    };
    let Some((stroke, _)) = table.address(m.index) else {
        return NodeDrag::default();
    };
    let (dx, dy) = (m.target.x - m.old.x, m.target.y - m.old.y);
    let edit = InkEdit::MoveStroke { stroke, dx, dy };
    let id = m.shape.id;
    if let Err(why) = m.session.reshape_ink_preview(id, &edit) {
        if m.phase == Phase::Complete {
            declined(m.actions, id, stroke, &why);
            return NodeDrag::default();
        }
        return NodeDrag {
            segments: Some(m.shape.segments()),
            snap: None,
        };
    }
    if m.phase == Phase::Complete {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI.
            format!(
                "{TRACE_MOVE_STROKE} id={} stroke={stroke} dx={dx:.2} dy={dy:.2} snap={}",
                id.num,
                m.snap
                    .map_or_else(|| "none".to_owned(), |c| format!("{:?}", c.kind))
            )
        });
        m.actions.push(Action::Annot(Plan::Ink(edit).action(id)));
        return NodeDrag::default();
    }
    let mut after = m.shape.clone();
    for (i, p) in after.points.iter_mut().enumerate() {
        if table.address(i).is_some_and(|(s, _)| s == stroke) {
            *p = Point::new(p.x + dx, p.y + dy);
        }
    }
    NodeDrag {
        segments: Some(after.segments()),
        snap: m.snap,
    }
}

fn declined(actions: &mut Vec<Action>, id: ObjId, stroke: usize, why: &EditError) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI.
        format!(
            "{TRACE_DECLINED} id={} stroke={stroke} family=ink-stroke reason={why}",
            id.num
        )
    });
    actions.push(Action::Annot(AnnotAction::DeclineNodeEdit {
        why: refusal_for(why),
    }));
}
