//! # `canvas::rotating` — the ninth grip, and the one gesture the eight could
//! never express
//!
//! ## What this closes
//!
//! `ui-conventions/handles.md` H2 — *"the standard set is eight resize grips, a
//! body, and a rotate handle"* — and the operator's own report, which is the
//! sentence that corpus row quotes:
//!
//! > *"unfortunately there was no way to reposition, resize, or rotate it on
//! > the screen. Can I please please please have that too?"*
//!
//! Reposition landed with `canvas::moving`'s transform fork and resize with
//! `canvas::resizing`'s, both on 2026-08-20. **Rotate is the third word in that
//! sentence and it had no affordance at all** — the verb rotates, and nothing
//! on the canvas reached it.
//!
//! ## ★★ Why a rotate is not a resize with different arithmetic
//!
//! The eight grips answer *"how big"*, which is a **distance**, so a resize is
//! a delta in two axes and every one of them has an opposite corner that must
//! not move. A rotation answers *"which way round"*, which is an **angle**, so:
//!
//! | | resize | rotate |
//! |---|---|---|
//! | measured from | the grip's opposite corner | the selection's **centre** |
//! | what the drag reads | a displacement | the change in bearing between two rays |
//! | what the pointer's *distance* means | everything | **nothing** |
//! | modifier | Shift preserves aspect | Shift snaps to 15° |
//!
//! The third row is the one that decides the module boundary. A rotate drag
//! must ignore how far the pointer is from the centre entirely — an operator
//! swinging a long arc for precision is doing exactly what the gesture invites,
//! and a build that scaled with radius would shrink the object as they did it.
//!
//! ## The handle sits ABOVE the box, on a stem
//!
//! Which is PowerPoint, Illustrator, Figma, Inkscape, Visio and Konva's
//! `Transformer`. The offset is what makes it reachable on a selection whose
//! top edge is crowded with the north grip and whatever is behind it, and the
//! stem is what says the two belong together — without it the handle reads as
//! an unrelated dot floating over the page.
//!
//! ★ **It is drawn as a circle**, not a square. Every square on this canvas
//! resizes; a shape that resized in one place and rotated in another would be
//! a private convention the operator has to learn, which is
//! `handles.md` H2's stated failure mode.
//!
//! ## ★★★ THREE DESTINATIONS SHARE THIS ONE GESTURE — 2026-08-28
//!
//! The header above describes a rotation of **page content**, which is what
//! this module did on the day it was written. `pdfcer-core` `Pass 155.0` and
//! `Pass 159.0` then shipped rotation for the annotation family and for ce
//! dimensions, and this module became the decision point for all three:
//!
//! | selected | verb | operand | refused by |
//! |---|---|---|---|
//! | page **content** | `transform_objects` | paint-order indices + a `Matrix` | — |
//! | a **markup** annotation | `rotate_annotation` | `ObjId` + pivot + degrees | — |
//! | a **ce dimension** | `rotate_dimension` | `DimensionId` + pivot + degrees | `rotate_annotation`, **by name** |
//! | a **form field's** box | *none* | — | `rotate_annotation`, **by name** (`/MK /R`, unbuilt) |
//!
//! **Everything above the branch is shared and must be**: the bearing between
//! two rays from the box's centre, the 15° snap, the wrap that stops a drag
//! past 180° spinning a whole turn, the travel threshold and the single
//! screen→page negation are one gesture whatever is under it. What differs is
//! one call. `canvas::resizing` cuts the identical seam in the identical place
//! and a reader who has understood one has understood both.
//!
//! ★★ The three operands are the **same shape** — a fixed point and a scalar —
//! because the engine chose it that way on this shell's request: *"the same
//! anchor+factors shape as move and resize, so your grip code needs no third
//! convention."* So [`commit_annotation`] is a routing decision rather than a
//! second arithmetic.
//!
//! ## ★★★ Why the box comes from `pressing::grabbable` and not `overlay::grip_box`
//!
//! [`Frame::bounds`] is filled by `canvas::interact` from
//! `crate::canvas::pressing::grabbable`, and that is **the single line the
//! annotation rotation hangs on**.
//!
//! `overlay::grip_box` derives its answer from the selection's cached *content*
//! outlines, which `select_annot` clears — an annotation is not content and has
//! nothing decomposed to cache. Over a selected markup or dimension it answers
//! `None`, so `bounds` would be `None`, so this function would return at its
//! first line and **the entire gesture would be a no-op with nothing said
//! anywhere.**
//!
//! ⇒ `grabbable` is also the function `pressing::look` hit-tests against and the
//! function `canvas::painting` paints from. **One box: what the operator can
//! see, what they can grab, and what turns.** That is rule H7, and it is the
//! guard against the failure this canvas has produced four times — a working
//! gesture aimed at the wrong verb, which never looks broken from a chair
//! because something moves. The most recent instance is recorded in
//! `canvas::presspick`: `covers()` tested the selection's move box alone, the
//! rotate handle sits *outside* that box, and a press on it selected the object
//! underneath, so the rotate became a select-and-move.
//!
//! ## ★★★ …AND THE SECTION ABOVE WAS RIGHT AND STILL SHIPPED THE DEFECT —
//! 2026-08-29
//!
//! On the **first ever driven run** of `rotating_a_markup_turns_it` the rotate
//! handle was painted, was pressed at the centre of the rect this application
//! itself declared, and committed nothing, with nothing said anywhere. The
//! check's own report named [`Frame::bounds`] as the suspect — the section
//! above, quoted back at it — and **it was wrong**: `canvas::interact` has
//! passed `pressing::grabbable`'s box since the day this module landed.
//!
//! The real cause was **fifteen lines further down the same function**: a
//! `selection.object_indices_on(page_index).is_empty()` guard standing *in
//! front of* the annotation branch. It counts page **content**, which
//! `select_annot` clears, so it answered "empty" on every markup and every ce
//! dimension and returned before the routing decision was reached. [`drag`]'s
//! own body carries the full account at the line that moved.
//!
//! ⇒ **The sixth instance of this hazard, and the lesson it adds is about where
//! a guard stands rather than about what it reads.** This one asked a perfectly
//! correct question — *has the content verb got an operand?* — of a gesture
//! that had already been routed away from the content verb. Three destinations
//! share this gesture, so a test written in one destination's vocabulary
//! belongs **after** the branch that picks the destination, never before it.
//! `canvas::resizing` already had it in the right place: its `NothingSelected`
//! test lives inside `resizing::action`, the pure builder for the *content*
//! verb, which the annotation branch returns before ever calling. Every
//! remaining caller of `overlay::grip_box` was audited the same day and none of
//! them was this bug — which is precisely why it survived a header section
//! written to prevent it.
//!
//! ## ★★ No options type, for any of the three, and that is a property of the
//! operation rather than an omission
//!
//! A rotation is an **isometry**: every length is preserved, including the
//! drawn stroke width. So there is no `scale_stroke_width` question here and no
//! `allow_appearance_distortion` — `resize_annotation` has both and needs them,
//! because §12.5.5's placement matrix scales artwork *after* stroking. Rotation
//! composes into the appearance's own `/Matrix` instead, so **a foreign
//! appearance rotates correctly** where it cannot be resized.
//!
//! ⇒ `pdfcer-core` drew the UI consequence and this module is built on it: *"if
//! your grip UI offers rotate and resize together, **rotate needs no
//! confirmation step and no distortion warning.** Resize does."* There is no
//! Tool-row switch for this gesture and no dialog in front of it.
//!
//! ## conventions: drag-moves
//!
//! Corpus: `ui-conventions/drag-moves.md`. The handle rows are answered by
//! `canvas::handles`, which owns the grip.
//!
//! - D1 live-preview: the ghost is the selection's outlines rotated about the
//!   same centre by the same angle the release commits.
//! - D2 derived-from-commit: [`angle`] is called once per frame and its result
//!   feeds the ghost and the commit; there is no second derivation.
//! - D3 escape-cancels: the gesture machine drops it and nothing is written
//!   before `Complete`.
//! - D4 one-undo-entry: one `transform_objects` call over every selected index.
//! - D5 modifiers-constrain: **Shift snaps to 15°**, which is the rotate
//!   flavour of the convention rather than the axis lock — see [`STEP_DEGREES`]
//!   — and it is announced on the status row like every other constraint.
//! - D6 snapping: WAIVED — there is nothing on a page to snap an angle to. A
//!   future "match that line's angle" is a different feature with a target.
//! - D7 no-op-is-not-an-edit: a release at exactly the bearing it began at
//!   raises nothing; see [`is_travel`].
//! - D8 grab-point: WAIVED, and it is the one row that genuinely does not
//!   apply. There is no point under the pointer to preserve — the pointer is
//!   holding a *bearing*, and the handle stays on its stem at the top of the
//!   box because that is where the box's top is.
//! - D9 disclosure: waived for page **content**, and NOT waived for the two
//!   annotation destinations — see `crate::text::rotating`'s header for the
//!   table of what is disclosed and what deliberately is not. Two things are
//!   owed: an annotation's `/Rect` grows at any angle that is not a quarter
//!   turn (§12.5.2 requires it upright), and **this shell draws its selection
//!   outline from `/Rect`**, so the operator watches a box swell around artwork
//!   that did not; and a `Linear` ce dimension's axis lock cannot survive a
//!   rotation, which the engine relaxes and asks us to say — *"an operator
//!   whose dimension silently stopped being axis-locked will find out later and
//!   blame something else."*

