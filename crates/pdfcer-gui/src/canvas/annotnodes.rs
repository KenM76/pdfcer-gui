//! # `canvas::annotnodes` — **the nodes of a markup shape, and moving them**
//!
//! ## The operator's report, verbatim
//!
//! > *"I also can't edit or delete nodes of a markup shape once it is drawn."*
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/annotnodes.md`.
//!
//! ## conventions: drag-moves
//!
//! Corpus `ui-conventions/drag-moves.md`, answered row by row because the
//! unanswered ones are the ones the operator finds.
//!
//! - **D1 live-preview** — the shape follows the pointer from the first frame,
//!   drawn from [`preview_of`], which is the same point list the release
//!   commits. `from` and `at` arrive **already in canvas space** and are not
//!   converted again; that double hop is the defect the operator reported on
//!   2026-08-20 (*"moves at a different speed than my mouse movements"*) and
//!   `dimdrag::inner` carries the post-mortem.
//! - **D2 derived-from-commit** — [`edited`] returns the point list, and both
//!   the preview and the action are built from that one `Vec`.
//! - **D3 escape-cancels** — WAIVED, as for every drag here: the gesture
//!   machine owns Escape and drops the drag before this module is reached.
//!   Nothing is written until `Phase::Complete`.
//! - **D4 one-undo-entry** — `reshape_annotation` is one `CommandKind`, two
//!   objects (the dictionary and its `/N` stream), and `reshape_ink` is one
//!   `CommandKind::ReshapeInk` the same way. One gesture pushes exactly one
//!   action, so one gesture is one `Ctrl+Z`.
//! - **D5 modifiers-constrain** — Shift locks the node to one axis, applied by
//!   [`crate::canvas::vertexroute`] through
//!   [`crate::canvas::constrain::reposition`], which filters the displacement
//!   from the press so the grab point survives (D8).
//! - **D6 snapping** — a node drag snaps, through the same
//!   [`crate::canvas::measure::snap_point`] query, the same tolerance and the
//!   same operator settings a measure pick uses. Alt suspends it. One function,
//!   not two, is what stops a marker sitting away from the point it describes.
//! - **D7 no-op-is-not-an-edit** — **GAP**, inherited deliberately: a
//!   zero-travel release still raises the action, exactly as an annotation move
//!   does, on the engine's own argument that *"a drag that returns to its start
//!   should not make you special-case your own arithmetic"*.
//! - **D8 grab-point** — the node moves by the pointer's **delta**, so whatever
//!   part of the handle was grabbed stays under the finger. A snap overrides
//!   it, for `dimdrag`'s stated reason: a corner three pixels off the thing it
//!   snapped to is the worst of the three outcomes.
//! - **D9 disclosure** — a reshape can drop properties the regenerated
//!   appearance does not reproduce, and can leave a `/Measure` dictionary
//!   stating a distance that is no longer true; an ink reshape can replace
//!   another producer's smoothed stroke with pdfcer's straight segments. All
//!   three are disclosed off-canvas by `app::actions::annots`; see
//!   [`crate::text::markup::measure_stale`] and
//!   [`crate::text::markup::ink_redrawn_straight`].

use pdfcer_core::edit::{EditError, EditSession, InkEdit, VertexEdit};
use pdfcer_core::object::ObjId;
use pdfcer_core::vector::Point;
use pdfcer_core::vector::snap::SnapCandidate;

use crate::app::actions::Action;
use crate::app::actions::annot::AnnotAction;
use crate::app::state::OpenDoc;
use crate::canvas::dimdrag::VertexIntent;
use crate::canvas::gesture::Phase;
use crate::canvas::mapping::PageMapping;
use crate::canvas::selection::{AnnotKind, SelectionState};

/// The trace region each node anchor is published under, suffixed with its
/// index — `canvas.markup-node.0`, `.1`, …
pub const NODE_REGION: &str = "canvas.markup-node"; // ui-text-exempt: trace region name

/// `markup-node-move id=… index=… address=… family=… nodes=… x=… y=… snap=…` —
/// the shell's own report that a node **move** gesture was understood.
pub const TRACE_MOVE: &str = "markup-node-move"; // ui-text-exempt: diagnostic trace name

/// `markup-node-insert id=… index=… address=… family=… nodes=… x=… y=… snap=…`
pub const TRACE_INSERT: &str = "markup-node-insert"; // ui-text-exempt: diagnostic trace name

/// `markup-node-remove id=… index=… address=… family=… nodes=… x=… y=… snap=…`
pub const TRACE_REMOVE: &str = "markup-node-remove"; // ui-text-exempt: diagnostic trace name

/// `markup-node-declined id=… index=… address=… family=… intent=… reason=…` —
/// the preflight said no on the frame the operator let go. `reason=` is the
/// engine's own sentence, verbatim.
pub const TRACE_DECLINED: &str = "markup-node-declined"; // ui-text-exempt: diagnostic trace name

/// `markup-nodes-unavailable id=… subtype=… reason=…` — the sentence
/// [`explain_unreshapable`] recorded, in the machine's own words beside the
/// operator's.
pub const TRACE_UNAVAILABLE: &str = "markup-nodes-unavailable"; // ui-text-exempt: diagnostic trace name

/// How much slack a press gets around a node anchor, in points.
const NODE_GRAB_SLACK_PT: f32 = 3.0;

/// **The nodes of one selected markup shape**, and how the engine addresses
/// them.
#[derive(Debug, Clone, PartialEq)]
pub struct Geometry {
    /// The annotation, by stable object id.
    pub id: ObjId,
    /// Every node, in **page space** (PDF user space, y-up), in the order the
    /// file holds them. For an `/Ink` this is every point of every stroke,
    /// stroke after stroke.
    pub points: Vec<Point>,
    /// Whether the shape closes back to its first node — a `/Polygon` does,
    /// nothing else does. Always `false` when `strokes` is `Some`: an ink
    /// stroke is open by definition (`InkEdit::InsertPoint`'s doc).
    pub closed: bool,
    /// The stroke boundaries of an `/Ink`, or `None` for a single-list shape.
    pub strokes: Option<ink::StrokeTable>,
}

/// **The geometry of the selected markup shape**, if it is one with nodes.
#[must_use]
pub fn geometry(doc: &OpenDoc, selection: &SelectionState) -> Option<Geometry> {
    let annot = selection.annot()?;
    if annot.target.kind != AnnotKind::Markup || annot.target.locked {
        return None;
    }
    let id = annot.target.id;
    let page = doc.pages.get(annot.target.page)?;
    let found = pdfcer_core::annot::page_annotations(&doc.session.graph(), page.id)
        .into_iter()
        .find(|a| a.id == Some(id))?;
    let to_points = |pairs: &[(f64, f64)]| -> Vec<Point> {
        pairs.iter().map(|&(x, y)| Point::new(x, y)).collect()
    };
    let single = |points: Vec<Point>, closed: bool| Geometry {
        id,
        points,
        closed,
        strokes: None,
    };
    // Matched on the `/Subtype` bytes the read model carries rather than on
    // "does it have a `/Vertices` key". The keys are read subtype-agnostically
    // by `page_annotations` — a malformed `/Square` carrying a stray
    // `/Vertices` array would answer the key test and be refused by every verb.
    // The subtype is what the engine's own matrix is keyed on, so it is what
    // this is keyed on.
    match found.subtype.as_slice() {
        b"Polygon" => Some(single(to_points(found.vertices.as_ref()?), true)),
        b"PolyLine" => Some(single(to_points(found.vertices.as_ref()?), false)),
        // A `/Line`'s two ends are `/L`, not `/Vertices`, and the engine
        // addresses them as index 0 and index 1 of the same `VertexEdit::Move`.
        // So the shell's list is `[start, end]` and the indices line up by
        // construction rather than by a mapping that could be got backwards.
        b"Line" => {
            let [start, end] = found.line?;
            Some(single(
                vec![Point::new(start.0, start.1), Point::new(end.0, end.1)],
                false,
            ))
        }
        // `/Ink` — `Pass 278.0`. Every point of every stroke, flattened in
        // file order, with the table that remembers where each stroke begins.
        // `ink_list` is `None` for an `/InkList` the engine could not read as
        // an array, and that is the R9 answer: no anchors, and the sentence
        // `explain_unreshapable` raises names the mark's kind.
        b"Ink" => {
            let (points, strokes) = ink::StrokeTable::flatten(found.ink_list.as_ref()?);
            Some(Geometry {
                id,
                points,
                closed: false,
                strokes: Some(strokes),
            })
        }
        _ => None,
    }
}

/// **Every node of the selected markup shape, in CANVAS space**, in index
/// order.
///
/// Empty for every other selection, which is what lets both the painter and the
/// hit test be one call with no branch of their own.
#[must_use]
pub fn nodes(doc: &OpenDoc, selection: &SelectionState) -> Vec<egui::Pos2> {
    let Some(shape) = geometry(doc, selection) else {
        return Vec::new();
    };
    let Some(annot) = selection.annot() else {
        return Vec::new();
    };
    let Some(page) = doc.pages.get(annot.target.page) else {
        return Vec::new();
    };
    shape
        .points
        .iter()
        .filter_map(|p| {
            #[allow(clippy::cast_possible_truncation)]
            let as_pos = egui::Pos2::new(p.x as f32, p.y as f32);
            crate::viewer::pdf_space_to_canvas(as_pos, page)
        })
        .collect()
}

/// **Which node a press at `screen` landed on**, if any.
#[must_use]
pub fn node_at(
    doc: &OpenDoc,
    map: &PageMapping,
    selection: &SelectionState,
    screen: egui::Pos2,
) -> Option<usize> {
    let tolerance = crate::canvas::dimdrag::VERTEX_HANDLE_PT / 2.0 + NODE_GRAB_SLACK_PT;
    nodes(doc, selection)
        .into_iter()
        .enumerate()
        .rfind(|(_, canvas)| map.to_screen(*canvas).distance(screen) <= tolerance)
        .map(|(index, _)| index)
}

/// The page-space segments a shape with these nodes would be drawn as.
#[must_use]
pub fn preview_of(points: &[Point], closed: bool) -> Vec<(Point, Point)> {
    let mut out: Vec<(Point, Point)> = points.windows(2).map(|w| (w[0], w[1])).collect();
    if closed
        && points.len() >= 3
        && let (Some(first), Some(last)) = (points.first(), points.last())
    {
        out.push((*last, *first));
    }
    out
}

impl Geometry {
    /// **The index pairs a preview joins** — the one statement of which nodes
    /// are connected, read by the preview painter and by the right-click
    /// segment pick so the edge a menu offers *"Add a point here"* on is always
    /// an edge the preview draws.
    #[must_use]
    pub fn segment_pairs(&self) -> Vec<(usize, usize)> {
        if let Some(table) = &self.strokes {
            return table.segment_pairs();
        }
        let n = self.points.len();
        let mut out: Vec<(usize, usize)> = (1..n).map(|i| (i - 1, i)).collect();
        if self.closed && n >= 3 {
            out.push((n - 1, 0));
        }
        out
    }

    /// The page-space segments this shape would be drawn as — the preview.
    #[must_use]
    pub fn segments(&self) -> Vec<(Point, Point)> {
        self.segment_pairs()
            .into_iter()
            .filter_map(|(a, b)| Some((*self.points.get(a)?, *self.points.get(b)?)))
            .collect()
    }

    /// The shape this edit would produce.
    #[must_use]
    fn edited(&self, intent: VertexIntent, index: usize, target: Point) -> Option<Self> {
        let mut out = self.points.clone();
        match intent {
            VertexIntent::Move => *out.get_mut(index)? = target,
            // `index + 1`, matching the engine: `insert_annotation_vertex(after,
            // at)` puts the new node at `after + 1`, and there is deliberately no
            // "insert before the first" spelling — the engine refuses `after >=
            // count` and says to rotate the polygon's start instead, which is what
            // every other tool does as well. `InkEdit::InsertPoint` spells it the
            // same way, per stroke.
            VertexIntent::Insert => {
                if index >= out.len() {
                    return None;
                }
                out.insert(index + 1, target);
            }
            VertexIntent::Remove => {
                if index >= out.len() {
                    return None;
                }
                out.remove(index);
            }
        }
        let strokes = match &self.strokes {
            Some(table) => Some(table.after_edit(intent, index)?),
            None => None,
        };
        Some(Self {
            id: self.id,
            points: out,
            closed: self.closed,
            strokes,
        })
    }
}

