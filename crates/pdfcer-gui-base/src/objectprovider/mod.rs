//! # `objectprovider` — front-to-back page object decomposition
//!
//! The thin `pdfcer-gui` adapter that plugs `pdfcer-core`'s read-only vector
//! object model (`pdfcer_core::vector`) into the shell. The shape is fixed:
//! this adapter CALLS INTO the object model, which stays GUI-free; the adapter
//! owns the trait impl and the object model owns none of it.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/objectprovider/mod.md`.

/// **The same questions, asked of either index space** — the `_of` family that
/// lets the Part and Node rungs be offered for something painted inside a form
/// XObject (`OPERATOR_REQUESTS.md` O70). Its header carries why the `_of`
/// family is a seam rather than an arbitrary cut.
mod geometry;

/// **The Part rung's unit for text: one visual LINE, not one show operator.**
/// One line of a CAD title block is however many show operators its producer
/// chose to write — 237 operators forming 144 lines on the measured sheet —
/// and its header carries why the shell addresses lines and what follows for
/// the move.
mod line;

/// **The Point rung's pick sets** — which anchors belong to which subpath,
/// which number each answers to, which Bézier handle shapes which side of a
/// node, and which of them a press picks. Its header carries why the rung is a
/// seam rather than an arbitrary cut.
mod node_rung;

use egui::{Pos2, Rect};
use pdfcer_core::page_tree::Page;
use pdfcer_core::vector::{
    Bounds, FormMarquee, HitTarget, MarqueeMode, Matrix, PageObjects, Point, VectorObject,
    decompose_page, hit_test_point_deep, hit_test_rect_deep,
};
use pdfcer_core::view::DocumentView;
use pdfcer_render::page_device_geometry;

use crate::objectsummary::{ObjectSummary, describe_object};
use pdfcer_render::tiny_skia::{Point as SkPoint, Transform};

/// One selectable thing on a page, addressed opaquely — and **which of the
/// two index spaces it lives in**.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TargetId {
    /// An index into [`PageObjects::objects`] — the page's own paint order.
    /// Editable by the paint-order verbs.
    Object(u64),
    /// An index into [`PageObjects::leaves`] — an object painted from inside
    /// a form XObject.
    ///
    /// Selectable, measurable and reportable. **Not** editable by the
    /// paint-order verbs — see the type docs — and that is a statement
    /// about the INDEX SPACE, which is what this type is for.
    ///
    /// It is **not** a statement about editability. `is_editable` on a leaf
    /// means *"this leaf is a path"*, and a leaf reached through this variant
    /// is edited by the **form-scoped** verbs — `move_objects_in_form`,
    /// `move_subpath_in_form`, `move_node_in_form`, `move_nodes_in_form`,
    /// `move_handle_in_form`, `delete_objects_in_form` — every one of which
    /// this shell calls.
    ///
    /// [`Self::page_object_index`] returning `None` guards one thing only:
    /// *do not hand a leaf index to a paint-order verb*. It does not mean *do
    /// not edit this*, and reading it that way withholds working verbs.
    Leaf(u64),
}

impl TargetId {
    /// The index an [`pdfcer_core::edit::EditSession`] paint-order verb will
    /// accept — `None` for a leaf.
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
    /// Held for exactly one question:
    /// `pdfcer_gui::canvas::target::CanvasTargetProvider::container_is_worth_selecting`,
    /// which needs to know whether a form covers the whole sheet. It is
    /// `page_device_geometry(page, 1.0)`'s first two returns, kept alongside
    /// the transform this provider is really built from.
    ///
    /// `None` makes that predicate answer `true`, which is the behaviour
    /// before it existed. A provider that cannot measure must not guess.
    page_extent_px: Option<egui::Vec2>,
    /// Each object's [`describe_object`] answer, filled on first ask. A
    /// summary walks every anchor of a path, and one path can hold a whole
    /// drawing, so the panels must not recompute it every frame.
    summaries: Vec<std::sync::OnceLock<ObjectSummary>>,
    /// Recent [`Self::hit_test_all`] questions and answers, newest last. A hit
    /// test walks every subpath of every path, and one press asks it at two
    /// tolerances, again on every frame of the drag that follows.
    recent_hits: std::sync::Mutex<Vec<(HitKey, Vec<TargetId>)>>,
}

/// How many [`ObjectModelProvider::recent_hits`] are kept.
const RECENT_HITS: usize = 4;
/// An engine hit test at least this slow is traced as `hit-test ms=`.
const HIT_TRACED_MS: u128 = 5;

/// A point hit-test's inputs, compared bit for bit.
type HitKey = (usize, u32, u32, u64);

