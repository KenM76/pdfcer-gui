//! # `panels::properties::geometry` — **X, Y, W and H, typed rather than
//! dragged**
//!
//! ## What this closes
//!
//! `FEATURES.md`'s Phase 1 remainder, verbatim:
//!
//! > **Editable geometry** — X/Y/W/H in the Properties panel, typed rather
//! > than dragged.
//!
//! And it closes it *because the resize gesture landed first*. Every number in
//! this section is a call into machinery that already exists and is already
//! tested: a position change is [`VectorAction::MoveSelection.into()`], the same variant a
//! move drag raises, and a size change is [`crate::canvas::resizing::action`],
//! the same function the eight grips raise. **This module computes two scale
//! factors and a delta and contributes no geometry of its own.**
//!
//! That is the whole design and it is deliberate. A properties panel that
//! reimplemented "make it 40 points wide" would be a second scale
//! implementation with a second set of rounding, a second pivot convention and
//! a second answer to *what happens to line weights* — and the two would drift,
//! silently, because nothing compares them.
//!
//! ## ★ Why an operator needs this even though the grips work
//!
//! Because a grip cannot express *exactly 40.0 points*. The resize gesture is
//! excellent for "about this big" and incapable of "the same as the one above
//! it", and a drawing is full of the second kind. It is also the only route
//! for a **small** object: at fit-page zoom on an ISO A1 sheet a 6 mm symbol is
//! about four pixels across, so its eight grips overlap each other and the
//! gesture is unusable at the zoom where the operator can see the sheet.
//!
//! It is additionally the only **accessible** route to a resize. A gesture that
//! requires a sub-pixel drag is a gesture some operators cannot perform, and
//! `MODES_AND_PANELS.md` §7's accessibility line asks for a typed equivalent to
//! every direct-manipulation edit for exactly that reason.
//!
//! ## ★★ Why there is an Apply button and the fields do not commit as you type
//!
//! Because **every commit is an undo entry**, and a `DragValue` the operator
//! scrubs from 40 to 120 would raise eighty of them. That is not a theoretical
//! objection: `app::actions`' `MoveNodes` doc comment makes the identical
//! argument about looping the singular verbs — *"N undo entries for one drag,
//! and each planned against byte offsets the previous one invalidated"* — and a
//! live-committing spinner is that loop with a nicer face on it.
//!
//! So the four fields edit a **draft**, and one press turns the draft into at
//! most two commands. Two rather than one because a move and a scale are
//! different verbs in `EditSession` and this shell does not have a combined
//! one; the operator who changes only X gets exactly one entry, and the
//! operator who changes X and W gets two, in that order, which is the order
//! that makes the second one's pivot mean what the preview said.
//!
//! ## ★ Why the draft is discarded when the object changes underneath it
//!
//! The draft is stamped with `(page, object, edit epoch)`. If any of the three
//! moves — the operator selects something else, or *anything at all* edits the
//! document — the draft is dropped and re-seeded from the object's current
//! bounds.
//!
//! The epoch is the one that matters and it is the one that is easy to leave
//! out. Without it, this sequence silently destroys work: type `W = 40`, do not
//! press Apply, press `Ctrl+Z` to undo something unrelated, press Apply. The
//! draft would still hold the numbers computed against the *pre-undo* bounds,
//! and the scale factor would be `40 / (a width that no longer exists)`. The
//! object would end up some third size that the operator never typed and cannot
//! predict. Re-seeding on the epoch makes that unrepresentable rather than
//! merely unlikely.
//!
//! ## ★★★ 2026-09-06 — THE SECOND SUBJECT: A SELECTED MARKUP
//!
//! This section drew **nothing at all** over a selected annotation, and said
//! why, in a comment at the head of [`section`] which is reproduced here in
//! full because the shape of the correction is the useful part:
//!
//! > *An annotation's geometry is its `/Rect`, which no verb in this build
//! > rewrites — see `FEATURES.md`'s Format-tab row. Showing editable X/Y/W/H
//! > over one would be a control that accepts a value and discards it.*
//!
//! **That was true when it was written and stopped being true.**
//! `EditSession::move_annotation` (`pdfcer-core` `edit.rs:25391`) and
//! `EditSession::resize_annotation` (`edit.rs:24477`) both ship, and this shell
//! was *already calling both of them* — `canvas::annotdrag` raises
//! [`AnnotAction::Move`](crate::app::actions::annot::AnnotAction::Move) on the
//! release of a drag and `canvas::resizing` raises
//! [`AnnotAction::Resize`](crate::app::actions::annot::AnnotAction::Resize) on
//! the release of a grip. So the refusal above had outlived its premise by the
//! width of two verbs, and the operator could **drag** a mark to a place and
//! could not **type** one.
//!
//! ⇒ On an ISO A1 CAD sheet, typing the number is frequently the only accurate
//! route. The module header's own argument for the content half applies
//! unchanged and harder: a revision cloud that must sit exactly 25 mm inside
//! the title block cannot be placed by hand at fit-page zoom, where 25 mm is
//! about seven pixels.
//!
//! ★★ **The correction is recorded rather than the comment quietly deleted**,
//! which is this project's standing practice: a claim that was right and
//! expired teaches something a clean file does not — that a refusal written
//! against a missing capability must name the capability, so that whoever adds
//! it can grep for the refusals it unblocks. That comment named the verb
//! ("no verb in this build rewrites `/Rect`") and was still not found for the
//! nine days between `move_annotation` landing and this change.
//!
//! ### What the annotation half shares, and what it cannot
//!
//! | | page-content object | markup annotation |
//! |---|---|---|
//! | the four numbers | anchors' bounding box | the annotation's `/Rect`, normalised |
//! | the Y convention | **bottom edge, Y up** | *the same* — [`crate::text::panels::properties::geometry_units_note`] is drawn once and describes both |
//! | a move | `VectorAction::MoveSelection`, a delta | `AnnotAction::Move`, a delta |
//! | a resize | `resizing::action`, pivot + factors | `AnnotAction::Resize`, anchor + factors |
//! | refusals | off-canvas, not a path, no node model | **locked** (`/F` bit 8), and the engine's foreign-appearance refusal |
//!
//! ★★★ **`move_annotation` takes a DELTA and `resize_annotation` takes an
//! ANCHOR plus FACTORS.** Neither takes the absolute rectangle the operator
//! typed, so both fields are converted here, by [`annot_plan`], using the same
//! two helpers ([`delta`] and [`factors`]) the content half uses — one
//! arithmetic, two callers, so a typed move and a typed scale cannot acquire
//! different rounding from a dragged one.
//!
//! ★★ **A ce dimension is not this section's**, and the guard is a `match` on
//! [`AnnotKind`](crate::canvas::selection::AnnotKind) rather than a comparison
//! of `/Subtype` strings. `pdfcer-core` refuses both verbs by name for one —
//! *"a ce dimension must RE-MEASURE when it moves"* — and points at
//! `move_dimension` and `move_dimension_vertex`, which are
//! [`super::dimension`]'s to reach. A string test would compile, would read
//! `"Line"` for a dimension exactly as it does for a plain line, and would
//! route a measurement into a verb that scales its rectangle and leaves the
//! number it displays saying something else.
//!
//! ## What this deliberately does NOT offer
//!
//! - **Rotation.** `EditSession` has no rotate verb, and expressing one as
//!   `move_nodes` is not the same edit: it would rotate the anchors and leave
//!   every glyph, dash pattern and line cap in the original orientation. That
//!   is a shear of the outline, not a rotation of the object.
//! - **A units picker.** Every number here is in **PDF user-space points**,
//!   which is what `move_objects` and `move_nodes` take and what the rest of
//!   this panel already shows. A millimetre field would be a conversion this
//!   module owns, and the measure tools already own a scale model with a
//!   *different* answer — a drawing at 1:50 has a page millimetre and a world
//!   millimetre and they are not the same length. Two conversions, one label.
//! - **Multi-object geometry.** The same refusal `resizing` makes and for the
//!   same reason: `move_nodes` addresses one object, and *"set both of these to
//!   40 wide"* is a different feature (align/distribute) with a different
//!   surface.