/// **The edit one frame asks the engine for**, in whichever of the two verb
/// families the shape belongs to.
#[derive(Debug, Clone, PartialEq)]
pub enum Plan {
    /// A `/Polygon`, `/PolyLine` or `/Line` — one index.
    Vertex(VertexEdit),
    /// An `/Ink` — `(stroke, point)`, converted from the flat anchor index by
    /// [`ink::StrokeTable`].
    Ink(InkEdit),
}

impl Plan {
    /// **Ask the engine whether this edit is allowed**, without doing it.
    ///
    /// Each family's preview shares one body with its mutating verb —
    /// `reshape_plan` for vertices, `ink_plan` for ink — so neither can
    /// disagree with what the release would do. The forecast itself is not
    /// returned: the drag needs a yes or a worded no, and the disclosure the
    /// ink forecast carries (`appearance_was_pdfces`) is read at apply time by (old-name-exempt: the engine's own field name, quoted verbatim)
    /// `app::actions::annots`, where the sentence can reach the status line.
    pub fn preview(&self, session: &EditSession, id: ObjId) -> Result<(), EditError> {
        match self {
            Self::Vertex(edit) => session.reshape_annotation_preview(id, *edit).map(|_| ()),
            Self::Ink(edit) => session.reshape_ink_preview(id, edit).map(|_| ()),
        }
    }

