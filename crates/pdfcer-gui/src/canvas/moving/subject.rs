//! Which verb a move reaches ([`eligible`]) and the one [`Action`] a completed
//! move becomes ([`action`]). `eligible` is a pure function of the selection
//! and a [`MoveContext`], so the ghost and the commit read the same answer and
//! its tests need no decomposition.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/moving/subject.md`.

use pdfcer_core::vector::Point;

use super::{MoveContext, MoveSubject, PageDelta, Refusal};
use crate::app::actions::{Action, VectorAction};
use crate::canvas::selection::{Selection, SelectionLevel, SelectionState};
use crate::panels::objects::provider::PartKind;

/// Which verb a move drag on this selection would reach, or why it reaches
/// none. A zero-travel drag is still eligible; [`action`] declines it.
pub fn eligible(
    selection: &SelectionState,
    page: usize,
    ctx: MoveContext,
) -> Result<MoveSubject, Refusal> {
    match selection.level() {
        SelectionLevel::Object => object_rung(selection, page, ctx),
        SelectionLevel::Part => {
            let entry = entered_entry(selection, page)?;
            match entry.object.leaf_index() {
                Some(leaf) => part_in_form(selection, page, &entry, leaf, &ctx),
                None => part_on_page(selection, page, &entry, &ctx),
            }
        }
        SelectionLevel::Node => node_rung(selection, page, &ctx),
    }
}

/// The Object rung. The operand list is the clean, ascending, de-duplicated
/// one Delete uses: `move_objects` resolves every index before planning, so
/// one stale entry would refuse the batch.
fn object_rung(
    selection: &SelectionState,
    page: usize,
    ctx: MoveContext,
) -> Result<MoveSubject, Refusal> {
    let objects = selection.object_indices_on(page);
    if objects.is_empty() {
        // `object_indices_on` drops everything drawn inside a placed drawing,
        // so an empty list can still be a selection of parts.
        let leaves = selection.leaf_indices_on(page);
        return if leaves.is_empty() {
            Err(Refusal::NothingSelected)
        } else {
            Ok(MoveSubject::LeavesInForm { page, leaves })
        };
    }
    // Parts of placed drawings selected beside page objects: every one moves,
    // through the per-item commit.
    if !ctx.beside.is_empty() {
        return Ok(MoveSubject::Mixed {
            page,
            objects,
            leaves: ctx.beside,
        });
    }
    // A fork, not one verb for both: `move_objects` rewrites coordinates in
    // place and adds nothing, while `transform_objects` adds `q cm Q` per object
    // per gesture, which accumulates in a file that is nudged often. Only a
    // selection holding something without coordinate operands takes the wrap.
    match ctx.non_path {
        None => Ok(MoveSubject::Objects { page, objects }),
        Some(_) => Ok(MoveSubject::Transform { page, objects }),
    }
}

/// The Part rung inside a placed drawing.
fn part_in_form(
    selection: &SelectionState,
    page: usize,
    entry: &Selection,
    leaf: usize,
    ctx: &MoveContext,
) -> Result<MoveSubject, Refusal> {
    let subpath = entry.subpath.ok_or(Refusal::NoPartEntered)?;
    let lines = selection.selected_parts_on(page, entry.object);
    match ctx.part_kind {
        Some(PartKind::Subpath) => Ok(MoveSubject::SubpathInForm {
            page,
            leaf,
            subpath,
        }),
        Some(PartKind::TextLine) => match ctx.run_move {
            Some(block) => Err(Refusal::TextRunCannotMove(block)),
            None if lines.len() > 1 => Ok(MoveSubject::TextLinesInForm { page, leaf, lines }),
            None => Ok(MoveSubject::TextLineInForm {
                page,
                leaf,
                line: subpath,
            }),
        },
        None => Err(Refusal::InsideForm),
    }
}

/// The Part rung on a page object. Every selected text chunk moves, not only
/// the entered one; a path's subpaths have no plural verb, so a multi-subpath
/// selection moves the entered one. `ctx.run_move` is the engine's own
/// pre-check (`text_run_move_refusal`), so this cannot promise a move the
/// planner refuses.
fn part_on_page(
    selection: &SelectionState,
    page: usize,
    entry: &Selection,
    ctx: &MoveContext,
) -> Result<MoveSubject, Refusal> {
    let subpath = entry.subpath.ok_or(Refusal::NoPartEntered)?;
    let lines = selection.selected_parts_on(page, entry.object);
    let object = entry
        .object
        .page_object_index()
        .ok_or(Refusal::UnaddressableObject)?;
    match ctx.part_kind {
        Some(PartKind::Subpath) => Ok(MoveSubject::Subpath {
            page,
            object,
            subpath,
        }),
        Some(PartKind::TextLine) => match ctx.run_move {
            Some(block) => Err(Refusal::TextRunCannotMove(block)),
            None if lines.len() > 1 => Ok(MoveSubject::TextLines {
                page,
                object,
                lines,
            }),
            None => Ok(MoveSubject::TextLine {
                page,
                object,
                line: subpath,
            }),
        },
        None => Err(Refusal::NotAPath(object)),
    }
}