use egui::Ui;
use pdfcer_core::object::ObjId;
use pdfcer_core::vector::Point;

use crate::app::actions::annot::AnnotAction;
use crate::app::actions::{Action, VectorAction};
use crate::app::state::OpenDoc;
use crate::canvas::resizing;
use crate::canvas::selection::AnnotKind;
use crate::text::panels::annotgeometry as at;
use crate::text::panels::properties as t;

/// The `ui-rect` region this section publishes, so a driven check can find the
/// fields without knowing the panel's arrangement.
pub const REGION: &str = "properties.geometry"; // ui-text-exempt: diagnostic region name

/// The **Width** field's own region.
///
/// ★ Published per field rather than leaving a driven check to divide [`REGION`]
/// into quarters, because the row heights are the theme's and would change under
/// a UI-scale setting the operator can move. `D:/dev/rag/egui/` records the
/// general form: *a harness that computes a control's position from a container's
/// is asserting the layout it was written against, not the one that shipped.*
pub const WIDTH_REGION: &str = "properties.geometry.width"; // ui-text-exempt: diagnostic region name

/// The **Apply** button's own region.
pub const APPLY_REGION: &str = "properties.geometry.apply"; // ui-text-exempt: diagnostic region name

/// The `ui-rect` region the **annotation** arm publishes.
///
/// ★★ A DIFFERENT name from [`REGION`], deliberately, and it is not tidiness.
/// A driven check that found `properties.geometry` could not tell which subject
/// it had got, so *"the width field did nothing"* would be indistinguishable
/// between a broken annotation resize and a check that had selected a path by
/// accident. The region name is the one place the harness can learn what the
/// section is currently about without reading the document.
pub const ANNOT_REGION: &str = "properties.annotgeometry"; // ui-text-exempt: diagnostic region name

/// The annotation arm's **Width** field, published per field for [`WIDTH_REGION`]'s
/// stated reason — row heights are the theme's, and a harness that divides a
/// container into quarters is asserting a layout it was written against.
pub const ANNOT_WIDTH_REGION: &str = "properties.annotgeometry.width"; // ui-text-exempt: diagnostic region name

/// The annotation arm's **Apply** button.
pub const ANNOT_APPLY_REGION: &str = "properties.annotgeometry.apply"; // ui-text-exempt: diagnostic region name

/// The smallest width or height the fields will accept.
///
/// A quarter point, matching `resizing::is_usable`'s own floor on the factors
/// it will act on. Below it the scale is a degenerate collapse — every node of
/// the path onto one line — which `move_nodes` would happily perform and which
/// no operator means.
pub const MIN_EXTENT_PT: f64 = 0.25;

/// The typed values, and what they were seeded from.
///
/// # Why the seed is stored and not just the values
///
/// Because *what changed* is the question this section has to answer on Apply,
/// and it cannot be answered by comparing the draft to the object's **current**
/// bounds — those are the same numbers the draft was seeded from, so an
/// operator who typed nothing and one who typed the current value back in would
/// be indistinguishable. Storing the seed makes "the operator touched this
/// field" a fact rather than an inference.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct GeometryDraft {
    /// What the draft describes: `(page, subject, edit epoch)`.
    ///
    /// `None` before the first seed. See the module header for why the epoch
    /// is a member and not an optimisation, and [`Subject`] for why the middle
    /// member became an enum on 2026-09-06.
    stamp: Option<(usize, Subject, u64)>,
    /// Left edge, PDF user-space points.
    pub x: f64,
    /// **Bottom** edge, PDF user-space points — Y is up in PDF space, and this
    /// field is labelled from the bottom rather than silently flipped, because
    /// a panel that showed a top-down Y would disagree with every other number
    /// in the program including the status bar's cursor readout.
    pub y: f64,
    /// Width, PDF user-space points.
    pub w: f64,
    /// Height, PDF user-space points.
    pub h: f64,
}

/// **What a draft is a draft OF.**
///
/// # ★★★ Why this is an enum and was a bare `usize`
///
/// Because the two subjects number themselves in different, overlapping
/// address spaces and neither number knows it. A page-content object is a
/// **paint-order index** — 0, 1, 2 — and an annotation is a **stable object
/// id**, whose `num` is also a small integer on almost every real document.
///
/// With the stamp's middle member a bare `usize`, selecting content object 7
/// and then an annotation that happens to be object 7 0 R, on the same page,
/// in the same edit epoch, would have left [`GeometryDraft::sync`] thinking
/// the draft already described the new selection. **It would not re-seed**, so
/// the panel would show the path's numbers over the annotation, and pressing
/// Apply would move the annotation by the difference between two unrelated
/// rectangles.
///
/// ⇒ That is a wrong edit produced by a *coincidence of integers*, which is the
/// worst class of defect this project keeps naming: it is not reproducible on
/// most documents, so a test written against a fixture would pass and the
/// operator would report it once, on one drawing, and never again.
///
/// An enum makes the two spaces distinguishable to the compiler and to
/// `PartialEq`, and costs nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Subject {
    /// A page-content object, by **paint-order index** on the page.
    Object(usize),
    /// A markup annotation, by **stable object id**.
    ///
    /// Stable is the operative word and is why an id rather than an `/Annots`
    /// position: `/Annots` renumbers when anything is added or removed, and a
    /// draft stamped with a position would survive a delete somewhere else on
    /// the page and then describe a different comment.
    Annot(ObjId),
}

/// The axis-aligned bounds of a path object, in PDF user space.
///
/// Derived from the object's **anchors**, which is the same set
/// [`resizing::action`] moves — deliberately, so the number this panel shows
/// and the number the edit acts on cannot disagree. A bounding box taken from
/// any other source (the object's `/BBox`, the canvas outline, the render
/// extent) would include curve bulges, line weight or a transform this section
/// does not apply, and the operator would type 40 and measure something else.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bounds {
    /// Minimum X.
    pub x0: f64,
    /// Minimum Y.
    pub y0: f64,
    /// Maximum X.
    pub x1: f64,
    /// Maximum Y.
    pub y1: f64,
}

impl Bounds {
    /// The bounds of a list of anchors, or `None` for an empty list.
    #[must_use]
    pub fn of(points: &[(usize, Point)]) -> Option<Self> {
        let (_, first) = points.first()?;
        let mut b = Self {
            x0: first.x,
            y0: first.y,
            x1: first.x,
            y1: first.y,
        };
        for (_, p) in points {
            b.x0 = b.x0.min(p.x);
            b.y0 = b.y0.min(p.y);
            b.x1 = b.x1.max(p.x);
            b.y1 = b.y1.max(p.y);
        }
        Some(b)
    }

    /// Width.
    #[must_use]
    pub fn w(self) -> f64 {
        self.x1 - self.x0
    }

    /// Height.
    #[must_use]
    pub fn h(self) -> f64 {
        self.y1 - self.y0
    }
}

impl GeometryDraft {
    /// Seed the draft from `bounds` if it does not already describe
    /// `(page, subject, epoch)`.
    ///
    /// Idempotent within a stamp, which is what lets it be called every frame:
    /// the operator's typing survives redraws and is discarded exactly when the
    /// thing it describes stops being the thing it described.
    pub fn sync(&mut self, page: usize, subject: Subject, epoch: u64, bounds: Bounds) {
        if self.stamp == Some((page, subject, epoch)) {
            return;
        }
        self.stamp = Some((page, subject, epoch));
        self.x = bounds.x0;
        self.y = bounds.y0;
        self.w = bounds.w();
        self.h = bounds.h();
    }