/// Which KIND of part the "Part" rung is standing on for a given object.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartKind {
    /// A subpath of a path object.
    Subpath,
    /// One visual **line** of a text object — which is however many show
    /// operators its producer wrote it as. `provider::line` carries the
    /// measurement and the argument.
    TextLine,
}

/// **Why moving one line of a text object would be refused**, asked before the
/// gesture rather than after it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunMoveBlock {
    /// The run takes its origin from the previous run's advance (9.4.2), so
    /// there is no operand to rewrite. `TextRunHasNoPositionOfItsOwn`.
    NoPositionOfItsOwn,
    /// The run AFTER this one takes its origin from this one's advance, so
    /// moving this one would drag that one along. `MoveWouldMoveNextRun`.
    WouldMoveNextRun,
    /// The index is not a run of this object at all — a stale selection, or
    /// an object that stopped being text under an edit.
    NotThere,
}

impl ObjectModelProvider {
    /// Build a provider for `page` (at `page_index`) from `view`.
    #[must_use]
    pub fn build(view: &DocumentView<'_>, page: &Page, page_index: usize) -> Option<Self> {
        Self::build_or_reason(view, page, page_index).ok()
    }

    /// [`Self::build`], keeping the reason the page would not decompose.
    pub fn build_or_reason(
        view: &DocumentView<'_>,
        page: &Page,
        page_index: usize,
    ) -> Result<Self, String> {
        // **TIMED, because this shell measures its own loop rather than
        // inheriting the engine's numbers.**
        //
        // The engine's measurement of `decompose_page` says the decode is
        // roughly three quarters of the cost and the decomposition the
        // remaining quarter — which is a statement about the engine's side, not
        // about how often this shell asks for one.
        //
        // ⇒ So the line carries **what was built** as well as how long: a
        // rebuild that produced no leaves is a different event from a slow one,
        // and a full object list with the deep-selection model silently missing
        // is a real failure mode that a duration alone cannot show.
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
            summaries: unfilled(&objects),
            recent_hits: std::sync::Mutex::new(Vec::new()),
            objects,
            to_canvas,
            to_pdf: to_canvas.invert(),
            // At scale 1.0 these ARE the page's canvas-space extent, which is
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
    #[doc(hidden)]
    pub fn from_parts(page_index: usize, objects: PageObjects, to_canvas: Transform) -> Self {
        Self {
            page_index,
            summaries: unfilled(&objects),
            recent_hits: std::sync::Mutex::new(Vec::new()),
            objects,
            to_canvas,
            to_pdf: to_canvas.invert(),
            // Headless tests construct from parts and have no page. `None`
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
    #[must_use]
    pub fn page_index(&self) -> usize {
        self.page_index
    }

    /// The current page's decomposed vector objects.
    #[must_use]
    pub fn page_objects(&self) -> &PageObjects {
        &self.objects
    }

    /// [`describe_object`] of object `index`, computed once per provider.
    #[must_use]
    pub fn summary(&self, index: usize) -> Option<&ObjectSummary> {
        let object = self.objects.objects.get(index)?;
        Some(
            self.summaries
                .get(index)?
                .get_or_init(|| describe_object(object)),
        )
    }

    /// Which subpath of `object` a canvas-space click lands on — the second
    /// selection level, for objects that hold a whole drawing.
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
            Some(VectorObject::Text(_)) => Some(PartKind::TextLine),
            _ => None,
        }
    }

    /// Which part of `object` a canvas-space click lands on — **whichever
    /// kind of part that object has**.
    #[must_use]
    pub fn part_hits(&self, object: usize, point: Pos2, tolerance: f64) -> Vec<usize> {
        match self.part_kind(object) {
            Some(PartKind::Subpath) => self.subpath_hits(object, point, tolerance),
            Some(PartKind::TextLine) => self.text_line_hits(object, point, tolerance),
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
            Some(PartKind::TextLine) => self.text_line_bounds_canvas(object, part),
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
            Some(PartKind::TextLine) => self.text_line_count(index),
            None => 0,
        }
    }

    /// Which **run** (show operator) of the text object at `object` a
    /// canvas-space click lands on — the text-side twin of
    /// [`Self::subpath_hits`].
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
    #[must_use]
    pub fn text_run_bounds_canvas(&self, object: usize, run: usize) -> Option<Rect> {
        let Some(VectorObject::Text(t)) = self.objects.objects.get(object) else {
            return None;
        };
        self.pdf_bounds_to_canvas(t.runs.get(run)?.bounds)
    }

    /// Whether deleting run `run` of text object `object` would be refused
    /// because the run AFTER it has no position of its own (§9.4.2).
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
    #[must_use]
    pub fn subpath_bounds_canvas(&self, object: usize, subpath: usize) -> Option<Rect> {
        let b = pdfcer_core::vector::subpath_bounds(&self.objects, object, subpath)?;
        self.pdf_bounds_to_canvas(b)
    }

    /// The page-space anchor sample points of the object at paint-order
    /// `index` — the circular best-fit tool's fit input.
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
    #[must_use]
    pub fn subpath_count(&self, index: usize) -> usize {
        match self.objects.objects.get(index) {
            Some(VectorObject::Path(path)) => path.subpaths.len(),
            _ => 0,
        }
    }

    // -----------------------------------------------------------------
    // The canvas target-provider surface.
    //
    // These are inherent methods, and `canvas::target` attaches
    // `CanvasTargetProvider` to them in one `impl` block that forwards. The
    // split is deliberate: the geometry and its tests live here, beside the
    // decomposition they read, and the trait stays a seam rather than a home.
    // -----------------------------------------------------------------

    /// Every target under the pointer, **front-most first**, *including what
    /// is painted inside form XObjects*.
    #[must_use]
    pub fn hit_test_all(&self, page_index: usize, point: Pos2, tolerance: f64) -> Vec<TargetId> {
        if page_index != self.page_index {
            return Vec::new();
        }
        let key = (
            page_index,
            point.x.to_bits(),
            point.y.to_bits(),
            tolerance.to_bits(),
        );
        let mut recent = self
            .recent_hits
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some((_, answer)) = recent.iter().find(|(asked, _)| *asked == key) {
            return answer.clone();
        }
        let Some(pdf) = self.canvas_to_pdf(point) else {
            return Vec::new();
        };
        let started = std::time::Instant::now();
        let answer: Vec<TargetId> = hit_test_point_deep(&self.objects, pdf, resolve(tolerance))
            .into_iter()
            .map(|hit| match hit {
                HitTarget::Object(i) => TargetId::Object(i as u64),
                HitTarget::Leaf(i) => TargetId::Leaf(i as u64),
            })
            .collect();
        let took = started.elapsed().as_millis();
        if took >= HIT_TRACED_MS {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            crate::diag::trace(|| format!("hit-test page={page_index} ms={took}"));
        }
        if recent.len() == RECENT_HITS {
            recent.remove(0);
        }
        recent.push((key, answer.clone()));
        answer
    }

    /// The page paint-order index of the **outermost form** enclosing a
    /// form-interior target - the *"select the container"* act.
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
    #[must_use]
    pub fn hit_test(&self, page_index: usize, point: Pos2, tolerance: f64) -> Option<TargetId> {
        self.hit_test_all(page_index, point, tolerance)
            .into_iter()
            .next()
    }

    /// Every object a canvas-space marquee rect takes, under `mode` and
    /// `forms`.
    #[must_use]
    pub fn hit_test_rect(
        &self,
        page_index: usize,
        rect: Rect,
        mode: MarqueeMode,
        forms: FormMarquee,
    ) -> Vec<TargetId> {
        if page_index != self.page_index {
            return Vec::new();
        }
        let Some(bounds) = self.canvas_rect_to_pdf_bounds(rect) else {
            return Vec::new();
        };
        // One call, one enclosure rule, one ordering. The mapping below is
        // the only thing this shell still owns about a marquee: the engine
        // answers in its own index spaces and this turns them into the two
        // `TargetId` variants the selection vocabulary uses. It is the same
        // mapping `hit_test_all` performs for a click, written the same way,
        // because they are the same two spaces.
        hit_test_rect_deep(&self.objects, bounds, mode, forms)
            .into_iter()
            .map(|hit| match hit {
                HitTarget::Object(i) => TargetId::Object(i as u64),
                HitTarget::Leaf(i) => TargetId::Leaf(i as u64),
            })
            .collect()
    }

    /// One object's canvas-space bounding rect, or `None` for a stale id.
    #[must_use]
    pub fn bounds(&self, page_index: usize, target: TargetId) -> Option<Rect> {
        if page_index != self.page_index {
            return None;
        }
        // Both lists, resolved by the id itself rather than by a caller
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
fn resolve(tolerance: f64) -> f64 {
    if tolerance.is_finite() && tolerance > 0.0 {
        tolerance
    } else {
        FALLBACK_SELECT_TOLERANCE
    }
}

/// One empty summary slot per object.
fn unfilled(objects: &PageObjects) -> Vec<std::sync::OnceLock<ObjectSummary>> {
    objects
        .objects
        .iter()
        .map(|_| std::sync::OnceLock::new())
        .collect()
}
