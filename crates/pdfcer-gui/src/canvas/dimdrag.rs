//! # `canvas::dimdrag` — **dragging a ce dimension to where it should be drawn**
//!
//! ## The operator's report, verbatim
//!
//! > *"I need to be able to move the dimension after it has been laid down,
//! > and there should be a preview of the dimensioning lines as I lay it down
//! > and click to position it when it is created and after the fact."*
//!
//! Two halves. The first half — *as I lay it down* — already shipped:
//! [`crate::canvas::measure`]'s third click places a new ce dimension and
//! previews it through `measure::pick::dimension_preview_segments`. This
//! module is the second half, *after the fact*, and it reuses that same
//! preview function for the same reason: a preview derived a second way is a
//! preview that can disagree with what commits.
//!
//! ## ★★ What "move a dimension" means, and why it is NOT `move_dimension`
//!
//! `pdfcer-core` offers two verbs and picking the wrong one is the whole design
//! decision here:
//!
//! | verb | what it changes | what the number does |
//! |---|---|---|
//! | `EditSession::move_dimension` | translates the **measured points** with the drawing | unchanged (a rigid motion preserves a distance) — but the dimension leaves the feature it was measuring |
//! | `EditSession::place_dimension` | writes `offset` and `text_along` only | **cannot** change, by construction: the value function does not read either field |
//!
//! Dragging a dimension is `place_dimension`. That is what SolidWorks does —
//! the attachment points stay on the geometry and the extension lines stretch
//! — and it is what the engine's own doc comment says the verb exists for:
//! *"This, not `move_dimension`, is what dragging a dimension does."*
//!
//! The consequence worth stating out loud, because it is the property that
//! makes this gesture safe enough to be the *default* action on a press:
//! **no drag, however far, can alter the printed number.** An operator can drag
//! a dimension across the sheet and back and the document's measurements are
//! unchanged. `move_dimension` has no such guarantee — it would take a
//! dimension off the feature it annotates — so it is deliberately not wired to
//! a drag and remains available only where the operator has said they mean it.
//!
//! ## The delta is resolved in the dimension's OWN frame
//!
//! A page-space delta is projected onto the two axes `axis_frame` gives:
//!
//! ```text
//! offset'     = offset     + delta · n    (perpendicular — how far the line stands off)
//! text_along' = text_along + delta · u    (parallel      — where the number sits along it)
//! ```
//!
//! **A delta, not `placement_from_point`.** Both were available and the
//! absolute form is shorter, but it resolves the placement from wherever the
//! *pointer* is, which means the dimension jumps on the first frame of the drag
//! so that its anchor lands under the cursor. A delta preserves the grab: the
//! dimension moves exactly as far as the hand does, and whatever part of it the
//! operator grabbed stays under their finger. The absolute form is the right
//! one for authoring — where there is no grab to preserve, because the
//! dimension does not exist yet — and that is precisely where `canvas::measure`
//! uses it.
//!
//! ## Two kinds may be dragged, and a perimeter is the easier of them
//!
//! **Linear** resolves the delta in the dimension's own axis frame, above.
//! **Perimeter** does not have one — a shape has no single axis to build a
//! frame around — so the engine anchors its label at `centroid + (text_along,
//! offset)` in the **page's** axes, and the delta is the answer with no
//! projection at all.
//!
//! One consequence is worth naming rather than discovering: a perimeter's label
//! is **strictly more free** than a linear one's. Drag a linear label
//! diagonally and it is flattened onto its axis, because that is where a
//! dimension line's text lives; drag a perimeter's and it lands where you
//! dropped it. That is not an inconsistency to iron out — it is the difference
//! between a label that belongs to a line and one that belongs to a shape.
//!
//! ## Why an ANGULAR dimension may not (and why that is not a stub)
//!
//! `place_dimension` accepts angular dimensions too, but its two arguments mean
//! something different there: `offset` is an **arc radius** from the apex and
//! `text_along` is in **degrees**. Adding a dot product measured in points to a
//! quantity measured in degrees is not a smaller version of the right answer,
//! it is arithmetic on mismatched units, so [`placed`] refuses.
//!
//! The refusal is honoured at the *press*, not at the release: [`grab_box`]
//! returns `None` for anything it cannot drag, so the press falls through to
//! the ordinary marquee and no gesture is ever started that could not finish.
//! That is this project's no-placeholders invariant applied to a gesture rather
//! than to a widget — an inert drag is a visible control that silently does
//! nothing, which is the exact failure class `DEFECTS.md` is made of.
//!
//! Angular placement by drag is worth having and is written up rather than
//! faked; see the TODO note on [`placed`].
//!
//! ## What this module does NOT decide
//!
//! * **Whether the press is a drag at all.** `canvas::gesture` owns that. This
//!   module only supplies the hit box that makes the press mean *move*.
//! * **Where the dimension is drawn once committed.** `pdfcer-render` draws the
//!   annotation; this module draws only the in-flight preview, and only from
//!   the same segment function a committed dimension is previewed from.
//! * **Undo granularity.** `place_dimension` is one command, so one drag is one
//!   undo entry, decided by the engine.
//!
//! ## conventions: drag-moves
//!
//! The corpus is `ui-conventions/drag-moves.md`. Every row answered, because
//! the unanswered ones are the ones the operator finds.
//!
//! - D1 live-preview: the dimension follows the pointer from the first frame,
//!   drawn through `dimension_preview_segments` — the same function a committed
//!   dimension is drawn from. **This row failed twice.** The label drag never
//!   previewed (the arm was written and unreachable), and the vertex drag
//!   converted screen→canvas twice, so it tracked at `1/zoom` and sat off by the
//!   scroll origin. Both fixed 2026-08-20; see `drag_vertex`.
//! - D2 derived-from-commit: `placed` returns the geometry AND the two scalars
//!   the commit writes, so preview and commit are one calculation. A caller
//!   cannot draw one placement and commit another without going out of its way.
//! - D3 escape-cancels: WAIVED — the gesture machine owns Escape and drops the
//!   drag before this module is reached. Nothing is written until `Complete`, so
//!   an abandoned drag leaves the document untouched by construction.
//! - D4 one-undo-entry: `place_dimension` and `move_dimension_vertex` are each
//!   one engine command, so one gesture is one Ctrl+Z.
//! - D5 modifiers-constrain: **Shift locks both drags to one axis**, applied
//!   by `canvas::interact` before either reaches this module —
//!   [`crate::canvas::constrain::translate`] for the label, whose outcome is a
//!   delta, and `reposition` for a vertex, whose outcome is a position and
//!   which therefore filters the displacement from the press so the grab point
//!   survives (D8). A label held to its *standoff* or its *slide* specifically —
//!   the dimension-space pair rather than the page axes — is a further
//!   refinement and is not built; recorded as a gap rather than claimed.
//! - D6 snapping: **a vertex drag snaps**, as of 2026-08-20, through the same
//!   `snap_candidates` query and the same operator settings the measure tools
//!   use — [`crate::canvas::measure::snap_point`], which exists precisely so
//!   there is one answer to *"where would this land"* rather than two. Alt
//!   suspends it, exactly as it does for a pick, and the marker is drawn at the
//!   target before the release. **The LABEL drag still does not snap**, and
//!   that is deliberate rather than pending: a label's position is
//!   presentational, it changes no measured value, and snapping a caption to a
//!   wall would move it onto the drawing rather than clear of it. The old row
//!   read: a vertex drag does not snap, while the tool that
//!   PLACED that vertex does. So an operator can pick a corner onto geometry and
//!   then be unable to put it back. The sharpest of the gaps here.
//! - D7 no-op-is-not-an-edit: **GAP** — a zero-travel release still raises the
//!   action. The engine may collapse it; this module does not check.
//! - D8 grab-point: the vertex moves by the pointer's DELTA, so whatever part of
//!   the handle was grabbed stays under the cursor. The label drag has always
//!   been a delta, and its header carries the argument for why the absolute form
//!   is right for authoring and wrong for moving.
//! - D9 disclosure: `MoveVertex` re-measures and says so off-canvas, with the
//!   label before and after — the "before" cannot be reconstructed once the
//!   geometry that produced it is gone. `Place` writes fields the value function
//!   does not read, so it has nothing to disclose and says nothing.