use egui::{Pos2, Vec2};

/// How many degrees a constrained rotation snaps to.
///
/// Fifteen, which is PowerPoint's, Illustrator's, Inkscape's and Figma's. It
/// divides 90 and 360 exactly, so the four right angles and the four diagonals
/// are all reachable — which is what the operator actually wants from the key,
/// and what a value like 10° would give them for 90 and take away for 45.
pub const STEP_DEGREES: f32 = 15.0;

/// The smallest rotation, in degrees, that counts as a gesture rather than a
/// twitch.
///
/// `drag-moves` D7: a drag that moves nothing is not an edit. A tenth of a
/// degree over a 200 pt box is a quarter of a pixel at the corner — invisible,
/// and not worth an undo entry for somebody who thought better of it.
const MIN_TRAVEL_DEGREES: f32 = 0.1;

/// **The angle a rotate drag has turned through, in radians.**
///
/// Positive is the direction the pointer went. Both rays are measured from
/// `centre`, and the pointer's *distance* from it is discarded — see the module
/// header for why that is the whole shape of the gesture rather than a detail.
///
/// # ★ Screen space in, screen space out, and the sign survives the hop
///
/// `centre`, `from` and `at` are all screen positions, where y runs **down**.
/// `atan2` therefore answers a bearing in a left-handed frame, so a clockwise
/// drag comes back positive. PDF user space is y-**up**, and
/// `Matrix::rotate(θ)` turns anticlockwise in it — so the caller negates once,
/// at the one place it converts, exactly as `canvas::mapping` does for every
/// other quantity that crosses.
///
/// Doing the flip here would put a page-space fact in a function that has never
/// seen a page, which is how a preview and a commit come to disagree about
/// which way round something went.
///
/// `None` when either ray is degenerate — the pointer exactly on the centre —
/// because a bearing from a zero-length ray is not a number and
/// `atan2(0.0, 0.0)` quietly answers zero rather than saying so.
#[must_use]
pub fn angle(centre: Pos2, from: Pos2, at: Pos2, constrain: bool) -> Option<f32> {
    let a = from - centre;
    let b = at - centre;
    if a.length() < f32::EPSILON || b.length() < f32::EPSILON {
        return None;
    }
    let delta = b.y.atan2(b.x) - a.y.atan2(a.x);
    // ★ Normalised into (-π, π] so a drag that crosses the ray behind the
    // centre turns the short way rather than jumping a full turn. Without it a
    // pointer moving smoothly through 180° makes the object spin the other way
    // round in one frame — a real defect in every naive implementation of this
    // gesture, and one that looks like a physics bug rather than an arithmetic
    // one.
    let delta = normalise(delta);
    Some(if constrain { snap(delta) } else { delta })
}

