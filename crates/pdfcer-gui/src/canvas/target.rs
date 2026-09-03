//! # `canvas::target` — the seam a hit-testable content model plugs into
//!
//! The canvas selects *things*. It does not know what a thing is, how it was
//! decomposed, or what coordinate frame its geometry was authored in. All it
//! needs is: **what is under this point, what is inside this rect, and where
//! is the thing I already have?** That question set is
//! [`CanvasTargetProvider`], and everything in `canvas/` is written against
//! it rather than against `pdfcer-core`.
//!
//! ## Why a trait rather than a direct call into the provider
//!
//! Three reasons, in order of how much they cost if ignored.
//!
//! 1. **The selection layer becomes headlessly testable.** Every invariant
//!    this stage is accountable for — *selection survives navigation* above
//!    all — is a property of the selection layer's *logic*, not of PDF
//!    decomposition. A test that had to build a `Document` to prove that
//!    zooming does not clear a selection would be a slow test of the wrong
//!    thing. [`StubTargets`] lets those tests state a page's contents in
//!    three lines.
//! 2. **The old shell already drew this line, and the provider was salvaged
//!    expecting it.** `panels::objects::provider`'s header, §2 of "What
//!    changed at salvage": *"The `CanvasTargetProvider` trait impl became
//!    inherent methods. The trait lives in `canvas/` and does not exist yet.
//!    The three methods keep their names and their exact semantics …
//!    Re-attaching the trait at S4 is a one-line `impl` block over methods
//!    that already have the right signatures."* This module is that
//!    re-attachment, and it is exactly that: [`impl CanvasTargetProvider for
//!    ObjectModelProvider`] delegates and adds nothing.
//! 3. **`GUI_ROADMAP.md` Phase 4** (continuous page display) changes *which
//!    pages* a provider answers for. A canvas written against the concrete
//!    single-page provider would have that assumption spread through it; a
//!    canvas written against a trait that takes `page_index` on every query
//!    already asks the right question.
//!
//! ## Every geometric argument here is CANVAS space
//!
//! Points, rects and tolerances crossing this trait are in canvas space —
//! Y-**down**, origin at the page's top-left, `/Rotate` already resolved. The
//! provider owns the hop into PDF user space (Y-**up**), because it owns the
//! page transform and inverts the *renderer's own* map to get there, so the
//! selection geometry and the raster agree by construction. See
//! [`crate::canvas::mapping`] for the full three-frame table and why
//! conflating any two of them is silent.
//!
//! ## The tolerance is a parameter, never a provider constant
//!
//! Stated on [`CanvasTargetProvider::hit_test`] and worth stating here too:
//! the only honest source for a hit tolerance is the live zoom, and the live
//! zoom belongs to the frame, not to the model. A provider that baked its own
//! tolerance would be a provider whose catch radius shrank as the operator
//! zoomed out — which is the defect
//! [`crate::canvas::mapping::SELECT_SCREEN_TOLERANCE_PX`] exists to close.

use egui::{Pos2, Rect};
use pdfcer_core::vector::MarqueeMode;

use crate::canvas::pick::PickClass;
use crate::panels::objects::provider::ObjectModelProvider;
pub use crate::panels::objects::provider::TargetId;

/// The seam a hit-testable content model plugs into.
///
/// Implemented today by [`ObjectModelProvider`] — the page's decomposed
/// vector objects. It will grow implementations for annotations and for
/// placed ce dimensions; those are separate object spaces with separate index
/// conventions, and the [`TargetId`] newtype is what keeps them from being
/// confused with each other.
///
/// ## "This page has no object model" is `None`, not a no-op provider
///
/// The old shell shipped an `EmptyTargetProvider` that hit nothing, enclosed
/// nothing and had no bounds. It is deliberately **not** carried across:
/// every consumer here takes `Option<&dyn CanvasTargetProvider>`, so the
/// absence is already representable, and two ways to say one thing is one way
/// too many. The `Option` is also the one that cannot be misread — a no-op
/// provider is indistinguishable from a page that decoded to nothing, and
/// [`crate::canvas::selection::SelectionState::resolve`] has to tell those
/// apart: the first must keep the selection and draw no outlines, the second
/// must drop entries that no longer exist.
pub trait CanvasTargetProvider {
    /// **Every** target at a canvas-space `point` within `tolerance`,
    /// **front-most first**.
    ///
    /// The required half of the point query, and the input to click-through
    /// cycling: an object entirely covered by another can only ever be
    /// selected by stepping past the cover, and a topmost-only query gives no
    /// click any way to do that.
    ///
    /// Empty for a miss, and empty for a query about a page this provider
    /// does not serve. Those two are deliberately the same answer — a caller
    /// that must distinguish them is asking the wrong object; the *canvas*
    /// knows which page it drew.
    fn hit_test_all(&self, page_index: usize, point: Pos2, tolerance: f64) -> Vec<TargetId>;