use egui::{Rect, Vec2};
use pdfcer_core::dimension::{DimensionId, DimensionKind};
use pdfcer_core::page_tree::Page;
use pdfcer_core::vector::Point;

use crate::app::actions::Action;
use crate::app::actions::dimensions::DimensionAction;
use crate::app::state::OpenDoc;
use crate::canvas::gesture::Phase;
use crate::canvas::mapping::PageMapping;
use crate::canvas::selection::{AnnotKind, SelectionState};

/// The trace channel a driven check reads to prove a placement committed.
///
/// An in-flight placement and a committed one are the same screenshot at the
/// moment of release, which is defect 8's lesson: the harness needs a line that
/// distinguishes *"the preview followed the pointer"* from *"the release
/// reached the verb"*, and no pixel can carry that.
pub const TRACE: &str = "dimension-place"; // ui-text-exempt: diagnostic trace name

/// The dimension under the selection, if one is selected **and** it is a kind
/// this module can drag.
///
/// Returns the record's id together with its geometry, because every caller
/// needs both and a second lookup could resolve differently after an edit.
///
/// # Why the whole model is walked rather than indexed
///
/// An annotation selection carries an *object* id — the annotation in the file
/// — and a dimension record carries a [`DimensionId`]. The sidecar holds the
/// mapping one way only (`record.annot`), so the reverse lookup is a scan. It
/// is a scan over the dimensions on the document, which is a handful even on a
/// heavily dimensioned sheet, and it runs once per press rather than per frame.
#[must_use]
pub fn selected(doc: &OpenDoc, selection: &SelectionState) -> Option<(DimensionId, DimensionKind)> {
    let annot = selection.annot()?;
    if annot.target.kind != AnnotKind::CeDimension {
        return None;
    }
    let model = doc.session.dimension_model();
    let record = model
        .dimensions()
        .iter()
        .find(|r| r.annot == Some(annot.target.id))?;
    // The gate that keeps an un-draggable kind from ever starting a gesture.
    // See the module header: an angular dimension's placement is a radius and
    // an angle, and this module's delta is in points.
    //
    // Perimeter joined Linear on 2026-08-20, when the engine shipped the kind
    // and confirmed that `place_dimension` carries it *"with no new semantics
    // and no new fields"*.
    if !matches!(
        record.kind,
        DimensionKind::Linear { .. } | DimensionKind::Perimeter { .. }
    ) {
        return None;
    }
    Some((record.id, record.kind.clone()))
}