    /// Whether anything was typed — i.e. whether Apply has work to do.
    ///
    /// Compared against the seed, not against the document. See the struct's
    /// own note for why that distinction is load-bearing.
    #[must_use]
    pub fn differs_from(&self, bounds: Bounds) -> bool {
        !near(self.x, bounds.x0)
            || !near(self.y, bounds.y0)
            || !near(self.w, bounds.w())
            || !near(self.h, bounds.h())
    }

    /// Whether the typed extents are large enough to act on.
    #[must_use]
    pub fn is_usable(&self) -> bool {
        self.w >= MIN_EXTENT_PT && self.h >= MIN_EXTENT_PT
    }
}

/// Two values that are the same number to within a tenth of a point.
///
/// ★ A tolerance rather than `==`, because these values make a round trip
/// through an `f64` spinner and back, and `40.0` typed into a field that was
/// seeded with `39.999999999999996` is the operator changing nothing. Without
/// it, merely selecting an object and pressing Apply would raise a move of
/// 4 × 10⁻¹⁵ points — a real undo entry, a real content-stream rewrite, and a
/// real cache invalidation, for an edit with no effect at any zoom.
///
/// A tenth of a point is about 35 µm on paper: finer than any plotter this
/// operator's drawings are printed on, and coarser than every float artefact.
fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 0.1
}

// ===========================================================================
// ★★★ THE ARITHMETIC, SHARED BY BOTH SUBJECTS — 2026-09-06
// ===========================================================================
//
// [`plan`] serves a page-content object and [`annot_plan`] serves a markup
// annotation, and the two engine verbs behind them take **different shapes**:
//
//     move_nodes / move_objects   ← a delta        `resizing::action` builds it
//     move_annotation(id, dx, dy) ← a delta
//     resize_annotation(id, anchor, sx, sy, opts)  ← an ANCHOR plus FACTORS
//
// The delta and the factors are therefore computed once each, here, and the
// two `*_plan` functions do nothing but package them for the verb they feed.
//
// ★★ That is not a line-count economy. Two copies of `draft.w / bounds.w()`
// would be two places for the zero-extent guard to be remembered, two answers
// to *what is "unchanged"*, and — the one that would actually have bitten —
// two roundings, so a mark placed by typing and the same mark placed by
// dragging would land a hundredth of a point apart and a "did the typed route
// agree with the dragged one?" test would have to carry a tolerance to pass.
// One arithmetic makes that agreement structural.

/// **The translation the draft asks for**, in PDF user-space points.
///
/// The bottom-left corner is the reference on both subjects: `x` is the left
/// edge and `y` is the **bottom** edge, Y increasing upward, exactly as
/// [`crate::text::panels::properties::geometry_units_note`] tells the operator
/// under the heading. There is one convention in this file and it is that one.
fn delta(draft: &GeometryDraft, bounds: Bounds) -> (f64, f64) {
    (draft.x - bounds.x0, draft.y - bounds.y0)
}

/// **The scale factors the draft asks for.**
///
/// ★ A zero-extent axis cannot be scaled and is not an error: a horizontal line
/// has no height, and asking for `h_new / 0` is how a NaN reaches `move_nodes`
/// — or, on the annotation side, how a NaN reaches `resize_annotation`, which
/// refuses a non-finite factor by name (`EditError::ResizeFactorInvalid`) but
/// would have been asked a question nobody meant. The factor is 1 — leave that
/// axis alone — which is also what the operator means, because a field showing
/// `0.0` for a flat line is describing a fact rather than offering an edit.
fn factors(draft: &GeometryDraft, bounds: Bounds) -> (f64, f64) {
    let sx = if bounds.w() > f64::EPSILON {
        draft.w / bounds.w()
    } else {
        1.0
    };
    let sy = if bounds.h() > f64::EPSILON {
        draft.h / bounds.h()
    } else {
        1.0
    };
    (sx, sy)
}

/// Whether the operator changed the position.
fn moved(draft: &GeometryDraft, bounds: Bounds) -> bool {
    !near(draft.x, bounds.x0) || !near(draft.y, bounds.y0)
}

/// Whether the operator changed the size.
fn scaled(draft: &GeometryDraft, bounds: Bounds) -> bool {
    !near(draft.w, bounds.w()) || !near(draft.h, bounds.h())
}

/// What Apply should raise, given a draft and the bounds it was seeded from.
///
/// Returns the commands **in the order they must run**: the move first, then
/// the scale. That order is not cosmetic — the scale's pivot is the object's
/// bottom-left corner *as the operator sees it after the move*, so computing
/// the scale against pre-move bounds and applying it after the move would put
/// the object somewhere neither number described.
///
/// # Why this is a pure function taking `Bounds` rather than a method on `Ui`
///
/// So it can be tested without a document, a provider or an `egui::Context`.
/// The four cases that matter — position only, size only, both, neither — are
/// four assertions here and would each be a driven check otherwise.
#[must_use]
pub fn plan(draft: &GeometryDraft, bounds: Bounds) -> Plan {
    let (dx, dy) = delta(draft, bounds);
    let (sx, sy) = factors(draft, bounds);

    Plan {
        translate: moved(draft, bounds).then_some((dx, dy)),
        // The pivot is the bottom-left of the box *after* the move, so the
        // corner the operator pinned with X and Y is the corner that stays.
        scale: scaled(draft, bounds).then_some((
            Point {
                x: draft.x,
                y: draft.y,
            },
            (sx as f32, sy as f32),
        )),
    }
}

/// What one press of Apply amounts to.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Plan {
    /// `(dx, dy)` in PDF points, when the position changed.
    pub translate: Option<(f64, f64)>,
    /// `(pivot, (sx, sy))`, when the size changed.
    pub scale: Option<(Point, (f32, f32))>,
}

impl Plan {
    /// Whether the plan would do anything.
    #[must_use]
    pub fn is_empty(self) -> bool {
        self.translate.is_none() && self.scale.is_none()
    }
}

/// What one press of Apply amounts to **over a markup annotation**.
///
/// # ★★★ Why this is a second type and not [`Plan`] reused
///
/// Because the two engine verbs do not take the same numbers, and a shared
/// type would have had to lie about one of them.
///
/// * `EditSession::move_annotation(id, dx, dy)` takes a **delta**, like the
///   content move — that half genuinely is the same shape.
/// * `EditSession::resize_annotation(id, anchor, sx, sy, opts)` takes an
///   **anchor and two `f64` factors**. [`Plan::scale`] carries a
///   [`Point`](pdfcer_core::vector::Point) and two **`f32`s**, because that is
///   what `move_nodes` takes on the content side.
///
/// Reusing `Plan` would therefore have meant widening `f32` factors back to
/// `f64` at the call site — a round trip through a narrower type that loses
/// about seven significant figures for no reason, on numbers an operator typed
/// to two decimal places precisely so they would be exact. `40.0 / 27.0` in
/// `f32` and then in `f64` is not the ratio the engine would have computed.
///
/// ⇒ Both are built from the same [`delta`] and [`factors`], so the *decision*
/// is shared and only the packaging differs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnnotPlan {
    /// `(dx, dy)` in PDF points, when the position changed. Raised as
    /// [`AnnotAction::Move`].
    pub translate: Option<(f64, f64)>,
    /// `(anchor, (sx, sy))`, when the size changed. Raised as
    /// [`AnnotAction::Resize`].
    ///
    /// The anchor is the point that **stays still** — the bottom-left corner
    /// the operator pinned with the Left and Bottom fields, taken *after* the
    /// move, which is why the move is raised first. `canvas::resizing` calls
    /// the same value the grip's `pivot` and states the trap: using the corner
    /// being dragged instead makes the shape grow away from the hand.
    pub resize: Option<((f64, f64), (f64, f64))>,
}

