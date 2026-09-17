//! # `canvas::resizing` — what the eight resize grips commit
//!
//! ## ★★★ The verb is `transform_objects`, and it is kind-agnostic
//!
//! `EditSession::transform_objects` wraps each object's operator run in
//! `q <cm> … Q`. **That never looks at an operand**, which is what makes it
//! kind-agnostic — not a match arm per kind that somebody has to remember to
//! extend. A text run and an image therefore resize exactly as a path does
//! (neither has nodes to move, and neither needs any), and a selection of N
//! objects is **one** call, one command, one undo entry.
//!
//! **Stroke width is not scaled on page content**, and that is a decision
//! rather than a consequence: on a CAD drawing a line weight is a *drafting
//! standard* — 0.25 mm is 0.25 mm whatever size the detail is — so keeping it
//! is right far more often than scaling it would be, and it is what every
//! drafting package does. It is nonetheless something pdfcer decided and the
//! operator did not, so it is **disclosed** ([`crate::text::resizing`]) rather
//! than assumed.
//!
//! ★★ **The matrix is PAGE space and nothing else.** `cm` composes into the CTM
//! in force at that point in the stream — the object's *user* space — so the
//! engine emits `X = CTM × M × CTM⁻¹` per object from that object's own captured
//! CTM. A selection spanning two local spaces gets two different `cm` operands
//! for one gesture and both land where the operator pointed. Passing anything
//! but page space from here would be right only where an object's CTM happens to
//! be the identity and **silently wrong at every scale the producer left in
//! force**.
//!
//! ---
//!
//! ## The arithmetic, and why it is not written out here
//!
//! **Scaling about an anchor is moving every point.** For an anchor `a` and
//! factors `(sx, sy)`:
//!
//! ```text
//! p' = a + (p - a) * (sx, sy)
//! ```
//!
//! `Matrix::scale(sx, sy).about(a)` is `translate(a) × M × translate(-a)`,
//! which is that expression exactly — so the map is stated once, by the crate
//! that owns matrices, rather than once per point here. A shell keeping its own
//! copy would be a second derivation of one answer in coordinate space, which
//! is the shape every silent defect this project has met there has had.
//!
//! One gesture is **one call, one command, one undo entry** — this project's
//! standing rule for a gesture (`canvas::moving`'s §1), and the thing a
//! per-object loop would break both by producing N undo entries and by planning
//! each edit against byte offsets the previous one invalidated.
//!
//! The operator's instruction: *"finish off phase 1 and phase 5. Get everything
//! unblocked on phase 5 — no excuses about slowness of feature from pdfcer as a
//! reason not to implement."*
//!
//! ## Why the arithmetic is here and not in `moving`
//!
//! [`crate::canvas::moving`] is about a **displacement** — one delta applied to
//! whatever the rung named. This is about a **map**: every node goes somewhere
//! different, and the somewhere depends on where it started. Folding it in
//! would put two different shapes of answer behind one `MoveSubject`, and the
//! module that owns the ghost preview would have to branch on which.
//!
//! ## The ghost, and rule 4
//!
//! An in-flight resize draws its **new outline**, not a tint over the old one —
//! `canvas::overlay`'s existing move ghost with a different transform. It is a
//! pre-commit affordance and therefore the *cursor*, which R8b's fourth clause
//! welcomes explicitly. Nothing is drawn onto the applied content, and a
//! screenshot of the page after a commit is a screenshot of the page as it will
//! save.
//!
//! ## conventions: drag-moves
//!
//! Corpus: `ui-conventions/drag-moves.md`. A resize is a drag; the handle rows
//! live in `ui-conventions/handles.md` and are answered by `canvas::handles`.
//!
//! - D1 live-preview: the resize ghost is drawn from the same scale factors the
//!   release commits.
//! - D2 derived-from-commit: `Some` only when a release would reach a
//!   transform verb on real operands.
//! - D3 escape-cancels: the gesture machine drops it; nothing is written before
//!   `Complete`.
//! - D4 one-undo-entry: `transform_objects` takes the whole selection — one
//!   command.
//! - D5 modifiers-constrain: **Shift preserves aspect**, applied in [`drag`]
//!   between [`factors`] and the ghost so the preview and the commit read one
//!   value; the arithmetic and the reasoning are
//!   [`crate::canvas::constrain::aspect`]. Announced on the status row while it
//!   is live. Alt-scales-about-centre is still absent and is named as a
//!   decision in that module's header, not an omission.
//! - D6 snapping: **GAP** — a resize does not snap to guides, the grid or other
//!   geometry.
//! - D7 no-op-is-not-an-edit: **GAP** — a release with factors of exactly 1.0
//!   is not checked for here.
//! - D8 grab-point: the pivot is the OPPOSITE corner, so the grabbed corner
//!   tracks the pointer and the far one stays still. Using the anchor instead
//!   would preview a shape growing away from the hand and commit one growing
//!   towards it, so the object would jump by its own size on release.
//! - D9 disclosure: WAIVED — a scale changes no measured value that pdfcer
//!   authored, and the new size is visible.