/// Wrap a radian difference into `(-π, π]`.
///
/// Its own function so the property has somewhere to be tested. See [`angle`]
/// for the defect it prevents.
#[must_use]
pub fn normalise(mut radians: f32) -> f32 {
    let turn = std::f32::consts::TAU;
    while radians > std::f32::consts::PI {
        radians -= turn;
    }
    while radians <= -std::f32::consts::PI {
        radians += turn;
    }
    radians
}

/// Round a radian angle to the nearest [`STEP_DEGREES`].
///
/// ★ It snaps the **total turn**, not the increment. Accumulating snapped
/// increments would let a slow drag through 90° arrive at 87°, because each
/// frame's small delta rounds to zero — the classic error, and the reason this
/// takes the whole angle rather than a per-frame one.
#[must_use]
pub fn snap(radians: f32) -> f32 {
    let step = STEP_DEGREES.to_radians();
    (radians / step).round() * step
}

/// Whether a rotation is big enough to be an edit — `drag-moves` D7.
#[must_use]
pub fn is_travel(radians: f32) -> bool {
    radians.abs() >= MIN_TRAVEL_DEGREES.to_radians()
}

/// Rotate a screen point about a screen centre, for the ghost.
///
/// ★ The ghost is drawn from **this** function and the commit from
/// `Matrix::rotate(θ).about(centre)`, which are the same map in two spaces —
/// and that is the one duplication in this module. It is not avoidable: the
/// preview must be drawn in screen space before the page conversion, and the
/// commit must be expressed in the engine's own type. What makes it safe is
/// that both take **the same θ from [`angle`]**, so the two can differ only in
/// the y-flip, which is a sign a unit test can pin.
#[must_use]
pub fn rotate_about(centre: Pos2, p: Pos2, radians: f32) -> Pos2 {
    let (s, c) = radians.sin_cos();
    let v = p - centre;
    centre + Vec2::new(v.x * c - v.y * s, v.x * s + v.y * c)
}