impl AnnotPlan {
    /// Whether the plan would do anything.
    #[must_use]
    pub fn is_empty(self) -> bool {
        self.translate.is_none() && self.resize.is_none()
    }
}

/// What Apply should raise over an annotation, given a draft and the `/Rect`
/// it was seeded from.
///
/// Returns the two verbs **in the order they must run**: the move first, then
/// the resize. Same reason as [`plan`]'s and it matters more here, because the
/// anchor is an absolute point rather than a factor — computing it against the
/// pre-move rectangle and applying it after the move would pin a corner the
/// annotation no longer has.
///
/// # ★★ Two undo entries for one press, disclosed rather than hidden
///
/// `move_annotation` and `resize_annotation` are separate `EditSession`
/// commands and there is no combined one, so an operator who changes Left *and*
/// Width gets two `Ctrl+Z` steps. That is exactly what the content half already
/// does — the module header's *"the operator who changes only X gets exactly
/// one entry, and the operator who changes X and W gets two"* — and it is
/// stated here too because a reader arriving at the annotation arm should not
/// have to infer it from the other one.
#[must_use]
pub fn annot_plan(draft: &GeometryDraft, bounds: Bounds) -> AnnotPlan {
    let (dx, dy) = delta(draft, bounds);
    let (sx, sy) = factors(draft, bounds);
    AnnotPlan {
        translate: moved(draft, bounds).then_some((dx, dy)),
        resize: scaled(draft, bounds).then_some(((draft.x, draft.y), (sx, sy))),
    }
}

/// Draw the section, returning whether it drew anything.
///
/// Returns `false` — and draws **nothing**, per R9 — whenever the selection has
/// no subject this section can act on. That covers no selection, several
/// objects, a text run, an image, and a **ce dimension** (whose geometry has
/// its own verbs and its own section). In every one of those cases the correct
/// surface is silence rather than four greyed spinners: the fields are not
/// *temporarily* unavailable, they have no subject.
///
/// # ★★★ The annotation fork, and the comment that used to be here
///
/// Until 2026-09-06 this function's first act was:
///
/// ```text
/// // An annotation's geometry is its `/Rect`, which no verb in this build
/// // rewrites — see `FEATURES.md`'s Format-tab row. Showing editable X/Y/W/H
/// // over one would be a control that accepts a value and discards it.
/// if doc.selection.annot().is_some() {
///     return false;
/// }
/// ```
///
/// **The premise expired.** `EditSession::move_annotation` and
/// `EditSession::resize_annotation` both ship and this shell already calls
/// both, from `canvas::annotdrag` and `canvas::resizing`. The module header
/// carries the full account of the correction and of why it is recorded rather
/// than quietly removed. What replaced the `return false` is a fork to
/// [`annot_section`], which raises those same two verbs.
pub fn section(
    ui: &mut Ui,
    doc: &OpenDoc,
    draft: &mut GeometryDraft,
    actions: &mut Vec<Action>,
) -> bool {
    // ★ The annotation arm runs FIRST and returns unconditionally, because the
    // selection model makes the two mutually exclusive by construction
    // (`canvas::selection::SelectionState` — *"one canvas, one selection"*), so
    // falling through to the content arm after an annotation arm that declined
    // could only ever read a stale content selection.
    if let Some(annot) = doc.selection.annot() {
        return annot_section(ui, doc, &annot.target, draft, actions);
    }
    let page = doc.view.page_index;
    let objects = doc.selection.object_indices_on(page);
    let [object] = objects.as_slice() else {
        return false;
    };
    let object = *object;
    let Some(provider) = doc.page_objects() else {
        return false;
    };
    let points = provider.object_node_points(object);
    let Some(bounds) = Bounds::of(&points) else {
        return false;
    };
    // The `Ref` into the document's object cache is released before anything is
    // drawn — the same short-borrow discipline `object_section` states, and it
    // matters more here because `apply` will want `&mut OpenDoc` this frame.
    drop(provider);

    draft.sync(page, Subject::Object(object), doc.edit_epoch, bounds);

    // ★★★ `ui_rect_visible`, not `ui_rect` — 2026-08-26, and the same lesson
    // `dialogs::settings::widgets::group` learned for its headings.
    //
    // The Properties panel is a `ScrollArea`, and in an ordinary dock layout
    // this section is taller than the slot it gets. A rect published for a
    // control that is scrolled out of sight does not merely mislead a reader:
    // a driven check **clicks** it, and the click lands on whatever is
    // genuinely at those coordinates.
    //
    // That is not hypothetical. `ui-verify geometry_fields_resize_a_shape`
    // reported *"THE WIDTH FIELD WAS SCRUBBED BY 80 PIXELS AND APPLY COMMITTED
    // NOTHING AND DECLINED NOTHING"* — a report that reads as a dead button and
    // was filed as one. The trace said otherwise:
    //
    //     properties.geometry        [[786.0 591.7] - [1100.0 762.0]]
    //     properties.geometry.apply  [[786.0 776.7] - [ 835.0 804.7]]
    //
    // Apply was **14 points below the panel's viewport**, the click went to
    // empty canvas, and nothing was pressed. The button was never broken.
    //
    // ★ Publishing only what is visible turns that false failure into an honest
    // SKIP naming the real condition — the panel is shorter than its content.
    // An absent region is a much better lie-free answer than a present one that
    // cannot be clicked.
    //
    // ★ And it is the section's REAL extent, not `ui.max_rect()`. That was the
    // available space when the section started drawing — which in a scroll area
    // is the remaining viewport — so it published a rect ending at 762 while
    // its own Apply button laid out at 776. A region that does not contain its
    // own controls is not a region.
    // No `.strong()` — R84 / DEFECTS.md D11.
    ui.label(t::geometry_heading());
    ui.label(egui::RichText::new(t::geometry_units_note()).small().weak());

    // ★ `None` for the disabled reason: a page-content object carries no
    // per-object lock. `/F` bit 8 is an ANNOTATION flag (§12.5.3 Table 165) and
    // has no counterpart in a content stream, so there is no state in which
    // these four are drawn and dead.
    field(ui, t::geometry_x(), &mut draft.x, None, None);
    field(ui, t::geometry_y(), &mut draft.y, None, None);
    field(ui, t::geometry_w(), &mut draft.w, Some(WIDTH_REGION), None);
    field(ui, t::geometry_h(), &mut draft.h, None, None);

    let changed = draft.differs_from(bounds);
    let usable = draft.is_usable();
    // ★ The draft AND what was inferred from it, on the trace channel, because
    // the three ways this section fails are indistinguishable from outside:
    // Apply greyed because the draft was wiped, Apply greyed because the scrub
    // never landed, and Apply live but `plan` returning nothing. A line saying
    // only "Apply was greyed" would have sent the first driven run of this
    // feature looking in the wrong one of the three.
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "geometry-draft x={:.2} y={:.2} w={:.2} h={:.2} bw={:.2} bh={:.2} \
             changed={changed} usable={usable}",
            draft.x,
            draft.y,
            draft.w,
            draft.h,
            bounds.w(),
            bounds.h()
        )
    });

    // ★ Greying, not hiding, and this is the case R9 reserves it for: Apply is
    // *temporarily* unavailable — type a different number and it works — which
    // is exactly the distinction the rule draws against a capability that is
    // absent. Both reasons are on the hover, because a dead button with no
    // explanation is the defect this project keeps finding.
    let enabled = changed && usable;
    let response = ui
        .add_enabled(enabled, egui::Button::new(t::geometry_apply()))
        .on_disabled_hover_text(if usable {
            t::geometry_nothing_typed()
        } else {
            t::geometry_too_small()
        });
    crate::diag::ui_rect_visible(APPLY_REGION, response.rect, ui.clip_rect());

    if response.clicked() {
        let plan = plan(draft, bounds);
        if let Some((dx, dy)) = plan.translate {
            actions.push(
                VectorAction::MoveSelection {
                    page,
                    objects: vec![object],
                    dx,
                    dy,
                }
                .into(),
            );
        }
        if let Some((pivot, factors)) = plan.scale {
            // ★ Routed through `resizing::action` rather than assembled here,
            // so the six refusals — not a path, no nodes, no object model — are
            // asked once and answered the same way for a typed edit as for a
            // dragged one. A second construction of `MoveNodes` in this file
            // would be a second place for "can this object be scaled?" to have
            // an opinion.
            let planned = resizing::action(
                &doc.selection,
                page,
                doc.page_objects().as_deref(),
                pivot,
                factors,
            );
            match planned {
                Ok(action) => actions.push(action),
                // Off-canvas, in words, through the SAME `decline` the grips
                // use — so a typed refusal and a dragged one produce one
                // sentence from one place. See `resizing::decline` for why a
                // refusal is recorded against epoch zero.
                Err(refusal) => resizing::decline(refusal),
            }
        }
    }

    ui.separator();
    // ★ Published HERE, at the end, and that placement is the fix.
    //
    // `ui.min_rect()` after the section has drawn is what it actually occupies.
    // Before it draws, `min_rect` is empty and `max_rect` is the space
    // *available* — which in a scroll area is the remaining viewport, and which
    // is what this used to publish: a rect ending at 762 while its own Apply
    // button laid out at 776. A region that does not contain its own controls
    // is not a region, and a check dividing it into quarters to find a field
    // would have been dividing the wrong box.
    crate::diag::ui_rect_visible(REGION, ui.min_rect(), ui.clip_rect());
    true
}