/// The screen-space box a press must land in to mean *move this dimension*.
///
/// The annotation's `/Rect`, projected. That is the same rectangle
/// `canvas::overlay::draw_selection` already strokes when a dimension is
/// selected, which is the property that matters: **the drawn outline and the
/// live target are the same shape.** An operator aims at what they can see.
///
/// # Why this is not `overlay::grip_box`
///
/// That function derives its box from the selection's cached content outlines,
/// which `select_annot` clears — an annotation is not content and has no
/// decomposed outline to cache. So `grip_box` answers `None` over a selected
/// dimension, which is why a press on one used to start a marquee and replace
/// the selection the operator was trying to act on. Keeping the two functions
/// separate rather than teaching `grip_box` about annotations keeps the resize
/// grips out of this: `grip_box`'s box is also what the eight scale handles are
/// laid out on, and a dimension has no scale verb.
#[must_use]
pub fn grab_box(doc: &OpenDoc, map: &PageMapping, selection: &SelectionState) -> Option<Rect> {
    selected(doc, selection)?;
    let annot = selection.annot()?;
    Some(map.rect_to_screen(annot.outline))
}

/// **The rule.** A page-space delta, applied in the dimension's own frame.
///
/// Returns the placed geometry together with the two scalars `place_dimension`
/// takes, so the preview and the commit are derived from one calculation rather
/// than two that could disagree. That pairing is the point of the return type:
/// a caller cannot draw one placement and commit another without going out of
/// its way.
///
/// `None` when the delta cannot be resolved — an **angular** dimension (see the
/// module header: its placement is a radius and an angle, and this delta is in
/// points), a **circular** one (which the engine refuses outright, having no
/// axis to place along), or a degenerate `Aligned` linear one whose two picks
/// coincide and which `axis_frame` refuses rather than fabricating.
///
/// # TODO — angular placement
///
/// `place_dimension` accepts an angular dimension, taking an arc radius and a
/// position in degrees. Dragging one is a genuinely different calculation
/// (radial distance from the apex; angle subtended) rather than this one with
/// different names, and it needs its own preview and its own tests. Filed
/// rather than approximated.
#[must_use]
pub fn placed(kind: &DimensionKind, dx: f64, dy: f64) -> Option<(DimensionKind, f64, f64)> {
    // ★★ A PERIMETER'S PLACEMENT IS IN PAGE AXES, AND THAT IS WHY IT NEEDS NO
    // PROJECTION.
    //
    // A linear dimension's `offset` and `text_along` are measured along its own
    // axis frame — the whole reason [`placed`]'s linear arm takes two dot
    // products. A perimeter has no single axis to have a frame around, so the
    // engine anchors its label at `centroid + (text_along, offset)` in the
    // PAGE's own axes, and says so:
    //
    // > *"It resolves with no projection at all — the pointer delta IS the
    // > answer, so unlike the linear case, dropping the label anywhere is
    // > expressible rather than flattened onto one axis."*
    //
    // So this arm is the delta, unchanged. `text_along` takes x and `offset`
    // takes y, which reads backwards until you remember that `offset` is
    // "away from the thing" and for a perimeter that direction is page +y by
    // definition rather than by derivation.
    //
    // ★ It is strictly MORE expressive than the linear case: a linear label
    // dragged diagonally is flattened onto its axis, and this one lands where
    // the operator dropped it. That is not an inconsistency to fix — it is the
    // difference between a label that belongs to a line and one that belongs to
    // a shape.
    if let DimensionKind::Perimeter {
        points,
        closed,
        offset,
        text_along,
    } = kind
    {
        let (offset, text_along) = (offset + dy, text_along + dx);
        return Some((
            DimensionKind::Perimeter {
                points: points.clone(),
                closed: *closed,
                offset,
                text_along,
            },
            offset,
            text_along,
        ));
    }
    let DimensionKind::Linear {
        a,
        b,
        constraint,
        offset,
        text_along,
    } = *kind
    else {
        return None;
    };
    let (u, n) = kind.axis_frame()?;
    let offset = offset + dx * n.x + dy * n.y;
    let text_along = text_along + dx * u.x + dy * u.y;
    Some((
        DimensionKind::Linear {
            a,
            b,
            constraint,
            offset,
            text_along,
        },
        offset,
        text_along,
    ))
}