/// **Apply one frame of a rotate drag: preview it, or commit it.**
///
/// Mirrors [`crate::canvas::resizing::drag`] deliberately, down to the return
/// type, so a reader who has understood one has understood both. What it hands
/// back is the **angle** for the ghost, where the resize hands back two factors
/// and the move hands back a displacement.
///
/// Returns `Some(radians)` only when a ghost should be drawn — which, by this
/// project's honesty contract, is exactly when a release would commit.
///
/// # ★★ The negation, and it happens exactly once
///
/// [`angle`] measures in **screen** space, where y runs down, so a clockwise
/// drag comes back positive. `Matrix::rotate` turns anticlockwise in PDF user
/// space, where y runs **up**. The single `-` below is that crossing, and it
/// lives here rather than in `angle` for the reason `canvas::mapping`'s header
/// gives about every other conversion: one place, or the preview and the commit
/// eventually disagree about which way round something went — which is a defect
/// that looks like a deliberate feature.
///
/// The ghost is drawn from the **un-negated** angle, in screen space, by
/// `overlay::draw_rotate_ghost`. Both come from one call to [`angle`], so the
/// only thing that can differ between them is this sign.
pub fn drag(
    ctx: &egui::Context,
    frame: Frame<'_>,
    selection: &crate::canvas::selection::SelectionState,
    actions: &mut Vec<crate::app::actions::Action>,
) -> Option<f32> {
    let Frame {
        from,
        at,
        phase,
        bounds,
        page_index,
        constrain,
        map,
        page,
        dimension,
    } = frame;
    let bounds = bounds?;
    // ★ The pivot is the CENTRE, taken from `Grip::pivot` rather than from
    // `bounds.center()` here. Same number today; one statement of "what does a
    // rotate turn about", so the ghost, the commit and any future third reader
    // cannot drift.
    let centre = crate::canvas::handles::Grip::Rotate.pivot(bounds);
    let map = map?;
    // Screen space throughout, because that is what `bounds` is: the two rays
    // and the centre have to be in one frame, and the grip box is the frame the
    // handle was drawn in.
    let theta = angle(centre, map.to_screen(from), map.to_screen(at), constrain)?;
    if constrain {
        crate::canvas::constrain::announce(ctx, crate::canvas::constrain::Lock::Angle);
    }
    if phase == crate::canvas::gesture::Phase::InFlight {
        return Some(theta);
    }

    // ---- commit ------------------------------------------------------
    if !is_travel(theta) {
        // `drag-moves` D7. Silent: a release at the bearing it began at is an
        // operator who thought better of it, and a sentence about it would be
        // reporting their change of mind back to them.
        return None;
    }
    let page = page?;
    // The SAME two hops every other commit on this canvas takes, in the same
    // order, through the same two functions — screen → canvas → PDF user space.
    let pdf = crate::viewer::canvas_to_pdf_space(map.to_page(centre), page)?;
    let pivot = pdfcer_core::vector::Point::new(f64::from(pdf.x), f64::from(pdf.y));
    // ★★★ AN ANNOTATION TAKES A DIFFERENT VERB, and the branch is here — after
    // the angle and the pivot, before the content action.
    //
    // The same seam `canvas::resizing` cuts, in the same place, for the same
    // reason: **everything above this line is shared and must be.** The bearing
    // between two rays, the centre it is measured from, the 15° snap, the wrap
    // that stops a drag past 180° spinning a whole turn, the travel threshold
    // and the screen→page conversion are one gesture whatever is under it. What
    // differs is which verb the angle is handed to.
    //
    // ★★ And the operand shape is the SAME for all three — pivot plus a scalar
    // angle — because the engine chose it that way deliberately: *"the same
    // anchor+factors shape as move and resize, so your grip code needs no third
    // convention."* This branch is therefore a **routing decision** rather than
    // a second arithmetic, exactly as the resize's is.
    if let Some(annot) = selection.annot() {
        commit_annotation(annot, dimension, pivot, theta, actions);
        return None;
    }
    // ★★★ **THE CONTENT GUARD, AND IT IS BELOW THE ANNOTATION BRANCH.** Moved
    // here on 2026-08-29, on the FIRST driven run of
    // `rotating_a_markup_turns_it`, and it is the whole defect that run found.
    //
    // It used to stand five lines above the `is_travel` check's successor —
    // *before* the annotation branch — as:
    //
    //     let objects = selection.object_indices_on(page_index);
    //     if objects.is_empty() { return None; }
    //
    // ## What that cost
    //
    // `object_indices_on` counts **page content**. `select_annot` clears the
    // content selection — an annotation is not content — so on every markup and
    // every ce dimension it answers an empty vector. The guard therefore
    // returned `None` before `selection.annot()` was ever asked, and the entire
    // annotation rotation was **consumed and discarded with nothing said
    // anywhere**: the handle painted, the press hit, the ghost turned, the
    // release did nothing, and no line reached the trace to say why. That is
    // this project's founding defect shape, D4a, reproduced inside the module
    // written to close it.
    //
    // ## ★★★ THIS IS THE SIXTH INSTANCE OF ONE HAZARD IN THIS CANVAS
    //
    // The fifth was `presspick::covers()`, fixed on 2026-08-28 — the same wrong
    // question at a different call site — and the rule written there is the one
    // this fix obeys:
    //
    // > **A guard that must agree with another module has to CALL it, not
    // > resemble it.**
    //
    // The generalisation the sixth instance adds, because *this* guard called
    // nothing and resembled nothing: **a guard derived from CONTENT state must
    // not stand in front of a branch about an ANNOTATION.** Three destinations
    // share this gesture (the header's table) and only one of them is content,
    // so any test written in content's vocabulary belongs *after* the routing
    // decision, never before it. `canvas::resizing` — the sibling gesture,
    // cutting the identical seam — already has it in the right place: its
    // `NothingSelected` test lives inside `resizing::action`, the pure builder
    // for the **content** verb, which the annotation branch returns before ever
    // reaching.
    //
    // ## ★★ And it now SPEAKS, which the old guard did not
    //
    // A leaf-only selection is the reachable case: `object_indices_on` keeps
    // entries with a `page_object_index` and drops the ones with only a
    // `leaf_index`, so an operator who has clicked *into* a form XObject has an
    // outline, a grip box, a painted rotate handle — and no operand. Before
    // this, that drag returned silently. `SelectionState::leaf_indices_on`'s
    // own header states the distinction it exists to let a caller word: *"you
    // selected nothing"* versus *"you selected something this verb cannot
    // reach"*. This is the second, and it is now a sentence.
    let objects = selection.object_indices_on(page_index);
    if objects.is_empty() {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            //
            // ★ It carries the page, because the guard is per-page: a selection
            // made on another sheet is the commonest way to arrive here with a
            // non-empty selection and no operand.
            format!("rotate-declined reason=nothing-selected page={page_index}")
        });
        // ★ Epoch zero, for `resizing::decline`'s stated reason: a refusal
        // changed nothing, so there is no edit for it to be about, and it must
        // retire on the operator's NEXT act rather than on the document's next
        // edit.
        crate::app::actions::record_note(
            0,
            crate::text::rotating::RotateRefusal::NothingSelected
                .line()
                .to_owned(),
        );
        return None;
    }
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        //
        // ★ It carries the ANGLE IN DEGREES and the pivot, which is what a
        // wrong build gets wrong: one that turned the other way, one that
        // pivoted about a corner instead of the centre, and one that snapped
        // when it should not are all `rotate-commit` otherwise.
        format!(
            "rotate-commit deg={:.2} px={:.2} py={:.2} objects={} constrained={}",
            (-theta).to_degrees(),
            pivot.x,
            pivot.y,
            objects.len(),
            u8::from(constrain),
        )
    });
    actions.push(
        crate::app::actions::VectorAction::TransformObjects {
            page: page_index,
            objects,
            // ★★ NEGATED here, and nowhere else. See this function's header.
            matrix: pdfcer_core::vector::Matrix::rotate(f64::from(-theta)).about(pivot),
        }
        .into(),
    );
    None
}