    /// **The action a release raises** for this plan — one of the six node
    /// variants of [`AnnotAction`], each of which reaches exactly one engine
    /// verb.
    #[must_use]
    pub fn action(self, id: ObjId) -> AnnotAction {
        match self {
            Self::Vertex(VertexEdit::Move { index, dx, dy }) => {
                AnnotAction::MoveNode { id, index, dx, dy }
            }
            Self::Vertex(VertexEdit::Insert { after, at }) => {
                AnnotAction::InsertNode { id, after, at }
            }
            Self::Vertex(VertexEdit::Remove { index }) => AnnotAction::RemoveNode { id, index },
            Self::Ink(InkEdit::MovePoint {
                stroke,
                point,
                dx,
                dy,
            }) => AnnotAction::MoveInkPoint {
                id,
                stroke,
                point,
                dx,
                dy,
            },
            Self::Ink(InkEdit::InsertPoint { stroke, after, at }) => AnnotAction::InsertInkPoint {
                id,
                stroke,
                after,
                at,
            },
            Self::Ink(InkEdit::RemovePoint { stroke, point }) => {
                AnnotAction::RemoveInkPoint { id, stroke, point }
            }
            // `InkEdit` is `#[non_exhaustive]` and carries three whole-stroke
            // variants this shell does not build — `ReplaceStroke`,
            // `MoveStroke`, `RemoveStroke`. [`planned`] is the only
            // constructor of a `Plan::Ink` and it builds only the three point
            // variants above, so this arm is unreachable by construction;
            // mapped to the general decline rather than a panic, for the same
            // reason every other "cannot happen" in a drag is: a wrong sentence
            // on release is recoverable and a crash mid-gesture is not.
            Self::Ink(_) => AnnotAction::DeclineNodeEdit {
                why: crate::text::markup::NodeEditRefusal::Refused,
            },
        }
    }

