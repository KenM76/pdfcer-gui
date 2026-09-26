//! # `canvas::overlay` — what the selection looks like, and what it must never look like
//!
//! ## Rule 4 is the whole design constraint of this file
//!
//! `D:\Dev\FeatureRequests\pdfce_FeatureRequests\README.md`, second and fourth
//! clauses of the disclosure rule:
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/overlay.md`.

use egui::{Color32, CornerRadius, Painter, Rect, Stroke, StrokeKind, Visuals};

use crate::canvas::handles;
use crate::canvas::mapping::PageMapping;
use crate::canvas::selection::{SelectionLevel, SelectionState};

/// The anchor marks and the Bézier handles.
pub mod anchors;

pub use anchors::{
    ANCHOR_PX, HANDLE_PX, MAX_UNSELECTED_ANCHORS, PUBLISHED_ANCHORS, anchor_region, draw_anchors,
    draw_handles, handle_region,
};

/// The travelling copy of the selection's own pixels.
///
/// Re-exported below so `canvas/` writes `overlay::draw_raster_ghost` and
/// nothing outside this module learns that it has a file of its own.
mod raster;

pub(super) use raster::{draw_raster_ghost, raster_ghost_is_owed};

/// The region the selection's grip box publishes.
pub const SELECTION_OUTLINE_REGION: &str = "canvas.selection-outline"; // ui-text-exempt: trace region name, never displayed

/// The minimum on-screen extent, in egui logical points, that a selection
/// outline is guaranteed to have on each axis.
pub const MIN_OUTLINE_EXTENT_PX: f32 = 6.0;

/// Grow a degenerate outline rect, about its own centre, until it is at least
/// `min_extent` on both axes — **the fix for a selection that is correct and
/// paints nothing.**
#[must_use]
pub fn visible_outline_rect(rect: Rect, min_extent: f32) -> Rect {
    if !rect.min.x.is_finite()
        || !rect.min.y.is_finite()
        || !rect.max.x.is_finite()
        || !rect.max.y.is_finite()
        || !min_extent.is_finite()
        || min_extent <= 0.0
    {
        return rect;
    }
    // Normalise: the canvas→screen projection is handed rects the provider
    // built by bounding a mapped quad, so `min` is not guaranteed to be the
    // smaller corner by the time it arrives.
    let rect = Rect::from_two_pos(rect.min, rect.max);
    let grow = |lo: f32, hi: f32| -> (f32, f32) {
        let extent = hi - lo;
        if extent >= min_extent {
            return (lo, hi);
        }
        let pad = (min_extent - extent) / 2.0;
        (lo - pad, hi + pad)
    };
    let (x0, x1) = grow(rect.min.x, rect.max.x);
    let (y0, y1) = grow(rect.min.y, rect.max.y);
    Rect::from_min_max(egui::pos2(x0, y0), egui::pos2(x1, y1))
}

/// The screen-space box the grips are laid out on, or `None` when nothing is
/// selected.
#[must_use]
pub fn grip_box(mapping: &PageMapping, selection: &SelectionState) -> Option<Rect> {
    let union = selection.outline_union()?;
    Some(visible_outline_rect(
        mapping.rect_to_screen(union),
        MIN_OUTLINE_EXTENT_PX,
    ))
}

/// **The box a GHOST is measured against** — annotation first, then page
/// content, in the same order [`draw_move_ghost`] has always used.
#[must_use]
pub fn ghost_box(
    mapping: &PageMapping,
    selection: &SelectionState,
    widget: Option<Rect>,
) -> Option<Rect> {
    if let Some(screen) = widget {
        return Some(visible_outline_rect(screen, MIN_OUTLINE_EXTENT_PX));
    }
    if let Some(annot) = selection.annot() {
        return Some(visible_outline_rect(
            mapping.rect_to_screen(annot.outline),
            MIN_OUTLINE_EXTENT_PX,
        ));
    }
    grip_box(mapping, selection)
}

/// Trace region for one rectangle of the resize preview, published per
/// outline so a driven check can assert the ghost tracks the drag.
const RESIZE_GHOST_REGION: &str = "canvas-resize-ghost"; // ui-text-exempt: trace region name, never displayed

/// The move ghost's alpha, out of 255.
const GHOST_ALPHA: u8 = 150;

/// Paint the selection: one outline per entry, plus the grips.
pub fn draw_selection(
    painter: &Painter,
    visuals: &Visuals,
    mapping: &PageMapping,
    selection: &SelectionState,
    // The whole answer, not just its grip set — `OPERATOR_REQUESTS.md` O69.
    // The caller already holds it and was narrowing it to one field; the
    // outline needs a second, and re-deriving that second one here would be a
    // predicate spelled in two places. See `Grabbable::outline`.
    grab: crate::canvas::pressing::Grabbable,
) {
    if selection.is_empty() {
        return;
    }
    let stroke = Stroke::new(1.5, ink(painter));

    // The selected ANNOTATION, if the selection is one.
    //
    // The **same stroke** as a content outline, deliberately. An operator does
    // not need to be taught that pdfcer distinguishes a `/Annots` entry from a
    // content object — they clicked a stamp and the stamp is now selected, and
    // a second visual language for that would be a distinction the *implementer*
    // finds interesting. What is selected is said in words, off-canvas, where
    // rule 4 puts every other disclosure.
    //
    // The grips around a selected annotation, painted here so the predicate
    // that paints them is the one that hit-tests them (H7).
    //
    // **`offer` IS THAT PREDICATE, AND IT IS PASSED IN RATHER THAN
    // RE-DERIVED HERE.** Three annotation kinds offer three different sets:
    //
    // | selected | painted | hit-tested |
    // |---|---|---|
    // | markup | eight squares **and** the circle | `GripSet::all()` |
    // | ce dimension | the circle **only** | `GripSet::rotate_only()` |
    // | form field | eight squares only | `GripSet::scale_only()` |
    //
    // Spelling that table out locally would be a second copy of it, and the day
    // two copies disagree the symptom is either a handle nobody can grab or —
    // worse — an invisible target that steals the press aimed at what is under
    // it. A handle painted from the selection but hit-tested behind a
    // capability the mode does not have is visible and untouchable in exactly
    // the mode that needs it.
    //
    // ⇒ One value, one decision, two consumers. `canvas::painting` asks
    // `pressing::grabbable` once and hands the answer to both.
    if let Some(annot) = selection.annot() {
        let screen =
            visible_outline_rect(mapping.rect_to_screen(annot.outline), MIN_OUTLINE_EXTENT_PX);
        // **THE OUTLINE IS DRAWN AT THE MARK'S OWN ANGLE** —
        // `OPERATOR_REQUESTS.md` O147: *"the box outlined when an
        // object is selected should be in the same angled orientation as the
        // object."*
        //
        // `annot.oriented` is `None` for everything that is not turned, so the
        // line below this is the code it has always been for the overwhelming
        // majority of selections. When it is `Some`, `/Rect` is **not** where
        // the mark is: §12.5.2 requires that rectangle upright, so it bounds a
        // turned mark rather than describing it, so an upright outline swells
        // around artwork that has not changed size. The outline follows the
        // mark instead; the discrepancy is not something to explain in words,
        // it is something not to draw.
        //
        // The published region stays the UPRIGHT bound, deliberately. Every
        // driven check that aims at this selection derives grips and offsets
        // from it, and a region that changed shape with the annotation's angle
        // would break each of them for a fact none of them is asking about.
        // `canvas.selection-angle` below is where the angle is stated, in the
        // trace, where a machine can read it without a screenshot.
        let frame = annot
            .oriented
            .map_or(crate::canvas::handles::GripFrame::Upright(screen), |quad| {
                crate::canvas::handles::GripFrame::Turned(quad.map(|p| mapping.to_screen(p)))
            });
        match frame {
            crate::canvas::handles::GripFrame::Turned(corners) => {
                for i in 0..4 {
                    painter.line_segment([corners[i], corners[(i + 1) % 4]], stroke);
                }
            }
            crate::canvas::handles::GripFrame::Upright(box_) => {
                painter.rect_stroke(box_, CornerRadius::ZERO, stroke, StrokeKind::Middle);
            }
        }
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            //
            // It carries `turned=1|0` **and the corners**, because a build
            // that drew the upright box and one that drew a quad which happened
            // to coincide with it are the same screenshot — and the second is
            // the one a broken projection produces. A check reading only
            // `turned=` would pass on a quad drawn in the wrong place.
            format!(
                "canvas-selection-angle turned={} corners={:?}",
                u8::from(matches!(
                    frame,
                    crate::canvas::handles::GripFrame::Turned(_)
                )),
                frame.corners_for_trace()
            )
        });
        // Published under the SAME region name the content selection uses,
        // so a driven check aiming at a grip reads one name whatever is
        // selected. `handles::grip_rects` derives all eight from this box, so a
        // harness that has it cannot disagree with the application about where
        // they are.
        crate::diag::ui_rect(SELECTION_OUTLINE_REGION, screen);
        // Whatever `offer` says, and nothing else. A **ce dimension** gets
        // the rotate handle and none of the eight — its extent IS its
        // measurement, so pdfcer has no verb that scales one and declines to
        // grow one; a rotation is an isometry, so the number is identical
        // either side of it and turning one is a legitimate drafting
        // operation. A **locked** annotation (§12.5.3 bit 8) and every
        // annotation kind this shell cannot address get `GripSet::default()`
        // — no box at all from `grabbable`, so nothing is painted and nothing
        // is grabbable. **R9**: rendering nothing is the honest answer for a
        // capability that does not exist.
        draw_grips_in(painter, visuals, frame, grab.offer);
        return;
    }

    // **THE BOX IS NOT DRAWN OVER THE NODES** — `OPERATOR_REQUESTS.md`
    // O69: *"If we are at a point where we are showing the nodes in an
    // editable state there shouldn't be a bounding box around the objects."*
    //
    // At the Part and Node rungs `selection::outline_rect` returns the entered
    // SUBPATH's bounding box, and it was stroked on top of that subpath's own
    // anchors — so the operator got a rectangle around the thing whose points
    // he was trying to see. The eight grips were already correctly withheld
    // there (`GripSet::default()`); the outline was the half nobody had gated.
    //
    // Traced rather than only changed, because "no box" and "no selection"
    // are the same screenshot. `canvas-outline` is a new first token — checked
    // against `tools/gates/check-trace-names.py`, which matches on first
    // tokens, and deliberately not `canvas-selection`, which
    // `canvas::clicking` already owns.
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            "canvas-outline drawn={} entries={}",
            grab.outline,
            selection.outlines().len()
        )
    });
    for (_, page_rect) in
        selection
            .outlines()
            .iter()
            .take(if grab.outline { usize::MAX } else { 0 })
    {
        let screen =
            visible_outline_rect(mapping.rect_to_screen(*page_rect), MIN_OUTLINE_EXTENT_PX);
        painter.rect_stroke(screen, CornerRadius::ZERO, stroke, StrokeKind::Middle);
    }

    if let Some(box_) = grip_box(mapping, selection) {
        // Published so a driven check can AIM AT A GRIP.
        //
        // It is the difference between a check that measures the feature and
        // one that measures the harness's guesswork: a grip sits at a corner
        // of this box, and the
        // box's extent is a fact only the application knows. A check that
        // guessed "a few pixels down and right of where I clicked" would land
        // inside the object on any shape larger than a grip — which is a MOVE
        // drag, and the check would then pass while exercising the wrong
        // gesture entirely.
        //
        // The name is the SELECTION's rather than the grips', because the box
        // is what is published: `handles::grip_rects` derives all eight from
        // it, so a harness that has this rect has every grip and cannot
        // disagree with the application about where they are.
        crate::diag::ui_rect(SELECTION_OUTLINE_REGION, box_);
        // The SAME `offer` the hit test was given, which for page content is
        // `GripSet::all()` at the Object rung and `GripSet::default()` at every
        // inner one. Deliberately NOT re-spelled here as
        // `selection.level() == SelectionLevel::Object`, for the reason the
        // annotation branch above gives: two spellings of one predicate are one
        // refactor away from disagreeing, and when they disagree a handle is
        // either painted and not hit-tested — the "visible control, silently
        // inert" failure — or hit-tested and not painted, which is worse,
        // because it is an invisible target that steals the press aimed at the
        // anchor underneath it.
        draw_grips(painter, visuals, box_, grab.offer);
    }
}

/// Paint the grips `offer` says a screen-space box has.
pub fn draw_grips(
    painter: &Painter,
    visuals: &Visuals,
    bounds: Rect,
    offer: crate::canvas::handles::GripSet,
) {
    draw_grips_in(
        painter,
        visuals,
        crate::canvas::handles::GripFrame::Upright(bounds),
        offer,
    );
}

/// [`draw_grips`] in an arbitrary [`handles::GripFrame`].
pub fn draw_grips_in(
    painter: &Painter,
    visuals: &Visuals,
    frame: crate::canvas::handles::GripFrame,
    offer: crate::canvas::handles::GripSet,
) {
    let bounds = frame.bounds();
    let ink = ink(painter);
    let stroke = Stroke::new(1.0, ink);
    if offer.resize {
        for (_, rect) in handles::grip_rects_in(frame) {
            painter.rect(
                rect,
                CornerRadius::ZERO,
                visuals.window_fill,
                stroke,
                StrokeKind::Middle,
            );
        }
    }
    // …and the rotate handle, which is a CIRCLE ON A STEM and not a ninth
    // square.
    //
    // Every square on this canvas resizes, so a shape that resized in one place
    // and rotated in another would be a private convention the operator has to
    // learn — `handles.md` H2's stated failure mode. The stem is what says the
    // handle belongs to this box; without it the circle reads as an unrelated
    // dot floating over the page.
    //
    // Gated **separately** from the eight, which is the whole reason
    // `GripSet` has two fields. On a selected **ce dimension** this is the only
    // thing drawn: there are no squares at all, because there is no verb that
    // scales one and `pdfcer-core` has declined to build one — *"either the
    // displayed value stays fixed while the geometry grows, so the dimension
    // lies about the drawing; or both change, so nothing was measured"*. A lone
    // circle on a stem over a dimension is therefore the correct picture rather
    // than an incomplete one.
    if offer.rotate {
        let handle = handles::rotate_rect_in(frame);
        let centre = handle.center();
        // The stem runs from the frame's own top edge, not from the page's.
        // On a turned frame those are different points, and a stem drawn to the
        // page's top edge would cross the mark diagonally at any angle past a
        // few degrees — reading as a line through the object rather than as the
        // handle's tether.
        let foot = frame.anchor(crate::canvas::handles::Grip::North);
        let _ = bounds;
        painter.line_segment([foot, centre], Stroke::new(1.0, ink));
        painter.circle(
            centre,
            handles::GRIP_SIZE_PX / 2.0,
            visuals.window_fill,
            stroke,
        );
        // **Published so a driven check can aim at the ninth handle.**
        //
        // The eight are derivable from `SELECTION_OUTLINE_REGION` — they sit on
        // its corners and edge midpoints — and the rotate handle is **not**: it
        // is offset by `ROTATE_STEM_PX`, a number the harness would have to
        // duplicate and could get wrong silently. `checks/rotate.rs` mirrors
        // that constant today and says in its own comment that it *"does not
        // aim at this number directly"*; this region is what lets a check stop
        // mirroring it at all.
        //
        // It is published only when the handle is actually drawn, so a check
        // reading it is reading the application's own statement that the
        // affordance exists — not a rectangle where one would be if the
        // selection had a rotate verb.
        crate::diag::ui_rect(ROTATE_HANDLE_REGION, handle);
    }
}

/// The region the **rotate handle** publishes when it is drawn.
pub const ROTATE_HANDLE_REGION: &str = "canvas.rotate-handle"; // ui-text-exempt: trace region name, never displayed

/// Paint the **annotation move ghost**: one rectangle, where the markup would
/// land.
pub fn draw_annot_ghost(painter: &Painter, mapping: &PageMapping, rect: egui::Rect) {
    let stroke = Stroke::new(1.5, ghost(ink(painter)));
    let screen = visible_outline_rect(mapping.rect_to_screen(rect), MIN_OUTLINE_EXTENT_PX);
    painter.rect_stroke(screen, CornerRadius::ZERO, stroke, StrokeKind::Middle);
}

/// Paint the **move ghost**: the selection's outlines, displaced by an
/// in-flight drag.
pub fn draw_move_ghost(
    painter: &Painter,
    mapping: &PageMapping,
    selection: &SelectionState,
    delta: egui::Vec2,
    // The same flag `draw_selection` takes — `OPERATOR_REQUESTS.md` O69.
    // At the object rung the ghost is always owed; at an inner rung it depends
    // on the companion below.
    outline: bool,
    // **Whether the operator can already see the real thing moving.**
    // `MovePreview` returns a ghost for `MoveSubject::Node` and `Nodes` as
    // well, and there the shape preview draws the actual anchors travelling.
    // A perimeter box on top of that is O63's complaint word for word — *"it
    // just had a perimeter box around it"* — in the one gesture O63 is about,
    // so it is withheld.
    //
    // ⚠ The condition is *geometry is travelling*, not *an inner rung*. Those
    // coincided until a text chunk became selectable, and then they did not:
    // a chunk is an inner rung with no path geometry at all, so gating on the
    // rung withheld the only feedback the gesture had and a drag showed the
    // operator nothing whatsoever. The ghost is the feedback of **last
    // resort** and is suppressed only when something better is already on
    // screen.
    //
    // It is the PREVIEW rather than a boolean derived from it, so that the
    // reduction lives in [`ghost_is_owed`] where it can be tested, rather than
    // at a call site where it cannot.
    already_travelling: Option<&crate::canvas::shapes::ShapePreview>,
) {
    // Read from the selection rather than inferred from `outline`: the two
    // agree today because `pressing::grabbable` sets that flag from this very
    // comparison, and a trace field that restates its own gate cannot witness
    // the gate being wrong.
    let part_rung = selection.level() != SelectionLevel::Object;
    if !ghost_is_owed(outline, already_travelling) {
        crate::canvas::trace::move_ghost(0, part_rung, true);
        return;
    }
    let stroke = Stroke::new(1.5, ghost(ink(painter)));
    let mut boxes = 0_usize;
    for (_, page_rect) in selection.outlines() {
        let screen = visible_outline_rect(
            mapping.rect_to_screen(page_rect.translate(delta)),
            MIN_OUTLINE_EXTENT_PX,
        );
        painter.rect_stroke(screen, CornerRadius::ZERO, stroke, StrokeKind::Middle);
        boxes += 1;
    }
    crate::canvas::trace::move_ghost(boxes, part_rung, false);
}

/// **Whether the move ghost is owed** — `OPERATOR_REQUESTS.md` O69 and O63.
pub(super) fn ghost_is_owed(
    outline: bool,
    already_travelling: Option<&crate::canvas::shapes::ShapePreview>,
) -> bool {
    outline || raster_ghost_is_owed(already_travelling)
}

/// Paint the **rotate ghost**: the selection's outlines turned about the
/// selection's centre.
pub fn draw_rotate_ghost(
    painter: &Painter,
    mapping: &PageMapping,
    selection: &SelectionState,
    centre: egui::Pos2,
    radians: f32,
) {
    let stroke = Stroke::new(1.5, ghost(ink(painter)));
    // The quadrilateral, not the rotated bounding box. Drawing the box would
    // show the operator a shape that GREW as they turned it — which is a
    // preview of something the release does not do, and doubly misleading here:
    // an annotation's `/Rect` really does grow on commit (§12.5.2 requires it
    // upright), and previewing that growth would suggest the artwork grows too.
    // It does not; only the rectangle around it does.
    let quad = |screen: Rect| {
        let corners = [
            screen.left_top(),
            screen.right_top(),
            screen.right_bottom(),
            screen.left_bottom(),
        ]
        .map(|p| crate::canvas::rotating::rotate_about(centre, p, radians));
        for i in 0..4 {
            painter.line_segment([corners[i], corners[(i + 1) % 4]], stroke);
        }
    };
    if let Some(annot) = selection.annot() {
        quad(visible_outline_rect(
            mapping.rect_to_screen(annot.outline),
            MIN_OUTLINE_EXTENT_PX,
        ));
        return;
    }
    for (_, page_rect) in selection.outlines() {
        quad(mapping.rect_to_screen(*page_rect));
    }
}

/// Paint the **resize ghost**: the selection's outlines, scaled about the
/// grip's anchor.
pub fn draw_resize_ghost(
    painter: &Painter,
    mapping: &PageMapping,
    selection: &SelectionState,
    widget: Option<Rect>,
    anchor: egui::Pos2,
    (sx, sy): (f32, f32),
) {
    let stroke = Stroke::new(1.5, ghost(ink(painter)));
    // **The annotation arm** — the same arm [`draw_move_ghost`] carries.
    //
    // `selection.outlines()` holds **page-content** entries. A markup
    // annotation's box lives on `AnnotSelection` instead, so without this the
    // loop below iterates nothing and a stamp being resized previews no change
    // at all — *"the bounding box stays the same size when I drag the
    // handles"* (`OPERATOR_REQUESTS.md` O154), which reads as the resize not
    // working rather than as a missing preview.
    //
    // ⚠ Written as a slice built once rather than as an early `return` with a
    // duplicated body: the scaling arithmetic below is the part that must not
    // exist twice, because it is the half that has to agree with what
    // `canvas::resizing` commits. Two copies of `anchor + (p - anchor) * s` is
    // how a preview and a commit come to disagree about where a corner went.
    //
    // **The widget arm** — O209, the same defect one surface along. A form
    // widget is in neither `annot()` nor `outlines()`; its screen box arrives
    // as a parameter, from the same `widgetdrag::grab_box` the drag measured
    // against. See [`ghost_box`] for why it is a parameter and not a third
    // place this module looks.
    //
    // When it is `Some` the other two are empty by construction — a widget
    // selection clears both — so the three are chained rather than ordered.
    let widget_box = widget.map(|screen| visible_outline_rect(screen, MIN_OUTLINE_EXTENT_PX));
    let annot_box = selection
        .annot()
        .map(|a| visible_outline_rect(mapping.rect_to_screen(a.outline), MIN_OUTLINE_EXTENT_PX));
    let content: Vec<Rect> = selection
        .outlines()
        .iter()
        .map(|(_, page_rect)| mapping.rect_to_screen(*page_rect))
        .collect();
    for screen in widget_box.into_iter().chain(annot_box).chain(content) {
        // `anchor + (p - anchor) * s`, per corner — the same map the commit
        // applies to every node, one level up, so what the operator sees is the
        // outline of what they will get.
        let scaled = egui::Rect::from_min_max(
            egui::pos2(
                anchor.x + (screen.min.x - anchor.x) * sx,
                anchor.y + (screen.min.y - anchor.y) * sy,
            ),
            egui::pos2(
                anchor.x + (screen.max.x - anchor.x) * sx,
                anchor.y + (screen.max.y - anchor.y) * sy,
            ),
        );
        // Published so a driven check can measure the preview rather than
        // photograph it. Without this the only oracle for "did the ghost
        // change size" is a screenshot diff, and a ghost is a 1.5 px stroke at
        // low alpha over arbitrary linework — which is the least reliable
        // pixel assertion this project owns.
        crate::diag::ui_rect(RESIZE_GHOST_REGION, scaled);
        painter.rect_stroke(
            visible_outline_rect(scaled, MIN_OUTLINE_EXTENT_PX),
            CornerRadius::ZERO,
            stroke,
            StrokeKind::Middle,
        );
    }
}

// ---------------------------------------------------------------------------
// Find highlights
// ---------------------------------------------------------------------------

/// One search hit, ready to paint.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FindHighlight {
    /// The hit's box in canvas space — Y-down, page top-left, `/Rotate`
    /// applied. The same space the selection outlines are cached in, so it
    /// projects through the same [`PageMapping`].
    pub rect: Rect,
    /// Whether this is the hit the position readout is counting.
    pub current: bool,
}

/// How opaque a non-current hit's wash is, out of 255.
const HIT_ALPHA: u8 = 40;

/// How opaque the current hit's wash is.
const CURRENT_ALPHA: u8 = 96;

/// Paint the search hits on the page currently shown.
pub fn draw_find_hits(
    painter: &Painter,
    mapping: &PageMapping,
    hits: impl IntoIterator<Item = FindHighlight>,
) {
    let (ink, fill) = pair(painter);
    let stroke = Stroke::new(2.0, ink);
    for hit in hits {
        let screen = visible_outline_rect(mapping.rect_to_screen(hit.rect), MIN_OUTLINE_EXTENT_PX);
        let alpha = if hit.current {
            CURRENT_ALPHA
        } else {
            HIT_ALPHA
        };
        painter.rect_filled(screen, CornerRadius::ZERO, at_alpha(fill, alpha));
        if hit.current {
            painter.rect_stroke(screen, CornerRadius::ZERO, stroke, StrokeKind::Middle);
        }
    }
}

// ---------------------------------------------------------------------------
// The text selection
// ---------------------------------------------------------------------------

/// How opaque the **text selection** wash is, out of 255.
const TEXT_SELECTION_ALPHA: u8 = 40;

/// Paint the operator's **text selection**: one wash per line of it.
pub fn draw_text_selection(painter: &Painter, mapping: &PageMapping, boxes: &[Rect]) {
    for page_rect in boxes {
        let screen =
            visible_outline_rect(mapping.rect_to_screen(*page_rect), MIN_OUTLINE_EXTENT_PX);
        painter.rect_filled(
            screen,
            CornerRadius::ZERO,
            at_alpha(fill(painter), TEXT_SELECTION_ALPHA),
        );
    }
}

/// The alpha the chunk outlines are drawn at.
const CHUNK_OUTLINE_ALPHA: u8 = 110;

/// **One hairline box per chunk of a selected text block** —
/// `OPERATOR_REQUESTS.md` O215 ask 3.
#[must_use]
pub fn draw_chunk_boxes(painter: &Painter, mapping: &PageMapping, boxes: &[Rect]) -> usize {
    let stroke = Stroke::new(1.0, at_alpha(ink(painter), CHUNK_OUTLINE_ALPHA));
    let mut drawn = 0;
    for page_rect in boxes {
        let screen =
            visible_outline_rect(mapping.rect_to_screen(*page_rect), MIN_OUTLINE_EXTENT_PX);
        painter.rect_stroke(screen, CornerRadius::ZERO, stroke, StrokeKind::Middle);
        drawn += 1;
    }
    drawn
}

/// A themed colour at a chosen alpha.
pub(super) fn at_alpha(base: Color32, alpha: u8) -> Color32 {
    let [r, g, b, _] = base.to_srgba_unmultiplied();
    // NOT A THEME COLOUR: arithmetic on the theme's own colour, not a choice
    // of one. The hue arrives from `visuals.selection.*` and only the alpha
    // is set here, so a restyle still reaches every surface built on this —
    // naming a role for each alpha would freeze them to palette entries and
    // break the "these are all the selection colour" relationship they exist
    // to keep.
    Color32::from_rgba_unmultiplied(r, g, b, alpha)
}

/// Paint the rubber-band, given its **canvas-space** rect.
pub fn draw_marquee(painter: &Painter, mapping: &PageMapping, page_rect: Rect) {
    let screen = mapping.rect_to_screen(page_rect);
    painter.rect_filled(screen, CornerRadius::ZERO, wash(fill(painter)));
    painter.rect_stroke(
        screen,
        CornerRadius::ZERO,
        Stroke::new(1.0, ink(painter)),
        StrokeKind::Middle,
    );
}

/// **Wash a fillable field**, so the operator can see what accepts typing.
pub fn draw_field_shade(painter: &Painter, visuals: &Visuals, mapping: &PageMapping, rect: Rect) {
    let screen = mapping.rect_to_screen(rect);
    painter.rect_filled(
        screen,
        CornerRadius::ZERO,
        at_alpha(visuals.hyperlink_color, FIELD_WASH_ALPHA),
    );
}

/// **Outline every widget the authoring surface can select** — O209, *"when I
/// am in edit mode I can't see these boxes."*
pub fn draw_field_target(painter: &Painter, visuals: &Visuals, mapping: &PageMapping, rect: Rect) {
    let screen = mapping.rect_to_screen(rect);
    painter.rect_filled(
        screen,
        CornerRadius::ZERO,
        at_alpha(visuals.hyperlink_color, FIELD_WASH_ALPHA),
    );
    painter.rect_stroke(
        screen,
        CornerRadius::ZERO,
        Stroke::new(1.0, at_alpha(visuals.hyperlink_color, FIELD_TARGET_ALPHA)),
        // Outside, for `draw_field_spotlight`'s reason: a middle-aligned
        // hairline on a tight text box eats the first and last glyph.
        StrokeKind::Outside,
    );
}

/// The authoring outline's alpha.
const FIELD_TARGET_ALPHA: u8 = 150;

/// **Outline the field the Forms panel is pointing at** —
/// `OPERATOR_REQUESTS.md` O98.
pub fn draw_field_spotlight(painter: &Painter, mapping: &PageMapping, rect: Rect) {
    let screen = mapping.rect_to_screen(rect);
    painter.rect_stroke(
        screen,
        CornerRadius::ZERO,
        Stroke::new(SPOTLIGHT_WIDTH, ink(painter)),
        StrokeKind::Outside,
    );
}

/// How thick the spotlight's outline is, in points.
const SPOTLIGHT_WIDTH: f32 = 2.0;

/// The alpha [`draw_field_shade`] washes a field at.
const FIELD_WASH_ALPHA: u8 = 28;

/// **The ink every outline, grip, ghost and band in this module is drawn
/// with** — the theme's *content-area* selection role.
fn ink(painter: &Painter) -> Color32 {
    egui_shell::theme::Theme::canvas_selection_ink(painter.ctx())
}

/// **The translucent tint a selected or enclosed region is washed with** — the
/// theme's content-area selection fill, 27 % alpha by design so the operator
/// can still see what they are picking.
fn fill(painter: &Painter) -> Color32 {
    egui_shell::theme::Theme::canvas_selection_fill(painter.ctx())
}

/// **Both content-area selection roles at once**, as `(ink, fill)`.
fn pair(painter: &Painter) -> (Color32, Color32) {
    egui_shell::theme::Theme::canvas_selection_pair(painter.ctx())
}

/// The rubber-band's fill: the theme's selection colour at low alpha.
fn wash(base: Color32) -> Color32 {
    // NOT A THEME COLOUR: arithmetic on the theme's own colour, not a choice
    // of one. The hue arrives from [`fill`] — the theme's content-area
    // selection role — and only the alpha is set here, so a restyle still
    // reaches this band. Naming a role for the washed colour instead would
    // freeze it to one palette entry and break the "the band is the selection
    // colour" relationship it exists to keep.
    Color32::from_rgba_unmultiplied(base.r(), base.g(), base.b(), 48)
}

/// The ghost outline's colour: the theme's selection stroke at [`GHOST_ALPHA`].
fn ghost(base: Color32) -> Color32 {
    let [r, g, b, _] = base.to_srgba_unmultiplied();
    // NOT A THEME COLOUR: the same arithmetic-on-a-themed-colour case as
    // `wash` — `base` is the content-area selection ink from [`ink`] and only
    // the alpha is chosen here.
    Color32::from_rgba_unmultiplied(r, g, b, GHOST_ALPHA)
}

#[cfg(test)]
mod tests;