    /// The **front-most** target at a canvas-space `point`, or `None`.
    ///
    /// `tolerance` is the canvas-space slack the click may miss an object's
    /// edge by, and it is a **parameter, not a constant** — see the module
    /// docs. Callers hand it
    /// [`crate::canvas::mapping::PageMapping::tolerance`], which is the one
    /// place the screen radius is divided by the zoom.
    ///
    /// **A provided method, not a required one.** Defined as the head of
    /// [`Self::hit_test_all`], which is what makes *"what does a plain click
    /// select?"* and *"what does cycling start from?"* structurally the same
    /// answer rather than a convention two implementations have to keep.
    fn hit_test(&self, page_index: usize, point: Pos2, tolerance: f64) -> Option<TargetId> {
        self.hit_test_all(page_index, point, tolerance)
            .into_iter()
            .next()
    }

    /// Which **class** a target belongs to, for the operator's selection
    /// filter — or `None` for a target this provider no longer knows.
    ///
    /// ★ This exists so that [`crate::canvas::input::probe`] can skip
    /// candidates whose class is switched off *without* knowing anything
    /// about how objects are stored. The alternative — handing `probe` the
    /// decomposition and letting it match on `VectorObject` — would put a
    /// second kind classifier in the codebase, which
    /// `crate::panels::objects::summary` exists to prevent.
    ///
    /// A provided method returning `None`, so the test doubles in this
    /// module and elsewhere do not all have to implement it. `None` means
    /// *"I cannot say"*, and the filter's contract for that is to let the
    /// candidate through: a provider that does not classify must not become
    /// a provider whose objects are all unselectable.
    fn object_class(&self, page_index: usize, target: TargetId) -> Option<PickClass> {
        let _ = (page_index, target);
        None
    }

    /// **The outermost form XObject a target is painted inside**, as a page
    /// object, or `None` for a target that is not inside one.
    ///
    /// ★★★ Why this is on the trait rather than only on the live provider
    /// (2026-08-31, `OPERATOR_REQUESTS.md` O70): `canvas::smart` substitutes a
    /// container for a leaf on the click path, which runs against
    /// `&dyn CanvasTargetProvider`. Reaching past the trait to the concrete
    /// provider there would put the one rule this feature has in a place the
    /// test doubles cannot reach — and the rule is exactly the kind that needs
    /// a stub to state it: *a click selects the container until you are inside
    /// it.*
    ///
    /// A provided method answering `None`, like [`Self::object_class`], so a
    /// double that has no forms is unaffected. `None` means *"nothing to
    /// substitute"*, which is the honest answer for a page with no forms and
    /// for a provider that does not model them.
    fn containing_form(&self, page_index: usize, target: TargetId) -> Option<TargetId> {
        let _ = (page_index, target);
        None
    }

    /// Every target **fully enclosed** by a canvas-space marquee rect.
    ///
    /// Fully-enclosed rather than touched is the shipped convention
    /// (decision 011, matching Inkscape's default and the old shell): a
    /// marquee that grabs everything it grazes is unusable on a dense
    /// drawing, which is the document class pdfcer is for.
    /// **Is this container worth selecting instead of what is inside it?**
    ///
    /// # ★★★ The question the Smart-Selector forgot to ask
    ///
    /// `canvas::smart::Scope::resolve` maps a leaf to its containing form so
    /// that a first click selects the container and a double-click descends —
    /// `OPERATOR_REQUESTS.md` O70, and the right model for a title block or a
    /// stamp.
    ///
    /// It is the **wrong** model for the commonest form in the world.
    /// Every CAD exporter this project has seen wraps a drawing's whole visible
    /// body in one page-sized form XObject, and a `/BBox` is a clipping extent
    /// (§8.10.1) rather than a claim about ink — so that wrapper contains
    /// everything, wins every click, and "select the container first" becomes
    /// "select the whole drawing, every time".
    ///
    /// ⇒ Which is the operator's **headline complaint**, verbatim, restored by
    /// the feature built to improve selection:
    ///
    /// > *"There are obviously more than one item on the page, but when I click
    /// > on one of the objects all I get is the page selected."*
    ///
    /// ★★ So a container is worth resolving to only when selecting it says
    /// something selecting the leaf does not. A container that holds
    /// **everything on the page** says nothing: it IS the page, under another
    /// name.
    ///
    /// # What it does NOT change
    ///
    /// **Entering** such a form still works, and must. A double-click descends
    /// into it, the Objects panel lists it, and the canvas menu's *"select the
    /// containing form"* reaches it. Reachable on purpose was always the
    /// design; winning by default is what was wrong, both times.
    ///
    /// Defaults to `true` — a provider that cannot measure says yes, which is
    /// the behaviour before this existed.
    fn container_is_worth_selecting(&self, page_index: usize, container: TargetId) -> bool {
        let _ = (page_index, container);
        true
    }

    /// Every target a marquee rect takes, under `mode`.
    ///
    /// ★ `mode` is a parameter as of 2026-09-02 (`OPERATOR_REQUESTS.md` O88):
    /// a left-to-right drag encloses, a right-to-left drag touches. The caller
    /// decides from the drag's direction and every implementor obeys, rather
    /// than each deciding for itself — see the live provider's own
    /// `hit_test_rect` for the report that motivated it.
    fn hit_test_rect(&self, page_index: usize, rect: Rect, mode: MarqueeMode) -> Vec<TargetId>;

