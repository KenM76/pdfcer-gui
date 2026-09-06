//! # `canvas::annotnodes` — **the nodes of a markup shape, and moving them**
//!
//! ## The operator's report, verbatim
//!
//! > *"I also can't edit or delete nodes of a markup shape once it is drawn."*
//!
//! That sentence named three separate absences and this module closes the one
//! it literally describes. The **ce dimension** half was closed the same day by
//! [`crate::canvas::dimdrag`] — a ce dimension's corners have had engine verbs
//! since `Pass 107.0`. The **markup** half could not be built at all until
//! `Pass 255.0`, because `pdfcer-core` did not model a markup annotation's
//! geometry: `/Vertices`, `/L` and `/InkList` were not in the read model and
//! there was no verb that could rewrite one. It was filed
//! (`request_a_markup_shapes_vertices_cannot_be_read_or_edited.md`) and
//! deliberately **not** worked around — re-parsing the annotation dictionary in
//! this shell would have been a second, weaker implementation of geometry the
//! engine owns, and the engine's own note explains exactly what a shell that
//! did so would ship:
//!
//! > A move has two halves and **only one of them shows up in a render.**
//! > `/Rect` moves the painted result for free; the geometry keys hold
//! > *absolute page coordinates*, and they are what any **other** tool
//! > regenerates an appearance from.
//!
//! ⇒ Every instrument this project owns — the rendered canvas, a screenshot, a
//! driven pixel check — reads the appearance stream. A shell that moved a node
//! by rewriting `/AP` alone would pass all of them and be wrong in Acrobat a
//! week later.
//!
//! ## What the engine gives us, and the matrix it enforces
//!
//! `Pass 255.0` shipped one planner,
//! `EditSession::reshape_annotation(annot_id, VertexEdit, modified)`, its
//! preflight `reshape_annotation_preview`, and three one-line wrappers —
//! `move_annotation_vertex`, `insert_annotation_vertex`,
//! `remove_annotation_vertex`. The matrix is the engine's and is **not**
//! restated as a condition here; it is asked, per frame, through the preflight:
//!
//! | `/Subtype` | move | insert | remove | floor |
//! |---|---|---|---|---|
//! | `/Polygon` (plain or cloudy `/BE`) | yes | yes | yes | 3 |
//! | `/PolyLine` | yes | yes | yes | 2 |
//! | `/Line` (incl. arrows) | yes (index 0/1) | refused | refused | — |
//! | `/Ink` | refused | refused | refused | — |
//! | `/Square`, `/Circle`, text markup | refused | refused | refused | — |
//!
//! ★★ **This shell knows the first column and nothing else in that table.**
//! [`geometry`] decides which shapes have *anchors to draw* — that is a
//! painting question and it has to be answered locally — and every question
//! about whether an edit is **allowed** goes to the engine. The distinction
//! matters because the two lists are not the same one: `/Ink` has readable
//! geometry (`Annotation::ink_list`) and no editable geometry, so a shell that
//! derived "draggable" from "readable" would put handles on every ink stroke
//! and refuse every drag from them.
//!
//! ## ★★★ The preflight is asked EVERY FRAME, including for a plain move
//!
//! `reshape_annotation_preview` shares one body with the mutating verb
//! (`reshape_plan`), so it cannot disagree with what a release would do. The
//! engine's standing advice, given to this project when the ce-dimension
//! vertex verbs landed and repeated on this Pass, is not optional:
//!
//! > *ask the preview verb every frame rather than catching the error
//! > afterwards* — *"a verb with no preflight makes the UI find out by
//! > pressing."*
//!
//! What it buys is the honesty contract every drag in this canvas is held to:
//! **the preview is a shape the release would commit, or it is the shape that
//! is already there.** A drag that begins and then fails is worse than a drag
//! that never starts, because it looks like it worked until the next frame
//! repaints.
//!
//! ★ It is asked for the **move** as well, which is where this module differs
//! from [`crate::canvas::dimdrag`]. That module does not preflight a corner
//! move, on an explicit engine ruling: a ce dimension's move cannot be refused
//! once the drag has begun, because a self-intersecting polyline has a
//! well-defined length and every remaining refusal is structural. A **markup**
//! move can still be refused mid-drag — `AnnotationVertexNotPlaceable` fires on
//! a non-finite coordinate, which is precisely what a page whose transform will
//! not round-trip produces — so the cheaper reasoning does not carry over and
//! is not borrowed.
//!
//! ## Rule 9 — what an unavailable capability draws
//!
//! **Nothing.** A `/Square`, a `/Circle`, an `/Ink` stroke and a text markup
//! get no anchors, no greyed anchors and no ghost anchors: [`geometry`] answers
//! `None` and the painter's loop is empty. There is no *temporarily*
//! unavailable case here to grey — the refusal is a property of the shape's
//! kind and will not change while the operator looks at it.
//!
//! ★★ What they get instead is a **sentence**, and it is delivered by
//! [`explain_unreshapable`] at the moment the operator asks: with the Points
//! tool armed — the deliberate act of arming the tool whose whole subject is
//! nodes — a selected markup that has no nodes says so, once, naming its own
//! kind. That is the difference between *"this shape has no nodes"* and *"this
//! program forgot to draw them"*, and it is the only difference the operator
//! can see.
//!
//! ## Rule 4 — this is the cursor, not the document
//!
//! Node anchors and the in-flight polyline are **pre-commit affordances**,
//! which R8b rule 4 names by name: *"a snap indicator, a hover highlight, a
//! rubber-band … these are the cursor"*. They are drawn for the selected
//! annotation only, they vanish with the selection, and nothing already applied
//! to the page is tinted, badged or flagged. The one-line test passes: a
//! screenshot of the canvas mid-drag differs from the saved file by a marching
//! outline and some small squares, which is where the pointer is and not what
//! the document says.
//!
//! ## Rule 15
//!
//! Everything here is about a **markup shape** — a `/Polygon`, `/PolyLine` or
//! `/Line` the operator drew as a comment. A **ce dimension** is also a `/Line`
//! and is claimed by [`crate::canvas::dimdrag`] before this module is reached;
//! the engine refuses it from these verbs by name
//! (`EditError::AnnotationIsCeDimension`) as the backstop. **pdf dimensions** —
//! CAD page content — are not annotations at all and are nowhere near this
//! module.
//!
//! ## conventions: drag-moves
//!
//! Corpus `ui-conventions/drag-moves.md`, answered row by row because the
//! unanswered ones are the ones the operator finds.
//!
//! - **D1 live-preview** — the shape follows the pointer from the first frame,
//!   drawn from [`preview_of`], which is the same point list the release
//!   commits. ★ `from` and `at` arrive **already in canvas space** and are not
//!   converted again; that double hop is the defect the operator reported on
//!   2026-08-20 (*"moves at a different speed than my mouse movements"*) and
//!   `dimdrag::inner` carries the post-mortem.
//! - **D2 derived-from-commit** — [`edited`] returns the point list, and both
//!   the preview and the action are built from that one `Vec`.
//! - **D3 escape-cancels** — WAIVED, as for every drag here: the gesture
//!   machine owns Escape and drops the drag before this module is reached.
//!   Nothing is written until `Phase::Complete`.
//! - **D4 one-undo-entry** — `reshape_annotation` is one `CommandKind`, two
//!   objects (the dictionary and its `/N` stream). One gesture pushes exactly
//!   one action, so one gesture is one `Ctrl+Z`.
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
//!   stating a distance that is no longer true. Both are disclosed off-canvas
//!   by `app::actions::annots`; see [`crate::text::markup::measure_stale`].

