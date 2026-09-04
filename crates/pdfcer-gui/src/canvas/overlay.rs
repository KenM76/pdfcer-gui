//! # `canvas::overlay` — what the selection looks like, and what it must never look like
//!
//! ## ★ Rule 4 is the whole design constraint of this file
//!
//! `D:\Dev\FeatureRequests\pdfce_FeatureRequests\README.md`, second and fourth
//! clauses of the disclosure rule:
//!
//! > **Applied content renders exactly as saved content will render.** No
//! > badge, red flag, dashed outline or "provisional" layer drawn into the
//! > page view. […] **A pre-commit affordance is not content marking.** A
//! > snap indicator, a hover highlight, a rubber-band, a selection handle —
//! > these are the *cursor*; they describe what is about to happen and they
//! > are welcome.
//!
//! Everything this module paints is in the second category and nothing is in
//! the first. Outlines, grips, rubber-bands and a move ghost all describe
//! *what the operator is about to act on*, and all of them disappear the
//! instant the selection does. Nothing here is keyed on a property of the
//! **content** — not "this text was OCRed", not "this bound is approximate",
//! not "this font was substituted". Those are inferences, they owe an
//! off-canvas report, and `panels`' own header records where the old shell's
//! dashed-outline version of one of them used to live and where its
//! replacement now is (a sentence in the Properties panel).
//!
//! The one-line test, from the same source: *would a screenshot of the
//! editing canvas differ from a screenshot of the same document saved and
//! reopened?* With nothing selected, this module paints **nothing at all**,
//! so the answer is no by construction.
//!
//! ## Colours come from the theme, never from a literal — and by their ROLE NAME
//!
//! Every colour here is read from the theme through the two purpose-named
//! accessors at the bottom of this file, [`ink`] and [`fill`]. A hard-coded
//! colour would be correct in one theme and invisible or shouting in the
//! other, and `panels`' scroll-bar note records that exact failure already
//! measured once in this project: a control that was present, opaque,
//! correctly sized and invisible in a capture.
//!
//! ★★★ **They used to be read from `visuals.selection.stroke` and
//! `visuals.selection.bg_fill`, and that was the wrong address** —
//! `REVIEW_TRIAGE.md` defect **T2**, fixed 2026-09-04. `egui::Visuals::selection`
//! is `egui`'s channel for styling **selected widgets**, not a canvas role, and
//! for as long as this theme handed it to the canvas every selected chrome
//! control in the application — nineteen `selectable_label` and
//! `Button::selected` sites — was painted with the colours on this page.
//! Measured luminance gap in the Dark preset: **72.5**, against a floor of 90.
//!
//! The values are unchanged; only their address is. That is the whole lesson
//! and it is this project's standing one: *a correctly-sourced value used for
//! the wrong role passes every gate — expose the pair behind a purpose-named
//! function.* `tools/gates/check-selection-channel.sh` now fails the build for
//! any file outside the theme module that reads the widget channel.
//!
//! ## Why the outline is grown before it is drawn
//!
//! [`visible_outline_rect`], salvaged with its reasoning. A horizontal rule
//! has a real, finite page bbox that is **exactly zero high**; it hit-tests,
//! selects and lists correctly, and its outline puts nothing on the screen.
//! The operator's click was right, the selection state was right, and the
//! feedback was a blank page — a correct action with no feedback is
//! indistinguishable from a broken one.

use egui::{Color32, CornerRadius, Painter, Rect, Stroke, StrokeKind, Visuals};

use crate::canvas::handles;
use crate::canvas::mapping::PageMapping;
use crate::canvas::selection::SelectionState;

/// The anchor marks and the Bézier handles — split out under R2 on 2026-08-31.
///
/// ★ Re-exported below rather than left behind a module path, so every
/// existing `overlay::ANCHOR_PX` / `overlay::draw_anchors` call site keeps its
/// spelling. Nothing moved but its address, which is the property that makes
/// an R2 split reviewable.
pub mod anchors;

pub use anchors::{
    ANCHOR_PX, HANDLE_PX, MAX_UNSELECTED_ANCHORS, PUBLISHED_ANCHORS, anchor_region, draw_anchors,
    draw_handles, handle_region,
};

/// The region the selection's grip box publishes.
///
/// ★ It is the box the eight grips are laid out on, not the union of the
/// outlines: `visible_outline_rect` widens a degenerate outline to a minimum
/// extent so a hairline is still grabbable, and a check aiming at the un-widened
/// rect would miss the grips on exactly the objects that needed the widening.
///
/// ★★ It stayed in the parent when the anchors were split out on 2026-08-31,
/// because it names the OUTLINE and the grips — `overlay`'s subject — and not
/// the points. Three call sites publish it: the annotation branch, the content
/// grip box, and `canvas::forms`' widget box.
pub const SELECTION_OUTLINE_REGION: &str = "canvas.selection-outline"; // ui-text-exempt: trace region name, never displayed

/// The minimum on-screen extent, in egui logical points, that a selection
/// outline is guaranteed to have on each axis.
///
/// Sized to be unmistakably visible without materially misreporting where the
/// object is: at 6 pt a horizontal rule's outline reads as a thin band centred
/// on the rule. The Properties panel states the object's true size, so the
/// enlargement can never be mistaken for the object's real extent — which is
/// what keeps this a legibility fix rather than a silent widening (rule 4 is
/// satisfied by disclosure, not by declining to draw).
pub const MIN_OUTLINE_EXTENT_PX: f32 = 6.0;

/// Grow a degenerate outline rect, about its own centre, until it is at least
/// `min_extent` on both axes — **the fix for a selection that is correct and
/// paints nothing.**
///
/// # The bug this closes
///
/// A horizontal rule (`100 200 m 300 200 l S`) has the page bbox
/// `100,200 → 300,200`: real, finite, and exactly zero high. `rect_stroke`
/// with `StrokeKind::Inside` then has no interior band to fill and puts
/// nothing on the screen.
///
/// # Why in SCREEN space, and why symmetric
///
/// Applied after the canvas→screen projection, so the guaranteed thickness is
/// a constant number of on-screen points at every zoom — the same
/// zoom-invariance discipline
/// [`crate::canvas::mapping::screen_tolerance_to_page`] applies to the catch
/// radius. Growing symmetrically about the centre keeps the band straddling
/// the rule rather than sitting to one side of it, so the outline still says
/// truthfully *the object is here*.
///
/// A non-finite rect is returned unchanged: there is no meaningful centre to
/// grow about, and a NaN box is a bug to leave visible upstream rather than
/// repair here.
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
///
/// Shared by the painter and the hit test so the drawn grips and the live
/// grips are the same squares. Two derivations of one box is how an operator
/// ends up aiming at a handle and getting a marquee.
#[must_use]
pub fn grip_box(mapping: &PageMapping, selection: &SelectionState) -> Option<Rect> {
    let union = selection.outline_union()?;
    Some(visible_outline_rect(
        mapping.rect_to_screen(union),
        MIN_OUTLINE_EXTENT_PX,
    ))
}