    /// One target's canvas-space bounding rect, or `None` for a target this
    /// provider no longer knows.
    ///
    /// **`None` rather than a panic is the contract**, and it is what makes
    /// re-resolution possible: a selection can outlive an edit that removed
    /// what it named, and the correct response is to drop the entry silently,
    /// not to crash the frame that is trying to draw.
    fn bounds(&self, page_index: usize, target: TargetId) -> Option<Rect>;

    /// Which **part** of `object` a canvas-space click lands on — subpaths
    /// for a path object, show-operator runs for a text object — nearest
    /// first.
    ///
    /// One query for both kinds, because the alternative is a kind match at
    /// every call site and the failure when two of them drift is that
    /// descending works for a drawing and not for a label. The dispatch lives
    /// in the provider ([`ObjectModelProvider::part_hits`]).
    ///
    /// Empty for an object with no part rung at all (an image), which is why
    /// the ladder caps itself at the Object rung for images by construction
    /// rather than by a check.
    /// The page's decomposed geometry, for a consumer that needs the model
    /// itself rather than a hit test over it.
    ///
    /// ★ **The one consumer is the two-line measure tool**, whose pick is
    /// `pdfcer_core::vector::linepick::pick_line_in_page` — a query this trait
    /// deliberately does not wrap. Wrapping it would put a *second* line-pick
    /// rule in the shell beside the engine's, and the whole point of the
    /// two-line dimension is that the shell and `pdfcer dimension-add`
    /// resolve the same click to the same line.
    ///
    /// `None` is a real answer, not a failure: a provider that has no
    /// `PageObjects` for `page_index` (the test double has none at all) simply
    /// cannot be asked, and the caller's correct response is to take no pick
    /// rather than to substitute one.
    fn page_objects_model(&self, page_index: usize) -> Option<&pdfcer_core::vector::PageObjects> {
        let _ = page_index;
        None
    }

    /// One object's page-space anchor samples — **the circular measure tool's
    /// fit input**, and the only query on this trait whose result is not
    /// canvas space.
    ///
    /// ★ **PDF user space, deliberately**, where every other geometric value
    /// crossing this trait is canvas space. The reason is the same one that
    /// keeps the two-line pick going through
    /// [`Self::page_objects_model`]: these points are handed straight to
    /// [`fit_circle_taubin`](pdfcer_core::dimension::fit_circle_taubin), which
    /// is the **engine's** fit, and the engine works in PDF user space. A
    /// canvas-space sample set would have to be converted back before the fit,
    /// which is a second Y-flip in a shell whose header says there is exactly
    /// one. The provider already stores its `PageObjects` in that frame, so
    /// this is a read rather than a conversion.
    ///
    /// Empty is a real answer and the common one: a text, image or form object
    /// carries no anchors — the same exclusion the snap engine applies — and so
    /// does a query about a page this provider does not serve or an index it
    /// does not have. A caller that must distinguish those is asking the wrong
    /// object; the *canvas* knows which page it drew.
    ///
    /// **A provided method returning nothing**, so a future provider over
    /// annotations or placed dimensions does not have to invent anchors for
    /// objects that have none in order to compile.
    fn object_sample_points(
        &self,
        page_index: usize,
        index: usize,
    ) -> Vec<pdfcer_core::vector::Point> {
        let _ = (page_index, index);
        Vec::new()
    }

    // ===================================================================
    // ★★★ THE SAME THREE QUESTIONS, FOR EITHER INDEX SPACE
    // ===================================================================
    //
    // `OPERATOR_REQUESTS.md` O70, 2026-09-01. The three above take a page
    // paint-order index, which is the only address the Part and Node rungs
    // have ever had — so those rungs were structurally unavailable for
    // anything painted inside a form XObject. `canvas::input::probe` said so
    // in a comment: *"the ladder stopping at the Object rung for a leaf,
    // expressed where the address space runs out."*
    //
    // These take a `TargetId` and are the ones `probe` now asks. The
    // page-index forms stay for the callers that legitimately hold one, and
    // the default implementations here delegate to them so a test double that
    // has no leaves is unaffected.

    /// [`Self::part_hits`], for either index space.
    fn part_hits_of(
        &self,
        page_index: usize,
        target: TargetId,
        point: Pos2,
        tolerance: f64,
    ) -> Vec<usize> {
        match target.page_object_index() {
            Some(object) => self.part_hits(page_index, object, point, tolerance),
            // ★ A double that does not model leaves answers "no parts" rather
            // than pretending — the same shape `object_class` uses for "I
            // cannot say", and the honest answer for a provider with one list.
            None => Vec::new(),
        }
    }

    /// [`Self::part_bounds`], for either index space.
    fn part_bounds_of(&self, page_index: usize, target: TargetId, part: usize) -> Option<Rect> {
        self.part_bounds(page_index, target.page_object_index()?, part)
    }

