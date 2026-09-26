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
///
/// Published by the painter so a driven check can **aim** at a node. Where a
/// handle is sits at the end of a page → canvas → screen conversion and is a
/// fact only the running application knows; a harness that guessed *"somewhere
/// near the corner I clicked"* would land on the page instead, start a marquee,
/// and then pass while exercising a completely different gesture.
///
/// Deliberately **not** `canvas.dimension-vertex`. Two subjects that reach
/// two different engine verb families must be distinguishable in the trace, or
/// a check that aimed at a ce dimension and hit a markup shape would report a
/// working build as broken and vice versa.
pub const NODE_REGION: &str = "canvas.markup-node"; // ui-text-exempt: trace region name

/// `markup-node-move id=… index=… address=… family=… nodes=… x=… y=… snap=…` —
/// the shell's own report that a node **move** gesture was understood.
///
/// `address=` is `stroke/point` for an `/Ink` and `none` otherwise;
/// `family=` is `vertex` or `ink`, naming which engine planner was asked.
///
/// **`markup-node-`, NOT `markup-vertex-`, and the rename is a caught
/// defect rather than a preference.** `canvas::markup::vertex` has written
/// `markup-vertex kind=… page=… n=… x=… y=…` since polygons became
/// authorable — one line per CLICK while the operator is drawing a shape.
/// A move line under the same first token would have made `Trace::last("markup-vertex")`
/// return whichever came later, so a check asserting a node MOVED would have
/// read a line about a node being PLACED and reported a working build as
/// broken, or the reverse. `tools/gates/check-trace-names.py` catches this
/// collision only against **funnel labels**; a module-to-module collision is
/// still found by reading, which is how this one was found.
///
/// Distinct from the funnel's `move-annotation-vertex`, which is the engine's
/// acknowledgement that the document changed. A check that read only one of the
/// two could not tell a shell that never asked from an engine that refused; see
/// `tools/gates/check-trace-names.py` for the three times that cost a day.
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
///
/// Returned by [`geometry`], read by the painter, the hit test, the preview and
/// the right-click menu — one value, so the anchor the operator sees, the anchor
/// the press finds and the address the engine is asked about are derived from
/// the same list in the same frame.
///
/// # `strokes` is the one field that knows which verb family this is
///
/// `None` is a `/Polygon`, `/PolyLine` or `/Line`: the engine addresses a node
/// by one index into `/Vertices` (or `/L`), through `VertexEdit`. `Some` is an
/// `/Ink`: the engine addresses a point as `(stroke, point)`, through `InkEdit`,
/// and the table converts this list's flat index to that pair. [`Plan`] is the
/// only place that branches on it, and everything else in this module treats
/// the two alike — which is what makes a freehand mark's drag look and behave
/// exactly like a polyline's, the property the operator asked for.
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
///
/// # Which shapes answer, and why the list is short
///
/// | `/Subtype` | source | closed | addressed by |
/// |---|---|---|---|
/// | `Polygon` | `/Vertices` | yes | one index, `VertexEdit` |
/// | `PolyLine` | `/Vertices` | no | one index, `VertexEdit` |
/// | `Line` | `/L`, two points | no | index 0 or 1, `VertexEdit` |
/// | `Ink` | `/InkList`, flattened | no | `(stroke, point)`, `InkEdit` — see [`ink`] |
///
/// Everything else answers `None`, and the exclusions are the interesting ones:
///
/// * **`/Square` and `/Circle`.** Defined by `/Rect`, not by vertices — they
///   already have eight resize grips, which is the verb for them.
/// * **Text markup.** `/QuadPoints` follow the words they cover and have no
///   corners of their own; the engine refuses them by name.
/// * **An `/Ink` with no readable `/InkList`.** `Annotation::ink_list` is
///   `None` for a malformed or absent array, and an anchor list with nothing
///   in it is the honest answer — the engine would refuse every address.
///
/// `/Ink` was on the refused list until `pdfcer-core` `Pass 278.0`
/// (`c8a6697`), and the argument for refusing it then is the argument for
/// drawing it now: an anchor is drawn only where a verb can act on it. The
/// verbs exist, so the anchors do. [`ink`]'s header carries what changed.
///
/// # A cloudy `/Polygon`'s anchors are on its VERTICES, not on its outline
///
/// A revision cloud is a `/Polygon` carrying `/BE << /S /C >>`; its scallops
/// are baked into `/AP` from the pre-bulge vertex list, and `/Rect` bounds the
/// **bulged** outline. The engine states this and states what a shell should
/// do with it: *"A shell drawing anchors draws them here, not on the cloud's
/// outline."* So the anchors sit slightly inside the ink, which is correct and
/// is what every editor in the class does with a stylised stroke.
///
/// # Three "no"s, and they are different kinds of no
///
/// | condition | what it means |
/// |---|---|
/// | an annotation is selected | otherwise the content branch owns the press |
/// | it is [`AnnotKind::Markup`] | a ce dimension is `dimdrag`'s, and it re-measures |
/// | it is not **locked** | §12.5.3 Table 165 bit 8 — *the file* says the user interface may not change this |
///
/// The locked case is honoured **here**, before an anchor is drawn, rather than
/// being left to the engine's refusal. A handle drawn on a shape the document
/// forbids changing is a promise the release cannot keep. Note this is bit 8
/// (`Locked`, 128) and **not** bit 10 (`LockedContents`, 512) — the engine
/// consults exactly the same one, and its own note records that treating either
/// as the other is a spec-contradicting bug in one direction or the other.
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
///
/// # The comparison is in SCREEN space, and that is why this converts rather
/// than the caller
///
/// An anchor is a screen-space affordance of a fixed size. Comparing in canvas
/// or page space would make the target shrink as the operator zooms out —
/// exactly when a shape's nodes are closest together and precision matters most
/// — and balloon as they zoom in, so that at 800 % a press anywhere near a node
/// would grab it. The conversion has to happen on the side of the boundary
/// where the tolerance is meaningful.
///
/// # Ties go to the LAST node, deliberately
///
/// Two coincident nodes are legal in a `/Vertices` array and the engine does
/// not de-duplicate. If the operator has made one and wants it gone, the one
/// they can reach is the one they can drag away, and the later index is the one
/// they just placed. `dimdrag::vertex_at` resolves the identical tie the
/// identical way; two node gestures on one canvas that disagreed about which
/// coincident point they grabbed would be a difference nobody could see and
/// everybody would trip over.
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
///
/// One function, used by the preview and by nothing else that could disagree
/// with it. A closed shape gets its closing segment here rather than at the
/// call site, because a caller that had to remember to add it is a caller that
/// will one day draw an open triangle over a closed one.
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
    ///
    /// A single-list shape joins consecutive nodes and, when closed with three
    /// or more, the last back to the first — [`preview_of`]'s rule, restated
    /// here as indices so the menu can name the segment's first node. An
    /// `/Ink` joins consecutive points **within** each stroke only; the
    /// boundary between two strokes is not a segment, because the file does
    /// not hold one and the release would not commit one.
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
    ///
    /// One derivation with [`Self::segment_pairs`], so a pair the menu offers
    /// is a segment the painter draws; and for a single-list shape identical
    /// to [`preview_of`], which the tests assert rather than assume.
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
///
/// Built once by [`planned`] and handed to **both** the preflight and the
/// action, so the question asked and the question answered are literally the
/// same value. A shell that preflighted `Remove { index }` and then committed
/// `Remove { index: index + 1 }` would pass every unit test either half has.
///
/// Two variants and not a trait object, for `AnnotAction`'s own reason: the
/// two families reach **two different engine planners** with two different
/// refusal vocabularies, and the one thing that must never happen on this
/// canvas is a gesture aimed at the wrong verb. An `/Ink` handed to
/// `reshape_annotation` is refused by name (`GeometryNotReshapable`); a
/// `/Polygon` handed to `reshape_ink` is refused by name (`InkVerbOnNonInk`).
/// Both refusals are worded in [`refusal_for`] as the backstop, and neither
/// is reachable while this enum is built from [`Geometry::strokes`].
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
///
/// A struct rather than ten parameters — `dimdrag::VertexFrame`'s own argument,
/// adopted rather than re-argued: three members are `Option`s of borrowed
/// things and two are `Pos2`s in the same space, both of which a positional
/// list would let a caller swap silently.
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
///
/// Two fields rather than one, for `dimdrag::VertexDrag`'s reason: the polyline
/// is page-space geometry drawn through the canvas transform, and the marker is
/// a screen-space glyph at the snap candidate. Folding them would make the
/// caller unpack a tuple whose members it uses in two places, forty lines
/// apart.
#[derive(Default)]
pub struct NodeDrag {
    /// The shape the release would commit, as page-space segments, or `None`
    /// when this frame previews nothing.
    pub segments: Option<Vec<(Point, Point)>>,
    /// What the node is snapping to, if anything.
    pub snap: Option<SnapCandidate>,
}

