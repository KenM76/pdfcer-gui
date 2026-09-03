//! # `panels::objects::provider` — front-to-back page object decomposition
//!
//! The thin `pdfcer-gui` adapter that plugs `pdfcer-core`'s read-only vector
//! object model (`pdfcer_core::vector`) into the shell. Salvaged from the old
//! shell's `object_provider.rs` (694 code lines, 313 test lines) per
//! `SALVAGE.md`'s Class A row. Decision 011 §2.1 set its shape:
//!
//! > Pass 9a's real provider is a thin `pdfcer-gui` adapter that CALLS INTO
//! > `pdfcer-core`'s read-only object model (which stays GUI-free); the
//! > adapter owns the trait impl, the object model owns none of it.
//!
//! ## What lives here vs in core (GUI–core separation)
//!
//! ALL geometry — decomposition, hit-testing, marquee enclosure — is
//! `pdfcer_core::vector`, in PDF user space. This module owns exactly two
//! things core cannot: (1) the **coordinate-space translation** between the
//! canvas's device convention and PDF user space, and (2) the [`TargetId`] ↔
//! object-index encoding. The translation reuses the SAME transform
//! [`pdfcer_render::page_device_geometry`] computes to rasterize the page (at
//! scale 1.0, which *is* canvas space), inverted — so selection geometry and
//! the render agree by construction, exactly as
//! [`crate::viewer::canvas_to_pdf_space`] does (this provider is the
//! batched, object-model-backed sibling of that per-point bridge).
//!
//! ## Single-page by design
//!
//! The canvas shows one page at a time and only ever queries
//! `view.page_index`, so a provider is built for the **current page** and
//! rebuilt on page change / edit. A query for any other `page_index` returns
//! nothing — cheap, and it keeps the decomposition off the hot path of a
//! large document (only the visible page is decomposed, not all N).
//!
//! `RIBBON_IA.md` §5.2 names this as the reason continuous page display is
//! "a larger build than it looks": this file returns nothing for any page
//! but the current one, and continuous mode needs a page *range*. That is
//! `GUI_ROADMAP.md` Phase 4 work, and it is a real change here rather than a
//! wiring change above.
//!
//! ## [`TargetId`] encoding
//!
//! A [`TargetId`] is the object's index into
//! [`pdfcer_core::vector::PageObjects::objects`] (paint order), cast to
//! `u64`. Consumers treat it opaquely; only this module mints and decodes
//! it.
//!
//! ---
//!
//! # What is live at S3, and what is waiting for S4
//!
//! This whole file came across, because `SALVAGE.md`'s procedure forbids
//! salvaging by snippet — *"the old GUI's value is disproportionately in its
//! doc comments; a snippet leaves those behind and the next engineer
//! re-derives a decision that was already made and already paid for."* But
//! only some of it has a consumer today, and pretending otherwise would be
//! its own dishonesty:
//!
//! | Method group | S3 consumer | Waiting on |
//! |---|---|---|
//! | [`ObjectModelProvider::build`], [`page_objects`](ObjectModelProvider::page_objects) | the Objects panel's row list | — |
//! | [`part_kind`](ObjectModelProvider::part_kind), [`part_count`](ObjectModelProvider::part_count), [`subpath_count`](ObjectModelProvider::subpath_count), [`text_run_count`](ObjectModelProvider::text_run_count) | the Objects panel's **object → part → point** nesting | — |
//! | [`subpath_node_points`](ObjectModelProvider::subpath_node_points), [`object_node_points`](ObjectModelProvider::object_node_points), [`subpath_handle_points`](ObjectModelProvider::subpath_handle_points) | the Objects panel's point rows and the Properties panel's node readout | — |
//! | [`hit_test_all`](ObjectModelProvider::hit_test_all), [`hit_test`](ObjectModelProvider::hit_test), [`hit_test_rect`](ObjectModelProvider::hit_test_rect), [`bounds`](ObjectModelProvider::bounds) | none | **S4** — the canvas selection layer and the `CanvasTargetProvider` trait |
//! | [`part_hits`](ObjectModelProvider::part_hits), [`subpath_hits`](ObjectModelProvider::subpath_hits), [`text_run_hits`](ObjectModelProvider::text_run_hits), [`nearest_node`](ObjectModelProvider::nearest_node), [`nearest_handle`](ObjectModelProvider::nearest_handle) | none | **S4** — click-to-select and the level ladder |
//! | [`part_bounds_canvas`](ObjectModelProvider::part_bounds_canvas) and friends | none | **S4** — selection outlines |
//! | [`object_sample_points`](ObjectModelProvider::object_sample_points) | none | **S5** — the measure tools' snap query and Taubin best-fit circle |
//!
//! **Every one of them is under test below.** That is the difference between
//! carrying a method forward and leaving a stub: the S4 canvas will attach a
//! trait to a working, proven implementation rather than to code nobody has
//! run since it was pasted.
//!
//! ## What changed at salvage
//!
//! 1. **`use eframe::egui` → `use egui`**, the crate-wide S0 convention.
//! 2. **The `CanvasTargetProvider` trait impl became inherent methods.**
//!    The trait lives in `canvas/` and does not exist yet. The three
//!    methods keep their names and their exact semantics, and
//!    [`ObjectModelProvider::hit_test`] — which was the *trait's provided
//!    method* over `hit_test_all` — is written out here as an inherent
//!    method with its derivation intact, so the two still cannot disagree.
//!    Re-attaching the trait at S4 is a one-line `impl` block over methods
//!    that already have the right signatures.
//! 3. **[`TargetId`] moved here from `canvas`.** It is the *encoding*, and
//!    the encoding belongs with the thing that mints it. When `canvas`
//!    grows its substrate it re-exports this rather than defining a second
//!    one — two id types over one index space is precisely the divergence
//!    this file's own docs warn about.
//! 4. **One test did not come across:
//!    `screen_tolerance_keeps_the_on_screen_catch_radius_constant`.** It
//!    asserts a law about `canvas::screen_tolerance_to_page` and
//!    `canvas::SELECT_SCREEN_TOLERANCE_PX`, neither of which exists in this
//!    crate yet, and re-declaring those constants here to keep a test green
//!    would put the tolerance in two places — which is the *cause* of the
//!    defect the test guards, not a way to guard it. It lands in `canvas/`
//!    at S4 with the functions it is about. **The substantive regression
//!    test came across intact**:
//!    [`tests::selection_tolerance_is_honoured_per_query_not_baked_in`]
//!    proves the tolerance is a per-query parameter rather than a baked
//!    constant, which is the half that lives here.
//! 5. **Two doc cross-references were repointed** at things that exist:
//!    `crate::canvas::EmptyTargetProvider` (the shippable no-op provider)
//!    and `crate::vector_edit_tool::nearest_anchor` are both S4/S5 modules,
//!    so the claims they anchored are stated directly instead of by link.
//!
//! No arithmetic, no tolerance rule, no hit ordering and no index
//! convention changed.

/// **The same questions, asked of either index space** — the `_of` family that
/// lets the Part and Node rungs be offered for something painted inside a form
/// XObject (`OPERATOR_REQUESTS.md` O70). Split out 2026-09-01 under R2; its
/// header carries why that is a seam rather than an arbitrary cut.
mod geometry;

use egui::{Pos2, Rect};
use pdfcer_core::page_tree::Page;
use pdfcer_core::vector::{
    Bounds, Handle, HitTarget, MarqueeMode, Matrix, PageObjects, Point, Segment, VectorObject,
    decompose_page, hit_test_point_deep, hit_test_rect,
};
use pdfcer_core::view::DocumentView;
use pdfcer_render::page_device_geometry;
use pdfcer_render::tiny_skia::{Point as SkPoint, Transform};