    /// [`Self::nearest_node`], for either index space.
    fn nearest_node_of(
        &self,
        page_index: usize,
        target: TargetId,
        part: usize,
        point: Pos2,
        tolerance: f64,
    ) -> Option<usize> {
        self.nearest_node(
            page_index,
            target.page_object_index()?,
            part,
            point,
            tolerance,
        )
    }

    fn part_hits(
        &self,
        page_index: usize,
        object: usize,
        point: Pos2,
        tolerance: f64,
    ) -> Vec<usize>;

    /// A part's own canvas-space bounds, for its outline.
    ///
    /// The *part's* box, never the object's. An object-sized rectangle drawn
    /// around a part tells the operator they selected the whole thing again
    /// — which is the misunderstanding entering the object exists to
    /// resolve, and on a measured CAD export that rectangle spans the entire
    /// drawing.
    fn part_bounds(&self, page_index: usize, object: usize, part: usize) -> Option<Rect>;

    /// The **object-scoped** index of the anchor of `part` nearest a
    /// canvas-space `point` within `tolerance` — the Node rung's pick.
    ///
    /// Object-scoped, not part-scoped, and that is load-bearing rather than a
    /// convention: it is the space `vector::anchor_count` reports and the
    /// space `pdfcer node-move --node N` addresses. A second numbering
    /// would make the number pdfcer shows disagree with the number the
    /// operator can act on.
    fn nearest_node(
        &self,
        page_index: usize,
        object: usize,
        part: usize,
        point: Pos2,
        tolerance: f64,
    ) -> Option<usize>;
}

/// **How much of a page's content a container may cover and still be worth
/// selecting**, as a fraction of the union of every page object's bounds.
///
/// ★ 0.9 — a container over nine tenths of everything on the sheet **is** the
/// sheet. See [`CanvasTargetProvider::container_is_worth_selecting`] for the
/// defect this number exists to prevent, and for why it errs generous.
const COVERS_EVERYTHING: f32 = 0.9;