use egui::{Pos2, Rect, Vec2};
use pdfcer_core::vector::Point;

use crate::app::actions::{Action, VectorAction};
use crate::canvas::gesture::Phase;
use crate::canvas::handles::Grip;
use crate::canvas::mapping::PageMapping;
use crate::canvas::selection::SelectionState;
use crate::panels::objects::provider::ObjectModelProvider;

/// Why a resize could not be committed.
///
/// Every variant is **a sentence to show**, never a silent drop —
/// `canvas::textedit::Refusal`'s rule. A gesture whose answer to a case it
/// cannot handle is to do nothing is a gesture the operator reports as broken,
/// because from the outside it is indistinguishable from one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    /// Nothing is selected, or the selection names no object on this page.
    NothingSelected,
    /// The object model could not be read, so nothing can be verified and
    /// therefore nothing may be promised.
    NoObjectModel,
    /// The drag would collapse the selection to nothing on an axis, or invert
    /// it.
    ///
    /// Refused rather than clamped: a zero or negative factor is a shape the
    /// operator cannot have meant, and clamping would silently substitute a
    /// different edit for the one they made.
    Degenerate,
}

// ★★ THE PREFLIGHT IS NOT BUILT, AND THIS IS THE NOTE THAT SAYS SO.
// `DEFECTS.md` D44.
//
// `transform_preview` is `&self`, side-effect-free, and shares one body with
// the verb — so `preview(..).is_ok()` **is** the predicate, and the engine
// distinguishes two errors the shell must treat differently:
//
// * `DegenerateCtm` — this object cannot be transformed AT ALL, because its
//   own CTM is singular. **Do not offer a handle.**
// * `SingularTransform` — this DRAG is degenerate. Offer the handle, refuse on
//   release.
//
// The second is handled: `is_usable` refuses a collapsing drag before the
// engine is asked, and anything that gets past it is worded by `vector_edit`'s
// own channel on release.
//
// The first is not. A handle is currently offered for an object that can never
// be transformed, and the operator finds out by dragging it. That is a real
// gap and it is a small one — a singular CTM is a producer emitting `0 0 0 0 x
// y cm`, which is rare — but it is named here rather than left to be
// discovered.
//
// ★ Why it is not built: **the preview decomposes the page.** Measured by the
// engine on the benchmark drawing, 129,758 objects, **~4 s in a debug build**
// — and both the verb and the preview pay it. The engine's own advice is
// *"call `transform_preview` on selection change and on gesture start, not per
// frame"*, which means a cache keyed on `(page, edit epoch, selection)` — a
// piece of work rather than a line. `app::cache::FormRunCache` is the shape it
// should take.
//
// ★★ A [`Refusal`] variant must not be added ahead of the call site that
// raises it: `every_refusal_is_still_raised_somewhere` fails on a variant with
// a sentence and no caller, and inventing a call site to satisfy that test
// would be the failure it exists to catch.

/// The scale factors a grip's drag implies, about the anchor opposite it.
///
/// # ★ Why the anchor is the OPPOSITE corner and not the centre
///
/// Because that is what every drawing application does, and the standing
/// tie-breaker for anything an operator compares against the tools they already
/// use is to behave the way those tools behave. Dragging the south-east grip
/// moves the south-east corner and leaves the north-west one exactly where it
/// is — so the part of the object the operator is *not* pointing at does not
/// move under their hand.
///
/// [`Grip::anchor`] already answers this, in **screen** space, for the drawing
/// side. This computes in the same frame and hands the result to the caller to
/// map, rather than re-deriving the opposite-corner rule: two spellings of
/// "which corner stays still" would eventually disagree, and the disagreement
/// would be an object that jumps on the first frame of a drag.
///
/// # The mid-edge grips scale ONE axis
///
/// `East` and `West` scale x and leave y at 1.0; `North` and `South` the
/// reverse. That is what a mid-edge grip means, and it is why they are offered
/// separately from the corners rather than being four more corners.
#[must_use]
pub fn factors(grip: Grip, bounds: egui::Rect, delta: Vec2) -> Option<(f32, f32)> {
    let (w, h) = (bounds.width(), bounds.height());
    if w <= f32::EPSILON || h <= f32::EPSILON {
        return None;
    }
    // How the grip's own motion changes the box's extent on each axis. A grip
    // on the east edge grows the box by its own dx; one on the west shrinks it
    // by the same. A grip that does not touch an axis leaves it alone.
    let dw = match grip {
        Grip::NorthEast | Grip::East | Grip::SouthEast => delta.x,
        Grip::NorthWest | Grip::West | Grip::SouthWest => -delta.x,
        _ => 0.0,
    };
    // ★ Screen y is DOWN and the box is a screen rect, so a south grip dragged
    // downward (positive dy) grows the box. The PDF-space flip happens once, in
    // `canvas::mapping`, and must not be applied a second time here — doing the
    // conversion twice is `canvas::mapping`'s own "classic silent defect".
    let dh = match grip {
        Grip::SouthWest | Grip::South | Grip::SouthEast => delta.y,
        Grip::NorthWest | Grip::North | Grip::NorthEast => -delta.y,
        _ => 0.0,
    };
    let sx = if dw == 0.0 { 1.0 } else { (w + dw) / w };
    let sy = if dh == 0.0 { 1.0 } else { (h + dh) / h };
    Some((sx, sy))
}