use pdfcer_core::edit::{EditError, VertexEdit};
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
/// ★ Deliberately **not** `canvas.dimension-vertex`. Two subjects that reach
/// two different engine verb families must be distinguishable in the trace, or
/// a check that aimed at a ce dimension and hit a markup shape would report a
/// working build as broken and vice versa.
pub const NODE_REGION: &str = "canvas.markup-node"; // ui-text-exempt: trace region name

/// `markup-node-move id=… index=… nodes=… x=… y=… snap=…` — the shell's own report that
/// a node **move** gesture was understood.
///
/// ★★★ **`markup-node-`, NOT `markup-vertex-`, and the rename is a caught
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

/// `markup-node-insert id=… index=… nodes=… x=… y=…`
pub const TRACE_INSERT: &str = "markup-node-insert"; // ui-text-exempt: diagnostic trace name

/// `markup-node-remove id=… index=… nodes=… x=… y=…`
pub const TRACE_REMOVE: &str = "markup-node-remove"; // ui-text-exempt: diagnostic trace name

/// `markup-node-declined id=… index=… intent=… reason=…` — the preflight said
/// no on the frame the operator let go.
pub const TRACE_DECLINED: &str = "markup-node-declined"; // ui-text-exempt: diagnostic trace name

/// `markup-nodes-unavailable id=… subtype=… reason=…` — the sentence
/// [`explain_unreshapable`] recorded, in the machine's own words beside the
/// operator's.
pub const TRACE_UNAVAILABLE: &str = "markup-nodes-unavailable"; // ui-text-exempt: diagnostic trace name