/// The move ghost's alpha, out of 255.
///
/// High enough to read as a *second* outline over dense linework — the whole
/// point is that the operator can see where the object is going — and low
/// enough that it never competes with the real outline, which is still on
/// screen showing where the object still is. Both boxes are visible during a
/// drag on purpose: the pair states the displacement, which one box alone
/// cannot.
const GHOST_ALPHA: u8 = 150;

/// Paint the selection: one outline per entry, plus the grips.
///
/// # The move ghost lives next door, and why it came back
///
/// A first draft of this function drew a translucent copy of the outline
/// offset by an in-flight move drag, and it was removed before it shipped:
/// `pdfcer-core` has no resize verb, the move drag was not wired to one either,
/// and *"a pre-commit affordance that describes something which does not
/// happen is not an affordance, it is a lie with a low alpha. It returns in
/// the same change as the verb."*
///
/// This is that change. [`draw_move_ghost`] is the ghost, and the condition
/// under which it may be drawn is exactly the one that note demanded: only
/// when [`crate::canvas::moving::eligible`] has already established that the
/// release will reach a real verb on real operands. **Resize is still not
/// wired**, and the grips still commit nothing — there is no scale verb — so
/// no ghost is offered for a grip drag either.
pub fn draw_selection(
    painter: &Painter,
    visuals: &Visuals,
    mapping: &PageMapping,
    selection: &SelectionState,
    // ★ The whole answer, not just its grip set — `OPERATOR_REQUESTS.md` O69.
    // The caller already holds it and was narrowing it to one field; the
    // outline needs a second, and re-deriving that second one here would be a
    // predicate spelled in two places. See `Grabbable::outline`.
    grab: crate::canvas::pressing::Grabbable,
) {
    if selection.is_empty() {
        return;
    }
    let stroke = Stroke::new(1.5, ink(painter));

    // ★ The selected ANNOTATION, if the selection is one.
    //
    // The **same stroke** as a content outline, deliberately. An operator does
    // not need to be taught that pdfcer distinguishes a `/Annots` entry from a
    // content object — they clicked a stamp and the stamp is now selected, and
    // a second visual language for that would be a distinction the *implementer*
    // finds interesting. What is selected is said in words, off-canvas, where
    // rule 4 puts every other disclosure.
    //
    // No grips. Grips promise a resize, and `set_markup_style` deliberately
    // does **not** include move or resize — the engine left them out of the
    // first slice by name. Drawing eight handles around a stamp that cannot be
    // resized is the "visible control, silently inert" failure this project
    // keeps finding, in its most literal form.
    // ★★★ The grips around a selected annotation, painted here so the predicate
    // that paints them is the one that hit-tests them (H7).
    //
    // ★★★ **`offer` IS THAT PREDICATE, AND IT IS NOW PASSED IN RATHER THAN
    // RE-DERIVED HERE — 2026-08-28.**
    //
    // Until this change the condition below was written out locally: *"draw the
    // eight if the kind is `Markup` and it is not locked"*, which happened to
    // agree with what `pressing::grabbable` decided, in a second place, in
    // different words. That was survivable while every annotation offered the
    // same set. It stopped being survivable the moment three annotation kinds
    // offered three different sets:
    //
    // | selected | painted | hit-tested |
    // |---|---|---|
    // | markup | eight squares **and** the circle | `GripSet::all()` |
    // | ce dimension | the circle **only** | `GripSet::rotate_only()` |
    // | form field | eight squares only | `GripSet::scale_only()` |
    //
    // A local re-derivation of that table would be a second copy of it, and the
    // day the two copies disagree the symptom is either a handle nobody can
    // grab or — worse — an invisible target that steals the press aimed at what
    // is under it. `handles::GripSet`'s own header records the 2026-08-20
    // incident that made this rule: a dimension's vertex handles were painted
    // from the selection and hit-tested behind a capability the mode did not
    // have, so they were visible and untouchable in the very mode that authors
    // dimensions.
    //
    // ⇒ One value, one decision, two consumers. `canvas::painting` asks
    // `pressing::grabbable` once and hands the answer to both.
    if let Some(annot) = selection.annot() {
        let screen =
            visible_outline_rect(mapping.rect_to_screen(annot.outline), MIN_OUTLINE_EXTENT_PX);
        painter.rect_stroke(screen, CornerRadius::ZERO, stroke, StrokeKind::Middle);
        // ★★ Published under the SAME region name the content selection uses,
        // so a driven check aiming at a grip reads one name whatever is
        // selected. `handles::grip_rects` derives all eight from this box, so a
        // harness that has it cannot disagree with the application about where
        // they are.
        crate::diag::ui_rect(SELECTION_OUTLINE_REGION, screen);
        // ★★ Whatever `offer` says, and nothing else. A **ce dimension** gets
        // the rotate handle and none of the eight — its extent IS its
        // measurement, so pdfcer has no verb that scales one and declines to
        // grow one; a rotation is an isometry, so the number is identical
        // either side of it and turning one is a legitimate drafting
        // operation. A **locked** annotation (§12.5.3 bit 8) and every
        // annotation kind this shell cannot address get `GripSet::default()`
        // — no box at all from `grabbable`, so nothing is painted and nothing
        // is grabbable. **R9**: rendering nothing is the honest answer for a
        // capability that does not exist.
        draw_grips(painter, visuals, screen, grab.offer);
        return;
    }

    // ★★★ **THE BOX IS NOT DRAWN OVER THE NODES** — `OPERATOR_REQUESTS.md`
    // O69: *"If we are at a point where we are showing the nodes in an
    // editable state there shouldn't be a bounding box around the objects."*
    //
    // At the Part and Node rungs `selection::outline_rect` returns the entered
    // SUBPATH's bounding box, and it was stroked on top of that subpath's own
    // anchors — so the operator got a rectangle around the thing whose points
    // he was trying to see. The eight grips were already correctly withheld
    // there (`GripSet::default()`); the outline was the half nobody had gated.
    //
    // ★ Traced rather than only changed, because "no box" and "no selection"
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
        // ★★ Published so a driven check can AIM AT A GRIP.
        //
        // Added 2026-08-19 with the resize commit, and it is the difference
        // between a check that measures the feature and one that measures the
        // harness's guesswork: a grip sits at a corner of this box, and the
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
        // ★★ The SAME `offer` the hit test was given, which for page content is
        // `GripSet::all()` at the Object rung and `GripSet::default()` at every
        // inner one — the identical condition this line used to spell out as
        // `selection.level() == SelectionLevel::Object`.
        //
        // Converted on 2026-08-28 with the annotation branch above, and for the
        // stronger version of the same reason: two spellings of one predicate
        // are one refactor away from disagreeing, and when they disagree a
        // handle is either painted and not hit-tested — the "visible control,
        // silently inert" failure — or hit-tested and not painted, which is
        // worse, because it is an invisible target that steals the press aimed
        // at the anchor underneath it. That second case is the defect that made
        // this rule necessary in the first place.
        draw_grips(painter, visuals, box_, grab.offer);
    }
}