/// ★ **The re-attachment.**
///
/// `panels::objects::provider` carried these methods across salvage as
/// inherent methods with their signatures and semantics unchanged, precisely
/// so this block would be a delegation and nothing else. It is: no
/// arithmetic, no tolerance rule, no hit ordering and no index convention is
/// decided here. Every one of the delegated methods is already under test in
/// that module against a real decomposition.
///
/// [`Self::hit_test`] is deliberately **not** overridden. The provider has an
/// inherent `hit_test` with the identical derivation (the head of
/// `hit_test_all`), and its own doc comment says the comment carries the
/// guarantee *"until the trait comes back"*. The trait is back, so the
/// guarantee is structural again and the inherent one is the redundant copy
/// — overriding here would reinstate two derivations of one answer.
///
/// The three ladder methods take `page_index` even though the provider is
/// single-page and its inherent methods do not, so the guard is applied on
/// this side. A canvas that descended into a part of a page the provider does
/// not serve would be addressing paint-order indices in the wrong page's
/// index space, which is the same class of error the `TargetId` newtype
/// exists to prevent — and the guard costs one comparison.
impl CanvasTargetProvider for ObjectModelProvider {
    /// Measured, not assumed. See the trait for why the question exists.
    ///
    /// # ★★ The measurement, and why it is not "is it page-sized"
    ///
    /// The obvious predicate — compare the form's `/BBox` with the page's media
    /// box — needs a page rect this provider does not hold, and it answers the
    /// wrong question anyway. A form can be *smaller* than the page and still
    /// contain every mark on it, which is common: an exporter that wraps the
    /// drawing body but not the margin produces exactly that, and selecting
    /// that wrapper is just as useless.
    ///
    /// So the comparison is against **what is actually on the sheet** — the
    /// union of every page object's bounds. If the container covers essentially
    /// all of it, the container IS the page's content, and selecting it tells
    /// the operator nothing they did not already know.
    ///
    /// ★ [`COVERS_EVERYTHING`] is deliberately generous. What it guards against
    /// is severe and constant — every click on a CAD drawing — and the cost of
    /// being slightly too generous is that one unusually large title block
    /// stops being offered as a container on the *first* click, while staying
    /// reachable by double-click, by the Objects panel and by the canvas menu.
    /// A mild inconvenience against a headline defect.
    fn container_is_worth_selecting(&self, page_index: usize, container: TargetId) -> bool {
        let Some(bounds) = self.bounds(page_index, container) else {
            return true;
        };
        // ★★★ AGAINST THE PAGE — and the first version of this compared against
        // the union of every page object's bounds instead.
        //
        // That was wrong in a way only a SECOND fixture could show: when the
        // form is the only page object, the union IS the form, the ratio is
        // 1.0, and every lone container is judged "holds everything" — a stamp
        // on an otherwise empty sheet included.
        //
        // ★★ Caught by two of this project's own driven checks contradicting
        // each other within the hour, which is the most useful thing a suite
        // can do. One demanded the leaf on a page-sized wrapper; the other
        // demanded the container on a 320×220 form on a 400×300 page. Both are
        // right, and only a page-relative measure satisfies both.
        let Some(page) = self.page_extent() else {
            return true;
        };
        if page.x <= f32::EPSILON || page.y <= f32::EPSILON {
            return true;
        }
        let covers = (bounds.width() / page.x).min(1.0) * (bounds.height() / page.y).min(1.0);
        let worth = covers < COVERS_EVERYTHING;
        // ★★ Traced, because the alternative is inferring this from a selection
        // two layers away. When those two checks disagreed, neither could say
        // what the predicate had actually answered — the numbers had to be
        // reconstructed by hand from a fixture generator. One line ends that.
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!(
                "container-worth id={} covers={covers:.3} page={:.0}x{:.0} box={:.0}x{:.0} worth={worth}",
                container.raw(),
                page.x,
                page.y,
                bounds.width(),
                bounds.height()
            )
        });
        worth
    }

    /// The real model. Guarded on the page, because this provider decomposes
    /// exactly one page and answering for another would hand the measure tool
    /// geometry from a different sheet.
    fn page_objects_model(&self, page_index: usize) -> Option<&pdfcer_core::vector::PageObjects> {
        (page_index == self.page_index()).then(|| self.page_objects())
    }

    fn hit_test_all(&self, page_index: usize, point: Pos2, tolerance: f64) -> Vec<TargetId> {
        Self::hit_test_all(self, page_index, point, tolerance)
    }

    /// The real samples, guarded on the page for the reason the block's own
    /// docs give: the provider decomposes exactly one page, and answering for
    /// another would fit a circle to a different sheet's geometry.
    fn object_sample_points(
        &self,
        page_index: usize,
        index: usize,
    ) -> Vec<pdfcer_core::vector::Point> {
        if page_index != self.page_index() {
            return Vec::new();
        }
        Self::object_sample_points(self, index)
    }

    fn hit_test_rect(&self, page_index: usize, rect: Rect, mode: MarqueeMode) -> Vec<TargetId> {
        Self::hit_test_rect(self, page_index, rect, mode)
    }

    /// ★ `Self::containing_form`, spelled as the inherent call rather than as
    /// `self.containing_form(..)`, which would be ambiguous to a reader for
    /// the same reason the four lines above are spelled this way: the
    /// provider has an inherent method of that name and this is the trait's.
    /// Rust resolves the bare form to the inherent one, so it happens to be
    /// correct and reads as a recursion.
    fn containing_form(&self, page_index: usize, target: TargetId) -> Option<TargetId> {
        Self::containing_form(self, page_index, target)
    }

    // ★ The three `_of` overrides, which is where a leaf's Part and Node rungs
    // actually come from — the trait's defaults answer only for a page object.
    // `provider::geometry` holds the implementations and its header carries why
    // none of it needed an engine request.
    fn part_hits_of(
        &self,
        page_index: usize,
        target: TargetId,
        point: Pos2,
        tolerance: f64,
    ) -> Vec<usize> {
        if page_index != self.page_index() {
            return Vec::new();
        }
        use crate::panels::objects::provider::PartKind;
        match (self.part_kind_of(target), target.page_object_index()) {
            (Some(PartKind::Subpath), _) => self.subpath_hits_of(target, point, tolerance),
            // ★★★ **A TEXT OBJECT'S RUNS, and forgetting them here broke the
            // Points tool for text — caught by the full sweep, 2026-09-01.**
            //
            // The first version of this override handled `Subpath` and answered
            // empty for everything else, which silently dropped the `Run` arm
            // the index-based `part_hits` had always dispatched. The symptom
            // was `canvas-anchors-declined reason=not-entered`: a Points-tool
            // click on text found no part, so the rung was never entered and no
            // points drew. Every unit test passed; the driven suite caught it.
            //
            // ⇒ The lesson is the one this project keeps relearning about
            // generalising a function: the new axis (which index space) is easy
            // to see, and the axis that was ALREADY there (which kind of part)
            // is the one that gets dropped.
            (Some(PartKind::Run), Some(object)) => self.text_run_hits(object, point, tolerance),
            // ★ A text run INSIDE a form has no leaf-indexed hit test yet —
            // `text_run_hits` indexes the page's own list, and answering from it
            // would return another object's runs entirely. Empty is the honest
            // answer, and it is the next thing to build rather than an
            // oversight.
            (Some(PartKind::Run), None) | (None, _) => Vec::new(),
        }
    }

    fn part_bounds_of(&self, page_index: usize, target: TargetId, part: usize) -> Option<Rect> {
        (page_index == self.page_index())
            .then(|| self.subpath_bounds_canvas_of(target, part))
            .flatten()
    }

    fn nearest_node_of(
        &self,
        page_index: usize,
        target: TargetId,
        part: usize,
        point: Pos2,
        tolerance: f64,
    ) -> Option<usize> {
        (page_index == self.page_index())
            .then(|| Self::nearest_node_of(self, target, part, point, tolerance))
            .flatten()
    }

    /// The real classifier, guarded on the page for the same reason every
    /// other query here is: this provider decomposes exactly one page.
    ///
    /// Delegates to `panels::objects::summary::object_kind`, which is **the**
    /// kind classifier in this crate, and then maps its answer with
    /// `PickClass::of_object`. Two hops rather than one match, deliberately:
    /// the hop through `object_kind` is what keeps this from becoming a
    /// second, drifting copy of the same decision.
    fn object_class(&self, page_index: usize, target: TargetId) -> Option<PickClass> {
        if page_index != self.page_index() {
            return None;
        }
        // ★ Both lists. The classifier is `object_kind`, unchanged and
        // still the only one in this crate — a leaf holds a `VectorObject`
        // exactly like a page object does, so the same hop answers for it and
        // the operator's pick filter works identically inside a form.
        let model = self.page_objects();
        let object = match target {
            TargetId::Object(i) => model.objects.get(usize::try_from(i).ok()?)?,
            TargetId::Leaf(i) => &model.leaves.get(usize::try_from(i).ok()?)?.object,
        };
        Some(PickClass::of_object(
            crate::panels::objects::summary::object_kind(object),
        ))
    }

    fn bounds(&self, page_index: usize, target: TargetId) -> Option<Rect> {
        Self::bounds(self, page_index, target)
    }

    fn part_hits(
        &self,
        page_index: usize,
        object: usize,
        point: Pos2,
        tolerance: f64,
    ) -> Vec<usize> {
        if page_index != self.page_index() {
            return Vec::new();
        }
        Self::part_hits(self, object, point, tolerance)
    }

    fn part_bounds(&self, page_index: usize, object: usize, part: usize) -> Option<Rect> {
        if page_index != self.page_index() {
            return None;
        }
        Self::part_bounds_canvas(self, object, part)
    }

    fn nearest_node(
        &self,
        page_index: usize,
        object: usize,
        part: usize,
        point: Pos2,
        tolerance: f64,
    ) -> Option<usize> {
        if page_index != self.page_index() {
            return None;
        }
        Self::nearest_node(self, object, part, point, tolerance)
    }
}