    /// A short word for the trace. Never displayed.
    fn word(&self) -> &'static str {
        match self {
            // ui-text-exempt: diagnostic trace fragments, never displayed in the UI.
            Self::Vertex(_) => "vertex",
            Self::Ink(_) => "ink",
        }
    }
}

/// **Build the [`Plan`] for this frame's intent**, addressed the way the
/// shape's family addresses a node.
#[must_use]
fn planned(
    shape: &Geometry,
    intent: VertexIntent,
    index: usize,
    from: Point,
    target: Point,
) -> Option<Plan> {
    if let Some(table) = &shape.strokes {
        return table.plan(intent, index, from, target).map(Plan::Ink);
    }
    Some(Plan::Vertex(match intent {
        VertexIntent::Move => VertexEdit::Move {
            index,
            dx: target.x - from.x,
            dy: target.y - from.y,
        },
        VertexIntent::Insert => VertexEdit::Insert {
            after: index,
            at: target,
        },
        VertexIntent::Remove => VertexEdit::Remove { index },
    }))
}

/// Which of the shell's sentences an engine refusal is.
#[must_use]
fn refusal_for(error: &EditError) -> crate::text::markup::NodeEditRefusal {
    use crate::text::markup::NodeEditRefusal as R;
    match error {
        EditError::ReshapeWouldBreachVertexFloor { .. } => R::WouldLeaveTooFew,
        EditError::InkStrokeWouldBreachPointFloor { .. } => R::StrokeWouldLeaveTooFew,
        EditError::GeometryNotReshapable { subtype, .. } => R::ShapeHasNoNodes {
            subtype: shape_word(subtype),
        },
        EditError::AnnotationVertexNotPlaceable { .. } => R::Unplaceable,
        EditError::AnnotationLocked { .. } => R::Locked,
        // The two ink index spaces, one sentence: whichever list the engine
        // could not find the address in, the shell's anchors were drawn from a
        // `/InkList` the engine no longer holds in that shape, and the remedy
        // — reselect, so both are rebuilt from one walk — is the same.
        EditError::InkPointIndexOutOfRange { .. } | EditError::InkStrokeIndexOutOfRange { .. } => {
            R::PointNotFound
        }
        EditError::InkWouldBeEmpty { .. } => R::WouldLeaveNothing,
        // A non-ink shape reached the ink planner. Unreachable while
        // [`planned`] chooses the family from [`Geometry::strokes`]; named so
        // the day it is reached the trace line carries the engine's own
        // sentence — which names the subtype — beside the shell's general one.
        EditError::InkVerbOnNonInk { .. } => R::Refused,
        // Named rather than left to the `_` arm below, and it earns the line:
        // this is the refusal that fires when the anchors and the geometry have
        // gone out of step — the painter drew a handle at index `n` and the
        // engine can no longer find one there. It cannot happen while both read
        // the same `page_annotations` walk in the same frame, and the day
        // anything caches one of them it becomes the first symptom. Mapped to
        // the general sentence because there is no next act about nodes that
        // would help the operator; kept visible here because there IS one for
        // whoever reads the trace, and `markup-vertex-declined` carries the
        // engine's own count and index.
        EditError::AnnotationVertexIndexOutOfRange { .. } => R::Refused,
        _ => R::Refused,
    }
}