/// Paint the grips `offer` says a screen-space box has.
///
/// Filled with the theme's window background and stroked in the selection
/// colour: a filled square reads as a handle at any zoom and against any page
/// content, where an outline-only square disappears over dense linework —
/// which is precisely the document class pdfcer is for.
///
/// # ★★★ `offer` is the hit test's own value, and that is the contract
///
/// It is [`crate::canvas::pressing::grabbable`]'s answer, passed straight
/// through — **never a predicate recomputed here**. Rule H7, and it is the
/// difference between two representations of one decision and one:
///
/// * a handle painted and **not** hit-tested is the *"visible control, silently
///   inert"* failure this project spends its time removing;
/// * a handle hit-tested and **not** painted is worse — an invisible target
///   that steals the press aimed at whatever is under it.
///
/// Since 2026-08-28 the two flags genuinely differ per selection — a ce
/// dimension turns and does not scale, a form field's box scales and does not
/// turn — so this function can no longer treat *"there is a box"* as *"there
/// are nine handles"*. It draws exactly what it was told.
pub fn draw_grips(
    painter: &Painter,
    visuals: &Visuals,
    bounds: Rect,
    offer: crate::canvas::handles::GripSet,
) {
    let ink = ink(painter);
    let stroke = Stroke::new(1.0, ink);
    if offer.resize {
        for (_, rect) in handles::grip_rects(bounds) {
            painter.rect(
                rect,
                CornerRadius::ZERO,
                visuals.window_fill,
                stroke,
                StrokeKind::Middle,
            );
        }
    }
    // ★★ …and the rotate handle, which is a CIRCLE ON A STEM and not a ninth
    // square.
    //
    // Every square on this canvas resizes, so a shape that resized in one place
    // and rotated in another would be a private convention the operator has to
    // learn — `handles.md` H2's stated failure mode. The stem is what says the
    // handle belongs to this box; without it the circle reads as an unrelated
    // dot floating over the page.
    //
    // ★★★ Gated **separately** from the eight, which is the whole reason
    // `GripSet` has two fields. On a selected **ce dimension** this is the only
    // thing drawn: there are no squares at all, because there is no verb that
    // scales one and `pdfcer-core` has declined to build one — *"either the
    // displayed value stays fixed while the geometry grows, so the dimension
    // lies about the drawing; or both change, so nothing was measured"*. A lone
    // circle on a stem over a dimension is therefore the correct picture rather
    // than an incomplete one.
    if offer.rotate {
        let handle = handles::rotate_rect(bounds);
        let centre = handle.center();
        painter.line_segment(
            [egui::pos2(centre.x, bounds.top()), centre],
            Stroke::new(1.0, ink),
        );
        painter.circle(
            centre,
            handles::GRIP_SIZE_PX / 2.0,
            visuals.window_fill,
            stroke,
        );
        // ★★ **Published so a driven check can aim at the ninth handle.**
        //
        // The eight are derivable from `SELECTION_OUTLINE_REGION` — they sit on
        // its corners and edge midpoints — and the rotate handle is **not**: it
        // is offset by `ROTATE_STEM_PX`, a number the harness would have to
        // duplicate and could get wrong silently. `checks/rotate.rs` mirrors
        // that constant today and says in its own comment that it *"does not
        // aim at this number directly"*; this region is what lets a check stop
        // mirroring it at all.
        //
        // ★ It is published only when the handle is actually drawn, so a check
        // reading it is reading the application's own statement that the
        // affordance exists — not a rectangle where one would be if the
        // selection had a rotate verb.
        crate::diag::ui_rect(ROTATE_HANDLE_REGION, handle);
    }
}

/// The region the **rotate handle** publishes when it is drawn.
///
/// ★ Distinct from [`SELECTION_OUTLINE_REGION`] because the handle is the one
/// affordance that cannot be derived from the outline: it sits on a stem
/// outside the box. Its presence in a trace is also the honest answer to *"does
/// this selection offer a rotation at all?"* — which is a question a driven
/// check has to be able to ask about a **form field**, where the answer must be
/// no.
pub const ROTATE_HANDLE_REGION: &str = "canvas.rotate-handle"; // ui-text-exempt: trace region name, never displayed