/// Everything one frame of a placement drag needs, gathered at the call site.
pub struct Frame<'a> {
    /// How far the pointer has travelled since the press, in canvas space.
    pub delta: Vec2,
    /// Draw the preview, or commit the placement.
    pub phase: Phase,
    /// The page the dimension is on — needed to turn a canvas delta into a
    /// page-space one, which is the only place the y-flip is applied.
    pub page: Option<&'a Page>,
}

/// Advance one frame of a placement drag.
///
/// Returns the **page-space segments** the dimension would be drawn as, if the
/// operator released now, or `None` when the drag reaches no verb.
///
/// # The honesty contract, restated because it is the same one everywhere here
///
/// The preview is `Some` if and only if a release would commit, and it is
/// derived from the *same* [`placed`] result the commit uses. So the operator
/// cannot be shown a dimension standing off by 40 points and then get one
/// standing off by something else — the two numbers are literally the same
/// `f64`.
///
/// # Rule 4, and why this preview is allowed to exist at all
///
/// Rule 4 forbids marking *applied* content as provisional. This draws
/// something that has not been applied yet: it is the rubber-band of a drag in
/// flight, which the rule names explicitly as a pre-commit affordance — *"a
/// snap indicator, a hover highlight, a rubber-band … these are the cursor"*.
/// It disappears on release, and what replaces it is the annotation itself,
/// rendered by `pdfcer-render` with no marking of any kind.
pub fn drag(
    frame: Frame<'_>,
    doc: &OpenDoc,
    selection: &SelectionState,
    actions: &mut Vec<Action>,
) -> Option<Vec<(Point, Point)>> {
    let Frame { delta, phase, page } = frame;
    let (id, kind) = selected(doc, selection)?;
    let page = page?;
    let d = super::moving::page_delta(delta, page)?;
    let (moved, offset, text_along) = placed(&kind, d.dx, d.dy)?;

    if phase == Phase::Complete {
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "{TRACE} id={} offset={offset:.2} text_along={text_along:.2}",
                id.0
            )
        });
        actions.push(Action::Dimension(DimensionAction::Place {
            dimension: id,
            offset,
            text_along,
        }));
        // Nothing is previewed on the frame that commits: the annotation is
        // about to be regenerated and drawn for real, and a preview left on
        // screen over it would be a second copy of the same line, one frame
        // stale.
        return None;
    }
    Some(super::measure::pick::dimension_preview_segments(&moved))
}

// ===========================================================================
// Vertex editing — the perimeter's corners
// ===========================================================================

/// The screen-space size of a vertex handle, in points.
///
/// The same 7 pt the Bézier handles use (`canvas::handles::GRIP_SIZE_PX`'s
/// neighbourhood), because they are the same affordance to a hand: a small
/// square you grab. Zoom-invariant — it is a screen-space control, so it does
/// not grow with magnification, and a corner on a plan at 20 % is as grabbable
/// as one at 400 %.
pub const VERTEX_HANDLE_PT: f32 = 7.0;

/// The trace region prefix each vertex handle is published under, suffixed with
/// its index — `canvas.dimension-vertex.0`, `.1`, …
///
/// Published by the painter so a driven check can aim at a corner. See its call
/// site for why a harness must never guess this.
pub const VERTEX_REGION: &str = "canvas.dimension-vertex"; // ui-text-exempt: trace region name

/// How much slack a press gets around a vertex handle, in points.
///
/// The drawn square is the promise; this is the target. They differ because a
/// 7 pt square is a hard thing to hit with a mouse on a dense drawing, and the
/// standing convention here — stated at `handles::grip_at` — is that a grip's
/// live area may exceed its drawn one, never the reverse. A target smaller than
/// its picture is the operator missing something they can see.
const VERTEX_GRAB_SLACK_PT: f32 = 3.0;