/// The operator's word for a `/Subtype`.
#[must_use]
fn shape_word(subtype: &str) -> crate::text::markup::ShapeWord {
    use crate::text::markup::ShapeWord as W;
    match subtype {
        "Ink" => W::Ink,
        "Square" => W::Rectangle,
        "Circle" => W::Ellipse,
        "Line" => W::Line,
        "Highlight" | "Underline" | "StrikeOut" | "Squiggly" => W::TextMarkup,
        _ => W::Other,
    }
}

/// What one frame of a node drag needs, gathered at the call site.
pub struct NodeFrame<'a> {
    /// The frame's context. Read only — the snap settings and the live
    /// modifiers live in it.
    pub ctx: &'a egui::Context,
    /// Which node, sampled at the press.
    pub index: usize,
    /// Where the press landed, in **canvas** space — the grab point.
    pub from: egui::Pos2,
    /// Where the pointer is now, in **canvas** space.
    pub at: egui::Pos2,
    /// Draw, or commit.
    pub phase: Phase,
    /// The open document.
    pub doc: &'a OpenDoc,
    /// The current selection, which is what names the shape.
    pub selection: &'a SelectionState,
    /// The decomposition, for the snap query. `None` means no snapping this
    /// frame rather than an error.
    pub targets: Option<&'a dyn crate::canvas::target::CanvasTargetProvider>,
    /// The frame's mapping, which owns the snap tolerance in page units.
    pub map: &'a PageMapping,
    /// Whether Alt is down **this frame** — the operator saying *"not this
    /// time"*, and the same override a measure pick honours.
    pub alt_held: bool,
}