/// Paint the **move ghost**: the selection's outlines, displaced by an
/// in-flight drag.
///
/// `delta` is in **canvas space** — the same space the cached outlines are in,
/// which is what makes this a translation and nothing more.
///
/// # ★ Why this costs no re-raster and no re-decomposition
///
/// Three facts line up, and the preview is affordable because of all three:
///
/// 1. **The outlines are cached in canvas space.**
///    [`SelectionState`] keys them on
///    `(page, edit epoch)`, neither of which moves during a drag, so no
///    decomposition happens on any frame of the gesture.
/// 2. **Canvas space is zoom-independent**, so translating a cached rect by a
///    canvas-space delta and projecting the result is exact at every
///    magnification — there is no per-frame re-derivation of geometry.
/// 3. **Nothing touches the page texture.** A ghost is two strokes on the
///    painter that is already open. The raster is invalidated by
///    `Action::Move*` on *commit*, once, in `app::actions` — not by the
///    preview, which is the whole reason the preview is a preview.
///
/// A ghost that re-rendered the page per frame would be a different feature
/// wearing the same name: on the CAD sheets pdfcer exists for, one raster is
/// tens of milliseconds and a drag is sixty frames a second.
///
/// # Rule 4
///
/// This is a *pre-commit affordance* — the cursor describing what is about to
/// happen — and rule 4 admits those explicitly, alongside the snap indicator,
/// the hover highlight and the rubber-band. What rule 4 forbids is marking
/// content that has **already been applied**, and nothing here survives the
/// release: the ghost exists only while the pointer is down. The one-line test
/// in this module's header still answers no — with nothing being dragged, this
/// paints nothing at all.
/// Paint the **annotation move ghost**: one rectangle, where the markup would
/// land.
///
/// ★★ Its own function beside [`draw_move_ghost`] rather than a case of it, and
/// the reason is what the two iterate. That one walks
/// `SelectionState::outlines()` -- the CONTENT selection's rectangles -- which
/// is empty for an annotation selection by construction, because the two
/// selections are mutually exclusive and live in different fields. Handing an
/// annotation drag to it would draw nothing at all, silently, which is the
/// "the gesture does nothing" symptom in the place hardest to notice: the drag
/// would still commit on release.
///
/// ★ It takes the rectangle already computed rather than a delta plus the
/// selection, because `annotdrag` has to decide the same rectangle to know
/// whether a drag is eligible at all. One computation, one answer, and the
/// preview cannot promise a landing spot the commit disagrees with.
pub fn draw_annot_ghost(painter: &Painter, mapping: &PageMapping, rect: egui::Rect) {
    let stroke = Stroke::new(1.5, ghost(ink(painter)));
    let screen = visible_outline_rect(mapping.rect_to_screen(rect), MIN_OUTLINE_EXTENT_PX);
    painter.rect_stroke(screen, CornerRadius::ZERO, stroke, StrokeKind::Middle);
}

pub fn draw_move_ghost(
    painter: &Painter,
    mapping: &PageMapping,
    selection: &SelectionState,
    delta: egui::Vec2,
    // ★★★ The same flag `draw_selection` takes, for the same reason and then
    // some — O69. `MovePreview` returns a ghost for `MoveSubject::Node` and
    // `Nodes` too, so while DRAGGING a point the operator was getting the
    // subpath's box *and* a translated copy of it. That is O63's complaint
    // word for word — *"it just had a perimeter box around it"* — surviving in
    // the one gesture O63 was about.
    //
    // Nothing is lost by removing it: O63's shape preview already draws the
    // real geometry moving, which is what he asked to see instead.
    outline: bool,
) {
    if !outline {
        return;
    }
    let stroke = Stroke::new(1.5, ghost(ink(painter)));
    for (_, page_rect) in selection.outlines() {
        let screen = visible_outline_rect(
            mapping.rect_to_screen(page_rect.translate(delta)),
            MIN_OUTLINE_EXTENT_PX,
        );
        painter.rect_stroke(screen, CornerRadius::ZERO, stroke, StrokeKind::Middle);
    }
}