/// **A selected annotation's `/Rect`, normalised, in PDF user space.**
///
/// `None` when the annotation is not among the page's — reachable after an undo
/// or an external reload has removed it while the selection still names it —
/// and when it carries no `/Rect`, which `EditSession` refuses by name
/// (`EditError::AnnotationRectMissing`) rather than inventing one.
///
/// # ★★★ Read from the DOCUMENT, not from the selection's `outline`
///
/// [`AnnotSelection::outline`](crate::canvas::selection::AnnotSelection::outline)
/// is right there and is the wrong number. It is in **canvas space** — Y down
/// from the page's top-left, with `/Rotate` applied and the crop box's origin
/// subtracted — and getting back to PDF user space from it means running
/// `viewer::canvas_to_pdf_space` twice and through `f32`.
///
/// `canvas::mapping`'s header calls a second conversion *the classic silent
/// defect*, and here it would be worse than usually: the fields would show the
/// number that came back from a round trip through two transforms, the operator
/// would type `40.00`, and on a rotated page the value written into `/Rect`
/// would be neither what they typed nor what they saw. Reading the dictionary
/// is one hop and no convention.
///
/// # ★★ Normalised, because §7.9.5 does not require a `/Rect` to be
///
/// A rectangle may legitimately be written with its *upper-right* corner first,
/// and producers do it. `min`/`max` on both axes is what makes "Left" mean the
/// left edge rather than "whichever X the file happened to write first" — and
/// without it a width would come out negative, which
/// `resize_annotation` would divide by and turn into a mirror.
///
/// ★ [`crate::canvas::annotclip::rect_centre_of`] gets the same fact right by a
/// different route (it averages the pair, which needs no normalisation) and
/// says so; the two agree because both are reading §7.9.5 rather than a habit.
///
/// # Cost
///
/// One `/Annots` walk per frame, bounded by
/// `pdfcer_core::annot::MAX_ANNOTS_PER_PAGE`. The same price
/// [`crate::canvas::annotclip::carried_options`] pays for the same reason —
/// there is no public verb that models one annotation dictionary — and the same
/// order as the content arm's `doc.page_objects()`, which is also per frame.
fn annot_bounds(doc: &OpenDoc, page: usize, id: ObjId) -> Option<Bounds> {
    let graph = doc.session.graph();
    let page_ref = doc.pages.get(page)?;
    let annot = pdfcer_core::annot::page_annotations(&graph, page_ref.id)
        .into_iter()
        .find(|a| a.id == Some(id))?;
    let rect = annot.rect?;
    Some(Bounds {
        x0: rect.llx.min(rect.urx),
        y0: rect.lly.min(rect.ury),
        x1: rect.llx.max(rect.urx),
        y1: rect.lly.max(rect.ury),
    })
}