/// The frame's facts about a rotate drag in flight.
///
/// The `Frame` shape `canvas::resizing` and `canvas::handledrag` already use,
/// and for the reason they give: the members are read-only facts about one
/// frame, so grouping them says what they are and removes the failure a long
/// parameter list invites — `from` and `at` are both `Pos2` in the same space
/// and swapping them would compile and turn the object backwards.
#[derive(Clone, Copy)]
pub struct Frame<'a> {
    /// Canvas-space position of the press — the first ray.
    pub from: Pos2,
    /// Canvas-space position of the pointer now — the second ray.
    pub at: Pos2,
    /// Draw the ghost, or commit.
    pub phase: crate::canvas::gesture::Phase,
    /// The selection's grip box in **screen** space, or `None` if there is no
    /// outline to have grabbed.
    pub bounds: Option<egui::Rect>,
    /// The page the drag is on.
    pub page_index: usize,
    /// Whether Shift is down **this frame** — snap to [`STEP_DEGREES`].
    pub constrain: bool,
    /// The frame's screen ⟷ canvas mapping.
    pub map: Option<&'a crate::canvas::mapping::PageMapping>,
    /// The page itself, for the canvas → PDF hop.
    pub page: Option<&'a pdfcer_core::page_tree::Page>,
    /// **The selected ce dimension's sidecar record id**, when the selection is
    /// one.
    ///
    /// ★★★ Carried rather than looked up, exactly as
    /// [`crate::canvas::resizing::Frame::selected_field`] is, and for a
    /// stronger version of that field's reason.
    ///
    /// `rotate_dimension` addresses the **sidecar record**, not the annotation:
    /// the selection carries an `ObjId` and the record carries a `DimensionId`,
    /// and the sidecar stores the mapping one way only (`record.annot`), so the
    /// reverse lookup is a scan over the document's dimensions. That scan needs
    /// `&OpenDoc`, which this module deliberately does not take — everything in
    /// it is a pure function of its [`Frame`], which is what lets the whole
    /// gesture be unit-tested without a window or a document.
    ///
    /// ★★ Resolving it in the **gesture** rather than in the apply arm is also
    /// what lets a rotation with no record behind it **decline in words**. An
    /// action raised with no operand would reach the engine, be refused, and —
    /// on the generic arm — say nothing at all. See [`drag`]'s commit path.
    ///
    /// `None` for a markup annotation, for page content, and for a ce dimension
    /// whose record could not be resolved. The first two never read it.
    pub dimension: Option<pdfcer_core::dimension::DimensionId>,
}