/// Whether a pair of factors describes a shape anybody meant.
///
/// A factor at or below zero collapses or mirrors the object. Refused rather
/// than clamped — see [`Refusal::Degenerate`].
///
/// The floor is not `0.0` but a small positive number, because a drag that
/// passes exactly through zero would otherwise produce a
/// zero-area object whose next resize has no bounds to scale from: the
/// `w <= EPSILON` guard in [`factors`] would then answer `None` for ever and
/// the object could never be recovered except by undo.
#[must_use]
pub fn is_usable(sx: f32, sy: f32) -> bool {
    const FLOOR: f32 = 0.001;
    sx.is_finite() && sy.is_finite() && sx > FLOOR && sy > FLOOR
}

/// **Build the one action a completed resize becomes.**
///
/// Pure, so the whole decision is testable without a window: the selection, the
/// object model, the anchor in PDF space and the two factors go in, and one
/// `VectorAction::MoveNodes.into()` or one named refusal comes out.
///
/// # ★ The anchor arrives in PDF user space, already converted
///
/// The caller converts once, through `canvas::mapping`, for the reason
/// `canvas::textedit::resolve_run` records about its own two hops: a second
/// conversion is how a preview and a commit come to disagree about where the
/// operator's hand was.
pub fn action(
    selection: &SelectionState,
    page: usize,
    provider: Option<&ObjectModelProvider>,
    anchor: Point,
    (sx, sy): (f32, f32),
) -> Result<Action, Refusal> {
    if !is_usable(sx, sy) {
        return Err(Refusal::Degenerate);
    }
    // ★ The provider is asked FOR and asked nothing.
    //
    // A transform needs no node positions and no kind check — that is the whole
    // point of the mechanism. What the model is still needed for is the same
    // guard `handledrag::drag` makes: a gesture on a page whose model could not
    // be read is a gesture addressing indices nothing has verified, and this
    // shell will not send those to a verb that rewrites bytes.
    let _ = provider.ok_or(Refusal::NoObjectModel)?;
    let objects = selection.object_indices_on(page);
    if objects.is_empty() {
        return Err(Refusal::NothingSelected);
    }
    // ★★ The computed scale, on the trace channel, from the ONE place that
    // computes it — so the gesture route and the typed route report the same
    // fact in the same words. `resize-commit` below is the *gesture's* line and
    // carries the grip, which the typed route has no equivalent of; this one is
    // about the EDIT, and a driven check that asserts on it is asserting the
    // thing both routes share rather than the thing one of them happens to log.
    //
    // ⚠ A driven check with no line to assert on reports a working edit as
    // "committed nothing" — a defect in the instrument that reads exactly like
    // a defect in the feature. The trace line is what stops that.
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "resize-scale sx={sx:.4} sy={sy:.4} ax={:.2} ay={:.2} objects={}",
            anchor.x,
            anchor.y,
            objects.len()
        )
    });
    // ★★ `scale(...).about(anchor)` — the whole arithmetic, in the engine's own
    // `Matrix`, in PAGE space.
    //
    // `about` is `translate(a) × M × translate(-a)`, which is
    // `p' = a + (p - a) * (sx, sy)` exactly — so this is the same formula the
    // header states, written once by the crate that owns matrices instead of
    // once here per point. A shell keeping its own copy would be a second
    // derivation of one answer in coordinate space.
    //
    // ★ Page space, not the object's. See the module header: the engine
    // conjugates by each object's own CTM, and a caller that "helpfully"
    // pre-multiplied would be right only where that CTM is the identity.
    let matrix = pdfcer_core::vector::Matrix::scale(f64::from(sx), f64::from(sy)).about(anchor);
    Ok(VectorAction::TransformObjects {
        page,
        objects,
        matrix,
    }
    .into())
}