/// **Draw the four fields over a selected markup annotation**, returning
/// whether anything was drawn.
///
/// # ★★★ The three refusals, and why each takes the surface it takes
///
/// | condition | surface | why |
/// |---|---|---|
/// | a **ce dimension** | draws nothing, returns `false` | it is not this section's subject at all — [`super::dimension`] owns it, and both engine verbs refuse it **by name** |
/// | the annotation is **gone** or has no `/Rect` | draws nothing, returns `false` | there is no number to show; four spinners over `0.0` would be an invitation to place a mark at the sheet's corner |
/// | `/F` bit 8 — **locked** | draws the fields and Apply, **greyed**, with [`crate::text::panels::annotgeometry::locked`] on hover | R9's reserved case exactly: the capability is present and this annotation is out of bounds, so selecting a different one restores it |
///
/// ★★ **The ce-dimension guard is an [`AnnotKind`] match, never a `/Subtype`
/// string comparison**, and that is rule 15 made mechanical. A ce dimension IS
/// a `/Line` — `/IT /LineDimension` — so `subtype == "Line"` reads `true` for a
/// dimension and for a plain arrow alike, and routing a measurement into
/// `resize_annotation` would scale its rectangle and its baked appearance and
/// leave the sidecar geometry the displayed number is derived from where it
/// was. The mark would then say `1250 mm` about a line that is 900 long.
/// `canvas::selection::annot::AnnotKind`'s header states why it is an enum:
/// *a bool is a fact a caller may forget to read; a variant is one the compiler
/// makes them handle.*
///
/// ★ The engine would in fact catch it — `move_annotation` returns
/// `AnnotationMoveWrongVerb` naming `move_dimension` — so this guard is not the
/// last line of defence. It is the one that keeps the shell from **offering**
/// the affordance, which is R83: a control that can only produce a refusal is
/// not drawn.
///
/// # ★★ The foreign-appearance refusal is NOT guarded here
///
/// `resize_annotation` refuses a non-uniform scale over an `/AP` pdfcer did not
/// draw, unless `allow_appearance_distortion` is set. That condition cannot be
/// evaluated without rebuilding the appearance and comparing bytes, so there is
/// nothing honest to grey. It is surfaced **after** the press, by name, because
/// the action raised here is the same [`AnnotAction::Resize`] the eight grips
/// raise and `app::actions::annots::resize` already catches that error and
/// records `decline::record_resize_not_rebuildable`. A typed Width that the
/// engine declines therefore says exactly what a dragged one says.
/// `crate::text::panels::annotgeometry`'s header carries the whole argument.
fn annot_section(
    ui: &mut Ui,
    doc: &OpenDoc,
    target: &crate::canvas::selection::AnnotTarget,
    draft: &mut GeometryDraft,
    actions: &mut Vec<Action>,
) -> bool {
    // ★ Exhaustive, so a third `AnnotKind` fails to compile here rather than
    // falling into whichever arm was written first. That is the same property
    // `annotclip::translated` buys with its exhaustive `MarkupSpec` match and
    // for the same reason: the failure of a wildcard is silent.
    match target.kind {
        AnnotKind::Markup => {}
        AnnotKind::CeDimension => return false,
    }
    let page = target.page;
    let Some(bounds) = annot_bounds(doc, page, target.id) else {
        return false;
    };
    draft.sync(page, Subject::Annot(target.id), doc.edit_epoch, bounds);

    // No `.strong()` — R84 / DEFECTS.md D11.
    //
    // ★★ The SAME heading, the SAME units note and the SAME four labels the
    // content arm draws. The units note is the load-bearing one: it says
    // *"Points, measured to the bottom-left corner. Y increases upward"*, which
    // is true of a `/Rect` in exactly the terms it is true of a path's bounding
    // box, and a second sentence phrased for annotations would be a second
    // statement of one coordinate convention. Two statements of a convention is
    // how a panel ends up measuring Y from the top in one half.
    ui.label(t::geometry_heading());
    ui.label(egui::RichText::new(t::geometry_units_note()).small().weak());

    // ★★★ **Greyed, not hidden, and the reason is on the hover** — R9. The
    // fields themselves and not only Apply, because a live spinner over a
    // locked annotation would accept a scrub and then refuse to commit it,
    // which is the "accepts a value and discards it" control this whole section
    // was once withheld to avoid.
    let locked = target.locked.then(at::locked);
    field(ui, t::geometry_x(), &mut draft.x, None, locked);
    field(ui, t::geometry_y(), &mut draft.y, None, locked);
    field(
        ui,
        t::geometry_w(),
        &mut draft.w,
        Some(ANNOT_WIDTH_REGION),
        locked,
    );
    field(ui, t::geometry_h(), &mut draft.h, None, locked);

    let changed = draft.differs_from(bounds);
    let usable = draft.is_usable();
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        //
        // ★ `annot-geometry-draft`, not `geometry-draft`: two subjects writing
        // one trace name would make `TraceLog::last("geometry-draft")` return
        // whichever arm drew most recently, and a driven check reading `bw=`
        // would silently be reading the other subject's box.
        format!(
            "annot-geometry-draft id={} x={:.2} y={:.2} w={:.2} h={:.2} \
             bx={:.2} by={:.2} bw={:.2} bh={:.2} locked={} changed={changed} usable={usable}",
            target.id.num,
            draft.x,
            draft.y,
            draft.w,
            draft.h,
            bounds.x0,
            bounds.y0,
            bounds.w(),
            bounds.h(),
            target.locked
        )
    });

    // ★ The three reasons, in the order that makes the most specific one win.
    // Locked first because it is a fact about the file rather than about the
    // typing — an operator whose fields are dead needs to know the file said
    // so, not that they have not typed anything yet, and "type a different
    // number" over a locked mark is advice that cannot work.
    let enabled = !target.locked && changed && usable;
    let why = if target.locked {
        at::locked()
    } else if usable {
        t::geometry_nothing_typed()
    } else {
        t::geometry_too_small()
    };
    let response = ui
        .add_enabled(enabled, egui::Button::new(t::geometry_apply()))
        .on_disabled_hover_text(why);
    crate::diag::ui_rect_visible(ANNOT_APPLY_REGION, response.rect, ui.clip_rect());

    if response.clicked() {
        let plan = annot_plan(draft, bounds);
        // ★★★ THE MOVE FIRST. `resize_annotation`'s anchor is an ABSOLUTE
        // point, so the corner the operator pinned with Left and Bottom must
        // already be where they said before it is used as the fixed point.
        // Raising the resize first would anchor on a corner the annotation is
        // about to stop having, and the mark would end up somewhere neither
        // number described — the same trap `plan` states for the content arm,
        // sharper here because a factor tolerates a stale origin and a point
        // does not.
        if let Some((dx, dy)) = plan.translate {
            // ★★ A **delta**, which is what `move_annotation(id, dx, dy)`
            // takes. The field holds an absolute Left/Bottom, so the conversion
            // is `delta`'s subtraction and it happens exactly once, in the pure
            // function, rather than in this arm where it could not be tested
            // without a document.
            actions.push(Action::Annot(AnnotAction::Move {
                id: target.id,
                dx,
                dy,
            }));
        }
        if let Some((anchor, (sx, sy))) = plan.resize {
            actions.push(Action::Annot(AnnotAction::Resize {
                id: target.id,
                anchor,
                sx,
                sy,
                // ★★ Whether the two factors are equal, computed the same way
                // `canvas::resizing` computes it from a grip drag. The engine
                // asked for this by name — it reports what the operator's hand
                // did, and a uniform scale of a foreign appearance is always
                // safe where a non-uniform one is refused.
                //
                // ★ Typing `40` into Width and leaving Height alone is a
                // NON-uniform scale even though the operator touched one field.
                // That is correct and is the case the refusal exists for.
                uniform: (sx - sy).abs() <= f64::EPSILON,
                // ★★★ **The operator's Tool-row switches, read live** —
                // `OPERATOR_REQUESTS.md` O51. `AnnotAction::Resize::modifiers`
                // documents why a *drag* must carry them rather than let the
                // apply arm read them: the gesture completed frames before the
                // queue drained. A press of Apply has no such gap — the click
                // and the read are the same frame — so this is
                // `CommitTextAnnot`'s case rather than `CommitMarkup`'s, and
                // reading them here keeps one store rather than adding a second
                // copy in the draft.
                modifiers: crate::canvas::scaling::read(ui.ctx()),
            }));
        }
    }

    ui.separator();
    // Published at the END, for the reason the content arm's own publication
    // gives at length: before it draws, `min_rect` is empty and `max_rect` is
    // the *available* space, which in a scroll area is the remaining viewport
    // — a region that does not contain its own Apply button is not a region.
    crate::diag::ui_rect_visible(ANNOT_REGION, ui.min_rect(), ui.clip_rect());
    true
}

/// One labelled numeric field.
///
/// `DragValue` rather than a `TextEdit`, because it accepts both — an operator
/// can scrub it *or* click and type an exact number — and the scrubbing costs
/// nothing here precisely because the fields edit a draft. On a live-committing
/// surface a scrubbable field would be the eighty-undo-entries problem the
/// module header describes; on a drafted one it is a free second input method.
///
/// ★ `disabled` is `Some(reason)` when the control must be drawn and dead —
/// today, a locked annotation. R9 requires the reason on the hover, and it is
/// attached to **each field** rather than to a wrapper because
/// `add_enabled_ui` produces no response to hang a hover on: an operator
/// pointing at the greyed Width would get nothing, which is the dead control
/// with no explanation this project keeps finding.
fn field(
    ui: &mut Ui,
    label: &str,
    value: &mut f64,
    region: Option<&'static str>,
    disabled: Option<&str>,
) {
    ui.horizontal(|ui| {
        ui.label(label);
        let widget = egui::DragValue::new(value).speed(SPEED).fixed_decimals(2);
        let response = match disabled {
            Some(reason) => ui.add_enabled(false, widget).on_disabled_hover_text(reason),
            None => ui.add(widget),
        };
        if let Some(region) = region {
            // Visible-clipped, for the reason the section's own publication
            // gives: this panel scrolls, and a field a harness can see but an
            // operator cannot is a coordinate that scrubs whatever is behind
            // it.
            crate::diag::ui_rect_visible(region, response.rect, ui.clip_rect());
        }
    });
}

/// PDF points per screen pixel of horizontal scrub.
///
/// Half a point, so a hundred-pixel drag spans fifty points — about the range a
/// draughtsman adjusts a symbol by — and so the value visibly moves on a slow
/// drag rather than jumping. A driven check depends on this being **exact**:
/// scrubbing `n` pixels changes the field by `n × SPEED`, which is what lets the
/// harness assert the number it expects instead of merely that something
/// changed.
const SPEED: f64 = 0.5;

#[cfg(test)]
mod tests {
    use super::*;