/// Paint the **rotate ghost**: the selection's outlines turned about the
/// selection's centre.
///
/// [`draw_resize_ghost`]'s sibling, and split from it for the same reason that
/// one is split from the move ghost: this transforms four CORNERS rather than
/// two, because a rotated rectangle is not an axis-aligned one and
/// `Rect::from_min_max` of two rotated corners is a different shape entirely.
///
/// ★ **It draws a quadrilateral, not a rect**, and that is the whole visible
/// difference. Drawing the rotated bounding box instead would show the operator
/// a shape that grew as they turned it — a preview of something the release
/// does not do.
///
/// The angle is the **un-negated** screen-space one from
/// `rotating::angle`, and the rotation is `rotating::rotate_about` — the same
/// function that module's own test pins against the measured bearing. The
/// commit negates once, at the page crossing; see `rotating::drag`.
///
/// # ★★★ An ANNOTATION's ghost is drawn by this same function — 2026-08-28
///
/// …unlike the **move** ghost, which `canvas::painting` carries in a separate
/// `annot_ghost` slot. The asymmetry is deliberate and is about the arithmetic
/// rather than about tidiness:
///
/// * a move ghost is a **translated rectangle**, and `annotdrag` computes it in
///   canvas space as part of deciding whether the move is eligible at all — so
///   the value already exists and carrying it is free;
/// * a rotate ghost is **four corners turned about a centre**, and that is the
///   identical calculation whether the corners came from a content outline or
///   from an annotation's `/Rect`. Two functions would be one function written
///   twice, and the second copy is where a preview and a commit come to
///   disagree about which way round something went.
///
/// ★★ The annotation case **returns early**, mirroring `draw_selection`'s own
/// structure one screen up. An annotation selection and a content selection are
/// mutually exclusive by construction (`SelectionState` enforces it in one
/// place), so the early return is a statement of that invariant rather than a
/// precedence: `selection.outlines()` is empty behind an annotation anyway, and
/// falling through would have drawn nothing — silently, which is how a missing
/// preview reads as *"the drag stopped tracking"*.
pub fn draw_rotate_ghost(
    painter: &Painter,
    mapping: &PageMapping,
    selection: &SelectionState,
    centre: egui::Pos2,
    radians: f32,
) {
    let stroke = Stroke::new(1.5, ghost(ink(painter)));
    // ★ The quadrilateral, not the rotated bounding box. Drawing the box would
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
///
/// [`draw_move_ghost`]'s sibling, and split from it rather than folded in for
/// the reason `canvas::resizing`'s own header gives about the arithmetic: a
/// move is one displacement applied to everything, a resize is a **map** whose
/// answer depends on where each corner started. One function taking an
/// `enum { Move(Vec2), Resize(Pos2, Vec2) }` would branch inside a loop that is
/// otherwise two lines.
///
/// ★ The anchor is in **screen** space, because that is the space the outlines
/// are projected into and the space [`crate::canvas::handles::Grip::anchor`]
/// already answers in. Converting to PDF for the preview and back again would
/// be two conversions for a picture that is thrown away next frame — and, worse,
/// a second place for the ghost and the commit to disagree about which corner
/// stayed still.
pub fn draw_resize_ghost(
    painter: &Painter,
    mapping: &PageMapping,
    selection: &SelectionState,
    anchor: egui::Pos2,
    (sx, sy): (f32, f32),
) {
    let stroke = Stroke::new(1.5, ghost(ink(painter)));
    for (_, page_rect) in selection.outlines() {
        let screen = mapping.rect_to_screen(*page_rect);
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
///
/// The whole vocabulary this module needs about Find: **where**, in canvas
/// space, and **whether it is the one the view is on**. Deliberately not a
/// `TextMatch`, not a page index and not a `Quad` — the projection from
/// unrotated PDF user space happens once, at search time, in
/// [`crate::find::Hit::canvas`], so this file is never told what a PDF is and
/// that file is never told what a `Painter` is.
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
///
/// Low enough that the *text under it stays readable* — the operator is
/// scanning the page to decide whether the hit is the one they want, and a
/// highlight that obscures its own subject defeats the purpose. Deliberately
/// the same order as the marquee's wash, which exists for the identical
/// reason.
const HIT_ALPHA: u8 = 40;

/// How opaque the current hit's wash is.
///
/// ★ **Emphasis, not hue, is what distinguishes the current hit** — and that
/// is a constraint rather than a preference. Every colour on this canvas comes
/// from the theme (see this module's header, and
/// `tools/gates/check-theme-colors.sh`), and the theme has no role meaning
/// "the search hit you are on". Borrowing one that means something else —
/// `warn_fg_color`, `error_fg_color` — would say *warning* on a control that
/// is not warning about anything, and would then be wrong the first time
/// somebody restyled the warning colour for warnings.
///
/// So the current hit is the same colour, more than twice as opaque, **and
/// stroked**. Two independent signals rather than one: alpha alone is a weak
/// difference on a dense drawing, and the outline is what carries it at a
/// glance. Acrobat and every browser use a second hue for this; pdfcer cannot,
/// and this is the honest substitute.
///
/// ### ★ Why it is 96 and not 168, which is what it was
///
/// **Measured on a screenshot of the running binary.** The first value was
/// chosen for contrast against neighbouring hits and produced a solid block
/// over the current one: on `reflow.pdf`, searching `the`, the word `The` at
/// the head of the paragraph was completely covered by its own highlight. A
/// highlight that hides the text it is highlighting has defeated its purpose —
/// the operator's next act is to *read* the hit and decide whether it is the
/// one they wanted.
///
/// That is exactly the failure a passing test could not have caught, and the
/// reason this project's founding rule is to drive the binary: both alphas
/// were within their asserted bounds, the hue assertion passed, and the
/// picture was wrong. The stroke went from 1.5 pt to 2.0 pt in the same change
/// to carry the emphasis the alpha gave up.
const CURRENT_ALPHA: u8 = 96;

/// Paint the search hits on the page currently shown.
///
/// `hits` comes from [`crate::find::FindState::page_highlights`], which
/// yields **nothing at all** when the results are not current — so an edit,
/// a changed query or a closed bar all stop the highlights here by supplying
/// an empty iterator rather than by a check in this function. That is the
/// mechanism by which rule 4 is kept: this module cannot paint a mark over
/// content the search no longer describes, because it is never handed one.
///
/// # Rule 4
///
/// A find highlight is a **pre-commit affordance in the second category** of
/// this module's header — it describes what the operator is looking at, not a
/// property of the content, and it disappears the instant the bar closes.
/// Nothing here is keyed on a property of the document: it marks *the answer
/// to a question the operator just asked*, which is the same class as a hover
/// highlight. The one-line test still answers no — with the bar closed, this
/// paints nothing at all.
///
/// # Why the rects are grown
///
/// Through [`visible_outline_rect`], for the same reason a selection outline
/// is: a text run's quad can be degenerate on one axis (a page whose
/// producer emitted a zero-height box, or a hit so small at the current zoom
/// that it rounds to nothing), and a highlight that puts no pixels on the
/// screen is indistinguishable from a search that did not work.
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
///
/// ★ **The number this project already paid for once.** `HANDOFF.md` §2's
/// defect 3 is *"Find's current-hit highlight completely covered the word it
/// highlighted"*, found by driving the binary and fixed by taking
/// [`CURRENT_ALPHA`] from 168 to 96 — with the lesson recorded there as
/// *"the operator's next act after finding a hit is to READ it"*.
///
/// It applies here with more force. A find hit is something the operator is
/// deciding about; a text selection is something they are **about to copy**,
/// and the only way to tell whether they swept the right words is to read them
/// through the wash. So this sits at the low end deliberately, at the same
/// value as a non-current find hit rather than the emphasised one: there is
/// nothing here to emphasise *against*, because a selection has no neighbours.
///
/// Equal to [`HIT_ALPHA`] and stated as its own constant rather than aliased to
/// it, because the two are equal by coincidence of judgement rather than by
/// construction — they answer different questions ("one of several answers" vs
/// "the thing you are copying") and a future change to either must not silently
/// move the other.
const TEXT_SELECTION_ALPHA: u8 = 40;

/// Paint the operator's **text selection**: one wash per line of it.
///
/// `boxes` come from [`crate::canvas::textsel::TextSelection::highlights`],
/// which yields **nothing at all** for another page or for a revision the
/// selection no longer describes — so an edit stops the wash here by supplying
/// an empty slice rather than by a check in this function, exactly as
/// `find::FindState::page_highlights` arranges for the search wash. That is the
/// mechanism by which rule 4 is kept: this module cannot paint a mark over
/// glyphs the selection no longer describes, because it is never handed one.
///
/// # Rule 4
///
/// A selection wash is a **pre-commit affordance** in the second category of
/// this module's header — it is the cursor, describing what a copy would take —
/// and it disappears the instant the selection does. Nothing here is keyed on a
/// property of the *content*: it marks a range the operator just swept. The
/// one-line test still answers no; with nothing selected this paints nothing.
///
/// # ★ Unstroked, where a find hit is stroked
///
/// [`draw_find_hits`] strokes the **current** hit because it has to be told
/// apart from the other hits on the page. A text selection is one thing, so
/// there is nothing to distinguish it from — and a stroke round each line box
/// would draw a visible seam **between** the lines of one selection, which is a
/// boundary the operator did not make and which no text application draws.
///
/// # Why the boxes are grown
///
/// Through [`visible_outline_rect`], for the reason [`draw_find_hits`] gives:
/// a glyph box can be degenerate on one axis — a producer that emitted a zero
/// size, or a line so small at the current zoom that it rounds away — and a
/// selection that puts no pixels on the screen is indistinguishable from a
/// gesture that did not work.
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

/// A themed colour at a chosen alpha.
///
/// Read back through `to_srgba_unmultiplied`, for the reason [`ghost`]
/// documents at length and [`wash`] pre-dates:
/// [`Color32`] stores **premultiplied** components, so the plain accessors
/// return a hue already darkened by whatever alpha the source carried, and
/// re-premultiplying that darkens it a second time.
///
/// `pub(super)` rather than private since the rulers landed: [`super::rulers`]
/// needs the theme's hairline at two grid alphas and [`super::guides`] needs
/// the selection hue at two guide alphas, and every one of those is the same
/// premultiplication trap. Four more spellings of it would be four more
/// chances to reach for `.r()` and produce a colour that is subtly wrong in
/// exactly the theme nobody tests in — which is the failure `wash`'s and
/// `ghost`'s own docs were written after. The *alphas* stay with the surfaces
/// that chose them, because each is an argument about legibility over
/// linework and belongs beside that argument.
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
///
/// A wash plus an outline. The wash matters on a dense drawing: an
/// outline-only band over a hatched region is hard to see at all, and a
/// rubber-band the operator cannot see is a rubber-band they cannot aim.
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
///
/// `OPERATOR_REQUESTS.md` O96 — *"in our display section we should have an
/// option to shade the form fields like acrobat does."*
///
/// # ★★★ Why this is an affordance and not the tint rule 4 forbids
///
/// The standing rule is *applied content renders exactly as saved content will
/// render*. A field is **not content**: it is a control, and this wash is the
/// same class of thing as the pointing hand `crate::canvas::forms` already puts
/// over a widget. It marks no inference and says nothing about pdfcer's
/// confidence in anything.
///
/// ★★ The property that keeps that true is **where it is painted**: here, in the
/// canvas overlay, over the finished page texture. It reaches no rasterizer, so
/// it cannot appear in a print, an export, a Save or a `render-page`.
///
/// # The colour, and why it is derived rather than named
///
/// The theme's own **hyperlink** colour at a low alpha. Not `selection`, which
/// is what a selected object wears and would make every fillable field look
/// selected; and not a literal, which would not track light and dark. Hyperlink
/// is the theme's "this is interactive, click it" role and that is exactly what
/// a fillable field is.
///
/// ★ [`FIELD_WASH_ALPHA`] is lower than the marquee's, deliberately. A band is
/// transient and the operator is looking *at* it; this sits under the page's own
/// content for as long as the document is open, and a wash that made a filled
/// field's own text harder to read would have traded one legibility problem for
/// another.
pub fn draw_field_shade(painter: &Painter, visuals: &Visuals, mapping: &PageMapping, rect: Rect) {
    let screen = mapping.rect_to_screen(rect);
    painter.rect_filled(
        screen,
        CornerRadius::ZERO,
        at_alpha(visuals.hyperlink_color, FIELD_WASH_ALPHA),
    );
}

/// **Outline the field the Forms panel is pointing at** —
/// `OPERATOR_REQUESTS.md` O98.
///
/// A **stroke**, where [`draw_field_shade`] is a fill, and the pair is
/// deliberate: every fillable field already wears the wash, so a spotlight that
/// was a stronger wash would be a difference of degree that an operator has to
/// compare two boxes to notice. An outline is a difference of *kind* and reads
/// at a glance, which is the whole job.
///
/// ★ The theme's **selection stroke**, because that is what the operator is
/// doing — they have picked this field out of a list, and every other "this is
/// the one I mean" on this canvas wears the same colour.
///
/// ★★ `StrokeKind::Outside`, so the outline sits *around* the field rather than
/// over its first and last characters. A middle-aligned stroke on a tight text
/// box eats the glyphs at both ends, which is worst on exactly the short fields
/// — a date, a revision letter — where every character matters.
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
///
/// ★ 2.0 rather than the marquee's 1.0. This one has to be seen against a page
/// that may be dense linework at a fitted zoom, and it is transient — it is on
/// screen only while a row is focused, so it can afford to be assertive in a
/// way a permanent mark could not.
const SPOTLIGHT_WIDTH: f32 = 2.0;

/// The alpha [`draw_field_shade`] washes a field at.
///
/// ★ 28, against the marquee's 48. See that function's ★ for why lower: this one
/// is on screen for as long as the document is open and sits under the field's
/// own text, where the band is transient and has nothing under it that has to
/// stay readable.
const FIELD_WASH_ALPHA: u8 = 28;

/// **The ink every outline, grip, ghost and band in this module is drawn
/// with** — the theme's *content-area* selection role.
///
/// # ★★★ Why this is a call and not `visuals.selection.stroke.color`
///
/// It used to be exactly that, everywhere in this file, and that address was
/// wrong — `REVIEW_TRIAGE.md` defect **T2**. `egui::Visuals::selection` is
/// `egui`'s styling channel for **selected widgets**: `Style::button_style`
/// takes both fills *and the text colour* from it for anything drawn with
/// `Button::selected(true)` or `ui.selectable_label(true, …)`. While the theme
/// pointed that channel at this canvas, the canvas won, and every selected
/// chrome control in the application was painted with canvas ink — accent text
/// on a 27 % wash, a luminance gap of 72.5 in the Dark preset against the
/// project's own readable floor of 90.
///
/// Nothing about the *picture* changed when it was fixed:
/// [`egui_shell::theme::Theme::canvas_selection_ink`] returns `palette.accent`,
/// which is bit-for-bit what that channel used to carry. What changed is that
/// the colour now arrives under a name that says which role it is, so
/// re-tuning chrome can no longer silently re-tune the page overlay.
/// `tools/gates/check-selection-channel.sh` keeps the old address unreachable.
///
/// ★ Takes the [`Painter`] rather than a `&Context` because every drawing
/// function in this module already holds one and `Painter::ctx` is free. That
/// is deliberate: a helper whose argument the caller must go and *find* is a
/// helper people work around.
fn ink(painter: &Painter) -> Color32 {
    egui_shell::theme::Theme::canvas_selection_ink(painter.ctx())
}

/// **The translucent tint a selected or enclosed region is washed with** — the
/// theme's content-area selection fill, 27 % alpha by design so the operator
/// can still see what they are picking.
///
/// See [`ink`] for the whole argument. This is the other half of the pair that
/// used to be read from `visuals.selection.bg_fill`, and it returns
/// `palette.selection_fill` — again, the identical value.
fn fill(painter: &Painter) -> Color32 {
    egui_shell::theme::Theme::canvas_selection_fill(painter.ctx())
}

/// **Both content-area selection roles at once**, as `(ink, fill)`.
///
/// For the functions that draw a washed rectangle *and* an outline around it —
/// [`draw_find_hits`] is the one — where fetching the two separately would
/// read as two unrelated colours and would take the theme lock twice inside a
/// loop.
fn pair(painter: &Painter) -> (Color32, Color32) {
    egui_shell::theme::Theme::canvas_selection_pair(painter.ctx())
}

/// The rubber-band's fill: the theme's selection colour at low alpha.
///
/// Derived from the theme rather than named, so it tracks light and dark
/// without a second literal — and low enough that the content under the band
/// stays readable, because the operator is choosing what to enclose *by
/// looking at it*.
fn wash(base: Color32) -> Color32 {
    // NOT A THEME COLOUR: arithmetic on the theme's own colour, not a choice
    // of one. The hue arrives from [`fill`] — the theme's content-area
    // selection role — and only the alpha is set here, so a restyle still
    // reaches this band — naming a role
    // for it would freeze the wash to one palette entry and break the
    // "the band is the selection colour" relationship it exists to keep.
    Color32::from_rgba_unmultiplied(base.r(), base.g(), base.b(), 48)
}

/// The ghost outline's colour: the theme's selection stroke at [`GHOST_ALPHA`].
///
/// Read back through `to_srgba_unmultiplied` rather than through `.r()`/`.g()`
/// /`.b()`. [`Color32`] stores **premultiplied** components, so the plain
/// accessors return a hue already darkened by whatever alpha the source
/// carried; re-premultiplying that at a new alpha darkens it a second time.
/// The selection stroke is opaque in both shipped themes, so the two spellings
/// agree today — which is exactly why the wrong one would go unnoticed until a
/// theme with a translucent selection stroke made the ghost a different colour
/// from the outline it is a copy of.
fn ghost(base: Color32) -> Color32 {
    let [r, g, b, _] = base.to_srgba_unmultiplied();
    // NOT A THEME COLOUR: the same arithmetic-on-a-themed-colour case as
    // `wash` — `base` is the content-area selection ink from [`ink`] and only
    // the alpha is chosen here.
    Color32::from_rgba_unmultiplied(r, g, b, GHOST_ALPHA)
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{Pos2, pos2};

    /// ★ A zero-height rule gets a visible band rather than nothing.
    #[test]
    fn a_degenerate_outline_is_grown_until_it_can_be_seen() {
        // The measured case: `100 200 m 300 200 l S`, projected to screen.
        let rule = Rect::from_min_max(pos2(100.0, 200.0), pos2(300.0, 200.0));
        let out = visible_outline_rect(rule, MIN_OUTLINE_EXTENT_PX);
        assert!(out.height() >= MIN_OUTLINE_EXTENT_PX);
        assert!(
            (out.width() - 200.0).abs() < f32::EPSILON,
            "the axis that was already visible must not be touched"
        );
        assert!(
            (out.center().y - 200.0).abs() < f32::EPSILON,
            "the band must straddle the rule, not sit to one side of it"
        );
    }

    /// A comfortable rect is returned unchanged — the growth is a repair, not
    /// a permanent inflation that would misreport every object's extent.
    #[test]
    fn a_healthy_outline_is_left_alone() {
        let r = Rect::from_min_max(pos2(0.0, 0.0), pos2(100.0, 80.0));
        assert_eq!(visible_outline_rect(r, MIN_OUTLINE_EXTENT_PX), r);
    }

    /// An inside-out rect normalises before it is grown, so a projection that
    /// swapped the corners still paints.
    #[test]
    fn an_inside_out_rect_normalises_before_growing() {
        let backwards = Rect::from_min_max(pos2(300.0, 240.0), pos2(100.0, 200.0));
        let out = visible_outline_rect(backwards, MIN_OUTLINE_EXTENT_PX);
        assert!(out.width() > 0.0 && out.height() > 0.0);
        assert!(out.contains(pos2(200.0, 220.0)));
    }

    /// A non-finite rect is left exactly as it arrived: there is no
    /// meaningful centre to grow about, and repairing it here would hide a
    /// bug that belongs upstream.
    #[test]
    fn a_non_finite_rect_is_returned_unchanged() {
        let nan = Rect::from_min_max(pos2(f32::NAN, 0.0), pos2(10.0, 10.0));
        let out = visible_outline_rect(nan, MIN_OUTLINE_EXTENT_PX);
        assert!(out.min.x.is_nan());
        // And a nonsense minimum is refused rather than shrinking the rect.
        let r = Rect::from_min_max(Pos2::ZERO, pos2(10.0, 10.0));
        assert_eq!(visible_outline_rect(r, -1.0), r);
        assert_eq!(visible_outline_rect(r, f32::NAN), r);
    }

    /// The wash keeps its hue and drops its alpha, so the content under a
    /// rubber-band stays readable.
    ///
    /// Asserted through `to_srgba_unmultiplied` rather than through `.r()`,
    /// and **approximately**. Both halves of that are the point:
    ///
    /// - [`Color32`] stores **premultiplied** components, so a translucent
    ///   blue reads back as `(11, 23, 38)` from the plain accessors and looks
    ///   as though the hue was lost. It was not, and "fixing" that by dropping
    ///   the alpha would be the wrong repair.
    /// - Premultiplying at alpha 48 and dividing back out is lossy — 60
    ///   returns as 58 — so exact equality would be asserting the precision of
    ///   egui's colour storage rather than the property this function has.
    #[test]
    fn the_marquee_wash_is_translucent_and_keeps_the_themes_hue() {
        // NOT A THEME COLOUR: a test fixture standing in for whatever the
        // theme supplies; the assertion is that the hue survives, so the
        // exact input has to be a known literal.
        let base = Color32::from_rgb(60, 120, 200);
        let [r, g, b, a] = wash(base).to_srgba_unmultiplied();
        for (got, want) in [(r, 60u8), (g, 120), (b, 200)] {
            assert!(
                got.abs_diff(want) <= 4,
                "the wash drifted off the theme's hue: {got} vs {want}"
            );
        }
        assert!(a < 64, "a rubber-band must not hide what it encloses");
    }

    /// The ghost keeps the theme's hue and is translucent — visibly a *copy*
    /// of the outline rather than a second, competing selection.
    ///
    /// Asserted through `to_srgba_unmultiplied` for the reason [`ghost`]'s own
    /// docs give, and approximately because premultiplying and dividing back
    /// out is lossy.
    #[test]
    fn the_move_ghost_is_translucent_and_keeps_the_themes_hue() {
        // NOT A THEME COLOUR: test fixture, as above.
        let base = Color32::from_rgb(60, 120, 200);
        let [r, g, b, a] = ghost(base).to_srgba_unmultiplied();
        for (got, want) in [(r, 60u8), (g, 120), (b, 200)] {
            assert!(
                got.abs_diff(want) <= 4,
                "the ghost drifted: {got} vs {want}"
            );
        }
        assert_eq!(a, GHOST_ALPHA);
        assert!(
            a > 64,
            "the ghost must be readable over dense linework, unlike the marquee wash"
        );
    }

    /// ★ **The current find hit is distinguished by emphasis, not by hue.**
    ///
    /// Both halves are asserted because both are the design:
    ///
    /// - the two alphas differ by enough to read at a glance, so a page of
    ///   hits shows *which one* the readout is counting;
    /// - the hue is the theme's, unchanged, in both — a find highlight that
    ///   borrowed `warn_fg_color` would say *warning* about something that is
    ///   not a warning, and would break the first time somebody restyled the
    ///   warning colour for warnings. `tools/gates/check-theme-colors.sh`
    ///   enforces the general rule; this asserts the specific consequence.
    ///
    /// The second signal — the stroke on the current hit — is structural
    /// rather than a colour and is asserted by reading [`draw_find_hits`],
    /// which strokes if and only if `current`.
    #[test]
    fn the_current_find_hit_differs_by_emphasis_and_keeps_the_themes_hue() {
        // NOT A THEME COLOUR: a test fixture standing in for whatever the
        // theme supplies; the assertion is that the hue survives, so the exact
        // input has to be a known literal.
        let base = Color32::from_rgb(60, 120, 200);
        let ordinary = at_alpha(base, HIT_ALPHA);
        let current = at_alpha(base, CURRENT_ALPHA);

        for colour in [ordinary, current] {
            let [r, g, b, _] = colour.to_srgba_unmultiplied();
            for (got, want) in [(r, 60u8), (g, 120), (b, 200)] {
                assert!(
                    got.abs_diff(want) <= 6,
                    "a find highlight drifted off the theme's hue: {got} vs {want}"
                );
            }
        }

        // ★ The three relations between the two alphas are checked at
        // COMPILE time rather than here.
        //
        // They are properties of two constants, so a run-time assertion would
        // only re-discover what the compiler can refuse outright — the same
        // argument `crate::app::status`'s `HEIGHT_PTS > ROW_HEIGHT_PTS`
        // makes. They live inside this test rather than beside the constants
        // so the whole colour argument is readable in one place.
        const _: () = assert!(
            CURRENT_ALPHA > HIT_ALPHA * 2,
            // ui-text-exempt: compile-error text, never displayed in the UI
            "the current hit must be obviously different from its neighbours; alpha is one \
             of the two signals and it must not be a subtle one"
        );
        const _: () = assert!(
            HIT_ALPHA < 96,
            // ui-text-exempt: compile-error text, never displayed in the UI
            "a highlight that hides the text it is highlighting defeats its own purpose"
        );
        // ★ The bound that came from a screenshot rather than from reasoning.
        // At 168 the current hit was a solid block over its own word; see
        // `CURRENT_ALPHA`'s docs. 112 is the ceiling that keeps ordinary black
        // text legible through the theme's selection blue in both presets.
        const _: () = assert!(
            CURRENT_ALPHA <= 112,
            // ui-text-exempt: compile-error text, never displayed in the UI
            "the operator's next act after finding a hit is to READ it; a wash this \
             opaque covers the word it is marking"
        );
    }

    /// ★ **The text-selection wash is readable through** — the bound the
    /// current-hit defect established, applied to the surface that needs it
    /// most.
    ///
    /// A find highlight marks one of several candidate answers; a text
    /// selection marks *the characters that are about to be copied*, and the
    /// operator's only way to check them is to read them. So the ceiling is
    /// asserted at compile time against the same value
    /// [`CURRENT_ALPHA`]'s own screenshot-derived bound uses, and the hue is
    /// asserted to be the theme's — a selection wash that borrowed a named
    /// palette entry would break the first time somebody restyled it for its
    /// real purpose. `tools/gates/check-theme-colors.sh` enforces the general
    /// rule; this asserts the specific consequence.
    #[test]
    fn the_text_selection_wash_is_readable_through_and_keeps_the_themes_hue() {
        // NOT A THEME COLOUR: a test fixture standing in for whatever the theme
        // supplies; the assertion is that the hue survives, so the exact input
        // has to be a known literal.
        let base = Color32::from_rgb(60, 120, 200);
        let [r, g, b, a] = at_alpha(base, TEXT_SELECTION_ALPHA).to_srgba_unmultiplied();
        for (got, want) in [(r, 60u8), (g, 120), (b, 200)] {
            assert!(
                got.abs_diff(want) <= 6,
                "the selection wash drifted off the theme's hue: {got} vs {want}"
            );
        }
        assert_eq!(a, TEXT_SELECTION_ALPHA);

        const _: () = assert!(
            TEXT_SELECTION_ALPHA <= CURRENT_ALPHA,
            // ui-text-exempt: compile-error text, never displayed in the UI
            "a text selection is what the operator is about to COPY, and the only way to \
             check it is to read it — it must never be more opaque than the find hit whose \
             opacity was already measured down from a solid block"
        );
        const _: () = assert!(
            TEXT_SELECTION_ALPHA > 0,
            // ui-text-exempt: compile-error text, never displayed in the UI
            "a selection nobody can see is a selection nobody can aim"
        );
    }

    /// A translucent source colour does not get darkened twice — the failure
    /// the accessor choice in [`ghost`] guards against.
    #[test]
    fn a_translucent_theme_colour_keeps_its_hue_through_the_ghost() {
        // NOT A THEME COLOUR: test fixture — a deliberately translucent
        // source, which is the input this test exists to exercise.
        let translucent = Color32::from_rgba_unmultiplied(60, 120, 200, 90);
        let [r, g, b, _] = ghost(translucent).to_srgba_unmultiplied();
        for (got, want) in [(r, 60u8), (g, 120), (b, 200)] {
            assert!(
                got.abs_diff(want) <= 6,
                "premultiplied components were re-premultiplied: {got} vs {want}"
            );
        }
    }
}