/// What one frame of a node drag produced.
#[derive(Default)]
pub struct NodeDrag {
    /// The shape the release would commit, as page-space segments, or `None`
    /// when this frame previews nothing.
    pub segments: Option<Vec<(Point, Point)>>,
    /// What the node is snapping to, if anything.
    pub snap: Option<SnapCandidate>,
}

/// Advance one frame of a **node** drag on a markup shape.
pub fn drag(frame: NodeFrame<'_>, actions: &mut Vec<Action>) -> NodeDrag {
    inner(frame, actions).unwrap_or_default()
}

fn inner(frame: NodeFrame<'_>, actions: &mut Vec<Action>) -> Option<NodeDrag> {
    let NodeFrame {
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
    } = frame;
    let shape = geometry(doc, selection)?;
    let annot = selection.annot()?;
    let page = doc.pages.get(annot.target.page)?;
    let old = *shape.points.get(index)?;

    //
    // And the GRAB POINT is preserved (D8): the node moves by the pointer's
    // DELTA, not to the pointer's position. Assigning the pointer straight to
    // the node teleports it under the cursor on the first frame, so an operator
    // who grabbed an anchor three pixels off centre sees the shape jump before
    // they have moved anything.
    let grab = at - from;
    let was = crate::viewer::pdf_space_to_canvas(
        #[allow(clippy::cast_possible_truncation)]
        egui::Pos2::new(old.x as f32, old.y as f32),
        page,
    )?;
    let free_pos = crate::viewer::canvas_to_pdf_space(was + grab, page)?;
    #[allow(clippy::cast_lossless)]
    let free = Point::new(f64::from(free_pos.x), f64::from(free_pos.y));

    // THE SNAP, and it deliberately OVERRIDES the grab point. D8 and D6 pull
    // in opposite directions here and every program in the class resolves it
    // the same way: snapping wins. The whole content of the gesture is landing
    // the node exactly on something, and preserving a three-pixel grab offset
    // would put it exactly three pixels off the thing it snapped to.
    //
    // The same query, the same tolerance and the same operator settings the
    // measure tools use — `measure::snap_point` exists precisely so there is
    // one answer to *"where would this land"* rather than two.
    let (target, snap) =
        crate::canvas::measure::snap_point(ctx, annot.target.page, free, alt_held, targets, map);

    let intent = crate::canvas::dimdrag::intent(ctx);
    Some(resolved(Resolve {
        session: &doc.session,
        shape: &shape,
        intent,
        index,
        old,
        target,
        phase,
        snap,
        actions,
    }))
}

/// Everything the second half of a node drag needs, once the geometry and the
/// snap have been resolved.
struct Resolve<'a> {
    /// The read side of the document, for the preflight.
    session: &'a EditSession,
    /// The shape as it stands — its id, its nodes in page space, whether it
    /// closes, and (for an `/Ink`) where its strokes begin.
    shape: &'a Geometry,
    /// Move, add or remove.
    intent: VertexIntent,
    /// The node the drag grabbed.
    index: usize,
    /// Where that node is now, page space — the operand of a move's delta.
    old: Point,
    /// Where the pointer is, page space, **after** snapping.
    target: Point,
    /// Draw, or commit.
    phase: Phase,
    /// What the node is snapping to, carried through to the painter.
    snap: Option<SnapCandidate>,
    /// Where the release's one action goes.
    actions: &'a mut Vec<Action>,
}