    fn pts(v: &[(f64, f64)]) -> Vec<(usize, Point)> {
        v.iter()
            .enumerate()
            .map(|(i, (x, y))| (i, Point { x: *x, y: *y }))
            .collect()
    }

    #[test]
    fn bounds_come_from_the_anchors() {
        let b = Bounds::of(&pts(&[(10.0, 20.0), (50.0, 20.0), (50.0, 80.0)])).unwrap();
        assert!((b.x0 - 10.0).abs() < 1e-9);
        assert!((b.y0 - 20.0).abs() < 1e-9);
        assert!((b.w() - 40.0).abs() < 1e-9);
        assert!((b.h() - 60.0).abs() < 1e-9);
    }

    #[test]
    fn no_anchors_means_no_bounds() {
        assert!(Bounds::of(&[]).is_none());
    }

    /// ★★ **Selecting an object and pressing Apply raises nothing.**
    ///
    /// The float round trip through the spinner is why this needs a test rather
    /// than being obvious: a seed of `39.999999999999996` and a typed `40.0`
    /// are different `f64`s and the same edit. Without the tolerance this would
    /// raise a move of 4 × 10⁻¹⁵ points — a real undo entry and a real
    /// content-stream rewrite for a change no zoom can show.
    #[test]
    fn an_untouched_draft_plans_nothing() {
        let b = Bounds {
            x0: 10.0,
            y0: 20.0,
            x1: 50.0,
            y1: 80.0,
        };
        let mut d = GeometryDraft::default();
        d.sync(0, Subject::Object(3), 7, b);
        assert!(!d.differs_from(b));
        assert!(plan(&d, b).is_empty());

        // The same number arriving with float dust on it is still no change.
        // Computed rather than written as a literal, because a literal with
        // that many digits is rounded back to 40.0 by the parser and the test
        // would assert nothing.
        d.w = (0.1_f64 + 0.2) * 100.0 + 10.000_000_000_000_004;
        assert!(plan(&d, b).is_empty());
    }

    /// Position only → one move, no scale.
    #[test]
    fn moving_it_plans_a_move_and_no_scale() {
        let b = Bounds {
            x0: 10.0,
            y0: 20.0,
            x1: 50.0,
            y1: 80.0,
        };
        let mut d = GeometryDraft::default();
        d.sync(0, Subject::Object(0), 0, b);
        d.x = 110.0;
        let p = plan(&d, b);
        assert_eq!(p.translate, Some((100.0, 0.0)));
        assert!(p.scale.is_none());
    }

    /// ★ Size only → one scale, pivoted on the corner the operator did NOT
    /// touch, so the object grows to the right and upward rather than about its
    /// middle. That is what a properties panel means by "X, Y, W, H": X and Y
    /// name a corner, and changing W moves the *other* edge.
    #[test]
    fn resizing_it_pivots_on_the_stated_corner() {
        let b = Bounds {
            x0: 10.0,
            y0: 20.0,
            x1: 50.0,
            y1: 80.0,
        };
        let mut d = GeometryDraft::default();
        d.sync(0, Subject::Object(0), 0, b);
        d.w = 80.0; // double the width
        let p = plan(&d, b);
        assert!(p.translate.is_none());
        let (pivot, (sx, sy)) = p.scale.unwrap();
        assert!((pivot.x - 10.0).abs() < 1e-9, "pivot is the stated X");
        assert!((pivot.y - 20.0).abs() < 1e-9, "pivot is the stated Y");
        assert!((sx - 2.0).abs() < 1e-6);
        assert!((sy - 1.0).abs() < 1e-6, "the untouched axis is not scaled");
    }

    /// ★★ **A flat object does not produce a NaN.**
    ///
    /// A horizontal line has zero height, and the obvious implementation
    /// computes `h_new / 0`. `move_nodes` would accept the resulting NaN
    /// coordinates and write them into the content stream, producing a page
    /// that no viewer — including this one — can render.
    #[test]
    fn a_zero_height_object_scales_only_the_axis_it_has() {
        let b = Bounds {
            x0: 0.0,
            y0: 50.0,
            x1: 100.0,
            y1: 50.0,
        };
        let mut d = GeometryDraft::default();
        d.sync(0, Subject::Object(0), 0, b);
        d.w = 200.0;
        let (_, (sx, sy)) = plan(&d, b).scale.unwrap();
        assert!((sx - 2.0).abs() < 1e-6);
        assert!(sy.is_finite() && (sy - 1.0).abs() < 1e-6);
    }

    /// ★ **The draft is discarded when the document changes underneath it**,
    /// which is the sequence in the module header: type a width, undo something
    /// unrelated, press Apply. Without the epoch in the stamp the factor would
    /// be computed against bounds that no longer exist.
    #[test]
    fn a_new_epoch_reseeds_the_draft() {
        let b = Bounds {
            x0: 0.0,
            y0: 0.0,
            x1: 40.0,
            y1: 40.0,
        };
        let mut d = GeometryDraft::default();
        d.sync(0, Subject::Object(0), 1, b);
        d.w = 400.0;
        // Same page, same object, but the document moved on.
        let after = Bounds {
            x0: 0.0,
            y0: 0.0,
            x1: 10.0,
            y1: 10.0,
        };
        d.sync(0, Subject::Object(0), 2, after);
        assert!(
            (d.w - 10.0).abs() < 1e-9,
            "the typed 400 must not survive an edit it was not computed against"
        );
    }

    /// Selecting a different object reseeds too.
    #[test]
    fn a_new_object_reseeds_the_draft() {
        let b = Bounds {
            x0: 0.0,
            y0: 0.0,
            x1: 40.0,
            y1: 40.0,
        };
        let mut d = GeometryDraft::default();
        d.sync(0, Subject::Object(0), 1, b);
        d.x = 999.0;
        d.sync(0, Subject::Object(1), 1, b);
        assert!((d.x - 0.0).abs() < 1e-9);
    }

    // =======================================================================
    // The ANNOTATION arm — 2026-09-06
    // =======================================================================

    /// A `/Rect`-shaped box for the annotation assertions below.
    fn rect(x0: f64, y0: f64, x1: f64, y1: f64) -> Bounds {
        Bounds { x0, y0, x1, y1 }
    }

    /// An annotation id, for stamping.
    fn id(num: u32) -> ObjId {
        ObjId::new(num, 0)
    }

    /// ★★★ **A CONTENT OBJECT AND AN ANNOTATION WITH THE SAME NUMBER ARE
    /// DIFFERENT SUBJECTS.**
    ///
    /// This is the assertion the [`Subject`] enum exists for, and the defect it
    /// prevents is a *coincidence of integers*: page-content objects are
    /// numbered by paint order (0, 1, 2 …) and annotations by object id, whose
    /// `num` is a small integer on almost every real document. With the stamp's
    /// middle member a bare `usize`, selecting content object 7 and then
    /// annotation `7 0 R` on the same page in the same epoch left `sync`
    /// believing the draft already described the new selection — so the panel
    /// showed the path's numbers over the annotation and Apply moved it by the
    /// difference between two unrelated rectangles.
    ///
    /// ⇒ Not reproducible on most documents, so a fixture-based test would pass
    /// and the operator would report it once and never again.
    ///
    /// **Falsified** by changing the stamp back to `(usize, usize, u64)` and
    /// keying on `id.num`: this went red on the `x` assertion — the draft kept
    /// the content object's `999` — and every other test in this file stayed
    /// green. Restored.
    #[test]
    fn an_annotation_and_a_content_object_with_the_same_number_do_not_share_a_draft() {
        let object_box = rect(0.0, 0.0, 40.0, 40.0);
        let annot_box = rect(500.0, 600.0, 560.0, 640.0);
        let mut d = GeometryDraft::default();
        d.sync(0, Subject::Object(7), 1, object_box);
        d.x = 999.0;
        // The same page, the same epoch, the same NUMBER — and a different
        // address space.
        d.sync(0, Subject::Annot(id(7)), 1, annot_box);
        assert!(
            (d.x - 500.0).abs() < 1e-9,
            "★ the draft must re-seed from the annotation's /Rect. A typed 999 surviving here \
             is Apply moving the annotation by the difference between two unrelated boxes"
        );
    }