/// **Every vertex of the selected perimeter, in CANVAS space**, in index order.
///
/// Empty for every other selection and for every other dimension kind, which is
/// what makes both the painter and the hit test one call with no branch of
/// their own.
#[must_use]
pub fn vertices(doc: &OpenDoc, selection: &SelectionState) -> Vec<egui::Pos2> {
    let Some((_, kind)) = selected(doc, selection) else {
        return Vec::new();
    };
    let Some((points, _)) = kind.polyline() else {
        return Vec::new();
    };
    let Some(page) = doc.pages.get(doc.view.page_index) else {
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

/// **Which vertex a press at `screen` landed on**, if any.
///
/// # ★ The comparison is in SCREEN space, and that is the whole of why this
/// function converts rather than the caller
///
/// A handle is a screen-space affordance of a fixed size. Comparing in canvas
/// or page space would make the target shrink as the operator zooms out —
/// exactly when a plan's corners are closest together and precision matters
/// most — and balloon as they zoom in, so that at 800 % a press anywhere near a
/// corner would grab it. The conversion has to happen on the side of the
/// boundary where the tolerance is meaningful, and that is here.
///
/// # Ties go to the LAST vertex, deliberately
///
/// Two coincident vertices are legal — [`super::measure::perimeter`] does not
/// de-duplicate, on the argument that a repeated point is invisible rather than
/// wrong. If the operator has made one and wants it gone, the one they can
/// reach is the one they can drag away, and the later index is the one they
/// just placed.
#[must_use]
pub fn vertex_at(
    doc: &OpenDoc,
    map: &PageMapping,
    selection: &SelectionState,
    screen: egui::Pos2,
) -> Option<usize> {
    let tolerance = VERTEX_HANDLE_PT / 2.0 + VERTEX_GRAB_SLACK_PT;
    vertices(doc, selection)
        .into_iter()
        .enumerate()
        .rfind(|(_, canvas)| map.to_screen(*canvas).distance(screen) <= tolerance)
        .map(|(index, _)| index)
}

/// Advance one frame of a **vertex** drag.
///
/// Returns the page-space segments the shape would be drawn as if the operator
/// released now, or `None` when the drag reaches no verb.
///
/// # ★★ This one RE-MEASURES, and that is the difference from every other
/// gesture in this module
///
/// [`drag`] writes `offset` and `text_along` — two fields the value function
/// does not read — so no label drag can alter the printed number. This one
/// moves a corner of the measured shape, so it changes the number **by
/// design**. The engine says so plainly: `move_dimension_vertex` is *"the first
/// ce-dimension verb that deliberately changes what a ce dimension measures"*.
///
/// The consequence for this shell is a disclosure obligation the label drag does
/// not have. `VertexOutcome` carries `previous_label` and `label` precisely
/// because **the old value cannot be reconstructed afterwards** — the geometry
/// it came from is gone — and a status line reading `12.40 m → 13.85 m` is a
/// disclosure where one reading `13.85 m` is just the number the operator can
/// already see on the page.
///
/// # No guard, no probe, no first-move check — the engine ruled on it
///
/// I asked whether the verb could refuse mid-drag, so that the preview could be
/// withheld. The answer was that it cannot, and it was a ruling rather than an
/// omission: a self-intersecting polyline has a perfectly well-defined total
/// length (a figure-eight is a real fence run), and a zero-length segment
/// contributes 0.0 and disappears the moment the vertex moves again. Every
/// remaining refusal is structural and knowable before the drag begins.
///
/// **⇒ Draw the preview. Always.**
pub fn drag_vertex(frame: VertexFrame<'_>, actions: &mut Vec<Action>) -> VertexDrag {
    let VertexFrame {
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
    inner(
        ctx, index, from, at, phase, doc, selection, targets, map, alt_held, actions,
    )
    .unwrap_or_default()
}

/// What one frame of a vertex drag needs, gathered at the call site.
///
/// A struct rather than eleven parameters — `canvas::resizing::Frame`'s own
/// argument, and this one crossed clippy's arity limit the moment snapping
/// arrived. Three of the members are `Option`s of borrowed things and two are
/// `Pos2`s in the same space, both of which a positional list would let a
/// caller swap silently.
pub struct VertexFrame<'a> {
    /// The frame's context. Read only — the snap settings live in it.
    pub ctx: &'a egui::Context,
    /// Which vertex, sampled at the press.
    pub index: usize,
    /// Where the press landed, in canvas space — the grab point.
    pub from: egui::Pos2,
    /// Where the pointer is now, in canvas space.
    pub at: egui::Pos2,
    /// Draw, or commit.
    pub phase: Phase,
    /// The open document.
    pub doc: &'a OpenDoc,
    /// The current selection, which is what names the perimeter.
    pub selection: &'a SelectionState,
    /// The decomposition, for the snap query. `None` means no snapping this
    /// frame rather than an error — see [`crate::canvas::measure::snap_point`].
    pub targets: Option<&'a dyn crate::canvas::target::CanvasTargetProvider>,
    /// The frame's mapping, which owns the snap tolerance in page units.
    pub map: &'a PageMapping,
    /// Whether Alt is down **this frame** — the operator saying *"not this
    /// time"*, and the same override a measure pick honours.
    pub alt_held: bool,
}

/// What one frame of a vertex drag produced.
///
/// ★ Two fields rather than one, because the preview and the snap indicator are
/// different pictures with different lifetimes: the polyline is drawn in page
/// space through the dimension painter, and the marker is a screen-space glyph
/// at the candidate. Folding them would make the caller unpack a tuple whose
/// members it uses in two different places, forty lines apart.
#[derive(Default)]
pub struct VertexDrag {
    /// The polyline the release would commit, as page-space segments, or `None`
    /// when this frame previews nothing.
    pub segments: Option<Vec<(Point, Point)>>,
    /// What the corner is snapping to, if anything.
    ///
    /// ★ `drag-moves` D6: *"a snap is an inference. It is announced by an
    /// indicator at the target while the drag is live — never applied
    /// silently."* This is what the painter draws that indicator from, and it
    /// is the **same candidate** the release commits — one derivation, which is
    /// the rule `measure::Resolved` exists to enforce and the reason a snap
    /// marker once sat away from the point it described for four days.
    pub snap: Option<pdfcer_core::vector::snap::SnapCandidate>,
}

#[allow(clippy::too_many_arguments)]
fn inner(
    ctx: &egui::Context,
    index: usize,
    from: egui::Pos2,
    at: egui::Pos2,
    phase: Phase,
    doc: &OpenDoc,
    selection: &SelectionState,
    targets: Option<&dyn crate::canvas::target::CanvasTargetProvider>,
    map: &PageMapping,
    alt_held: bool,
    actions: &mut Vec<Action>,
) -> Option<VertexDrag> {
    let (id, kind) = selected(doc, selection)?;
    let (points, closed) = kind.polyline()?;
    let page = doc.pages.get(doc.view.page_index)?;
    let old = *points.get(index)?;

    // ★★★ `from` and `at` are ALREADY CANVAS SPACE — the gesture machine says
    // so on the variant, and converting them again was the operator's bug of
    // 2026-08-20:
    //
    // > *"as soon as I click one, the preview of the dragging of it is offset
    // > from the mouse and moves at a different speed than my mouse movements,
    // > so the distance from the pointer varies as you move it."*
    //
    // This read `map.to_page(at)` first — the SCREEN -> canvas hop — applied to
    // a value that had already had it. So the corner tracked at `1/zoom` of the
    // pointer's speed and sat off by the scroll origin. `canvas::handledrag`
    // does the identical job correctly in one hop, eleven lines long, in the
    // module next door.
    //
    // ★ This is the second instance in this codebase and both were written by
    // somebody who had read the first one's post-mortem. `egui::Pos2` is screen,
    // canvas AND page space, so the compiler cannot object. The durable fix is
    // typed coordinates, not care — see `drag-moves` D1a.
    //
    // ★★ And the GRAB POINT is preserved (D8): the vertex moves by the
    // pointer's DELTA, not to the pointer's position. Assigning the pointer
    // straight to the vertex teleports the corner under the cursor on the first
    // frame, so an operator who grabbed a handle three pixels off centre sees
    // the shape jump before they have moved anything.
    let grab = at - from;
    let was = crate::viewer::pdf_space_to_canvas(
        #[allow(clippy::cast_possible_truncation)]
        egui::Pos2::new(old.x as f32, old.y as f32),
        page,
    )?;
    let new = crate::viewer::canvas_to_pdf_space(was + grab, page)?;

    let mut moved: Vec<Point> = points.to_vec();
    #[allow(clippy::cast_lossless)]
    let free = Point::new(f64::from(new.x), f64::from(new.y));

    // ★★ THE SNAP, and it deliberately OVERRIDES the grab point.
    //
    // D8 (the grab point is preserved) and D6 (snapping) pull in opposite
    // directions here, and every program in the class resolves it the same way:
    // **snapping wins.** The whole content of the gesture is landing the corner
    // exactly on something, and preserving a three-pixel grab offset would put
    // it exactly three pixels off the thing it snapped to — which is a corner
    // that looks snapped and is not, the worst of the three outcomes.
    //
    // The grab is still what decides *which* candidate is near, because `free`
    // is computed from the delta above; it is only the final placement that
    // yields.
    //
    // ★ The same query, the same tolerance and the same operator settings the
    // measure tools use. See `measure::snap_point` for why that is one function
    // and not two.
    let (target, snap) =
        crate::canvas::measure::snap_point(ctx, doc.view.page_index, free, alt_held, targets, map);
    *moved.get_mut(index)? = target;

    if phase == Phase::Complete {
        let (dx, dy) = (target.x - old.x, target.y - old.y);
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            //
            // ★ `snap=` carries the candidate KIND, not a boolean. A wrong
            // build that snapped to the nearest thing of any sort still reports
            // `snap=1`; one that reports `snap=Endpoint` when the operator was
            // over a midpoint is telling a driven check something a bool
            // cannot. `resize-commit`'s own note makes the same argument: a
            // trace line must carry the number a wrong build would get wrong.
            format!(
                "dimension-vertex id={} index={index} dx={dx:.2} dy={dy:.2} snap={}",
                id.0,
                snap.map_or_else(|| "none".to_owned(), |c| format!("{:?}", c.kind))
            )
        });
        actions.push(Action::Dimension(DimensionAction::MoveVertex {
            dimension: id,
            index,
            dx,
            dy,
        }));
        // Nothing is previewed on the frame that commits — the dimension is
        // about to be regenerated and drawn for real — and the marker goes with
        // it, because a snap indicator over a snap that has already happened is
        // describing the past.
        return Some(VertexDrag::default());
    }

    // The preview, through the same segment function a committed perimeter is
    // drawn from — this module's standing rule, and `measure::pick` supplies
    // the closing segment for a ring rather than this call site guessing at it.
    Some(VertexDrag {
        segments: Some(super::measure::pick::dimension_preview_segments(
            &DimensionKind::Perimeter {
                points: moved,
                closed,
                offset: 0.0,
                text_along: 0.0,
            },
        )),
        snap,
    })
}