/// A provider assembled from plain rectangles — the seam every selection
/// test in `canvas/` uses.
///
/// # Why the selection tests do not use the real provider
///
/// Because they are not about decomposition. *"Zooming out three rungs does
/// not clear the selection"* is a property of the selection layer's state
/// machine; proving it against a real PDF would mean a fixture, a
/// `Document`, a page tree and a content-stream walk, all to establish that
/// one `Vec` was not emptied. The real provider's geometry is already proven
/// in its own module, against real content streams, and duplicating that
/// coverage here would test `pdfcer-core` twice and the invariant once.
///
/// Objects are listed **back to front** (paint order, the same convention as
/// `PageObjects::objects`), so `hit_test_all`'s front-most-first contract is
/// this type reversing the scan — which is a real behaviour worth having in
/// the stub rather than a simplification that would let a caller depending on
/// the order pass here and fail live.
#[cfg(test)]
#[derive(Debug, Default, Clone)]
pub struct StubTargets {
    /// Which page this stub answers for.
    pub page: usize,
    /// One rect per object, in paint order.
    pub objects: Vec<Rect>,
    /// One rect per **form-interior leaf**, in the order the real provider
    /// would list them.
    ///
    /// ★ A second list rather than a flag on the first, because that is the
    /// shape the engine ships and the shape the two index spaces have. A stub
    /// that modelled a leaf as "an object with a marker" could not reproduce
    /// the one property every test here turns on: that `objects[1]` and
    /// `leaves[1]` are different things, and that only the first is an edit
    /// operand.
    ///
    /// Deliberately **not** hit by [`Self::hit_test_rect`], matching the live
    /// provider — see its `hit_test_rect` for why a marquee stays on the
    /// page's own list.
    pub leaves: Vec<Rect>,
    /// Optional per-object part rects, in part order. An object with no
    /// entry has no parts — the image case.
    pub parts: std::collections::BTreeMap<usize, Vec<Rect>>,
    /// Optional per-object anchor samples, in **PDF user space** — the
    /// circular measure tool's fit input.
    ///
    /// Absent for an object means *"carries no fit geometry"*, which is the
    /// text/image case the real provider answers the same way. Stated
    /// explicitly rather than derived from [`Self::objects`] because that
    /// distinction is precisely what `measure::circular::click` refuses on, and
    /// a stub that manufactured four corners for every object could not
    /// express the refusal at all.
    pub samples: std::collections::BTreeMap<usize, Vec<pdfcer_core::vector::Point>>,
    /// Which page object each leaf is painted inside, as `(leaf, object)`.
    ///
    /// Empty means *"these leaves have no container this stub knows about"*,
    /// which is what the live provider answers for a target that is not in a
    /// form — so a test that does not set it gets the no-substitution path.
    pub containers: std::collections::BTreeMap<usize, usize>,
}