    /// Selecting a different annotation reseeds, exactly as a different object
    /// does.
    #[test]
    fn a_different_annotation_reseeds_the_draft() {
        let b = rect(10.0, 10.0, 50.0, 50.0);
        let mut d = GeometryDraft::default();
        d.sync(0, Subject::Annot(id(3)), 1, b);
        d.w = 400.0;
        d.sync(0, Subject::Annot(id(4)), 1, b);
        assert!((d.w - 40.0).abs() < 1e-9);
    }

    /// ★★ **A typed Left becomes a DELTA**, because `move_annotation` takes one.
    ///
    /// The field holds an absolute coordinate and the verb takes a
    /// displacement, so the conversion is the whole content of this arm. A plan
    /// that handed `draft.x` straight to the verb would move the annotation to
    /// `x + draft.x` — off the sheet on any real drawing, and *further* off it
    /// the further right the mark already was, which reads as a random jump.
    #[test]
    fn a_typed_position_becomes_a_delta_and_no_resize() {
        let b = rect(100.0, 200.0, 160.0, 240.0);
        let mut d = GeometryDraft::default();
        d.sync(0, Subject::Annot(id(1)), 0, b);
        d.x = 130.0;
        let p = annot_plan(&d, b);
        assert_eq!(p.translate, Some((30.0, 0.0)), "a delta, not a coordinate");
        assert!(p.resize.is_none());
    }

    /// ★★ **A typed Width becomes an ANCHOR plus a FACTOR**, because
    /// `resize_annotation` takes those and not a target rectangle.
    ///
    /// The anchor is the corner the operator pinned with Left and Bottom, so
    /// the box grows to the right and upward — which is what "X, Y, W, H"
    /// means in every properties panel: X and Y name a corner, and changing W
    /// moves the *other* edge.
    #[test]
    fn a_typed_size_becomes_an_anchor_and_factors() {
        let b = rect(100.0, 200.0, 160.0, 240.0); // 60 × 40
        let mut d = GeometryDraft::default();
        d.sync(0, Subject::Annot(id(1)), 0, b);
        d.w = 120.0; // double
        let p = annot_plan(&d, b);
        assert!(p.translate.is_none());
        let (anchor, (sx, sy)) = p.resize.expect("a resize");
        assert!(
            (anchor.0 - 100.0).abs() < 1e-9,
            "the anchor is the stated Left"
        );
        assert!(
            (anchor.1 - 200.0).abs() < 1e-9,
            "the anchor is the stated Bottom"
        );
        assert!((sx - 2.0).abs() < 1e-12);
        assert!((sy - 1.0).abs() < 1e-12, "the untouched axis is not scaled");
    }

    /// ★★★ **Move first, and the anchor is the corner the operator TYPED, not
    /// the one the annotation has now.**
    ///
    /// `resize_annotation`'s anchor is an absolute point, so this is sharper
    /// than the content arm's equivalent: a factor tolerates a stale origin and
    /// a point does not. Anchoring on `bounds.x0` here would pin a corner the
    /// annotation is about to stop having, and the mark would land somewhere
    /// neither number described.
    ///
    /// **Falsified** by anchoring on `(bounds.x0, bounds.y0)`: red, and the
    /// two single-change tests above stayed green — which is why this case
    /// needs its own test rather than being implied by them.
    #[test]
    fn moving_and_resizing_anchors_on_the_typed_corner() {
        let b = rect(100.0, 200.0, 160.0, 240.0);
        let mut d = GeometryDraft::default();
        d.sync(0, Subject::Annot(id(1)), 0, b);
        d.x = 300.0;
        d.w = 30.0;
        let p = annot_plan(&d, b);
        assert_eq!(p.translate, Some((200.0, 0.0)));
        let (anchor, (sx, _)) = p.resize.expect("a resize");
        assert!(
            (anchor.0 - 300.0).abs() < 1e-9,
            "★ the anchor is the TYPED Left, i.e. the corner as it will be after the move — \
             not {}",
            b.x0
        );
        assert!((sx - 0.5).abs() < 1e-12);
    }

    /// An untouched annotation draft plans nothing, for [`plan`]'s reason.
    #[test]
    fn an_untouched_annotation_draft_plans_nothing() {
        let b = rect(10.0, 20.0, 50.0, 80.0);
        let mut d = GeometryDraft::default();
        d.sync(0, Subject::Annot(id(5)), 3, b);
        assert!(annot_plan(&d, b).is_empty());
    }

    /// ★★ **A zero-height annotation does not produce a NaN factor.**
    ///
    /// A `/Line` drawn perfectly horizontally has a degenerate `/Rect`, and the
    /// obvious implementation computes `h_new / 0`. `resize_annotation` refuses
    /// a non-finite factor by name — `EditError::ResizeFactorInvalid` — so this
    /// would surface as a worded refusal rather than as corruption, but it
    /// would be a refusal for a question nobody asked.
    #[test]
    fn a_flat_annotation_scales_only_the_axis_it_has() {
        let b = rect(0.0, 50.0, 100.0, 50.0);
        let mut d = GeometryDraft::default();
        d.sync(0, Subject::Annot(id(6)), 0, b);
        d.w = 200.0;
        let (_, (sx, sy)) = annot_plan(&d, b).resize.expect("a resize");
        assert!((sx - 2.0).abs() < 1e-12);
        assert!(sy.is_finite() && (sy - 1.0).abs() < 1e-12);
    }

    /// ★★★ **THE TWO SUBJECTS COMPUTE THE SAME NUMBERS**, because they share
    /// [`delta`] and [`factors`].
    ///
    /// The failure this pins is not a crash: it is a mark placed by typing
    /// landing a fraction of a point away from the same mark placed by
    /// dragging, because one arm rounded through `f32` on the way to its verb
    /// and the other did not. Nothing on screen would show it and nothing else
    /// in this suite would catch it.
    ///
    /// ★ The content plan's factors are `f32` — `move_nodes` takes those — so
    /// the comparison is at `f32` precision, which is the honest bound: what is
    /// asserted is that the two arms agree to the precision the narrower one
    /// can express, not that a widened `f32` equals an `f64`.
    #[test]
    fn the_two_arms_agree_about_what_the_operator_typed() {
        let b = rect(10.0, 20.0, 50.0, 80.0);
        let mut d = GeometryDraft::default();
        d.sync(0, Subject::Object(0), 0, b);
        d.x = 35.0;
        d.w = 55.0;

        let content = plan(&d, b);
        let annot = annot_plan(&d, b);
        assert_eq!(content.translate, annot.translate, "one delta, two callers");

        let (pivot, (csx, csy)) = content.scale.expect("a scale");
        let (anchor, (asx, asy)) = annot.resize.expect("a resize");
        assert!((pivot.x - anchor.0).abs() < 1e-9 && (pivot.y - anchor.1).abs() < 1e-9);
        assert!((f64::from(csx) - asx).abs() < 1e-6);
        assert!((f64::from(csy) - asy).abs() < 1e-6);
    }

    /// A collapse is refused before it can reach `move_nodes`.
    #[test]
    fn a_degenerate_extent_is_not_usable() {
        let mut d = GeometryDraft {
            w: 0.0,
            h: 40.0,
            ..Default::default()
        };
        assert!(!d.is_usable());
        d.w = MIN_EXTENT_PT;
        assert!(d.is_usable());
    }
}