/// Advance one frame of a **node** drag on a markup shape.
///
/// Returns the page-space segments the shape would be drawn as if the operator
/// released now, or an empty [`NodeDrag`] when the drag reaches no verb.
///
/// # The order, and why the preflight comes before the arithmetic that draws
///
/// 1. resolve the shape, the page and the grabbed node;
/// 2. turn the pointer delta into a page-space target, **snapped**;
/// 3. read the live intent — move, insert or remove;
/// 4. **ask the engine whether that edit is allowed**;
/// 5. draw the answer: the edited shape if it is, the shape as it stands if it
///    is not;
/// 6. on release, raise exactly one action — or record exactly one sentence.
///
/// Step 4 before step 5 is the whole design. A build that drew the node
/// vanishing and then refused on release would be showing the operator an edit
/// that never happens.
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
///
/// # Why this exists at all, and why it is not a greyed anchor
///
/// R9: *an unavailable capability renders **nothing**; greying is only for
/// temporarily unavailable, always explained on hover.* A `/Square` will never
/// grow nodes, so greyed anchors on one would be a control that is permanently
/// inert — the exact failure class this project's `DEFECTS.md` is made of, and
/// the shape of the operator's own report.
///
/// But *nothing* is also what a build that forgot to draw the anchors renders,
/// and the operator cannot tell those two apart by looking. So the absence is
/// **stated**, in a sentence, at the moment they ask for it.
///
/// # When it fires, and why that moment
///
/// The **Points tool is armed** and a markup shape with no nodes is selected.
/// Arming that tool is the deliberate act — its whole subject is nodes — so it
/// is the moment the question *"where are the nodes?"* is actually being asked.
/// It is the same reasoning `Declined::NodeToolNeedsEditMode` already uses one
/// step earlier: *a key that does nothing has no control to hover, which makes
/// it the case that most needs a sentence rather than the least.*
///
/// **Once per subject, not once per frame.** The pair
/// `(annotation, is-the-tool-armed)` is remembered in `egui::Memory` and the
/// sentence is raised only when it changes. Writing the decline slot sixty
/// times a second would work — the write is idempotent — and would silently
/// stamp on every other sentence the operator was reading. A status line that
/// cannot be replaced by anything else is not a status line.
///
/// Returns `true` when it raised the sentence, which is what the unit tests
/// assert and what makes "did it fire once?" a question with an answer.
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
pub mod ink;

#[cfg(test)]
mod tests;