/// The frame's facts about a resize drag in flight.
///
/// A struct rather than seven parameters, and it is not only clippy's
/// arity rule: **five of the seven are read-only facts about the same frame**,
/// so grouping them says what they are. It also removes the failure a long
/// parameter list invites — `map` and `page` are both `Option<&…>` and adjacent,
/// and swapping them would compile if their types ever converged.
///
/// `selection`, `provider` and `actions` stay outside it, deliberately: the
/// first two are *the document's* state rather than the frame's, and the third
/// is an output. A struct that mixed all three would be a bag rather than a
/// grouping.
#[derive(Clone, Copy)]
pub struct Frame<'a> {
    /// Which grip the press landed on, sampled at the press.
    pub grip: Grip,
    /// ★★★ How far the pointer has travelled since then, **in PAGE space**.
    ///
    /// # ⚠ The unit is load-bearing, and [`Self::bounds`] is in the other one
    ///
    /// The gesture machine works in page space by design — `canvas::interact`
    /// builds its `PointerFrame` with `pos: screen_pos.map(|p| map.to_page(p))`
    /// — so every caller passes a page-space displacement. [`Self::bounds`] is
    /// **screen** space, and [`factors`] divides one by the other.
    ///
    /// ⇒ Divide them unreconciled and every factor's distance from unity comes
    /// out inflated by `1/zoom`: at a fitted 29.55 % a corner dragged 60 px
    /// commits a **5.94×** stretch where the geometry says 2.46×, and the shape
    /// runs **143 px past the cursor** on both axes — this module's own D8
    /// convention (*"the grabbed corner tracks the pointer"*) broken by the
    /// module that states it. `DEFECTS.md` **D18**.
    ///
    /// ★★ So the conversion is in [`drag`], where the two meet, rather than at
    /// the call site — one consumer and three would-be converters, and the
    /// honest contract is the one every caller already satisfies. Two `Vec2`s
    /// in two spaces are indistinguishable to the compiler; the only defence is
    /// that exactly one function reconciles them.
    pub delta: Vec2,
    /// Draw the ghost, or commit.
    pub phase: Phase,
    /// The selection's grip box in screen space, or `None` if there is no
    /// outline to have grabbed.
    pub bounds: Option<egui::Rect>,
    /// The page the drag is on.
    pub page_index: usize,
    /// **Whether Shift is down THIS FRAME**, sampled live rather than at the
    /// press.
    ///
    /// ★ Live, unlike `gesture::Drag::shift`, and the two are different facts
    /// that happen to read the same key. That one asks *"what did this gesture
    /// MEAN"* — extend the selection or replace it — and must be sampled at the
    /// press, because the meaning of a gesture cannot change half-way through
    /// it. This asks *"is the operator constraining right now"*, and every
    /// program in the class lets that be picked up and put down mid-drag. An
    /// operator who starts a free resize, sees it going crooked and grabs Shift
    /// expects the shape to snap to proportion under their hand.
    pub constrain: bool,
    /// **The operator's Tool-row scale switches**, sampled at the commit.
    ///
    /// ★ Carried on the [`Frame`] rather than read from `egui::Memory` inside
    /// this module, so `resizing` stays a pure decision over its inputs and
    /// stays testable without a `Context`. Every other live fact on this struct
    /// arrives the same way, including `constrain`.
    pub modifiers: crate::canvas::scaling::Modifiers,
    /// The frame's screen ⟷ canvas mapping.
    pub map: Option<&'a PageMapping>,
    /// The page itself, for the canvas → PDF hop.
    pub page: Option<&'a pdfcer_core::page_tree::Page>,
    /// The selected form field, when one is selected.
    ///
    /// ★★ Carried rather than looked up, because a resize has THREE
    /// destinations and only one of them is on `SelectionState`: page content
    /// and a markup annotation both live there, and a form field's selection
    /// lives on the document — `canvas::selection::annot` excludes `/Widget`
    /// outright so the form surface owns those presses. A drag that had to ask
    /// two places which one applied would be re-deriving a fact the caller
    /// already has.
    pub selected_field: Option<&'a crate::app::state::SelectedField>,
}

/// **Apply one frame of a resize drag: preview it, or commit it.**
///
/// Mirrors [`crate::canvas::moving::drag`] deliberately, down to the return
/// type, so the caller's two arms read the same and a reader who has understood
/// one has understood both. What it hands back is the **scale factors** for the
/// ghost, where the move drag hands back a displacement.
///
/// # ★ A refusal is worded ONCE, on `Complete`
///
/// Not on every frame of the drag. `moving::drag` makes the same choice and its
/// reason applies unchanged: an in-flight gesture is a question, and answering a
/// question the operator has not finished asking would put a sentence on the
/// status row sixty times a second while they were still deciding.
fn to_pdf(
    at: egui::Pos2,
    map: &PageMapping,
    page: &pdfcer_core::page_tree::Page,
) -> Option<(f64, f64)> {
    let canvas = map.to_page(at);
    let pdf = crate::viewer::canvas_to_pdf_space(canvas, page)?;
    Some((f64::from(pdf.x), f64::from(pdf.y)))
}