/// **The preflight, the preview and the commit** — the half of a node drag that
/// has no pointer in it.
///
/// See [`drag`] for the ordering and why the preflight comes before the
/// arithmetic that draws.
fn resolved(edit: Resolve<'_>) -> NodeDrag {
    let Resolve {
        session,
        shape,
        intent,
        index,
        old,
        target,
        phase,
        snap,
        actions,
    } = edit;
    let id = shape.id;
    // `address=` in the trace lines below: the engine's own `(stroke, point)`
    // for an `/Ink`, so a driven check can tell *the right verb on the wrong
    // stroke* from a working gesture; `none` for a single-list shape, whose
    // address IS the index.
    let address = shape
        .strokes
        .as_ref()
        .and_then(|t| t.address(index))
        .map_or_else(
            || "none".to_owned(),
            |(stroke, point)| format!("{stroke}/{point}"),
        );
    let Some(plan) = planned(shape, intent, index, old, target) else {
        // Only an `/Ink` anchor index the stroke table cannot place — a press
        // the painter could not have drawn an anchor for. The engine cannot be
        // asked about an address this shell cannot produce, so the refusal is
        // worded here, by the same rule as every other: a sentence, not a
        // silence, and only on the frame the operator let go.
        if phase == Phase::Complete {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI.
                format!(
                    "{TRACE_DECLINED} id={} index={index} address={address} intent={intent:?} \
                     reason=anchor-index-outside-ink-list",
                    id.num
                )
            });
            actions.push(Action::Annot(AnnotAction::DeclineNodeEdit {
                why: crate::text::markup::NodeEditRefusal::PointNotFound,
            }));
        }
        return NodeDrag::default();
    };

    // --- the preflight ----------------------------------------------------
    //
    // `reshape_annotation_preview` shares one body with `reshape_annotation`
    // (`reshape_plan`), and `reshape_ink_preview` with `reshape_ink`
    // (`ink_plan`), so neither can disagree with what the release would do. It
    // costs one annotation walk per frame of a drag that lasts a second or
    // two, which is deliberate: the alternative is a second copy of the
    // engine's subtype matrix and its floors in this shell, and that is the
    // *"two things that must agree and eventually will not"* the engine's own
    // doc comment argues against by name.
    if let Err(why) = plan.preview(session, id) {
        if phase == Phase::Complete {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI.
                //
                // The engine's own sentence goes HERE, verbatim, and not to
                // the operator. This is where a developer reads it, and it is
                // the one place a refusal this shell mapped to its general
                // sentence can still be diagnosed precisely.
                format!(
                    "{TRACE_DECLINED} id={} index={index} address={address} family={} \
                     intent={intent:?} reason={why}",
                    id.num,
                    plan.word(),
                )
            });
            // Handed INWARD as an action rather than recorded here: the
            // decline store is `pub(super)` inside `crate::app` and the canvas
            // is outside that boundary. See `AnnotAction::DeclineNodeEdit`.
            actions.push(Action::Annot(AnnotAction::DeclineNodeEdit {
                why: refusal_for(&why),
            }));
            return NodeDrag::default();
        }
        // The shape exactly as it stands. See [`drag`]'s header: a preview that
        // showed the edit would be promising a release that refuses.
        return NodeDrag {
            segments: Some(shape.segments()),
            snap: None,
        };
    }

    let Some(after) = shape.edited(intent, index, target) else {
        // Unreachable behind the preflight, which refuses every index the list
        // does not hold. Returning empty rather than asserting: a frame with no
        // preview is recoverable, and a panic here would take the window with
        // it during a drag.
        return NodeDrag::default();
    };

    if phase == Phase::Complete {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI.
            //
            // `nodes=` carries the count AFTER the edit and `snap=` carries
            // the candidate KIND rather than a boolean, both for the reason
            // `dimension-vertex` states: a trace line must carry the number a
            // wrong build would get wrong. An insert on the wrong segment, a
            // remove that took the neighbour, and a working gesture all move
            // the shape; only the index and the count together say which.
            format!(
                "{} id={} index={index} address={address} family={} nodes={} x={:.2} y={:.2} \
                 snap={}",
                match intent {
                    VertexIntent::Move => TRACE_MOVE,
                    VertexIntent::Insert => TRACE_INSERT,
                    VertexIntent::Remove => TRACE_REMOVE,
                },
                id.num,
                plan.word(),
                after.points.len(),
                target.x,
                target.y,
                snap.map_or_else(|| "none".to_owned(), |c| format!("{:?}", c.kind))
            )
        });
        // The action is built from the SAME plan the preflight was asked
        // about — `Plan::action` is a pure relabelling — so the question asked
        // and the edit committed cannot differ by an index, a stroke or a sign.
        actions.push(Action::Annot(plan.action(id)));
        // Nothing is previewed on the frame that commits: the annotation is
        // about to be regenerated and drawn for real, and a preview laid over
        // it would be a second copy of the same shape, one frame stale.
        return NodeDrag::default();
    }

    NodeDrag {
        segments: Some(after.segments()),
        // A removal has no destination, so there is nothing for a snap marker
        // to describe and drawing one would point at a node that is about to
        // stop existing.
        snap: (intent != VertexIntent::Remove).then_some(snap).flatten(),
    }
}