/// One selectable thing on a page, addressed opaquely — and **which of the
/// two index spaces it lives in**.
///
/// # ★★★ WHY THIS IS AN ENUM AND NOT A NUMBER
///
/// A page has two lists of objects, not one, and they index **different
/// content streams**:
///
/// * [`PageObjects::objects`] — what the *page's own* content stream paints.
///   A paint-order index here is the number `pdfcer object-list` prints as
///   `index=` and `object-move` / `object-delete` / `node-move` take as an
///   operand, and it is the number every `EditSession` paint-order verb
///   resolves against the page's buffer.
/// * [`PageObjects::leaves`] — what a *form XObject invoked by the page*
///   paints, geometry already mapped into page space by
///   [`pdfcer_core::vector::decompose_page`]. A leaf's token range indexes
///   **the form's** buffer.
///
/// The engine keeps those two lists apart deliberately, and its own reason is
/// the one that governs here (`FormLeaf`'s header): *"eleven call sites in
/// `edit.rs` resolve a paint-order index and apply surgery to the page's
/// content stream. Put leaves in `PageObjects::objects` and every one of
/// those verbs would happily apply a form-relative token range to the page and
/// corrupt it — silently, because the range is in bounds."*
///
/// **In range and wrong is the dangerous combination**, and it is exactly the
/// combination a single `u64` would produce here. So this type carries the
/// list with the index, and the only way to obtain a number an edit verb will
/// accept is [`TargetId::page_object_index`], which answers `None` for a leaf.
/// A site that wants to edit therefore has to say what it does about a leaf,
/// at the point where it can still say something useful to the operator,
/// rather than silently addressing the wrong buffer.
///
/// # Why not two types
///
/// Because one *selection* holds both, and the whole point of the form work is
/// that an object inside a form is a first-class selection stop. A selection
/// set generic over which kind it holds would push the distinction into every
/// container in `canvas/`; an enum keeps it at the leaves of the call graph,
/// where the decision actually differs.
///
/// # `Ord`, and what its order means
///
/// Derived, so every `Object` sorts before every `Leaf`. That is an arbitrary
/// but stable total order, which is all the selection set needs it for
/// (de-duplication and a non-flickering outline paint order). It is **not** a
/// paint order and nothing may read it as one — leaves and page objects
/// interleave on [`pdfcer_core::vector::FormLeaf::paint_order`], and the one
/// place that ordering matters is the hit test, which the engine performs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TargetId {
    /// An index into [`PageObjects::objects`] — the page's own paint order.
    /// Editable by the paint-order verbs.
    Object(u64),
    /// An index into [`PageObjects::leaves`] — an object painted from inside
    /// a form XObject.
    ///
    /// Selectable, measurable and reportable. **Not** editable by the
    /// paint-order verbs: see the type docs, and
    /// [`pdfcer_core::vector::FormLeaf::is_editable`], which is `false` for
    /// every leaf until the engine grows editing-through-recursion.
    Leaf(u64),
}

impl TargetId {
    /// The index an [`pdfcer_core::edit::EditSession`] paint-order verb will
    /// accept — `None` for a leaf.
    ///
    /// ★ **This is the only supported way to turn a `TargetId` into an edit
    /// operand**, and its `None` is the guard that makes the two index spaces
    /// impossible to confuse. Do not pattern-match `Object(i)` and cast at a
    /// call site that is about to edit: the match compiles just as well when
    /// somebody later adds a third variant, and this method does not.
    #[must_use]
    pub fn page_object_index(self) -> Option<usize> {
        match self {
            Self::Object(i) => usize::try_from(i).ok(),
            Self::Leaf(_) => None,
        }
    }

    /// The index into [`PageObjects::leaves`], or `None` for a page object.
    #[must_use]
    pub fn leaf_index(self) -> Option<usize> {
        match self {
            Self::Object(_) => None,
            Self::Leaf(i) => usize::try_from(i).ok(),
        }
    }

    /// Whether this target lives inside a form XObject.
    #[must_use]
    pub const fn is_leaf(self) -> bool {
        matches!(self, Self::Leaf(_))
    }

    /// The raw number, **for a trace line or a label and nothing else**.
    ///
    /// Deliberately loses which list it came from, so it is useless as an edit
    /// operand and cannot be mistaken for one — every caller of this method is
    /// building a string. Pair it with [`Self::is_leaf`] when the string is
    /// shown to the operator, because "object 7" and "leaf 7" are different
    /// things and a trace that says only `7` is a trace that cannot be read.
    #[must_use]
    pub const fn raw(self) -> u64 {
        match self {
            Self::Object(i) | Self::Leaf(i) => i,
        }
    }
}

/// The fallback canvas-space slack a click may miss an object's edge by,
/// used ONLY when the caller cannot supply a live zoom (a non-finite or
/// non-positive zoom makes a screen-to-page tolerance conversion return
/// `0.0`, which would make selection impossible rather than merely fussy).
///
/// Canvas space is the page's device space at zoom 1.0, where one unit is
/// one PDF point (the `page_device_geometry` scale-1.0 map is
/// distance-preserving — a pure rotation + Y-flip + translation), so this is
/// also, in effect, a ~3 pt page-space tolerance.
///
/// **This used to be the only tolerance**, applied at every zoom level, and
/// that was a bug: the pointer is divided by `zoom` before it reaches
/// [`ObjectModelProvider::hit_test`], so a constant canvas-space tolerance
/// is a *shrinking* on-screen catch radius — 1.5 px at 50% zoom, 0.75 px at
/// 25%. Objects were effectively unclickable whenever the operator zoomed
/// out to see a whole drawing. The live tolerance arrives as a parameter,
/// derived at the call site from a screen-pixel constant divided by the
/// zoom.
pub const FALLBACK_SELECT_TOLERANCE: f64 = 3.0;

/// The object-model-backed provider for one page (module docs).
pub struct ObjectModelProvider {
    /// The page this provider answers for; queries for any other index miss.
    page_index: usize,
    /// The decomposed objects, in PDF user space (paint order).
    objects: PageObjects,
    /// PDF user space → canvas space (the render device map at scale 1.0).
    to_canvas: Transform,
    /// Canvas space → PDF user space (the inverse), or `None` for a
    /// degenerate (non-invertible) page — then the provider declines every
    /// query rather than fabricate geometry.
    to_pdf: Option<Transform>,
    /// **The page's own extent in canvas units**, or `None` when this
    /// provider was built from parts and nobody supplied one.
    ///
    /// ★★★ Held for exactly one question:
    /// [`crate::canvas::target::CanvasTargetProvider::container_is_worth_selecting`],
    /// which needs to know whether a form covers the whole sheet. It is
    /// `page_device_geometry(page, 1.0)`'s first two returns, which were
    /// discarded here until 2026-09-01 — the transform was wanted and the
    /// size was not.
    ///
    /// ★ `None` makes that predicate answer `true`, which is the behaviour
    /// before it existed. A provider that cannot measure must not guess.
    page_extent_px: Option<egui::Vec2>,
}

/// Which KIND of part the "Part" rung is standing on for a given object.
///
/// The rung is shared between path SUBPATHS and text RUNS, and almost
/// everything about it is identical — nearest-first hit order, an outline to
/// draw, Escape to ascend, Delete to remove. What differs is the **verb
/// set**, and that is exactly what this tells a caller:
///
/// | | `Subpath` | `Run` |
/// |---|---|---|
/// | Delete | `delete_subpath` | `delete_text_run` |
/// | Drag to move | `move_subpath` | **nothing — no core verb exists** |
/// | Descend to Point | yes | no (a run has no anchors) |
///
/// The Point-rung row needs no guard anywhere:
/// [`ObjectModelProvider::nearest_node`] reaches
/// [`ObjectModelProvider::subpath_node_points`], which matches
/// `VectorObject::Path` only, so a text entry can never produce a node hit.
/// The ladder caps itself at two rungs for text by construction rather than
/// by a check — which is also why the Objects panel's tree can nest a text
/// object one level and a path object two, with no special case in the row
/// builder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartKind {
    /// A subpath of a path object.
    Subpath,
    /// A show operator ("run") of a text object.
    Run,
}