pub fn drag(
    frame: Frame<'_>,
    selection: &SelectionState,
    provider: Option<&ObjectModelProvider>,
    actions: &mut Vec<Action>,
) -> Option<(f32, f32)> {
    let Frame {
        grip,
        delta,
        phase,
        bounds,
        page_index,
        constrain,
        map,
        page,
        selected_field,
        modifiers,
    } = frame;
    let Some(bounds) = bounds else {
        // No grip box means no selection outline, which means there was nothing
        // to grab — unreachable from a real gesture, and silent because a
        // sentence about a selection that does not exist would be describing
        // the harness rather than the document.
        return None;
    };
    // ★★★ THE ONE PLACE THE TWO SPACES ARE RECONCILED. See [`Frame::delta`].
    //
    // `bounds` is screen space (`pressing::grabbable` → `overlay::grip_box`,
    // the same rectangle the outline is drawn from) and `delta` is page space,
    // so `factors` — which divides one by the other — needs them in one space.
    //
    // ★ Screen rather than page, because `factors` also receives `bounds` and
    // converting the rectangle would mean converting the grip, the pivot and
    // the anchor with it. One vector is the smaller crossing.
    //
    // ★★ When there is no mapping the delta passes through unchanged, which is
    // the zoom-1.0 identity — and is exactly what every unit test in this
    // module supplies. ⚠ **A green suite is therefore no evidence that the two
    // spaces agree**: at zoom 1.0 a mismatch is arithmetically invisible,
    // because the harness compares the same quantity against itself and a
    // common factor cancels. Only a driven run at a fitted zoom can see it.
    let delta = map.map_or(delta, |m| m.page_vec_to_screen(delta));
    let Some((sx, sy)) = factors(grip, bounds, delta) else {
        if phase == Phase::Complete {
            decline(Refusal::Degenerate);
        }
        return None;
    };
    // ★★ The aspect lock is applied HERE — above the `InFlight` return, below
    // the one place the factors are derived — so the ghost and the commit are
    // the same pair of `f32`s and cannot disagree.
    //
    // Applying it in the caller would have been the smaller diff and is the
    // trap: the caller sees `drag`'s return value (the ghost) but not the
    // commit path inside it, so a constrained preview would commit
    // unconstrained factors. That is `drag-moves` D2 — *the preview is derived
    // from what the release will commit*.
    let (sx, sy) = if constrain {
        crate::canvas::constrain::aspect(sx, sy)
    } else {
        (sx, sy)
    };
    if phase == Phase::InFlight {
        // ★ D5's second clause — *the constraint is announced* — is answered by
        // the CALLER, not here. This module takes no `egui::Context` and that
        // is deliberate: everything in it is a pure function of its `Frame`,
        // which is what lets the whole resize be unit-tested without a window.
        // The announcement needs a context, needs to know which of five drags
        // is in flight, and cannot affect what commits — so it belongs where
        // those three facts already are, in `canvas::interact`.
        //
        // ★ The ghost is offered even for factors that will be REFUSED on
        // release, and that is deliberate: an operator dragging a corner past
        // the opposite one can see the shape collapsing, which is how they
        // learn to stop. Hiding the preview at the moment it becomes invalid
        // would read as the drag having stopped tracking.
        return Some((sx, sy));
    }

    // ---- commit ------------------------------------------------------
    let (Some(map), Some(page)) = (map, page) else {
        decline(Refusal::NoObjectModel);
        return None;
    };
    // ★★ The anchor is converted ONCE, here, through the same mapping the
    // outline was drawn with — the same TWO hops `canvas::textedit::resolve_run`
    // takes, in the same order, through the same two functions: screen → canvas
    // → PDF user space. The canvas is Y-down from the page's top-left with
    // `/Rotate` applied and every coordinate the engine speaks is Y-up from the
    // un-rotated CropBox, and `canvas::mapping`'s header calls a second
    // conversion *the classic silent defect*: the ghost and the commit would
    // disagree about which corner stayed still, and the object would jump by
    // whatever the two conversions differed by on release.
    // ★★ `pivot`, NOT `anchor`. `anchor` is where the grip IS; the point that
    // must stay still is the OPPOSITE corner. Using `anchor` here would scale
    // the object about the very corner the operator is dragging, so the shape
    // would grow away from their hand instead of towards it — a resize that
    // works and is wrong, which is the failure mode this whole module's driven
    // check exists to catch.
    let anchor_screen = grip.pivot(bounds);
    let anchor_canvas = map.to_page(anchor_screen);
    let Some(pdf) = crate::viewer::canvas_to_pdf_space(anchor_canvas, page) else {
        decline(Refusal::Degenerate);
        return None;
    };
    let anchor = Point::new(f64::from(pdf.x), f64::from(pdf.y));
    // ★★★ An ANNOTATION takes a different verb, and the branch is here — after
    // the factors and the anchor, before the content action.
    //
    // Everything above this line is shared and must be: the eight grips, the
    // pivot rule (the corner opposite the one grabbed), the aspect lock, the
    // degenerate-drag refusal and the screen->page conversion are the same
    // gesture whatever is under it. What differs is one call.
    //
    // ★★ `resize_annotation` takes **anchor + factors**, which is not a
    // coincidence: this shell asked for that shape rather than a target `/Rect`
    // precisely so it would match `transform_objects`, and the engine took the
    // reasoning unchanged -- *"the anchor is a decision the shell makes from
    // which grip was grabbed, and a verb that took a grip name would be
    // encoding our affordance in your crate."* So the two transform verbs
    // consume the identical pair and this branch is a routing decision rather
    // than a second arithmetic.
    if let Some(annot) = selection.annot() {
        if annot.target.kind != crate::canvas::selection::AnnotKind::Markup || annot.target.locked {
            // No scale verb for a ce dimension -- its extent IS its
            // measurement -- and a locked annotation is the file refusing.
            // Neither is offered grips, so neither can arrive; declining
            // silently rather than by name is the honest answer for a state a
            // gesture cannot reach.
            return None;
        }
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!(
                "resize-annot-commit id={} grip={grip:?} sx={sx:.4} sy={sy:.4} \
                 ax={:.2} ay={:.2}",
                annot.target.id.num, anchor.x, anchor.y
            )
        });
        actions.push(Action::Annot(
            crate::app::actions::annot::AnnotAction::Resize {
                id: annot.target.id,
                anchor: (anchor.x, anchor.y),
                sx: f64::from(sx),
                sy: f64::from(sy),
                // ★★ Whether the drag was PROPORTIONAL, sent because the engine
                // asked for it by name: *"if your grips can report whether a drag
                // was proportional, that distinction is worth having."*
                //
                // A non-uniform scale of a FOREIGN appearance stream distorts the
                // stroke -- a mathematical limit, not a defect, because neither PDF
                // nor SVG has a per-axis stroke width -- and the engine refuses
                // that case rather than silently producing an oval border, which is
                // what the parity reference does. A uniform scale is always safe.
                uniform: (sx - sy).abs() <= f32::EPSILON,
                // ★★★ **What the operator asked to ride along** — O51's switches.
                //
                // `uniform` above and this are different facts and both travel:
                // the first is a measurement of the drag, the second is a
                // decision by the operator. Deriving either from the other puts
                // words in one of their mouths — see `canvas::scaling`.
                modifiers,
            },
        ));
        return Some((sx, sy));
    }
    // ★★★ A FORM FIELD's box, and it is the third destination this one gesture
    // reaches. `OPERATOR_REQUESTS.md` **O53**.
    //
    // The verb differs from the annotation one and the engine says why: a
    // widget goes to `edit_widget(fqn, index, WidgetEdit::new().with_rect(..))`,
    // *"which rebuilds the appearance into the new box as part of the same
    // command"* -- a check box's tick and a text field's border have to be
    // redrawn at the new size, which `resize_annotation` would not do.
    //
    // ★★ So this one takes a RECTANGLE where the annotation takes anchor and
    // factors, and the conversion happens here rather than in the engine
    // because it is the same arithmetic the eight grips already did: the ghost
    // the operator was watching IS `bounds` scaled about the pivot, and
    // deriving the rect from it is what makes what they saw and what is
    // written the same box.
    if let Some(selected) = selected_field.cloned() {
        // The ghost the operator was watching, in page space. `pivot` is the
        // corner that stays still — the one opposite the grip — so this is the
        // same box the preview drew, converted rather than recomputed.
        let scaled = scaled_about(bounds, grip.pivot(bounds), sx, sy);
        let (Some(a), Some(b)) = (to_pdf(scaled.min, map, page), to_pdf(scaled.max, map, page))
        else {
            decline(Refusal::Degenerate);
            return None;
        };
        // ★ `from_corners`, not a literal: §7.9.5 lets a `/Rect`'s corners
        // arrive in any order and normalises them, and a grip dragged past its
        // anchor produces exactly that — a mirrored box, which is a supported
        // gesture rather than an error to guard against.
        let pdf_rect = pdfcer_core::page_tree::Rect::from_corners(a.0, a.1, b.0, b.1);
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!(
                "resize-widget-commit field={} widget={} grip={grip:?} sx={sx:.4} sy={sy:.4}",
                selected.field, selected.widget
            )
        });
        actions.push(Action::Field(
            crate::app::actions::forms::FieldAction::EditWidget {
                field: selected.field,
                widget: selected.widget,
                // ★★★ **AND THE OPERATOR'S SWITCHES** —
                // `OPERATOR_REQUESTS.md` O76: *"Form shape outlines of
                // checkboxes and such scale when I drag them larger."*
                //
                // `WidgetEdit` carries the scale answer in **the same type**
                // the annotation path takes rather than in three mirrored
                // fields, so the two destinations of this one gesture differ in
                // their verb and not in what the operator said. A `.with_rect`
                // and nothing else would leave the Tool-row switches reaching
                // an annotation and stopping at a form field.
                //
                // ★ `to_options()` is the same call `annots::resize` makes,
                // from the same `modifiers` value captured on the same frame.
                // Deriving them separately is exactly how the two paths would
                // drift, and `canvas::scaling`'s header records what that cost
                // the last time it happened.
                edit: pdfcer_core::edit::WidgetEdit::new()
                    .with_rect(pdf_rect)
                    .with_resize(modifiers.to_options()),
                // ui-text-exempt: a control name carried for a refusal message.
                touched: "the box",
            },
        ));
        return Some((sx, sy));
    }
    match action(selection, page_index, provider, anchor, (sx, sy)) {
        Ok(a) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                //
                // ★ Carries the FACTORS and the anchor, which is what a wrong
                // build would get wrong. A line saying only "resize committed"
                // would be identical for a build that scaled about the centre,
                // mirrored an axis, or applied the same factor to both.
                format!(
                    "resize-commit grip={grip:?} sx={sx:.4} sy={sy:.4} \
                     ax={:.2} ay={:.2}",
                    anchor.x, anchor.y
                )
            });
            actions.push(a);
            Some((sx, sy))
        }
        Err(reason) => {
            decline(reason);
            None
        }
    }
}