/// **Every ce dimension's drawn ink on the current page**, in canvas space,
/// keyed by its annotation id.
///
/// # Why this exists: a bounding box is not a shape
///
/// A click selects what is under the cursor, not what merely encompasses it.
/// A ce dimension's `/Rect` is the box around two witness lines, a dimension
/// line, two arrowheads and a label — mostly empty air for anything but a
/// perfectly horizontal one, and for a perimeter traced round a building it is
/// the entire footprint. Hit-testing that box meant the operator could not
/// select the drawing underneath their own dimensions.
///
/// ★ The segments come from `measure::pick::dimension_preview_segments` — **the
/// same function the dimension is previewed and drawn from**. That is this
/// module's standing rule applied to hit testing: what is clickable and what is
/// visible are one derivation, so they cannot drift apart. A second "where is
/// the ink" calculation would be a second thing to keep right.
///
/// # What it costs, and why it is per-click rather than cached
///
/// One pass over the sidecar's dimension records, projecting each one's
/// segments. A heavily dimensioned sheet carries tens of these, not thousands —
/// the 129,758 objects on the benchmark drawing are page CONTENT, and none of
/// them is here. It runs on a click, not on a frame.
///
/// The label is deliberately NOT included. It is drawn by `pdfcer-render` from
/// the appearance stream and this shell does not know its box; a dimension is
/// selected by its lines, which is the part an operator points at. If that
/// proves too strict in use, the fix is to ask the engine for the label's box
/// rather than to guess one here.
#[must_use]
pub fn annot_shapes(
    doc: &OpenDoc,
    ce_dimensions: &std::collections::BTreeSet<pdfcer_core::object::ObjId>,
) -> std::collections::BTreeMap<pdfcer_core::object::ObjId, Vec<(egui::Pos2, egui::Pos2)>> {
    let mut out = std::collections::BTreeMap::new();
    let Some(page) = doc.pages.get(doc.view.page_index) else {
        return out;
    };
    let model = doc.session.dimension_model();
    for record in model.dimensions() {
        let Some(annot) = record.annot else { continue };
        if !ce_dimensions.contains(&annot) {
            continue;
        }
        let segments: Vec<(egui::Pos2, egui::Pos2)> =
            super::measure::pick::dimension_preview_segments(&record.kind)
                .into_iter()
                .filter_map(|(a, b)| {
                    #[allow(clippy::cast_possible_truncation)]
                    let to_canvas = |p: Point| {
                        crate::viewer::pdf_space_to_canvas(
                            egui::Pos2::new(p.x as f32, p.y as f32),
                            page,
                        )
                    };
                    Some((to_canvas(a)?, to_canvas(b)?))
                })
                .collect();
        // Empty means "this kind reports no segments" — a circular dimension
        // today. Left OUT of the map rather than inserted empty, so the caller
        // falls back to the rectangle: an annotation nothing can claim is
        // unselectable, which is worse than one that claims too much.
        if !segments.is_empty() {
            out.insert(annot, segments);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfcer_core::vector::AxisConstraint;

    fn horizontal() -> DimensionKind {
        DimensionKind::Linear {
            a: Point::new(100.0, 200.0),
            b: Point::new(300.0, 200.0),
            constraint: AxisConstraint::Horizontal,
            offset: 0.0,
            text_along: 0.0,
        }
    }

    /// A horizontal dimension's frame is `u = +x`, `n = +y`. So a drag straight
    /// up is pure standoff and a drag sideways is pure text slide — the two
    /// components do not contaminate each other.
    #[test]
    fn a_delta_splits_into_standoff_and_slide_along_the_axis() {
        let (_, offset, along) = placed(&horizontal(), 0.0, 30.0).expect("linear places");
        assert!((offset - 30.0).abs() < 1e-9, "straight up is all standoff");
        assert!(along.abs() < 1e-9, "and none of it slides the text");

        let (_, offset, along) = placed(&horizontal(), 25.0, 0.0).expect("linear places");
        assert!(offset.abs() < 1e-9, "sideways changes no standoff");
        assert!(
            (along - 25.0).abs() < 1e-9,
            "and slides the text by the drag"
        );
    }

    /// ★ The property the whole design rests on: **placement never touches what
    /// is measured.** Whatever the drag, `a` and `b` come out unchanged, so the
    /// printed number cannot move.
    #[test]
    fn no_drag_can_move_the_measured_points() {
        let before = horizontal();
        for (dx, dy) in [(0.0, 0.0), (500.0, -900.0), (-12.5, 7.25), (1e6, 1e6)] {
            let (after, _, _) = placed(&before, dx, dy).expect("linear places");
            let (DimensionKind::Linear { a, b, .. }, DimensionKind::Linear { a: a0, b: b0, .. }) =
                (&after, &before)
            else {
                panic!("both are linear");
            };
            assert!(
                (a.x - a0.x).abs() < 1e-9 && (a.y - a0.y).abs() < 1e-9,
                "point a moved on a drag of {dx},{dy}"
            );
            assert!(
                (b.x - b0.x).abs() < 1e-9 && (b.y - b0.y).abs() < 1e-9,
                "point b moved on a drag of {dx},{dy}"
            );
        }
    }

    /// Placement accumulates: two drags of ten leave the dimension where one
    /// drag of twenty would. The delta form is what makes this true — an
    /// absolute `placement_from_point` would put it wherever the pointer last
    /// was, which is a different gesture.
    #[test]
    fn two_drags_compose_into_one() {
        let (once, _, _) = placed(&horizontal(), 0.0, 10.0).expect("places");
        let (twice, offset, _) = placed(&once, 0.0, 10.0).expect("places");
        let (_, direct, _) = placed(&horizontal(), 0.0, 20.0).expect("places");
        assert!((offset - direct).abs() < 1e-9);
        let DimensionKind::Linear { offset: o, .. } = twice else {
            panic!("linear")
        };
        assert!((o - 20.0).abs() < 1e-9);
    }

    /// An aligned dimension whose two picks coincide has no axis, so there is
    /// nothing to resolve a delta against. Refused rather than fabricated — see
    /// `axis_frame`, which makes the same call one level down.
    #[test]
    fn a_degenerate_dimension_has_no_frame_and_is_refused() {
        let degenerate = DimensionKind::Linear {
            a: Point::new(50.0, 50.0),
            b: Point::new(50.0, 50.0),
            constraint: AxisConstraint::Aligned,
            offset: 0.0,
            text_along: 0.0,
        };
        assert!(placed(&degenerate, 5.0, 5.0).is_none());
    }
    fn square_perimeter() -> DimensionKind {
        DimensionKind::Perimeter {
            points: vec![
                Point::new(0.0, 0.0),
                Point::new(100.0, 0.0),
                Point::new(100.0, 100.0),
                Point::new(0.0, 100.0),
            ],
            closed: true,
            offset: 0.0,
            text_along: 0.0,
        }
    }

    /// ★★ **A perimeter's label goes where it is dropped**, in both axes.
    ///
    /// The property that separates it from a linear dimension, and the one the
    /// engine went out of its way to point out: a perimeter's placement is in
    /// PAGE axes, so a diagonal drag is expressible. A linear dimension's
    /// diagonal drag is flattened onto its own axis, which is correct there and
    /// would be wrong here.
    #[test]
    fn a_perimeter_label_takes_the_delta_in_both_axes() {
        let (_, offset, along) = placed(&square_perimeter(), 25.0, -40.0).expect("places");
        assert!((along - 25.0).abs() < 1e-9, "x goes to text_along");
        assert!((offset + 40.0).abs() < 1e-9, "y goes to offset");
    }

    /// ★ And the same guarantee the linear case has, which is the whole reason
    /// this gesture is safe to be the default: **the measured shape is never
    /// touched**, so the number cannot change no matter where the label lands.
    #[test]
    fn no_drag_moves_a_perimeter_vertex() {
        let before = square_perimeter();
        let (after, _, _) = placed(&before, 900.0, -700.0).expect("places");
        let (
            DimensionKind::Perimeter { points, closed, .. },
            DimensionKind::Perimeter {
                points: p0,
                closed: c0,
                ..
            },
        ) = (&after, &before)
        else {
            panic!("both are perimeters");
        };
        assert_eq!(points.len(), p0.len());
        for (a, b) in points.iter().zip(p0) {
            assert!((a.x - b.x).abs() < 1e-9 && (a.y - b.y).abs() < 1e-9);
        }
        assert_eq!(closed, c0, "and the ring is still a ring");
    }
}