/// The Node rung: every selected anchor on the entered object, not only the
/// entered one.
fn node_rung(
    selection: &SelectionState,
    page: usize,
    ctx: &MoveContext,
) -> Result<MoveSubject, Refusal> {
    let entry = entered_entry(selection, page)?;
    let node = entry.node.ok_or(Refusal::NoNodeEntered)?;
    let nodes = selection.selected_nodes_on(page, entry.object);
    if let Some(leaf) = entry.object.leaf_index() {
        return match ctx.part_kind {
            Some(PartKind::Subpath) if nodes.len() > 1 => {
                Ok(MoveSubject::NodesInForm { page, leaf, nodes })
            }
            Some(PartKind::Subpath) => Ok(MoveSubject::NodeInForm { page, leaf, node }),
            Some(other) => Err(Refusal::NoVerbForPart(other)),
            None => Err(Refusal::InsideForm),
        };
    }
    let object = entry
        .object
        .page_object_index()
        .ok_or(Refusal::UnaddressableObject)?;
    match ctx.part_kind {
        Some(PartKind::Subpath) if nodes.len() > 1 => Ok(MoveSubject::Nodes {
            page,
            object,
            nodes,
        }),
        Some(PartKind::Subpath) => Ok(MoveSubject::Node { page, object, node }),
        Some(other) => Err(Refusal::NoVerbForPart(other)),
        None => Err(Refusal::NotAPath(object)),
    }
}

/// The entered entry of a deeper rung, whichever index space it names.
fn entered_entry(selection: &SelectionState, page: usize) -> Result<Selection, Refusal> {
    selection
        .entered_object()
        .filter(|e| e.page == page)
        .ok_or(Refusal::NothingSelected)
}

/// The ONE action a completed move becomes, in page space. `node_at` is the
/// entered anchor's position for a single-node move; `points` every anchor of
/// the entered object for a multi-node one.
pub fn action(
    subject: MoveSubject,
    delta: PageDelta,
    node_at: Option<Point>,
    points: &[(usize, Point)],
) -> Result<Action, Refusal> {
    if !delta.is_travel() {
        return Err(Refusal::NoTravel);
    }
    let PageDelta { dx, dy } = delta;
    let moved = |from: Point| Point::new(from.x + dx, from.y + dy);
    let raised: VectorAction = match subject {
        MoveSubject::Transform { page, objects } => VectorAction::TransformObjects {
            page,
            objects,
            matrix: pdfcer_core::vector::Matrix::translate(dx, dy),
        },
        MoveSubject::Objects { page, objects } => VectorAction::MoveSelection {
            page,
            objects,
            dx,
            dy,
        },
        MoveSubject::Mixed {
            page,
            objects,
            leaves,
        } => VectorAction::MoveMixed {
            page,
            objects: objects.into_iter().map(|i| (i, dx, dy)).collect(),
            leaves: leaves
                .into_iter()
                .map(|group| group.into_iter().map(|i| (i, dx, dy)).collect())
                .collect(),
            widgets: Vec::new(),
            annots: Vec::new(),
            notes: Vec::new(),
            // ui-text-exempt: diagnostic gesture name, never displayed
            gesture: "drag",
        },
        MoveSubject::LeavesInForm { page, leaves } => VectorAction::MoveLeavesInForm {
            page,
            leaves,
            dx,
            dy,
        },
        MoveSubject::NodeInForm { page, leaf, node } => VectorAction::MoveNodeInForm {
            page,
            leaf,
            node,
            to: moved(node_at.ok_or(Refusal::NodeNotFound(node))?),
        },
        MoveSubject::NodesInForm { page, leaf, nodes } => VectorAction::MoveNodesInForm {
            page,
            leaf,
            moves: nodes_moved(nodes, points, moved)?,
        },
        MoveSubject::Node { page, object, node } => VectorAction::MoveNode {
            page,
            object,
            node,
            to: moved(node_at.ok_or(Refusal::NodeNotFound(node))?),
        },
        MoveSubject::Nodes {
            page,
            object,
            nodes,
        } => VectorAction::MoveNodes {
            page,
            object,
            moves: nodes_moved(nodes, points, moved)?,
        },
        other => part_action(other, dx, dy),
    };
    Ok(raised.into())
}

/// The subpath and text-line subjects, each one verb with a uniform delta.
fn part_action(subject: MoveSubject, dx: f64, dy: f64) -> VectorAction {
    match subject {
        MoveSubject::SubpathInForm {
            page,
            leaf,
            subpath,
        } => VectorAction::MoveSubpathInForm {
            page,
            leaf,
            subpath,
            dx,
            dy,
        },
        MoveSubject::Subpath {
            page,
            object,
            subpath,
        } => VectorAction::MoveSubpath {
            page,
            object,
            subpath,
            dx,
            dy,
        },
        MoveSubject::TextLine { page, object, line } => VectorAction::MoveTextLine {
            page,
            object,
            line,
            dx,
            dy,
        },
        MoveSubject::TextLineInForm { page, leaf, line } => VectorAction::MoveTextLineInForm {
            page,
            leaf,
            line,
            dx,
            dy,
        },
        MoveSubject::TextLines {
            page,
            object,
            lines,
        } => VectorAction::MoveTextLines {
            page,
            object,
            lines,
            dx,
            dy,
        },
        MoveSubject::TextLinesInForm { page, leaf, lines } => VectorAction::MoveTextLinesInForm {
            page,
            leaf,
            lines,
            dx,
            dy,
        },
        whole => unreachable!("`action` answers {whole:?} itself"), // ui-text-exempt: a panic message, never displayed
    }
}

/// Each of `nodes` at its new position, read from `points`; a node missing
/// from them refuses the whole move rather than moving the rest.
fn nodes_moved(
    nodes: Vec<usize>,
    points: &[(usize, Point)],
    moved: impl Fn(Point) -> Point,
) -> Result<Vec<(usize, Point)>, Refusal> {
    nodes
        .into_iter()
        .map(|node| {
            points
                .iter()
                .find_map(|(i, p)| (*i == node).then_some(*p))
                .map(|from| (node, moved(from)))
                .ok_or(Refusal::NodeNotFound(node))
        })
        .collect()
}