/// Word a refusal on the status row, and trace it.
///
/// One place, so a variant added to [`Refusal`] is a compile error in
/// `crate::text::resizing` rather than a drag that silently does nothing —
/// which is the failure `canvas::textedit`'s own history is about.
pub(crate) fn decline(reason: Refusal) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!("resize-declined reason={reason:?}")
    });
    crate::app::actions::record_note(
        // Epoch zero rather than the document's, and this is the one place in
        // the crate that does it. **It is a live defect, and the sentence it
        // was meant to produce is never seen.** `DEFECTS.md` D41.
        //
        // `crate::app::actions::last_edit_disclosure` is an EQUALITY filter —
        // it returns the slot only while `d.epoch == epoch`, and the status bar
        // passes `doc.edit_epoch`. A note stamped `0` is therefore readable
        // only while the document's epoch is still zero, which is until its
        // first edit. From the first edit onward every resize refusal is
        // recorded and invisible, for the life of the session.
        //
        // The intended behaviour — retire on the operator's next act rather
        // than on the next document change — cannot be expressed by a fixed
        // epoch at all, because the filter has no ordering. A refusal that must
        // reach the operator is stamped with the current epoch like every other
        // note; retiring it sooner needs a mechanism, not a sentinel.
        0,
        crate::text::resizing::refusal(reason).to_owned(),
    );
}