/// **The sentence a shape with no nodes owes the operator**, raised once when
/// they arm the tool that looks for nodes.
pub fn explain_unreshapable(
    ctx: &egui::Context,
    doc: &OpenDoc,
    selection: &SelectionState,
    actions: &mut Vec<Action>,
) -> bool {
    let armed = crate::canvas::tool::active(ctx).is_node();
    let subject = selection
        .annot()
        .filter(|a| a.target.kind == AnnotKind::Markup)
        .map(|a| (a.target.id, a.target.subtype.clone()));
    // The memory slot holds what was last SAID about, so that re-selecting the
    // same shape after selecting another one says it again — which is right:
    // the operator asked twice.
    let key = egui::Id::new("markup-nodes-explained");
    let said: Option<(bool, Option<(ObjId, String)>)> = ctx.memory(|m| m.data.get_temp(key));
    let now = (armed, subject.clone());
    if said.as_ref() == Some(&now) {
        return false;
    }
    ctx.memory_mut(|m| m.data.insert_temp(key, now));
    if !armed {
        return false;
    }
    let Some((id, subtype)) = subject else {
        return false;
    };
    // Asked of [`geometry`] rather than of the subtype directly, so the
    // sentence and the anchors can never disagree: if this answers `Some` the
    // painter drew handles, and there is nothing to explain.
    if geometry(doc, selection).is_some() {
        return false;
    }
    let word = shape_word(&subtype);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI.
        format!(
            "{TRACE_UNAVAILABLE} id={} subtype={subtype} word={word:?}",
            id.num
        )
    });
    actions.push(Action::Annot(AnnotAction::DeclineNodeEdit {
        why: crate::text::markup::NodeEditRefusal::ShapeHasNoNodes { subtype: word },
    }));
    true
}

/// **The right-click route to these same three verbs.** See its header for
/// why a menu row needs no armed tool where the chord does, and for where the
/// *which node did they mean* operand is parked for the life of the popup.
pub mod menu;

/// **The stroke table of an `/Ink`** — flat anchor index ↔ `(stroke, point)`,
/// the within-stroke segment list, and why every point is an anchor for now.
pub use pdfcer_gui_base::inkaddress as ink;

#[cfg(test)]
mod tests;