/// How much slack a press gets around a node anchor, in points.
///
/// The drawn square is the **promise**; this is the **target**. They differ
/// because a 7 pt square is hard to hit on a dense drawing, and the standing
/// convention here — stated at `handles::grip_at` and again at
/// `dimdrag::vertex_at` — is that a grip's live area may exceed its drawn one
/// and never the reverse. A target smaller than its picture is the operator
/// missing something they can plainly see.
const NODE_GRAB_SLACK_PT: f32 = 3.0;

/// **The geometry of the selected markup shape**, if it is one with nodes.
///
/// Returns the annotation's id, its node list in **page space** (PDF user
/// space, y-up), and whether the shape closes.
///
/// # Which shapes answer, and why the list is short
///
/// | `/Subtype` | source | closed |
/// |---|---|---|
/// | `Polygon` | `/Vertices` | yes |
/// | `PolyLine` | `/Vertices` | no |
/// | `Line` | `/L`, two points | no |
///
/// Everything else answers `None`, and the two exclusions are the interesting
/// ones:
///
/// * **`/Ink`.** `Annotation::ink_list` is readable and every vertex edit on it
///   is refused by name. Drawing anchors from a readable-but-uneditable field
///   would be the *"visible control, silently inert"* failure this project's
///   `DEFECTS.md` is made of. R9: an unavailable capability renders nothing.
/// * **`/Square` and `/Circle`.** Defined by `/Rect`, not by vertices — they
///   already have eight resize grips, which is the verb for them.
///
/// # ★★ A cloudy `/Polygon`'s anchors are on its VERTICES, not on its outline
///
/// A revision cloud is a `/Polygon` carrying `/BE << /S /C >>`; its scallops
/// are baked into `/AP` from the pre-bulge vertex list, and `/Rect` bounds the
/// **bulged** outline. The engine states this and states what a shell should
/// do with it: *"A shell drawing anchors draws them here, not on the cloud's
/// outline."* So the anchors sit slightly inside the ink, which is correct and
/// is what every editor in the class does with a stylised stroke.
///
/// # ★ Three "no"s, and they are different kinds of no
///
/// | condition | what it means |
/// |---|---|
/// | an annotation is selected | otherwise the content branch owns the press |
/// | it is [`AnnotKind::Markup`] | a ce dimension is `dimdrag`'s, and it re-measures |
/// | it is not **locked** | §12.5.3 Table 165 bit 8 — *the file* says the user interface may not change this |
///
/// The locked case is honoured **here**, before an anchor is drawn, rather than
/// being left to the engine's refusal. A handle drawn on a shape the document
/// forbids changing is a promise the release cannot keep. ★ Note this is bit 8
/// (`Locked`, 128) and **not** bit 10 (`LockedContents`, 512) — the engine
/// consults exactly the same one, and its own note records that treating either
/// as the other is a spec-contradicting bug in one direction or the other.
#[must_use]
pub fn geometry(doc: &OpenDoc, selection: &SelectionState) -> Option<(ObjId, Vec<Point>, bool)> {
    let annot = selection.annot()?;
    if annot.target.kind != AnnotKind::Markup || annot.target.locked {
        return None;
    }
    let page = doc.pages.get(annot.target.page)?;
    let found = pdfcer_core::annot::page_annotations(&doc.session.graph(), page.id)
        .into_iter()
        .find(|a| a.id == Some(annot.target.id))?;
    let to_points = |pairs: &[(f64, f64)]| -> Vec<Point> {
        pairs.iter().map(|&(x, y)| Point::new(x, y)).collect()
    };
    // ★ Matched on the `/Subtype` bytes the read model carries rather than on
    // "does it have a `/Vertices` key". The keys are read subtype-agnostically
    // by `page_annotations` — a malformed `/Square` carrying a stray
    // `/Vertices` array would answer the key test and be refused by every verb.
    // The subtype is what the engine's own matrix is keyed on, so it is what
    // this is keyed on.
    match found.subtype.as_slice() {
        b"Polygon" => Some((annot.target.id, to_points(found.vertices.as_ref()?), true)),
        b"PolyLine" => Some((annot.target.id, to_points(found.vertices.as_ref()?), false)),
        // A `/Line`'s two ends are `/L`, not `/Vertices`, and the engine
        // addresses them as index 0 and index 1 of the same `VertexEdit::Move`.
        // So the shell's list is `[start, end]` and the indices line up by
        // construction rather than by a mapping that could be got backwards.
        b"Line" => {
            let [start, end] = found.line?;
            Some((
                annot.target.id,
                vec![Point::new(start.0, start.1), Point::new(end.0, end.1)],
                false,
            ))
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
    let Some((_, points, _)) = geometry(doc, selection) else {
        return Vec::new();
    };
    let Some(annot) = selection.annot() else {
        return Vec::new();
    };
    let Some(page) = doc.pages.get(annot.target.page) else {
        return Vec::new();
    };
    points
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
/// # ★ The comparison is in SCREEN space, and that is why this converts rather
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

/// The node list this edit would produce, applied to `points`.
///
/// ★ Returned rather than drawn, so the preview and the action are built from
/// **one** `Vec`. A second derivation of *"what would this look like"* is the
/// defect `measure::Resolved` exists to prevent, and it has shipped on this
/// canvas twice.
///
/// `None` for an index the list does not hold, which the preflight would also
/// refuse — asked here as well because this function is where the slice is
/// indexed and a panic mid-drag would take the window with it.
#[must_use]
fn edited(
    points: &[Point],
    intent: VertexIntent,
    index: usize,
    target: Point,
) -> Option<Vec<Point>> {
    let mut out = points.to_vec();
    match intent {
        VertexIntent::Move => *out.get_mut(index)? = target,
        // ★ `index + 1`, matching the engine: `insert_annotation_vertex(after,
        // at)` puts the new node at `after + 1`, and there is deliberately no
        // "insert before the first" spelling — the engine refuses `after >=
        // count` and says to rotate the polygon's start instead, which is what
        // every other tool does as well.
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
    Some(out)
}

/// The [`VertexEdit`] this frame's intent asks the engine for.
///
/// ★ Built once and handed to **both** the preflight and the action, so the
/// question asked and the question answered are literally the same value. A
/// shell that preflighted `Remove { index }` and then committed
/// `Remove { index: index + 1 }` would pass every unit test either half has.
#[must_use]
fn planned(intent: VertexIntent, index: usize, from: Point, target: Point) -> VertexEdit {
    match intent {
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
    }
}

/// Which of the shell's five sentences an engine refusal is.
///
/// ★ The mapping lives **here** rather than in `crate::text::markup`, for
/// `dimdrag::refusal_for`'s reason and this project's standing division: the
/// engine's error enum is a *shell* concern, and the string catalog holds
/// operator prose only. A `crate::text::` module that matched on `EditError`
/// would put the engine's vocabulary into the catalog and give the catalog a
/// reason to change every time the engine adds a variant.
///
/// ★★ The engine offers a `reason: &'static str` on
/// [`EditError::GeometryNotReshapable`] and says a shell may show it verbatim.
/// It is **not** shown verbatim, and the choice is deliberate rather than
/// squeamish: those sentences are written for a developer reading a CLI —
/// *"author a PolyLine instead"*, *"use resize_annotation"*, *"/QuadPoints are
/// text-anchored quadrilaterals"* — and they name verbs and PDF keys this
/// operator has never seen. The `subtype` field is what is used, because that
/// is the fact the operator can check against the shape in front of them. The
/// engine's sentence goes to the **trace**, where the developer is.
///
/// The `_` arm is not laziness. The remaining refusals —
/// `AnnotationNotFound`, `AnnotationIsCeDimension`, `AnnotationLocked`,
/// `AnnotationVertexIndexOutOfRange`, `DocumentEncrypted`, the certification
/// guard, `MarkupSpec` — are either unreachable from an anchor this shell drew
/// from this same geometry, or are properties of the FILE that no wording about
/// nodes would help with. They get the general sentence rather than a
/// fabricated specific one, and the operator learns that the press was heard.
#[must_use]
fn refusal_for(error: &EditError) -> crate::text::markup::NodeEditRefusal {
    use crate::text::markup::NodeEditRefusal as R;
    match error {
        EditError::ReshapeWouldBreachVertexFloor { .. } => R::WouldLeaveTooFew,
        EditError::GeometryNotReshapable { subtype, .. } => R::ShapeHasNoNodes {
            subtype: shape_word(subtype),
        },
        EditError::AnnotationVertexNotPlaceable { .. } => R::Unplaceable,
        EditError::AnnotationLocked { .. } => R::Locked,
        // ★ Named rather than left to the `_` arm below, and it earns the line:
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
///
/// ★ A mapping and not a passthrough. `"PolyLine"` is a PDF name; *"a
/// polyline"* is a shape. `"Square"` is the PDF name for what pdfcer's own
/// ribbon calls a **rectangle**, and showing the operator "Square" for the
/// thing they drew with the Rectangle tool is the surface disagreeing with
/// itself. The unknown arm keeps the raw name rather than inventing one,
/// because a subtype this shell has never heard of is better named exactly than
/// named wrongly.
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
    let (id, points, closed) = geometry(doc, selection)?;
    let annot = selection.annot()?;
    let page = doc.pages.get(annot.target.page)?;
    let old = *points.get(index)?;

    // ★★★ `from` and `at` are ALREADY CANVAS SPACE — the gesture machine says
    // so on the variant, and converting them a second time is the operator's
    // bug of 2026-08-20: the node tracked at `1/zoom` of the pointer's speed
    // and sat off by the scroll origin. `egui::Pos2` is screen, canvas AND page
    // space, so the compiler cannot object; `dimdrag::inner` carries the whole
    // post-mortem and this is the third module to inherit the rule rather than
    // rediscover it.
    //
    // ★★ And the GRAB POINT is preserved (D8): the node moves by the pointer's
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

    // ★★ THE SNAP, and it deliberately OVERRIDES the grab point. D8 and D6 pull
    // in opposite directions here and every program in the class resolves it
    // the same way: snapping wins. The whole content of the gesture is landing
    // the node exactly on something, and preserving a three-pixel grab offset
    // would put it exactly three pixels off the thing it snapped to.
    //
    // ★ The same query, the same tolerance and the same operator settings the
    // measure tools use — `measure::snap_point` exists precisely so there is
    // one answer to *"where would this land"* rather than two.
    let (target, snap) =
        crate::canvas::measure::snap_point(ctx, annot.target.page, free, alt_held, targets, map);

    let intent = crate::canvas::dimdrag::intent(ctx);
    Some(resolved(Resolve {
        session: &doc.session,
        id,
        points: &points,
        closed,
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
///
/// ★ A struct for [`NodeFrame`]'s reason and one more: it is the seam that lets
/// every rule below be tested **against the real engine** without a window, a
/// pointer or an `egui::Context`. `dimdrag::CountEdit` draws the identical seam
/// for the identical reason, and its own tests are the precedent — a test that
/// faked the annotation would be asking the engine about a shape that does not
/// exist and would get `AnnotationNotFound` for every case while looking
/// exactly like a test that passed for the right reason.
struct Resolve<'a> {
    /// The read side of the document, for the preflight.
    session: &'a pdfcer_core::edit::EditSession,
    /// The annotation being reshaped.
    id: ObjId,
    /// Its nodes as they stand, page space.
    points: &'a [Point],
    /// Whether the shape closes — a `/Polygon` does, the other two do not.
    closed: bool,
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
        id,
        points,
        closed,
        intent,
        index,
        old,
        target,
        phase,
        snap,
        actions,
    } = edit;
    let plan = planned(intent, index, old, target);

    // --- the preflight ----------------------------------------------------
    //
    // `reshape_annotation_preview` shares one body with `reshape_annotation`
    // (`reshape_plan`), so it cannot disagree with what the release would do.
    // It costs one annotation walk per frame of a drag that lasts a second or
    // two, which is deliberate: the alternative is a second copy of the
    // engine's subtype matrix and its two floors in this shell, and that is the
    // *"two things that must agree and eventually will not"* the engine's own
    // doc comment argues against by name.
    if let Err(why) = session.reshape_annotation_preview(id, plan) {
        if phase == Phase::Complete {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI.
                //
                // ★ The engine's own sentence goes HERE, verbatim, and not to
                // the operator. This is where a developer reads it, and it is
                // the one place a refusal this shell mapped to its general
                // sentence can still be diagnosed precisely.
                format!(
                    "{TRACE_DECLINED} id={} index={index} intent={intent:?} reason={why}",
                    id.num
                )
            });
            // ★ Handed INWARD as an action rather than recorded here: the
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
            segments: Some(preview_of(points, closed)),
            snap: None,
        };
    }

    let Some(shape) = edited(points, intent, index, target) else {
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
            // ★ `nodes=` carries the count AFTER the edit and `snap=` carries
            // the candidate KIND rather than a boolean, both for the reason
            // `dimension-vertex` states: a trace line must carry the number a
            // wrong build would get wrong. An insert on the wrong segment, a
            // remove that took the neighbour, and a working gesture all move
            // the shape; only the index and the count together say which.
            format!(
                "{} id={} index={index} nodes={} x={:.2} y={:.2} snap={}",
                match intent {
                    VertexIntent::Move => TRACE_MOVE,
                    VertexIntent::Insert => TRACE_INSERT,
                    VertexIntent::Remove => TRACE_REMOVE,
                },
                id.num,
                shape.len(),
                target.x,
                target.y,
                snap.map_or_else(|| "none".to_owned(), |c| format!("{:?}", c.kind))
            )
        });
        actions.push(Action::Annot(match intent {
            VertexIntent::Move => AnnotAction::MoveNode {
                id,
                index,
                dx: target.x - old.x,
                dy: target.y - old.y,
            },
            VertexIntent::Insert => AnnotAction::InsertNode {
                id,
                after: index,
                at: target,
            },
            VertexIntent::Remove => AnnotAction::RemoveNode { id, index },
        }));
        // Nothing is previewed on the frame that commits: the annotation is
        // about to be regenerated and drawn for real, and a preview laid over
        // it would be a second copy of the same shape, one frame stale.
        return NodeDrag::default();
    }

    NodeDrag {
        segments: Some(preview_of(&shape, closed)),
        // A removal has no destination, so there is nothing for a snap marker
        // to describe and drawing one would point at a node that is about to
        // stop existing.
        snap: (intent != VertexIntent::Remove).then_some(snap).flatten(),
    }
}

/// **The sentence a shape with no nodes owes the operator**, raised once when
/// they arm the tool that looks for nodes.
///
/// # ★★★ Why this exists at all, and why it is not a greyed anchor
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
/// ★★ **Once per subject, not once per frame.** The pair
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
    // ★ Asked of [`geometry`] rather than of the subtype directly, so the
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

/// ★★★ **The right-click route to these same three verbs.** See its header for
/// why a menu row needs no armed tool where the chord does, and for where the
/// *which node did they mean* operand is parked for the life of the popup.
pub mod menu;

#[cfg(test)]
mod tests;