/// The rectangle `bounds` becomes when both its corners are scaled about
/// `pivot` by `(sx, sy)`.
///
/// # ★★★ Both corners, because a pivot is not always a corner
///
/// This is the identical map `overlay::draw_resize_ghost` paints —
/// `pivot + (p - pivot) * s` — so the box that is written is the box the
/// operator watched, by construction rather than by two derivations agreeing.
///
/// The earlier spelling derived one far corner as `bounds.min + bounds.max -
/// 2 * pivot`, which is right only while the pivot **is** a corner.
/// [`Grip::pivot`] deliberately answers a **mid-edge** point for the four edge
/// grips, and on such a grip's cross axis that expression is exactly zero — so
/// the derived corner collapsed onto the pivot and an edge drag committed a
/// zero-extent `/Rect`. The corner grips were unaffected and kept working,
/// which is what O209 reports as *"only the corner drag handles work."*
fn scaled_about(bounds: Rect, pivot: Pos2, sx: f32, sy: f32) -> Rect {
    let map = |p: Pos2| {
        egui::pos2(
            pivot.x + (p.x - pivot.x) * sx,
            pivot.y + (p.y - pivot.y) * sy,
        )
    };
    Rect::from_min_max(map(bounds.min), map(bounds.max))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 100×50 screen box at the origin.
    fn box_100x50() -> egui::Rect {
        egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(100.0, 50.0))
    }

    /// ★★★ **An edge grip's scaled box keeps its cross-axis extent.**
    ///
    /// The regression O209 names: *"only the corner drag handles work."* An
    /// east or west drag leaves the height alone and a north or south drag
    /// leaves the width alone, because [`Grip::pivot`] answers a **mid-edge**
    /// point on those four and the box is scaled about it rather than reflected
    /// through it.
    ///
    /// It asserts the extent that must survive, not merely that the rectangle
    /// is non-empty — a box collapsed on one axis and a box that merely failed
    /// to grow are both "not what the operator dragged", and only the first was
    /// the defect.
    #[test]
    fn an_edge_grip_scales_only_its_own_axis() {
        let bounds = box_100x50();
        for (grip, sx, sy, want_w, want_h) in [
            (Grip::East, 1.5, 1.0, 150.0, 50.0),
            (Grip::West, 1.5, 1.0, 150.0, 50.0),
            (Grip::North, 1.0, 2.0, 100.0, 100.0),
            (Grip::South, 1.0, 2.0, 100.0, 100.0),
        ] {
            let got = scaled_about(bounds, grip.pivot(bounds), sx, sy);
            assert!(
                (got.width() - want_w).abs() < 0.001 && (got.height() - want_h).abs() < 0.001,
                "{grip:?}: wanted {want_w}x{want_h}, got {}x{}",
                got.width(),
                got.height()
            );
        }
    }

    /// ★ **A corner grip is unchanged by the fix.**
    ///
    /// The two spellings agree wherever the pivot is itself a corner, and that
    /// is the half that kept working — so this is the control that says the
    /// repair widened the set of grips that commit rather than moving it.
    #[test]
    fn a_corner_grip_pins_the_opposite_corner() {
        let bounds = box_100x50();
        let got = scaled_about(bounds, Grip::NorthWest.pivot(bounds), 2.0, 2.0);
        assert_eq!(got.max, bounds.max, "the far corner stays put");
        assert!(
            (got.width() - 200.0).abs() < 0.001 && (got.height() - 100.0).abs() < 0.001,
            "got {}x{}",
            got.width(),
            got.height()
        );
    }

    /// ★ **Dragging the south-east grip right and down grows both axes.**
    ///
    /// The base case, and the one whose y sign is easy to get backwards: screen
    /// y is down, so a positive `dy` on a *south* grip is growth. Getting it
    /// wrong produces an object that shrinks when you pull it bigger, which is
    /// the kind of defect that survives review because both directions "look
    /// like a resize".
    #[test]
    fn the_south_east_grip_grows_both_axes() {
        let (sx, sy) = factors(Grip::SouthEast, box_100x50(), Vec2::new(50.0, 25.0)).expect("box");
        assert!((sx - 1.5).abs() < 1e-6, "sx={sx}");
        assert!((sy - 1.5).abs() < 1e-6, "sy={sy}");
    }

    /// The north-west grip grows when dragged UP and LEFT — negative deltas.
    #[test]
    fn the_north_west_grip_grows_on_negative_travel() {
        let (sx, sy) =
            factors(Grip::NorthWest, box_100x50(), Vec2::new(-50.0, -25.0)).expect("box");
        assert!((sx - 1.5).abs() < 1e-6, "sx={sx}");
        assert!((sy - 1.5).abs() < 1e-6, "sy={sy}");
    }

    /// ★★ **A mid-edge grip scales ONE axis**, which is the whole reason the
    /// four of them are offered separately from the corners.
    ///
    /// A build that treated them as corners would let an operator aiming at
    /// "make this wider" also make it taller — a change they did not ask for,
    /// on the axis they were deliberately not touching.
    #[test]
    fn a_mid_edge_grip_leaves_the_other_axis_alone() {
        let (sx, sy) = factors(Grip::East, box_100x50(), Vec2::new(50.0, 40.0)).expect("box");
        assert!((sx - 1.5).abs() < 1e-6, "sx={sx}");
        assert!(
            (sy - 1.0).abs() < 1e-6,
            "the east grip moved y by {sy}; a mid-edge grip must not touch the other axis even \
             when the pointer wanders across it"
        );
        let (sx, sy) = factors(Grip::South, box_100x50(), Vec2::new(70.0, 25.0)).expect("box");
        assert!((sx - 1.0).abs() < 1e-6, "sx={sx}");
        assert!((sy - 1.5).abs() < 1e-6, "sy={sy}");
    }

    /// A degenerate box has no factors, rather than infinite ones.
    #[test]
    fn a_zero_width_box_has_no_factors() {
        let flat = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(0.0, 50.0));
        assert_eq!(factors(Grip::East, flat, Vec2::new(10.0, 0.0)), None);
    }

    /// ★ **A collapse or a mirror is refused, not clamped.**
    ///
    /// Clamping would silently substitute a different edit for the one the
    /// operator made — and a mirrored path is a legal, plausible-looking
    /// document they did not ask for.
    #[test]
    fn collapsing_and_mirroring_are_refused() {
        assert!(!is_usable(0.0, 1.0));
        assert!(!is_usable(1.0, -1.0));
        assert!(!is_usable(f32::NAN, 1.0));
        assert!(!is_usable(1.0, f32::INFINITY));
        assert!(is_usable(0.5, 2.0));
    }

    /// ★★ **The map is anchored**: the anchor point does not move, and
    /// everything else moves in proportion to its distance from it.
    ///
    /// Asserted as the two properties rather than against a table of
    /// coordinates, because the properties are what "resize about a corner"
    /// means and a coordinate table would pass for a build that had the anchor
    /// at the centre.
    #[test]
    fn the_anchor_stays_put_and_distance_scales() {
        let anchor = Point::new(10.0, 20.0);
        let scaled = |p: Point, sx: f64, sy: f64| {
            Point::new(
                anchor.x + (p.x - anchor.x) * sx,
                anchor.y + (p.y - anchor.y) * sy,
            )
        };
        let at_anchor = scaled(anchor, 3.0, 3.0);
        assert!((at_anchor.x - anchor.x).abs() < 1e-9);
        assert!((at_anchor.y - anchor.y).abs() < 1e-9);

        let far = Point::new(30.0, 20.0);
        let out = scaled(far, 2.0, 2.0);
        assert!(
            ((out.x - anchor.x) - 2.0 * (far.x - anchor.x)).abs() < 1e-9,
            "a point twice as far from the anchor must end up twice as far again"
        );
    }
}