#[cfg(test)]
impl StubTargets {
    /// A stub for `page` holding `objects` in paint order.
    pub fn new(page: usize, objects: impl IntoIterator<Item = Rect>) -> Self {
        Self {
            page,
            objects: objects.into_iter().collect(),
            leaves: Vec::new(),
            parts: std::collections::BTreeMap::new(),
            samples: std::collections::BTreeMap::new(),
            containers: std::collections::BTreeMap::new(),
        }
    }

    /// Say which page object each leaf lives inside, as `(leaf, object)`.
    #[must_use]
    pub fn with_containers(mut self, pairs: impl IntoIterator<Item = (usize, usize)>) -> Self {
        self.containers = pairs.into_iter().collect();
        self
    }

    /// Give the page some form-interior leaves, front-most last.
    ///
    /// A leaf is hit *before* every page object at the same point, which
    /// models the common real case this whole change exists for: the page
    /// object at that point is the page-sized **form**, the engine excludes
    /// forms from a deep hit test outright, and what is left is what is inside
    /// it. The stub does not model paint-order interleaving — the live
    /// provider gets that from
    /// [`pdfcer_core::vector::hit_test_point_deep`], which is where the
    /// ordering rule belongs and where it is tested.
    #[must_use]
    pub fn with_leaves(mut self, leaves: impl IntoIterator<Item = Rect>) -> Self {
        self.leaves = leaves.into_iter().collect();
        self
    }

    /// Give `object` some parts.
    #[must_use]
    pub fn with_parts(mut self, object: usize, parts: impl IntoIterator<Item = Rect>) -> Self {
        self.parts.insert(object, parts.into_iter().collect());
        self
    }

    /// Give `object` some page-space anchor samples — what makes it pickable
    /// by the circular measure tool.
    #[must_use]
    pub fn with_samples(
        mut self,
        object: usize,
        samples: impl IntoIterator<Item = pdfcer_core::vector::Point>,
    ) -> Self {
        self.samples.insert(object, samples.into_iter().collect());
        self
    }

    /// The rect grown by `tolerance` on every side — the stub's model of "a
    /// click may miss an edge by the catch radius". Crude next to the real
    /// per-segment distance test, and deliberately so: what the selection
    /// layer must get right is *that it passes a page-space tolerance at
    /// all*, and a stub that ignored the argument could not fail that way.
    fn caught(rect: Rect, tolerance: f64) -> Rect {
        #[allow(
            clippy::cast_possible_truncation,
            reason = "a catch radius is a handful of points; f32 is exact well past that" // ui-text-exempt: clippy lint justification, never displayed
        )]
        let pad = tolerance.max(0.0) as f32;
        rect.expand(pad)
    }
}

#[cfg(test)]
impl CanvasTargetProvider for StubTargets {
    fn hit_test_all(&self, page_index: usize, point: Pos2, tolerance: f64) -> Vec<TargetId> {
        if page_index != self.page {
            return Vec::new();
        }
        let leaves = self
            .leaves
            .iter()
            .enumerate()
            .filter(|(_, r)| Self::caught(**r, tolerance).contains(point))
            .map(|(i, _)| TargetId::Leaf(i as u64))
            .rev();
        let objects = self
            .objects
            .iter()
            .enumerate()
            .filter(|(_, r)| Self::caught(**r, tolerance).contains(point))
            .map(|(i, _)| TargetId::Object(i as u64))
            // Paint order is back to front; the contract is front-most first.
            .rev();
        leaves.chain(objects).collect()
    }

    fn object_sample_points(
        &self,
        page_index: usize,
        index: usize,
    ) -> Vec<pdfcer_core::vector::Point> {
        if page_index != self.page {
            return Vec::new();
        }
        self.samples.get(&index).cloned().unwrap_or_default()
    }

    fn containing_form(&self, page_index: usize, target: TargetId) -> Option<TargetId> {
        if page_index != self.page {
            return None;
        }
        let leaf = target.leaf_index()?;
        self.containers
            .get(&leaf)
            .map(|object| TargetId::Object(*object as u64))
    }

    fn hit_test_rect(&self, page_index: usize, rect: Rect, mode: MarqueeMode) -> Vec<TargetId> {
        if page_index != self.page {
            return Vec::new();
        }
        self.objects
            .iter()
            .enumerate()
            .filter(|(_, r)| match mode {
                MarqueeMode::Enclosed => rect.contains_rect(**r),
                MarqueeMode::Touched => rect.intersects(**r),
            })
            .map(|(i, _)| TargetId::Object(i as u64))
            .collect()
    }

    fn bounds(&self, page_index: usize, target: TargetId) -> Option<Rect> {
        if page_index != self.page {
            return None;
        }
        match target {
            TargetId::Object(i) => self.objects.get(usize::try_from(i).ok()?).copied(),
            TargetId::Leaf(i) => self.leaves.get(usize::try_from(i).ok()?).copied(),
        }
    }

    fn part_hits(
        &self,
        page_index: usize,
        object: usize,
        point: Pos2,
        tolerance: f64,
    ) -> Vec<usize> {
        if page_index != self.page {
            return Vec::new();
        }
        self.parts
            .get(&object)
            .map(|parts| {
                parts
                    .iter()
                    .enumerate()
                    .filter(|(_, r)| Self::caught(**r, tolerance).contains(point))
                    .map(|(i, _)| i)
                    .collect()
            })
            .unwrap_or_default()
    }