impl ObjectModelProvider {
    /// Build a provider for `page` (at `page_index`) from `view`.
    ///
    /// Returns `None` only if the page's content cannot be decoded/tokenized
    /// (the same failure the renderer would hit). A caller then says so in
    /// words rather than showing an empty list — a failure state must never
    /// be visually indistinguishable from a success state that happens to
    /// have no content.
    ///
    /// # Pass a SESSION view, not the base document (decision 018)
    ///
    /// Callers pass `&session.view()`. Passing `&session.document().view()`
    /// decomposes the *base revision*, so hit-testing, marquee selection and
    /// the measure tools' snapping all address geometry the operator can no
    /// longer see and miss geometry they can. The raster and this provider
    /// must be built from the *same* view, or the canvas shows one document
    /// and responds as another.
    ///
    /// At S3 the second half of that hazard is what bites: the Objects panel
    /// would list the pre-edit object set while the canvas draws the
    /// post-edit page, and the panel exists precisely to answer "what am I
    /// looking at".
    #[must_use]
    pub fn build(view: &DocumentView<'_>, page: &Page, page_index: usize) -> Option<Self> {
        Self::build_or_reason(view, page, page_index).ok()
    }

    /// [`Self::build`], keeping the reason the page would not decompose.
    ///
    /// # Why the reason is worth a second constructor
    ///
    /// [`Self::build`] throws the `ContentError` away, which is right for a
    /// *panel*: an operator is told the page's content could not be read, in
    /// the catalog's words, and a tokenizer's error text is not a sentence
    /// anybody outside this project can act on.
    ///
    /// It is wrong for the **diagnostic channel**.
    /// `crate::app::state::OpenDoc::trace_object_count` emits
    /// `objects-unavailable page=… reason=decompose-failed detail=…`, and the
    /// `detail=` is the whole value of the line: without it a harness learns
    /// that a page did not decompose and nothing about why, which is a
    /// question it then has to answer by hand.
    ///
    /// Before the decomposition cache moved onto `OpenDoc`, the trace kept
    /// that detail by running **its own** `decompose_page` — a second
    /// decomposition of the same page, which is precisely the *"two
    /// decompositions quietly diverge"* pattern decision 011 warns about and
    /// [`Self::page_objects`]' own docs exist to prevent. This constructor is
    /// what let that second call be deleted: one decomposition, and the
    /// failure reason survives it.
    ///
    /// The error is stringified here rather than propagated as a
    /// `ContentError` so the cache that stores it does not have to name a
    /// `pdfcer-core` error type in its own signature — the only consumer wants
    /// a line of trace text, and `ContentError` is `#[non_exhaustive]`.
    ///
    /// # Errors
    ///
    /// The page's `/Contents` could not be resolved, inflated or tokenized —
    /// the same failure the renderer would hit on the same page.
    pub fn build_or_reason(
        view: &DocumentView<'_>,
        page: &Page,
        page_index: usize,
    ) -> Result<Self, String> {
        // ★★★ **TIMED, and the line is the instrument this shell owes the
        // engine** — 2026-08-31.
        //
        // `pdfcer-core`'s reply to
        // `request_one_edit_costs_two_decompositions_of_the_same_page` measured
        // its own side and asked for ours by name: *"we have not measured your
        // loop; measure it rather than take that sentence."* It also corrected
        // a causal reading this project got wrong — the decode is roughly three
        // quarters of the cost and the decomposition the remaining quarter,
        // where this project had assumed one number was the other.
        //
        // ⇒ So the line carries **what was built** as well as how long: a
        // rebuild that produced no leaves is a different event from a slow one,
        // and between `a24868e` and `a8586cc` the engine's own memo returned
        // exactly that — a full object list with the deep-selection model
        // silently missing.
        let started = std::time::Instant::now();
        let objects =
            decompose_page(view, page, Matrix::IDENTITY).map_err(|err| err.to_string())?;
        let elapsed = started.elapsed();
        let (count, leaves) = (objects.objects.len(), objects.leaves.len());
        crate::diag::trace(move || {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!(
                "page-objects-built page={page_index} objects={count} leaves={leaves} ms={}",
                elapsed.as_millis()
            )
        });
        let (w, h, to_canvas) = page_device_geometry(page, 1.0);
        Ok(Self {
            page_index,
            objects,
            to_canvas,
            to_pdf: to_canvas.invert(),
            // ★ At scale 1.0 these ARE the page's canvas-space extent, which is
            // the space `bounds` answers in. Taken here rather than re-derived
            // from the crop box, so the geometry has one source.
            #[allow(
                clippy::cast_precision_loss,
                reason = "a page dimension in pixels is far below f32's exact-integer range" // ui-text-exempt: a lint justification
            )]
            page_extent_px: Some(egui::vec2(w as f32, h as f32)),
        })
    }

    /// Construct directly from parts — the seam the headless unit tests use
    /// (a [`PageObjects`] plus an explicit canvas↔PDF transform), so the
    /// adapter logic is proven without a live `Document` or an egui frame.
    #[cfg(test)]
    pub(crate) fn from_parts(
        page_index: usize,
        objects: PageObjects,
        to_canvas: Transform,
    ) -> Self {
        Self {
            page_index,
            objects,
            to_canvas,
            to_pdf: to_canvas.invert(),
            // ★ Headless tests construct from parts and have no page. `None`
            // makes `container_is_worth_selecting` answer `true`, which is what
            // those tests were written against — a unit test must not start
            // depending on a geometric judgement it never supplied the geometry
            // for.
            page_extent_px: None,
        }
    }

    /// The page's extent in canvas units, when this provider knows it.
    ///
    /// See the field for why it exists and why `None` is a real answer rather
    /// than a missing one.
    #[must_use]
    pub fn page_extent(&self) -> Option<egui::Vec2> {
        self.page_extent_px
    }

    /// Which page this provider answers for.
    ///
    /// Read by the caller that decides whether to rebuild after a page step.
    /// Exposed rather than re-derived because "is my provider still about
    /// the page I am looking at?" must have exactly one answer.
    #[must_use]
    pub fn page_index(&self) -> usize {
        self.page_index
    }

    /// The current page's decomposed vector objects.
    ///
    /// The **shared escape hatch** every consumer reads the already-
    /// decomposed objects through — the Objects panel's row list today, the
    /// snap engine and the Taubin best-fit circle later — so each reuses the
    /// ONE decomposition this provider built rather than running a second
    /// `decompose_page` per frame. That avoids the exact "two decompositions
    /// quietly diverge" pattern decision 011 warns against.
    ///
    /// Everything in [`PageObjects`] is in **PDF user / page space** — the
    /// frame the model stores — so a caller with a canvas-space point
    /// converts it first, either with [`Self::canvas_to_pdf`]'s public
    /// sibling [`crate::viewer::canvas_to_pdf_space`] or through this
    /// provider's own queries.
    #[must_use]
    pub fn page_objects(&self) -> &PageObjects {
        &self.objects
    }

    /// Which subpath of `object` a canvas-space click lands on — the second
    /// selection level, for objects that hold a whole drawing.
    ///
    /// A thin adapter over [`pdfcer_core::vector::hit_test_subpaths`], exactly
    /// like [`Self::hit_test_all`] is over the per-object query: convert
    /// canvas space to PDF user space, apply the same degenerate-tolerance
    /// fallback, and let the core own the geometry. Sharing that fallback
    /// matters — without it a click could select an object and then find none
    /// of its subpaths, which reads as "the second level is broken" rather
    /// than "the tolerance was zero".
    ///
    /// Nearest first. Empty for a non-path object or an out-of-range index.
    #[must_use]
    pub fn subpath_hits(&self, object: usize, point: Pos2, tolerance: f64) -> Vec<usize> {
        let Some(pdf) = self.canvas_to_pdf(point) else {
            return Vec::new();
        };
        pdfcer_core::vector::hit_test_subpaths(&self.objects, object, pdf, resolve(tolerance))
    }

    /// What kind of part the object at `index` is decomposed into, or `None`
    /// for an object with no Part rung at all (an image).
    #[must_use]
    pub fn part_kind(&self, index: usize) -> Option<PartKind> {
        match self.objects.objects.get(index) {
            Some(VectorObject::Path(_)) => Some(PartKind::Subpath),
            Some(VectorObject::Text(_)) => Some(PartKind::Run),
            _ => None,
        }
    }

    /// Which part of `object` a canvas-space click lands on — **whichever
    /// kind of part that object has**.
    ///
    /// ONE dispatcher rather than a kind match at each call site. The
    /// alternative is duplicated-predicate drift: two places deciding "which
    /// part is under the pointer" go out of step invisibly, and the operator
    /// finds that descending works for a drawing and not for a label.
    #[must_use]
    pub fn part_hits(&self, object: usize, point: Pos2, tolerance: f64) -> Vec<usize> {
        match self.part_kind(object) {
            Some(PartKind::Subpath) => self.subpath_hits(object, point, tolerance),
            Some(PartKind::Run) => self.text_run_hits(object, point, tolerance),
            None => Vec::new(),
        }
    }

    /// A part's bounds in **canvas** space, for drawing its outline —
    /// whichever kind of part it is. The dispatcher for
    /// [`Self::subpath_bounds_canvas`] / [`Self::text_run_bounds_canvas`],
    /// for the same anti-drift reason as [`Self::part_hits`].
    #[must_use]
    pub fn part_bounds_canvas(&self, object: usize, part: usize) -> Option<Rect> {
        match self.part_kind(object) {
            Some(PartKind::Subpath) => self.subpath_bounds_canvas(object, part),
            Some(PartKind::Run) => self.text_run_bounds_canvas(object, part),
            None => None,
        }
    }

    /// How many parts the object at `index` has, whichever kind they are.
    ///
    /// This is what the Objects panel's tree counts to decide whether a row
    /// gets an expander, and how many child rows it contributes when open.
    #[must_use]
    pub fn part_count(&self, index: usize) -> usize {
        match self.part_kind(index) {
            Some(PartKind::Subpath) => self.subpath_count(index),
            Some(PartKind::Run) => self.text_run_count(index),
            None => 0,
        }
    }

    /// Which **run** (show operator) of the text object at `object` a
    /// canvas-space click lands on — the text-side twin of
    /// [`Self::subpath_hits`].
    ///
    /// A thin adapter over [`pdfcer_core::vector::hit_test_text_runs`], with
    /// the same canvas→PDF conversion and the same degenerate-tolerance
    /// fallback its sibling uses. Sharing that fallback matters for the same
    /// reason: without it a click could select a text object and then find
    /// none of its runs, which reads as "the second level is broken" rather
    /// than "the tolerance was zero".
    ///
    /// Nearest first. **Empty for a non-text object, an out-of-range index,
    /// or a text object whose runs could not be laid out** — the core query
    /// deliberately does not fall back to the object's enclosing box there,
    /// because naming run 0 for an object whose runs were never measured
    /// would hand a caller a deletable target that is the wrong one.
    #[must_use]
    pub fn text_run_hits(&self, object: usize, point: Pos2, tolerance: f64) -> Vec<usize> {
        let Some(pdf) = self.canvas_to_pdf(point) else {
            return Vec::new();
        };
        pdfcer_core::vector::hit_test_text_runs(&self.objects, object, pdf, resolve(tolerance))
    }

    /// How many runs the text object at `object` has, or `0` for anything
    /// else — the text twin of [`Self::subpath_count`], and `0` for the same
    /// reason it is: a path has no runs, and a loop over none of them is
    /// exactly the right amount of work.
    #[must_use]
    pub fn text_run_count(&self, object: usize) -> usize {
        match self.objects.objects.get(object) {
            Some(VectorObject::Text(t)) => t.runs.len(),
            _ => 0,
        }
    }

    /// A text run's bounds in **canvas** space, for drawing its outline.
    ///
    /// Same argument as [`Self::subpath_bounds_canvas`]: the object's own
    /// bounds would draw a rectangle around every label on the sheet and
    /// tell the operator they had selected the whole thing again — which is
    /// the misunderstanding entering the object exists to resolve. On the
    /// measured CAD export that rectangle spans the entire drawing.
    #[must_use]
    pub fn text_run_bounds_canvas(&self, object: usize, run: usize) -> Option<Rect> {
        let Some(VectorObject::Text(t)) = self.objects.objects.get(object) else {
            return None;
        };
        self.pdf_bounds_to_canvas(t.runs.get(run)?.bounds)
    }

    /// Whether deleting run `run` of text object `object` would be refused
    /// because the run AFTER it has no position of its own (§9.4.2).
    ///
    /// A pure query the shell asks **before** offering the control (R83),
    /// answered from the same `positioned_by` flag
    /// [`pdfcer_core::edit::EditSession::delete_text_run`] refuses on — so a
    /// disabled affordance and the verb cannot disagree about which runs are
    /// deletable.
    ///
    /// `false` for a non-text object or an out-of-range index: there is no
    /// deletion to refuse.
    #[must_use]
    pub fn text_run_delete_would_move_next(&self, object: usize, run: usize) -> bool {
        let Some(VectorObject::Text(t)) = self.objects.objects.get(object) else {
            return false;
        };
        // The LAST run is never refused — nothing follows it to be moved.
        // And a single-run object deletes the whole text object, which the
        // core verb allows unconditionally.
        t.runs.len() > 1
            && t.runs.get(run + 1).is_some_and(|next| {
                next.positioned_by == pdfcer_core::vector::RunPositioning::Inherited
            })
    }

    /// A subpath's bounds in **canvas** space, for drawing its outline.
    ///
    /// The object's own bounds would draw a rectangle around the entire
    /// drawing and tell the operator they had selected the whole thing again
    /// — which is the misunderstanding entering the object exists to resolve.
    #[must_use]
    pub fn subpath_bounds_canvas(&self, object: usize, subpath: usize) -> Option<Rect> {
        let b = pdfcer_core::vector::subpath_bounds(&self.objects, object, subpath)?;
        self.pdf_bounds_to_canvas(b)
    }

    /// The page-space anchor sample points of the object at paint-order
    /// `index` — the circular best-fit tool's fit input.
    ///
    /// A path object contributes every anchor of every subpath, in **PDF
    /// user / page space** (the frame [`Self::page_objects`] stores and
    /// [`fit_circle_taubin`](pdfcer_core::dimension::fit_circle_taubin)
    /// consumes); a text/image/form object (or an out-of-range index)
    /// contributes nothing — they carry no snap/fit node geometry, the same
    /// exclusion the snap engine applies. Reuses the ONE decomposition this
    /// provider already built, never a second `decompose_page`.
    #[must_use]
    pub fn object_sample_points(&self, index: usize) -> Vec<Point> {
        match self.objects.objects.get(index) {
            Some(VectorObject::Path(path)) => path
                .page_subpaths()
                .iter()
                .flat_map(|sp| sp.anchors().collect::<Vec<_>>())
                .collect(),
            _ => Vec::new(),
        }
    }

    /// How many parts (subpaths) the path object at paint-order `index` has,
    /// or `0` for a non-path object.
    ///
    /// Exists so a caller can iterate an object's parts without reaching
    /// into `objects.objects` and re-doing the `VectorObject::Path` match at
    /// a call site whose job is drawing rows. `0` for a non-path is the
    /// honest answer rather than an `Option`: a text run has no subpaths, and
    /// a loop over none of them is exactly the right amount of work.
    #[must_use]
    pub fn subpath_count(&self, index: usize) -> usize {
        match self.objects.objects.get(index) {
            Some(VectorObject::Path(path)) => path.page_subpaths().len(),
            _ => 0,
        }
    }

    /// The anchors of ONE subpath, each paired with its **object-scoped**
    /// index — the Point rung's pick set (decision 028 §Q1).
    ///
    /// # Why not [`Self::object_sample_points`], which already returns anchors
    ///
    /// That one returns the whole object's flat list, and using it as a node
    /// pick set is a hazard decision 028 found already shipped: on a measured
    /// CAD export one path object holds **6,681 anchors**, so "the nearest
    /// anchor to the press" can easily belong to a subpath the operator is
    /// not pointing at, and nothing is drawn beforehand to say which. Scoping
    /// the pick set to the ENTERED subpath is what makes the grab predictable
    /// — the operator can only hit points they descended into and can see.
    ///
    /// The same number is why the Objects panel nests points under a *part*
    /// rather than listing an object's anchors directly: 6,681 sibling rows
    /// under one object is not a tree, it is a wall.
    ///
    /// # Why the index is object-scoped even though the set is subpath-scoped
    ///
    /// Decision 025 §1.3(b): the number pdfcer shows and the number
    /// `pdfcer node-move --node N` addresses must be the same number.
    /// `vector::anchor_count` counts across the whole object, so the running
    /// offset is added here rather than letting the GUI invent a second
    /// numbering that would disagree with every other consumer.
    ///
    /// Returns empty for a non-path object or an out-of-range index — the
    /// same exclusion [`Self::object_sample_points`] applies, for the same
    /// reason (text and image objects are not node-editable, decision 011
    /// §2.1).
    #[must_use]
    /// The **Bézier handles** of one anchor of one subpath, in PDF user space.
    ///
    /// Returns at most two: the control point governing the curve as it
    /// *arrives* at the anchor and the one governing it as it *leaves*. Either
    /// or both are absent when the neighbouring segment is a straight line or
    /// there is no neighbouring segment at all — which is the ordinary case on
    /// a CAD drawing, where almost every path is polygonal.
    ///
    /// # ★ Why this is per-ANCHOR and every other point accessor is per-subpath
    ///
    /// Because handles are only ever drawn for the anchors the operator has
    /// selected, and that is not a cosmetic decision. A subpath's anchors are
    /// its skeleton and are worth showing all at once; its handles are two per
    /// anchor and are *inside* the shape, so drawing every one turns a curve
    /// into a thicket and hides the outline the operator is working on. Every
    /// vector editor draws them for the selection alone, and this accessor's
    /// shape is what makes that the cheap path rather than a filter over a
    /// list that was expensive to build.
    ///
    /// # ★★ How an anchor index maps onto segments, and the off-by-one in it
    ///
    /// `Subpath` is `start` plus a list of `segments`, and `anchors()` yields
    /// `start` first and then each segment's end. So for object-scoped anchor
    /// `k` **within this subpath** (0-based):
    ///
    /// - its **incoming** handle is `segments[k - 1].c2` — the second control
    ///   point of the segment that ends *here*. Absent for `k == 0`, which has
    ///   no segment before it.
    /// - its **outgoing** handle is `segments[k].c1` — the first control point
    ///   of the segment that starts *here*. Absent for the last anchor.
    ///
    /// Getting that backwards produces handles that are drawn on the wrong side
    /// of the anchor and drag the wrong curve, which looks like a coordinate
    /// bug rather than an indexing one. It is stated here because
    /// `pdfcer_core::vector::Handle`'s own doc comment states it, and the two
    /// must agree: the `Handle` value returned here is passed straight to
    /// `EditSession::move_handle`.
    ///
    /// A **closed** subpath's first anchor also has an incoming handle — from
    /// the closing segment — and that is deliberately NOT returned. The closing
    /// segment of an `h`-terminated subpath has no operands of its own in the
    /// content stream, so there is nothing for `move_handle` to rewrite, and
    /// offering a handle the engine will refuse is the "visible control,
    /// silently inert" failure this project keeps finding.
    pub fn node_handles(
        &self,
        index: usize,
        subpath: usize,
        node: usize,
    ) -> Vec<(pdfcer_core::vector::Handle, Point)> {
        use pdfcer_core::vector::{Handle, Segment};

        let Some(VectorObject::Path(path)) = self.objects.objects.get(index) else {
            return Vec::new();
        };
        let subpaths = path.page_subpaths();
        // The object-scoped anchor index has to be brought back into the
        // subpath's own space, using the SAME running offset
        // `subpath_node_points` computes — see its comment for why the offset
        // is the object-scoped index of the subpath's first anchor.
        let mut offset = 0usize;
        for (i, sp) in subpaths.iter().enumerate() {
            let count = sp.anchors().count();
            if i == subpath {
                let Some(local) = node.checked_sub(offset).filter(|k| *k < count) else {
                    // The anchor is not in this subpath. A selection that
                    // out-ran a decomposition, refused rather than guessed at —
                    // the same posture `canvas::moving`'s `NodeNotFound` takes.
                    return Vec::new();
                };
                let mut out = Vec::with_capacity(2);
                // Incoming: the second control point of the segment BEFORE it.
                if let Some(Segment::Cubic { c2, .. }) =
                    local.checked_sub(1).and_then(|j| sp.segments.get(j))
                {
                    out.push((Handle::Incoming, *c2));
                }
                // Outgoing: the first control point of the segment AFTER it.
                if let Some(Segment::Cubic { c1, .. }) = sp.segments.get(local) {
                    out.push((Handle::Outgoing, *c1));
                }
                return out;
            }
            offset += count;
        }
        Vec::new()
    }

    pub fn subpath_node_points(&self, index: usize, subpath: usize) -> Vec<(usize, Point)> {
        let Some(VectorObject::Path(path)) = self.objects.objects.get(index) else {
            return Vec::new();
        };
        let subpaths = path.page_subpaths();
        // The running offset IS the object-scoped index of the target
        // subpath's first anchor, because `anchor_count` flattens the same
        // walk in the same order.
        let mut offset = 0usize;
        for (i, sp) in subpaths.iter().enumerate() {
            let anchors: Vec<Point> = sp.anchors().collect();
            if i == subpath {
                return anchors
                    .into_iter()
                    .enumerate()
                    .map(|(k, p)| (offset + k, p))
                    .collect();
            }
            offset += anchors.len();
        }
        Vec::new()
    }

    /// **Every** anchor of the path object at paint-order `index`, each with
    /// its object-scoped index — [`Self::subpath_node_points`] flattened
    /// across all subpaths.
    ///
    /// # Why the whole object and not one subpath
    ///
    /// A multi-node **selection** is object-scoped: nothing stops an operator
    /// Ctrl-clicking one anchor on a shape's outer subpath and another on a
    /// hole inside it, and a selection set holds both by their object-scoped
    /// index. A multi-node **drag** therefore has to look up positions across
    /// the whole object — asking per-subpath would mean the caller
    /// re-deriving which subpath each selected index falls in, which is
    /// exactly the offset arithmetic [`Self::subpath_node_points`] exists to
    /// keep in one place.
    ///
    /// Empty for a non-path object, for the same reason
    /// [`Self::subpath_count`] returns `0`.
    #[must_use]
    pub fn object_node_points(&self, index: usize) -> Vec<(usize, Point)> {
        let Some(VectorObject::Path(path)) = self.objects.objects.get(index) else {
            return Vec::new();
        };
        path.page_subpaths()
            .iter()
            .flat_map(|sp| sp.anchors())
            .enumerate()
            .collect()
    }

    /// The Bézier control points ("handles") of one subpath, each tagged with
    /// the **object-scoped index of the node it belongs to** and which side
    /// of that node it shapes.
    ///
    /// # Which handle belongs to which node
    ///
    /// A cubic segment carries two control points, and they belong to
    /// *different* nodes — this is the part that is easy to get backwards.
    /// Segment `k` runs from anchor `k` to anchor `k+1`, so its `c1` shapes
    /// the curve LEAVING anchor `k` and its `c2` shapes the curve ARRIVING
    /// at anchor `k+1`. That is exactly the split
    /// [`pdfcer_core::vector::Handle`] names, and it is why the enum is worded
    /// by direction of travel rather than "first/second": first-and-second
    /// are properties of a *segment*, and a segment says nothing about which
    /// node the operator selected.
    ///
    /// Straight segments contribute nothing. pdfcer refuses to invent a handle
    /// for a line — turning a line into a curve is a different operation with
    /// a different name — so a node with no curve on a side simply has no
    /// mark there, and the absence is stated in the readout rather than drawn
    /// as a ghost (decision 028 §Q2).
    ///
    /// `v`/`y` implicit control points need no special handling here: the
    /// decomposition already resolves them into explicit `c1`/`c2`
    /// (`Segment::Cubic`'s own doc comment), so this sees one uniform shape
    /// and the promotion-to-`c` happens far downstream in the planner.
    #[must_use]
    pub fn subpath_handle_points(
        &self,
        object: usize,
        subpath: usize,
    ) -> Vec<(usize, Handle, Point)> {
        let Some(VectorObject::Path(path)) = self.objects.objects.get(object) else {
            return Vec::new();
        };
        let subpaths = path.page_subpaths();
        let mut offset = 0usize;
        for (i, sp) in subpaths.iter().enumerate() {
            let anchors = sp.anchors().count();
            if i != subpath {
                offset += anchors;
                continue;
            }
            let mut out = Vec::new();
            for (k, seg) in sp.segments.iter().enumerate() {
                if let Segment::Cubic { c1, c2, .. } = *seg {
                    // `c1` shapes the curve leaving anchor k …
                    out.push((offset + k, Handle::Outgoing, c1));
                    // … and `c2` shapes the curve arriving at anchor k+1.
                    out.push((offset + k + 1, Handle::Incoming, c2));
                }
            }
            return out;
        }
        Vec::new()
    }

    /// The handle of `subpath` nearest `point` within `tolerance`, as
    /// `(node index, side)` — the Point rung's handle pick.
    ///
    /// # Why handles are hit-tested BEFORE nodes
    ///
    /// A handle sits close to its own node exactly when the curve is nearly
    /// flat there. If the node won ties, the handle would be unreachable
    /// precisely in the case where the operator most wants it — to pull a
    /// flat segment into a curve. Checking the smaller target first is the
    /// standard resolution and the one decision 028 §Q3 specifies.
    ///
    /// `point` is in **PDF page space**, unlike [`Self::nearest_node`]'s
    /// canvas-space input: the only caller is the drag classifier, which has
    /// already converted the press origin to page space to compute the drag's
    /// reference point. Converting back to canvas just to convert forward
    /// again would be two chances to disagree with itself for no benefit.
    #[must_use]
    pub fn nearest_handle(
        &self,
        object: usize,
        subpath: usize,
        pdf: Point,
        tolerance: f64,
    ) -> Option<(usize, Handle)> {
        let mut best: Option<((usize, Handle), f64)> = None;
        for (index, side, p) in self.subpath_handle_points(object, subpath) {
            if !p.is_finite() {
                continue;
            }
            let d = p.distance(pdf);
            if d <= tolerance && best.is_none_or(|(_, bd)| d < bd) {
                best = Some(((index, side), d));
            }
        }
        best.map(|(hit, _)| hit)
    }

    /// The object-scoped index of the anchor of `subpath` nearest `point`
    /// within `tolerance`, or `None` — the Point rung's pick.
    ///
    /// Takes canvas space and converts internally, exactly as
    /// [`Self::subpath_hits`] does, so the canvas→PDF frame conversion stays
    /// in the one place that owns it rather than being re-derived by each
    /// caller. `tolerance` is in PDF units, already converted from screen
    /// pixels by the caller.
    ///
    /// **Ties resolve to the lower index**, which is the same rule the vector
    /// edit tool's own nearest-anchor search uses — so a point equidistant
    /// from two anchors picks the same one whether it was reached by clicking
    /// or by dragging. (That tool lands at S5; the rule is stated here rather
    /// than cross-referenced so it survives being read alone.)
    #[must_use]
    pub fn nearest_node(
        &self,
        object: usize,
        subpath: usize,
        point: Pos2,
        tolerance: f64,
    ) -> Option<usize> {
        let pdf = self.canvas_to_pdf(point)?;
        let mut best: Option<(usize, f64)> = None;
        for (index, p) in self.subpath_node_points(object, subpath) {
            if !p.is_finite() {
                continue;
            }
            let d = p.distance(pdf);
            if d <= tolerance && best.is_none_or(|(_, bd)| d < bd) {
                best = Some((index, d));
            }
        }
        best.map(|(index, _)| index)
    }

    // -----------------------------------------------------------------
    // The canvas target-provider surface.
    //
    // These four were `impl CanvasTargetProvider for ObjectModelProvider`
    // in the old shell. The trait lives in `canvas/` and lands at S4; the
    // methods are inherent here in the meantime, with their signatures and
    // semantics unchanged, so re-attaching the trait is an `impl` block
    // and nothing else.
    // -----------------------------------------------------------------

    /// Every target under the pointer, **front-most first**, *including what
    /// is painted inside form XObjects*.
    ///
    /// A thin adapter, as the module docs promise: convert canvas space to
    /// PDF user space, resolve the tolerance, and hand both to
    /// [`pdfcer_core::vector::hit_test_point_deep`], which owns the geometry
    /// and the ordering.
    ///
    /// The list is what click-through cycling steps through. Without it, an
    /// object completely covered by another is unselectable by any click.
    ///
    /// # THE DEEP QUERY, AND WHY A FORM IS NOT IN THE ANSWER
    ///
    /// The operator, 2026-08-26: *"when I click on one of the objects all I
    /// get is the page selected."*
    ///
    /// He was clicking a real object. It was inside a form XObject, and this
    /// method used to call `pdfcer_core::vector::hit_test_point_all`, which
    /// sees a form as **one opaque object bounded by its `/BBox`**. A form
    /// declaring the whole `MediaBox` and drawing one small line is legal and
    /// common - 8.10.1 makes `/BBox` a *clipping* extent, a statement about
    /// where painting is allowed, not about where ink is. So a page-sized form
    /// sat in paint order above everything drawn before it and won every click
    /// at every point, and the outline it produced hugged the page edge, which
    /// looks exactly like a state this program does not have.
    ///
    /// `hit_test_point_deep` answers with what is **inside** the form and
    /// **excludes the form itself outright**. Measured on the fixtures this
    /// project uses:
    ///
    /// | page | page objects | forms | leaves |
    /// |---|---:|---:|---:|
    /// | the industry print-conformance suite's composite page 1 | 28 | 4 | **242** |
    /// | `ncored-benchmark-cad-drawing` p1 | 129,758 | 1 | **10,256** |
    /// | `SW41177` p1 | 5,903 | 0 | 0 |
    ///
    /// On the first two, almost everything the operator can see was outside
    /// the object model this method reported. On the third nothing changes at
    /// all, which is the shape of the fix: it costs nothing on a page with no
    /// forms.
    ///
    /// # There is deliberately NO fallback to the shallow query
    ///
    /// It is tempting to fall back to `hit_test_point_all` when the deep query
    /// comes back empty, so a click on a form with no reachable interior still
    /// selects *something*. **That reinstates the defect.** The commonest empty
    /// answer by far is a click on blank paper *inside* a page-sized form -
    /// and a fallback would answer it with the form, which is the operator's
    /// original complaint, verbatim, restored for the case that produces it
    /// most often.
    ///
    /// A form is still reachable, by two deliberate acts rather than by
    /// default: the canvas context menu's *"select the containing form"* on
    /// any object inside it (see [`Self::containing_form`]), and its row in
    /// the Objects panel, which lists `PageObjects::objects` and therefore
    /// lists every form. Reachable-on-purpose is the whole point; winning by
    /// default is what was wrong.
    ///
    /// # What this costs
    ///
    /// One extra scan of `PageObjects::leaves` per query, and a sort of the
    /// hits. The engine bounds the candidate list, and in practice a point is
    /// under one to three things. `canvas::clicking` asks twice per selecting
    /// click - once for the pick and once for the *"1 of 5 here"* count - and
    /// that was already true.
    #[must_use]
    pub fn hit_test_all(&self, page_index: usize, point: Pos2, tolerance: f64) -> Vec<TargetId> {
        if page_index != self.page_index {
            return Vec::new();
        }
        let Some(pdf) = self.canvas_to_pdf(point) else {
            return Vec::new();
        };
        hit_test_point_deep(&self.objects, pdf, resolve(tolerance))
            .into_iter()
            .map(|hit| match hit {
                HitTarget::Object(i) => TargetId::Object(i as u64),
                HitTarget::Leaf(i) => TargetId::Leaf(i as u64),
            })
            .collect()
    }

    /// The page paint-order index of the **outermost form** enclosing a
    /// form-interior target - the *"select the container"* act.
    ///
    /// # Why the container has to be offered somewhere
    ///
    /// Because [`Self::hit_test_all`] no longer answers with a form, ever, and
    /// a form is a perfectly legitimate thing to want: it is one page object,
    /// it has a paint-order index, and `object-move` / `object-delete` address
    /// it exactly like any other. Moving a title block or deleting a stamp is
    /// *the form*, not the 240 objects inside it.
    ///
    /// So the reach gained inside forms must not cost the reach to the form.
    /// The engine's own note on `hit_test_point_deep` says the same: *"the
    /// form itself is still reachable - `containment` names every enclosing
    /// form, so a shell can offer 'select the container' as a deliberate
    /// second act, which is a different thing from having it win by default."*
    ///
    /// # Why `paint_order` and not `containment`
    ///
    /// [`pdfcer_core::vector::FormLeaf::containment`] holds `ObjId`s, and an
    /// `ObjId` is not addressable by any paint-order verb - resolving one back
    /// to an index would mean a search, and a search that can find the *wrong*
    /// invocation when a page draws the same form twice.
    /// [`pdfcer_core::vector::FormLeaf::paint_order`] is *"the index, in
    /// `PageObjects::objects`, of the outermost form this object is inside"* -
    /// already the number this needs, already unambiguous about which
    /// invocation, and the same number the engine interleaves the two lists
    /// on.
    ///
    /// # Returns
    ///
    /// `None` for a page object (it has no container), for a stale leaf index,
    /// and for a query about another page. The **outermost** form, not the
    /// immediate parent: one act, one meaning, and it is the one whose index
    /// an edit verb can take.
    #[must_use]
    pub fn containing_form(&self, page_index: usize, target: TargetId) -> Option<TargetId> {
        if page_index != self.page_index {
            return None;
        }
        let leaf = self.objects.leaves.get(target.leaf_index()?)?;
        Some(TargetId::Object(leaf.paint_order as u64))
    }

    /// The **object id** of the outermost form enclosing a form-interior
    /// target — the operand half of the same question
    /// [`Self::containing_form`] answers in paint order.
    ///
    /// # ★★★ Why there have to be two of these, and it is not an oversight
    ///
    /// [`Self::containing_form`]'s own doc comment argues at length for
    /// answering in `paint_order` rather than in `containment`, and every word
    /// of it is still true **for the act it serves**. Selecting the container
    /// is a *selection*, and this shell's selection vocabulary is paint-order
    /// indices: `TargetId::Object(n)` is what `object-move`, `object-delete`
    /// and the outline renderer all take. An `ObjId` would have to be searched
    /// back into an index, and a search can find the wrong invocation when a
    /// page draws one form twice.
    ///
    /// `EditSession::unshare_form` inverts that. Its signature is
    /// `(page_index: usize, form: ObjId)` — it addresses the **stream object**,
    /// not a position in any paint order — and it is explicit that the unit of
    /// the operation is the PAGE rather than the invocation:
    ///
    /// > *"If this page invokes the form under several names, **all of them**
    /// > are re-pointed at the one copy."*
    ///
    /// ⇒ So the ambiguity `containing_form` refuses to resolve is one this verb
    /// **does not have**. "Which of the two invocations did you mean?" has no
    /// answer here, because the engine's answer is *both, always*. The two
    /// methods are therefore not two spellings of one fact; they are the two
    /// different facts two different verbs need, and offering only one of them
    /// is what left `unshare_form` unreachable from this shell until
    /// 2026-08-28.
    ///
    /// # Why `containment[0]` and not `parent()`
    ///
    /// [`pdfcer_core::vector::FormLeaf::containment`] is documented as *"the
    /// chain of enclosing form XObjects, **outermost first**, ending with the
    /// form this object is directly inside"*, and is never empty for a leaf.
    /// `FormLeaf::parent()` returns the **last** entry — the innermost form —
    /// and that is precisely the operand `unshare_form` refuses:
    /// `EditError::FormNestedInAnotherForm` fires when a form is reached only
    /// from inside another form, because re-binding a nested invocation means
    /// editing the parent, whose own blast radius depends on the document's
    /// nesting structure.
    ///
    /// ⇒ Passing `parent()` would therefore produce a **worded refusal on every
    /// nested drawing** where the outermost form is the one the operator wants
    /// and the one the engine can privatise. Taking position 0 is not a
    /// preference; it is the only element of the chain the verb accepts, and
    /// it is the same element `containing_form` reports the paint order of, so
    /// "select the form" and "unshare the form" cannot come to disagree about
    /// which form they mean.
    ///
    /// # Returns
    ///
    /// `None` for a page object (it is not inside anything), for a stale leaf
    /// index, for a query about another page, and — defensively — for a leaf
    /// with an empty containment chain, which the engine documents as
    /// impossible but which is cheaper to tolerate than to trust.
    #[must_use]
    pub fn containing_form_object(
        &self,
        page_index: usize,
        target: TargetId,
    ) -> Option<pdfcer_core::object::ObjId> {
        if page_index != self.page_index {
            return None;
        }
        let leaf = self.objects.leaves.get(target.leaf_index()?)?;
        leaf.containment.first().copied()
    }

    /// The **topmost** object under the pointer, or `None`.
    ///
    /// Defined as the head of [`Self::hit_test_all`] rather than as a second
    /// query, so "what does a plain click select?" and "what does cycling
    /// start from?" cannot come to different answers. This was the trait's
    /// *provided* method in the old shell — i.e. the same derivation,
    /// enforced by the trait rather than by this comment; the comment is what
    /// carries the guarantee until the trait comes back.
    #[must_use]
    pub fn hit_test(&self, page_index: usize, point: Pos2, tolerance: f64) -> Option<TargetId> {
        self.hit_test_all(page_index, point, tolerance)
            .into_iter()
            .next()
    }

    /// Every object a canvas-space marquee rect takes, under `mode`.
    ///
    /// # ★★★ `mode` is a parameter as of 2026-09-02, and it is `OPERATOR_REQUESTS.md` O88
    ///
    /// This used to hard-code [`MarqueeMode::Enclosed`], on decision 011's
    /// reasoning — *a marquee that grabs everything it grazes is unusable on a
    /// dense drawing, which is the document class pdfcer is for*. That reasoning
    /// is still right and is still the **default**; what was wrong was that it
    /// was the **only** answer.
    ///
    /// The operator's report: *"I can't box select the tables in the left or
    /// right top corners … it only picks up the lines of each table."* Both
    /// tables sit hard against the sheet edge, so a band that surrounds one has
    /// to start **outside the page** — and at fit zoom there is barely a pixel
    /// of margin to start in. The only band that can actually be drawn is one
    /// *inside* the table, which surrounds a few short rules and nothing else.
    ///
    /// ⇒ *"It only picks up the lines"* is what an enclosing band returns when
    /// it cannot be drawn big enough. It was never a hit test that excluded
    /// text.
    ///
    /// The caller chooses from the drag's **direction** — see
    /// `crate::canvas::gesture::GestureOutcome::Marquee::crossing`. Left to
    /// right encloses; right to left touches. That is AutoCAD's window /
    /// crossing-window rule, which SolidWorks drawings use too, and it is the
    /// convention rather than an invention: no modifier key, nothing new to
    /// learn, and the behaviour a drawing-office hand already has.
    ///
    /// ★ **Select All still passes `Enclosed` explicitly**
    /// (`app::actions::apply`), and must: it hands an infinite rect, under
    /// which the two modes agree, and stating the mode keeps the call readable
    /// rather than resting on that coincidence.
    #[must_use]
    pub fn hit_test_rect(&self, page_index: usize, rect: Rect, mode: MarqueeMode) -> Vec<TargetId> {
        if page_index != self.page_index {
            return Vec::new();
        }
        let Some(bounds) = self.canvas_rect_to_pdf_bounds(rect) else {
            return Vec::new();
        };
        // The page's own list, from the engine's rule.
        let mut out: Vec<TargetId> = hit_test_rect(&self.objects, bounds, mode)
            .into_iter()
            .map(|i| TargetId::Object(i as u64))
            .collect();
        // ...and the form interiors, by the SAME rule, applied here because
        // the engine has no deep marquee to call.
        //
        // Without this, a click could select an object inside a form and a
        // rubber-band across the identical object could not - two gestures
        // that mean "select this" disagreeing about what is selectable, which
        // is the kind of inconsistency an operator meets in the first minute.
        //
        // `contained_by` is the engine's own `MarqueeMode::Enclosed` predicate,
        // called directly rather than re-derived, so the two halves of this
        // answer cannot come to different conclusions about the same rect. A
        // leaf's `page_bbox` is already page space - `decompose_page` maps it
        // on the way out - so no second projection happens here.
        //
        // Written up for the request channel as a boundary finding rather than
        // kept quiet: `hit_test_rect` is the engine's marquee and it should
        // have a deep form the way `hit_test_point` now does, so that
        // `pdfcer` and this shell cannot drift on what a rubber-band
        // selects.
        out.extend(
            self.objects
                .leaves
                .iter()
                .enumerate()
                .filter(|(_, leaf)| match mode {
                    // The engine's own two predicates, called rather than
                    // re-derived, so the deep half and the page half of this
                    // answer cannot come to different conclusions about the
                    // same rect — which is the property this filter existed to
                    // keep before it had a mode to get wrong as well.
                    MarqueeMode::Enclosed => leaf.object.page_bbox().contained_by(bounds),
                    MarqueeMode::Touched => leaf.object.page_bbox().intersects(bounds),
                    // ★ No wildcard, deliberately. `MarqueeMode` is NOT
                    // `#[non_exhaustive]`, so a third variant added upstream
                    // breaks this build — which is the outcome to want. A
                    // wildcard would silently give the new mode whichever
                    // behaviour it fell through to, on a surface where the two
                    // existing behaviours differ by "did the operator get
                    // objects they did not ask for".
                })
                .map(|(i, _)| TargetId::Leaf(i as u64)),
        );
        out
    }

    /// One object's canvas-space bounding rect, or `None` for a stale id.
    ///
    /// A stale id resolving to `None` rather than panicking is the contract:
    /// a selection set can outlive an edit that removed what it named, and
    /// the correct response is to drop it silently, not to crash the frame
    /// that is trying to draw.
    #[must_use]
    pub fn bounds(&self, page_index: usize, target: TargetId) -> Option<Rect> {
        if page_index != self.page_index {
            return None;
        }
        // ★ Both lists, resolved by the id itself rather than by a caller
        // that had to remember which one it was holding. A leaf's geometry is
        // already in page space — `decompose_page` maps it there on the way
        // out — so this is the same projection, not a second one.
        let bbox = match target {
            TargetId::Object(i) => self
                .objects
                .objects
                .get(usize::try_from(i).ok()?)?
                .page_bbox(),
            TargetId::Leaf(i) => self
                .objects
                .leaves
                .get(usize::try_from(i).ok()?)?
                .object
                .page_bbox(),
        };
        self.pdf_bounds_to_canvas(bbox)
    }

    // ----------------------------- geometry -----------------------------

    /// Map a canvas-space point into PDF user space (the object model's
    /// frame), or `None` on a degenerate page.
    fn canvas_to_pdf(&self, p: Pos2) -> Option<Point> {
        let inv = self.to_pdf?;
        let mut pts = [SkPoint::from_xy(p.x, p.y)];
        inv.map_points(&mut pts);
        let out = pts[0];
        Some(Point::new(f64::from(out.x), f64::from(out.y)))
    }

    /// Map a PDF-space point into canvas space (for a selection outline).
    fn pdf_to_canvas(&self, p: Point) -> Pos2 {
        // Narrowing to f32 for egui; the object bounds are page geometry,
        // well within f32 range.
        #[allow(clippy::cast_possible_truncation)]
        let mut pts = [SkPoint::from_xy(p.x as f32, p.y as f32)];
        self.to_canvas.map_points(&mut pts);
        Pos2::new(pts[0].x, pts[0].y)
    }

    /// The canvas-space rect enclosing a PDF-space [`Bounds`] under the page
    /// transform (its four corners mapped, then bounded — the transform may
    /// rotate, so the axis-aligned canvas rect is the bound of the mapped
    /// quad).
    fn pdf_bounds_to_canvas(&self, b: Bounds) -> Option<Rect> {
        if b.is_empty() {
            return None;
        }
        let corners = [
            Point::new(b.min.x, b.min.y),
            Point::new(b.max.x, b.min.y),
            Point::new(b.max.x, b.max.y),
            Point::new(b.min.x, b.max.y),
        ];
        let mut rect: Option<Rect> = None;
        for c in corners {
            let p = self.pdf_to_canvas(c);
            rect = Some(match rect {
                None => Rect::from_min_max(p, p),
                Some(r) => r.union(Rect::from_min_max(p, p)),
            });
        }
        rect
    }

    /// The PDF-space bounding box of a canvas-space marquee rect (its four
    /// corners mapped back, then bounded).
    fn canvas_rect_to_pdf_bounds(&self, rect: Rect) -> Option<Bounds> {
        let corners = [
            rect.left_top(),
            rect.right_top(),
            rect.right_bottom(),
            rect.left_bottom(),
        ];
        let mut b = Bounds::EMPTY;
        for c in corners {
            b = b.union_point(self.canvas_to_pdf(c)?);
        }
        if b.is_empty() { None } else { Some(b) }
    }
}

/// Resolve a caller-supplied tolerance, falling back on a degenerate one.
///
/// A degenerate tolerance (`0.0` from a non-finite or zero zoom, or a
/// negative value) would silently make every query a miss. Falling back to
/// the fixed canvas-space value instead is the right trade: *fussy at low
/// zoom* is a far better failure than *selection is broken*.
///
/// Extracted into one function rather than repeated at each of the four call
/// sites that need it, because the four must agree — a click that selects an
/// object and then finds none of its subpaths reads as "the second level is
/// broken" when the real answer is that one site forgot the fallback.
fn resolve(tolerance: f64) -> f64 {
    if tolerance.is_finite() && tolerance > 0.0 {
        tolerance
    } else {
        FALLBACK_SELECT_TOLERANCE
    }
}

#[cfg(test)]
mod tests;

/// The Point rung's pick sets: which points belong to which part, and which
/// handle belongs to which node.
///
/// Separate from the module's main test block because these answer a
/// different question — not "does a click find the object" but "does the
/// index the operator sees mean what `node-move --node N` means".
#[cfg(test)]
mod node_rung_tests;