/// **Raise the right rotation verb for a selected annotation.**
///
/// ★★★ The routing, in one `match` the compiler checks — which is the whole
/// reason `canvas::selection::annot::AnnotKind` is an enum rather than an
/// `is_ce_dimension: bool` on the target. Its header states the rule this
/// function is the newest instance of: *"a bool is a fact a caller may forget
/// to read, while a variant is one the compiler makes them handle."*
///
/// # ★★★ The two verbs are NOT interchangeable, and the engine refuses to let
/// them be
///
/// `rotate_annotation` returns `AnnotationMoveWrongVerb` for a ce dimension and
/// points at `rotate_dimension`, with the reason attached: *"a ce dimension's
/// orientation is part of its measurement, so turning it must re-measure rather
/// than spin a rectangle."*
///
/// A dimension is a `/Line` with `/IT /LineDimension` and a record in the
/// document's `/PieceInfo` sidecar. Handing one to the annotation verb would
/// turn its `/Rect` and its baked `/AP` and leave the **sidecar geometry** —
/// the thing the displayed number is derived from — where it was, so the
/// dimension would draw at one angle and measure along another.
///
/// ⇒ `pdfcer-core` refuses that by name and this shell routes rather than
/// forces. The refusal stays as the backstop; this is what stops it being
/// reached.
///
/// # ★★ A widget cannot arrive here at all, and that is the R9 answer
///
/// `rotate_annotation` also refuses a **widget** by name — a widget's rotation
/// is `/MK /R` (§12.5.6.19 Table 189), a quantised 0/90/180/270 *declaration*
/// the field's appearance generator reads rather than a free-angle transform,
/// and it is not built.
///
/// There is no arm for it because there is no path to one:
/// `canvas::selection::annot` excludes `/Widget` from annotation selection
/// outright (the form surface owns those presses), so a widget is a
/// `doc.selected_field` rather than a `selection.annot()`, and
/// `pressing::grabbable` hands that selection `GripSet::scale_only()` — **no
/// rotate handle is painted and none is hit-tested.** R9: render nothing rather
/// than draw a handle that refuses.
fn commit_annotation(
    annot: &crate::canvas::selection::AnnotSelection,
    dimension: Option<pdfcer_core::dimension::DimensionId>,
    pivot: pdfcer_core::vector::Point,
    theta: f32,
    actions: &mut Vec<crate::app::actions::Action>,
) {
    // ★★ NEGATED here, on the same line of reasoning as the content commit
    // below and at the same point in the flow: `angle` measures in SCREEN
    // space, where y runs down, and `rotate_annotation` takes degrees
    // ANTICLOCKWISE in PDF user space, where y runs up. One negation, at the
    // one crossing. See this module's header.
    //
    // ★ Degrees rather than radians, because that is what both verbs take —
    // and unlike the content path, which builds a `Matrix::rotate(radians)`,
    // there is no matrix here to hide the unit in. The conversion is explicit
    // so a reader can check it against the trace, which also reports degrees.
    let degrees = f64::from((-theta).to_degrees());
    match annot.target.kind {
        crate::canvas::selection::AnnotKind::Markup => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                //
                // ★ It carries the ANGLE and the PIVOT — what a wrong build
                // gets wrong. A build that turned the other way, or pivoted
                // about a corner instead of the centre, is `rotate-annot-commit`
                // otherwise, and both are perfectly good rotations that look
                // entirely deliberate to anybody who did not watch the pointer.
                format!(
                    "rotate-annot-commit id={} deg={degrees:.2} px={:.2} py={:.2}",
                    annot.target.id.num, pivot.x, pivot.y
                )
            });
            actions.push(crate::app::actions::Action::Annot(
                crate::app::actions::annot::AnnotAction::Rotate {
                    id: annot.target.id,
                    pivot: (pivot.x, pivot.y),
                    degrees,
                },
            ));
        }
        crate::canvas::selection::AnnotKind::CeDimension => {
            // ★★★ **A REFUSAL MUST BE A SENTENCE, NEVER A SILENCE.**
            //
            // Without this arm a dimension whose sidecar record could not be
            // resolved would produce the exact defect this project was started
            // to remove: the operator drags the handle, watches the ghost turn,
            // lets go, and the dimension snaps back with nothing said anywhere.
            //
            // ★ It is a refusal the SHELL can answer — a query, not something
            // only the engine knows — so it is worded here rather than from the
            // apply phase.
            //
            // ★★★ **Through `record_note`, not through
            // `app::status::decline`,** and the difference is a module boundary
            // that is deliberate rather than incidental. `app::status::decline`
            // is `pub(super)` inside `crate::app`, with its own argument
            // attached: *"a decline is written by the one dispatcher and read
            // by the one bar."* The canvas is outside that boundary, and
            // `canvas::resizing::decline` — the sibling gesture, refusing the
            // sibling way — already takes the route this takes.
            //
            // ⇒ So the two rotation channels are split by WHO KNOWS: a
            // condition the gesture can see goes on the note channel from here,
            // and the three the engine returns go on the decline channel from
            // `app::actions::annots`, inside the boundary. One sentence
            // catalog, `crate::text::rotating`, serves both.
            //
            // ★ **Epoch zero**, exactly as `resizing::decline` uses and for its
            // stated reason: a refusal changed nothing, so there is no edit for
            // it to be about, and `record_note` keys on the epoch so a
            // disclosure retires when the document moves past it. Passing the
            // live epoch would leave this sentence on screen through every
            // subsequent edit.
            //
            // ★★ Unreachable while `pressing::grabbable` holds:
            // `dimdrag::grab_box` answers `Some` only for a dimension
            // `dimdrag::selected` resolved, which is where this id comes from.
            // It is worded anyway, for `crate::text::rotating::RotateRefusal`'s
            // stated reason — a routing bug with a sentence is a bug report,
            // and one without is a handle that does nothing.
            let Some(dimension) = dimension else {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    format!(
                        "rotate-declined reason=no-dimension-record annot={}",
                        annot.target.id.num
                    )
                });
                crate::app::actions::record_note(
                    0,
                    crate::text::rotating::RotateRefusal::NoDimensionRecord
                        .line()
                        .to_owned(),
                );
                return;
            };
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                //
                // ★ It carries BOTH ids. A wrong build here is one that
                // resolved the reverse lookup to a different record — a
                // perfectly good rotation of the wrong dimension — and the two
                // numbers side by side are the only place that is visible.
                format!(
                    "rotate-dim-commit dim={} annot={} deg={degrees:.2} px={:.2} py={:.2}",
                    dimension.0, annot.target.id.num, pivot.x, pivot.y
                )
            });
            actions.push(crate::app::actions::Action::Annot(
                crate::app::actions::annot::AnnotAction::RotateDimension {
                    dimension,
                    annot: annot.target.id,
                    pivot: (pivot.x, pivot.y),
                    degrees,
                },
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn deg(d: f32) -> f32 {
        d.to_radians()
    }

    /// ★ **A quarter turn clockwise on screen reads as +90°.**
    ///
    /// The base case, and the one whose sign is easy to get backwards: screen y
    /// is DOWN, so a pointer moving from due-east to due-south of the centre has
    /// gone clockwise, and `atan2` in a y-down frame calls that positive. The
    /// caller negates once when it crosses into page space; getting the sign
    /// wrong here would rotate the object the other way and look like a
    /// perfectly deliberate feature.
    #[test]
    fn a_clockwise_quarter_turn_on_screen_is_positive() {
        let c = Pos2::new(100.0, 100.0);
        let got = angle(c, Pos2::new(200.0, 100.0), Pos2::new(100.0, 200.0), false).expect("rays");
        assert!((got - deg(90.0)).abs() < 1e-4, "got {}", got.to_degrees());
    }

    /// ★★ **The pointer's DISTANCE from the centre changes nothing.**
    ///
    /// The property that separates this gesture from a resize, asserted rather
    /// than assumed: an operator swinging a long arc for precision is doing what
    /// the gesture invites, and a build that let the radius in would shrink or
    /// grow the object while they did it.
    #[test]
    fn the_radius_does_not_matter() {
        let c = Pos2::new(0.0, 0.0);
        let near = angle(c, Pos2::new(10.0, 0.0), Pos2::new(0.0, 10.0), false).expect("rays");
        let far = angle(c, Pos2::new(900.0, 0.0), Pos2::new(0.0, 4.0), false).expect("rays");
        assert!((near - far).abs() < 1e-4, "{near} vs {far}");
    }

    /// ★★ **A drag past 180° turns the SHORT way rather than spinning back.**
    ///
    /// Without the normalisation a pointer moving smoothly through the ray
    /// behind the centre makes the object jump a full turn in one frame. It
    /// looks like a physics bug and it is an arithmetic one.
    #[test]
    fn crossing_the_far_ray_does_not_spin_a_whole_turn() {
        assert!((normalise(deg(190.0)) - deg(-170.0)).abs() < 1e-4);
        assert!((normalise(deg(-190.0)) - deg(170.0)).abs() < 1e-4);
        assert!((normalise(deg(179.0)) - deg(179.0)).abs() < 1e-4);
    }

    /// Shift lands on the right angles and the diagonals, which is what 15°
    /// divides both of.
    #[test]
    fn the_constraint_reaches_the_angles_anybody_wants() {
        for want in [0.0, 15.0, 45.0, 90.0, 180.0] {
            let near = deg(want + 4.0);
            assert!(
                (snap(near) - deg(want)).abs() < 1e-4,
                "{want}° was not reachable from {}°",
                near.to_degrees()
            );
        }
    }

    /// ★ **The TOTAL turn snaps, not each increment.**
    ///
    /// Accumulating snapped increments lets a slow drag through 90° arrive at
    /// 87°, because each frame's small delta rounds to zero. Asserted as the
    /// property rather than by simulating frames: `snap` is called on the whole
    /// angle and there is nowhere for an increment to be rounded.
    #[test]
    fn a_slow_drag_still_reaches_the_step() {
        // Half a degree at a time is what a careful hand produces; each one
        // snaps to zero, and their sum does not.
        assert!((snap(deg(0.5))).abs() < 1e-6, "one small step is no turn");
        assert!(
            (snap(deg(88.0)) - deg(90.0)).abs() < 1e-4,
            "the accumulated angle snaps to the step it is nearest"
        );
    }

    /// A twitch is not an edit.
    #[test]
    fn a_twitch_is_not_travel() {
        assert!(!is_travel(deg(0.05)));
        assert!(is_travel(deg(1.0)));
    }

    /// The pointer exactly on the centre has no bearing, and says so rather
    /// than answering zero.
    #[test]
    fn a_degenerate_ray_has_no_angle() {
        let c = Pos2::new(50.0, 50.0);
        assert!(angle(c, c, Pos2::new(60.0, 50.0), false).is_none());
        assert!(angle(c, Pos2::new(60.0, 50.0), c, false).is_none());
    }

    /// ★ **The ghost's rotation agrees with the angle that produced it.**
    ///
    /// `rotate_about` is the one duplication in this module — the ghost is drawn
    /// from it and the commit from `Matrix::rotate`. This pins the half that can
    /// be checked without a document: a point rotated by the angle measured
    /// between two rays lands on the second ray.
    #[test]
    fn the_ghost_map_agrees_with_the_measured_angle() {
        let c = Pos2::new(0.0, 0.0);
        let from = Pos2::new(100.0, 0.0);
        let at = Pos2::new(0.0, 100.0);
        let theta = angle(c, from, at, false).expect("rays");
        let moved = rotate_about(c, from, theta);
        assert!((moved.x - at.x).abs() < 1e-3, "x {moved:?}");
        assert!((moved.y - at.y).abs() < 1e-3, "y {moved:?}");
    }
}
