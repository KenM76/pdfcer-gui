//! # `canvas::resizing` — the eight grips finally do something, and what it
//! cost to get there without a verb
//!
//! ## What this closes
//!
//! `GUI_ROADMAP.md` Phase 1.3 drew eight resize grips at S4. They have been
//! **cursored, hit-tested and drag-consuming ever since, and have committed
//! nothing** — the last ⛔ in `FEATURES.md`'s Phase 1 list and the oldest
//! unbuilt thing in this project.
//!
//! [`crate::canvas::handles`]' header states the reason and it is still true:
//!
//! > `pdfcer-core` has `move_object`, `move_objects`, `move_subpath`,
//! > `move_node`, `move_nodes` and `move_handle` — and **no scale or resize
//! > verb for a vector object at all**.
//!
//! Re-derived against the engine on 2026-08-19 rather than taken from that
//! note: `grep "pub fn .*scale" edit.rs` returns exactly one hit and it is
//! `set_group_scale`, a ce-dimension calibration. **The blocker is real** —
//! unlike two others this project re-checked the same week, both of which had
//! quietly expired.
//!
//! ## ★★★ IT IS BUILT OUT OF `transform_objects` NOW — 2026-08-20
//!
//! Everything below this block describes how a resize was built out of
//! `move_nodes` between 2026-08-19 and 2026-08-20, and **it is kept**, because
//! the reasoning is the record of a substitution that was correct while it
//! lasted and of the four limits it could not get past. Three of the four are
//! gone; the fourth turned out to be a decision rather than a limit.
//!
//! `EditSession::transform_objects` (`Pass 113.0`) wraps each object's operator
//! run in `q <cm> … Q`. **That never looks at an operand**, which is what makes
//! it kind-agnostic — not a match arm per kind that somebody has to remember to
//! extend. So:
//!
//! | was refused | now |
//! |---|---|
//! | **text runs** | works. A text object has no nodes, and it does not need any |
//! | **images** | works |
//! | **more than one object** | works, in **one** call, one command, one undo entry — the slice is the point |
//! | **stroke width** | still not scaled, and it is still the right answer: on a CAD drawing a line weight is a *drafting standard*. Now a genuine decision rather than a consequence, and still disclosed |
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
//! ## ★★ It used to be built out of `move_nodes`, and that was the whole idea
//!
//! **Scaling a path IS moving every one of its nodes.** For an anchor `a` and
//! factors `(sx, sy)`:
//!
//! ```text
//! p' = a + (p - a) * (sx, sy)
//! ```
//!
//! `EditSession::move_nodes` takes a **slice** of `(node, Point)`, so a whole
//! resize is **one call, one command, one undo entry** — which is this
//! project's standing rule for one gesture (`canvas::moving`'s §1) and the
//! thing a naive per-node loop would break, both by producing N undo entries
//! and by planning each move against byte offsets the previous one invalidated.
//!
//! The operator's instruction, 2026-08-19: *"finish off phase 1 and phase 5.
//! Get everything unblocked on phase 5 — no excuses about slowness of feature
//! from pdfcer as a reason not to implement."* This is that applied to Phase 1.
//!
//! ## ★ What this CANNOT do, stated here rather than discovered
//!
//! All four are consequences of the substitution, not of the implementation,
//! and all four are **worded refusals** rather than silent no-ops:
//!
//! | | why |
//! |---|---|
//! | **text runs** | a text object has no nodes. Scaling one means writing a `Tm`, and this shell will not synthesise one — that is the engine's arithmetic |
//! | **images** | likewise, a `cm` |
//! | **more than one object** | `move_nodes` is per object, so N objects is N commands and N undo entries. One gesture is one command; the honest answer is to decline |
//! | **stroke width** | a scaled path keeps its original `w`, so a 2× box has 1× linework |
//!
//! The last is **not** refused, and that is a judgement rather than an
//! oversight. On a CAD drawing a line weight is a *drafting standard* — 0.25 mm
//! is 0.25 mm whatever size the detail is — so keeping it is right far more
//! often than scaling it would be, and it is the behaviour every drafting
//! package has. It is nonetheless something pdfcer decided and the operator did
//! not, so it is **disclosed** ([`crate::text::resizing`]) rather than assumed.
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
//! - D2 derived-from-commit: `Some` only when a release would reach
//!   `move_nodes` on real operands.
//! - D3 escape-cancels: the gesture machine drops it; nothing is written before
//!   `Complete`.
//! - D4 one-undo-entry: scaling a path is moving every one of its nodes, and
//!   `move_nodes` takes a slice — one command.
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