    fn part_bounds(&self, page_index: usize, object: usize, part: usize) -> Option<Rect> {
        if page_index != self.page {
            return None;
        }
        self.parts.get(&object)?.get(part).copied()
    }

    fn nearest_node(
        &self,
        page_index: usize,
        object: usize,
        part: usize,
        point: Pos2,
        tolerance: f64,
    ) -> Option<usize> {
        // The stub's nodes are the part rect's four corners, numbered from
        // the object's first part — object-scoped, as the contract requires.
        if page_index != self.page {
            return None;
        }
        let parts = self.parts.get(&object)?;
        let offset = parts.iter().take(part).count() * 4;
        let rect = parts.get(part)?;
        let corners = [
            rect.left_top(),
            rect.right_top(),
            rect.right_bottom(),
            rect.left_bottom(),
        ];
        corners
            .iter()
            .enumerate()
            .map(|(i, c)| (i, f64::from(c.distance(point))))
            .filter(|(_, d)| *d <= tolerance)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(i, _)| offset + i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::from_min_size(Pos2::new(x, y), egui::vec2(w, h))
    }

    /// The stub reports front-most first, so a test that depends on stacking
    /// order is exercising the same contract the live provider honours.
    #[test]
    fn the_stub_reports_the_front_most_object_first() {
        let p = StubTargets::new(
            0,
            [rect(0.0, 0.0, 100.0, 100.0), rect(40.0, 40.0, 20.0, 20.0)],
        );
        assert_eq!(
            p.hit_test_all(0, Pos2::new(50.0, 50.0), 0.0),
            vec![TargetId::Object(1), TargetId::Object(0)]
        );
        assert_eq!(
            p.hit_test(0, Pos2::new(50.0, 50.0), 0.0),
            Some(TargetId::Object(1))
        );
        // A query about another page is a miss, not a panic.
        assert!(p.hit_test_all(1, Pos2::new(50.0, 50.0), 0.0).is_empty());
    }

    /// The stub actually consults the tolerance it is handed. A stub that
    /// ignored it could not fail the way a caller passing raw screen pixels
    /// fails, which would make every selection test blind to the defect this
    /// stage is most at risk of.
    #[test]
    fn the_stub_honours_the_tolerance_it_is_given() {
        let p = StubTargets::new(0, [rect(0.0, 0.0, 10.0, 10.0)]);
        let just_outside = Pos2::new(14.0, 5.0);
        assert!(p.hit_test_all(0, just_outside, 1.0).is_empty());
        assert_eq!(
            p.hit_test_all(0, just_outside, 6.0),
            vec![TargetId::Object(0)]
        );
    }

    /// The marquee encloses rather than touches, on both sides of the seam.
    #[test]
    fn the_stub_marquee_requires_full_enclosure() {
        let p = StubTargets::new(
            0,
            [rect(0.0, 0.0, 10.0, 10.0), rect(100.0, 100.0, 10.0, 10.0)],
        );
        let grazing = Rect::from_min_size(Pos2::new(5.0, 5.0), egui::vec2(200.0, 200.0));
        assert_eq!(
            p.hit_test_rect(0, grazing, MarqueeMode::Enclosed),
            vec![TargetId::Object(1)],
            "an object the marquee only grazes must not be selected"
        );
        // ★★ …and the SAME band as a crossing window takes BOTH — O88.
        //
        // The pair is the point. An `Enclosed`-only assertion passes against a
        // stub that ignores its mode argument entirely, which is exactly the
        // shape a hurried implementation of this change would have: the
        // parameter added, threaded, and never read. Asserting both modes over
        // one rect is the only way this test can tell them apart.
        assert_eq!(
            p.hit_test_rect(0, grazing, MarqueeMode::Touched),
            vec![TargetId::Object(0), TargetId::Object(1)],
            "a crossing window must take the object it only grazes as well"
        );
    }

    /// Node indices stay object-scoped across a part boundary — the same law
    /// the real provider's `node_rung_tests` pins, restated on the stub so a
    /// selection test reading a node index is reading the same numbering the
    /// live provider would have produced.
    #[test]
    fn stub_node_indices_keep_counting_across_parts() {
        let p = StubTargets::new(0, [rect(0.0, 0.0, 100.0, 100.0)]).with_parts(
            0,
            [rect(0.0, 0.0, 10.0, 10.0), rect(50.0, 50.0, 10.0, 10.0)],
        );
        assert_eq!(p.nearest_node(0, 0, 0, Pos2::new(0.0, 0.0), 2.0), Some(0));
        assert_eq!(
            p.nearest_node(0, 0, 1, Pos2::new(50.0, 50.0), 2.0),
            Some(4),
            "the second part's points must continue the object's numbering"
        );
        // Out of tolerance is nothing, rather than the nearest regardless.
        assert_eq!(p.nearest_node(0, 0, 0, Pos2::new(30.0, 30.0), 2.0), None);
    }
}