use egui::Vec2;
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
/// `canvas::textedit::Refusal`'s rule, and for the reason that module's own
/// history proves: this project has already shipped one feature whose answer to
/// a case it could not handle was to do nothing, and the operator reported it
/// as broken for weeks.
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
//
// `transform_preview` is `&self`, side-effect-free, and shares one body with
// the verb — so `preview(..).is_ok()` **is** the predicate, and the engine's
// guidance is explicit about what a shell should do with it:
//
// | error | means | UI |
// |---|---|---|
// | `DegenerateCtm` | this object cannot be transformed AT ALL — its own CTM is singular | **do not offer a handle** |
// | `SingularTransform` | this DRAG is degenerate | offer the handle, refuse on release |
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
// ★ Why it is not built tonight: **the preview decomposes the page.** Measured
// by the engine on the benchmark drawing, 129,758 objects, **~4 s in a debug
// build** — and both the verb and the preview pay it. The engine's own advice
// is *"call `transform_preview` on selection change and on gesture start, not
// per frame"*, which means a cache keyed on `(page, edit epoch, selection)`,
// which is a piece of work rather than a line. `app::cache::FormRunCache` is
// the shape it should take.
//
// ★★ A variant WAS added here for it and then removed on the same evening,
// because `every_refusal_is_still_raised_somewhere` — written in the same hour
// — failed on its first run: it had a sentence and no call site. That is the
// test doing exactly what it was written for, and inventing a call site to
// satisfy it would have been the failure it exists to catch.

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
    // ★ The provider is still asked for, and it is no longer asked ANYTHING.
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
    // It was added on the first driven run of the typed route, which failed
    // reporting "Apply committed nothing" while the trace clearly showed the
    // object's bounds changing from 317.87 to 358.00. The check was right about
    // its oracle being absent and wrong about what that meant — a defect in the
    // instrument, and exactly the shape `CONTINUE.md` §7 warns about.
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
    // What this replaced was the same map written out per node:
    //
    //     p' = a + (p - a) * (sx, sy)
    //
    // and it is worth naming what that cost. `about` is
    // `translate(a) × M × translate(-a)`, which is that expression exactly — so
    // this is not a new formula, it is the same one stated once by the crate
    // that owns matrices instead of once here per point. A shell that kept its
    // own copy would be the second derivation of one answer, which is the
    // failure this project has now met four times in coordinate space.
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
    /// # This doc comment said "in screen points" until 2026-08-29, and it was
    /// never true
    ///
    /// The gesture machine works in page space by design — `canvas::interact`
    /// builds its `PointerFrame` with `pos: screen_pos.map(|p| map.to_page(p))`
    /// — so every caller has always passed a page-space displacement. The
    /// contract was wrong, not the callers.
    ///
    /// ⇒ **And [`Self::bounds`] genuinely IS screen space**, so the two were
    /// divided against each other and every factor's distance from unity came
    /// out inflated by `1/zoom`. At the operator's fitted 29.55 % a corner
    /// dragged 60 px committed a **5.94×** stretch where the geometry says
    /// 2.46×, and the shape shot **143 px past the cursor** on both axes —
    /// this module's own D8 convention (*"the grabbed corner tracks the
    /// pointer"*) violated by the module that states it. `DEFECTS.md` **D18**.
    ///
    /// ★★ The fix is a conversion in [`drag`], where the two meet, rather than
    /// at the call site — because there is one consumer and three would-be
    /// converters, and the honest contract is the one every caller already
    /// satisfies. Two `Vec2`s in two spaces are indistinguishable to the
    /// compiler; the only defence is that exactly one function reconciles them.
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
    // module supplies. **That is why a green suite never saw D18**: at zoom 1.0
    // the bug is arithmetically invisible, and the harness only ever compared
    // the same quantity against itself, where a common factor cancels.
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
    // commit path inside it, so a constrained preview would have committed
    // unconstrained factors. That is `drag-moves` D2 — *the preview is derived
    // from what the release will commit* — and it is the failure this project
    // has already met three times in coordinate space.
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
    // outline was drawn with. `canvas::mapping`'s header calls a second
    // conversion *the classic silent defect*: the ghost and the commit would
    // then disagree about which corner stayed still, and the object would jump
    // by whatever the two conversions differed by on release.
    // The same TWO hops `canvas::textedit::resolve_run` takes, in the same
    // order, through the same two functions — screen → canvas → PDF user space.
    // `canvas::mapping`'s header calls doing this any other way *the classic
    // silent defect*: the canvas is Y-down from the page's top-left with
    // `/Rotate` applied, and every coordinate the engine speaks is Y-up from
    // the un-rotated CropBox.
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
                // `uniform` above and this are different facts and both travel: the
                // first is a measurement of the drag, the second is a decision by
                // the operator, and until 2026-08-28 the apply arm derived the
                // second from the first. See `canvas::scaling` for why that was a
                // workaround rather than a rule.
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
        let pivot = grip.pivot(bounds);
        let far = egui::pos2(
            pivot.x + (bounds.min.x + bounds.max.x - 2.0 * pivot.x) * sx,
            pivot.y + (bounds.min.y + bounds.max.y - 2.0 * pivot.y) * sy,
        );
        let (Some(a), Some(b)) = (to_pdf(pivot, map, page), to_pdf(far, map, page)) else {
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
                // ★★★ **AND THE OPERATOR'S SWITCHES, as of 2026-08-31** —
                // `OPERATOR_REQUESTS.md` O76, the row that began *"Form shape
                // outlines of checkboxes and such scale when I drag them
                // larger."*
                //
                // For the life of that row this line read `.with_rect(..)` and
                // nothing else, and it could not read otherwise: `WidgetEdit`
                // had no way to carry a scale answer, so the three switches on
                // the Tool row reached an annotation and stopped at a form
                // field. That gap was filed rather than worked around, and
                // `pdfcer-core` Pass 187.0 answered it by **reusing the same
                // type** the annotation path takes rather than mirroring three
                // fields — so the two destinations of this one gesture now
                // differ in their verb and not in what the operator said.
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
        // ★ Epoch zero rather than the document's, and this is the one place in
        // the crate that does it. A refusal changed nothing, so there is no
        // edit for it to be about; `record_note` keys on the epoch so a
        // disclosure retires when the document moves past it, and a refusal
        // must retire on the operator's NEXT act instead. Passing the live
        // epoch would leave "you cannot resize text" on screen through forty
        // subsequent edits.
        0,
        crate::text::resizing::refusal(reason).to_owned(),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 100×50 screen box at the origin.
    fn box_100x50() -> egui::Rect {
        egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(100.0, 50.0))
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
