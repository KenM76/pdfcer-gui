//! # `panels::pages::thumbnails` — what gets drawn, when, and what is kept
//!
//! The Pages panel's **rendering and caching policy**, separated from the
//! panel body so the decisions are readable and testable without an
//! `egui::Context`. The body asks three questions of this module — *what is
//! the state of tile N?*, *what should I draw next?*, *draw it* — and this
//! module owns every answer.
//!
//! Two passes per picture: a **draft** at [`THUMBNAIL_WIDTH_PTS`], quick on any
//! page, then — once every visible tile has one — a **fine** render at the
//! width the tile is actually drawn, when that is wider than the draft. The
//! draft stays on its tile, stretched, until the fine picture lands; a fine
//! render that fails or runs out of time leaves the draft. So a tile that has
//! a picture never loses it to a resize or a zoom.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/pages/thumbnails.md`.

use std::collections::HashMap;
use std::time::Duration;

use pdfcer_core::page_tree::Page;

use super::viewport_centre;
use crate::app::state::OpenDoc;
use crate::app::state::pageepoch::PageEpochs;
use crate::render::pressure::Surface;
use crate::render::raster::{PageTexture, texture_from_pixels};
use crate::render::worker::{BackgroundResult, RenderRequest};

/// How wide a thumbnail is rasterized, in PDF points.
pub const THUMBNAIL_WIDTH_PTS: f32 = 140.0;

pub use pdfcer_gui_base::pagebudget::{
    MAX_PAGE_BUDGET, MIN_PAGE_BUDGET, PAGE_BUDGET_DEFAULT, budget_from_millis, millis_from_budget,
};

/// How many uploaded thumbnails are kept at once.
pub const MAX_CACHED_THUMBNAILS: usize = 64;

/// How many fine pictures off screen are kept; the furthest goes first. The
/// visible ones are never counted against it, so a wide grid of large tiles
/// cannot evict what it is showing.
pub const MAX_OFFSCREEN_FINE: usize = 16;

/// Fine renders are made at a tile width rounded up to this many points, so
/// dragging the dock splitter does not ask for a new picture every frame.
pub const FINE_STEP_PTS: f32 = 64.0;

/// The resolution a held picture was drawn at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Grade {
    /// [`THUMBNAIL_WIDTH_PTS`] wide.
    Draft,
    /// This many points wide — a tile width rounded up to [`FINE_STEP_PTS`].
    Fine(u32),
}

impl Grade {
    /// The width in points it was drawn at.
    #[must_use]
    pub fn width(self) -> u32 {
        match self {
            #[allow(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "a positive constant of a few hundred points" // ui-text-exempt: clippy lint justification, never displayed
            )]
            Self::Draft => THUMBNAIL_WIDTH_PTS as u32,
            Self::Fine(w) => w,
        }
    }
}

/// Whether a picture drawn at `held` should be redrawn finer for tiles
/// `wanted` points wide (`None`: the draft is wide enough), given the last
/// fine render refused at this revision, `(width)`.
#[must_use]
pub fn wants_finer(held: Grade, wanted: Option<u32>, refused: Option<u32>) -> bool {
    let Some(wanted) = wanted else {
        return false;
    };
    held.width() < wanted && refused.is_none_or(|r| wanted < r)
}

/// Why a page has no picture, when the reason is not "not yet".
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Unavailable {
    /// The render was still going when [`ThumbnailCache::budget`] elapsed.
    ///
    /// Not a defect in the page. Kept as its own variant so the tile can say
    /// *"not finished"* rather than *"would not draw"*, which would blame a
    /// document that is merely large.
    Abandoned,
    /// `pdfcer-render` refused the page. The string is the renderer's own
    /// `Display`, kept for the trace — **not** shown on the tile, which has
    /// room for two words and a different job.
    Failed(String),
}

/// What a tile should draw.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileState {
    /// A picture of the page as it is now; draw it.
    Ready,
    /// A picture of an earlier revision of the same sheet. Drawn until its
    /// replacement lands, so an edit never blanks a tile.
    Stale,
    /// Queued, and previews are on: a picture is coming.
    NotDrawnYet,
    /// Queued, and previews are off: a picture is *not* coming until the
    /// operator says so. Deliberately distinct from [`Self::NotDrawnYet`] —
    /// waiting for something that will never arrive is being misled by a
    /// word.
    PreviewsOff,
    /// The render hit [`ThumbnailCache::budget`] and was abandoned.
    ///
    /// Not a failure and not a stop. The next page is drawn normally;
    /// only this one has no picture, and raising the budget brings it
    /// back ([`ThumbnailCache::set_budget`]).
    Abandoned,
    /// The renderer refused the page.
    Failed,
}

/// **The most recent page the budget skipped**, and how long it was given.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkippedPage {
    /// Which page (0-based).
    pub page_index: usize,
    /// The budget it exceeded, in milliseconds.
    pub millis: u128,
}

/// Every thumbnail this panel has, and the policy that fills it.
pub struct ThumbnailCache {
    /// The uploaded pictures, by 0-based page index.
    ready: HashMap<usize, PageTexture>,
    /// The pages that have no picture and are not waiting for one.
    unavailable: HashMap<usize, Unavailable>,
    /// The pixels-per-point bits every texture was drawn at, or `None` before
    /// the first frame. Document-wide because density is: moving the window
    /// to another monitor leaves every texture at the wrong resolution. The
    /// page index is deliberately not in it; turning a page changes no picture.
    key: Option<u32>,
    /// **The page epoch each held entry was built at**, keyed the same way
    /// [`Self::ready`] and [`Self::unavailable`] are.
    ///
    /// One entry per held picture *and* per held refusal — a page that failed
    /// to render is as much a claim about a revision as one that succeeded, and
    /// leaving the refusals unkeyed would mean an edit never got a second
    /// attempt at a page that had failed.
    built_at: HashMap<usize, u64>,
    /// The revisions [`Self::sync`] was last given; `sync` runs once per frame
    /// before any tile is drawn.
    synced: PageEpochs,
    /// [`PageEpochs::structure`] as of the last sync. A change means page
    /// indices may name different sheets, so every picture goes.
    structure: u64,
    /// The render in the worker's background slot, if this cache started one.
    requested: Option<Requested>,
    /// The most recent page the budget abandoned, if any.
    ///
    /// **A disclosure, not a decision.** Nothing reads this to decide whether
    /// to draw; it exists so the panel can say which page has no picture and
    /// why, per rule 4's *report separately*. Cleared by
    /// [`Self::set_budget`], because a verdict measured against a limit the
    /// operator has since changed is a stale claim.
    skipped: Option<SkippedPage>,
    /// **Whether the operator wants previews. Nothing but the operator
    /// writes this.**
    ///
    ///
    /// ⚠ If a future change makes this module write this field, the
    /// three-state problem comes straight back and so does the defect the
    /// operator reported. Skip the page, not the feature.
    on: bool,
    /// **The operator's per-page time limit**, from the box beside the
    /// checkbox — or `None`, meaning *never give up*.
    ///
    ///
    /// # `Option`, not a zero
    ///
    /// `Duration::ZERO` would have been a sentinel every reader of this field
    /// had to remember; `None` is one the compiler makes them handle. The
    /// operator's `0` is converted at exactly one place,
    /// [`budget_from_millis`], and turned back at [`millis_from_budget`].
    ///
    /// `None` lets a preview run as long as it takes, on the worker.
    budget: Option<Duration>,
    /// The resolution of each picture in [`Self::ready`].
    grade: HashMap<usize, Grade>,
    /// The width fine pictures are wanted at, or `None` while the tiles are
    /// no wider than the draft. Set each frame by [`Self::set_tile_width`].
    fine_width: Option<u32>,
    /// A fine render that failed or ran out of time, by page: the revision
    /// and the width. Not retried at that width or wider for that revision;
    /// the draft stays on the tile.
    fine_refused: HashMap<usize, (u64, u32)>,
    /// The pages on screen at the last [`Self::drive`]: fine pictures of
    /// these are never evicted for [`MAX_OFFSCREEN_FINE`].
    visible: Vec<usize>,
    /// The page indices in [`Self::ready`], newest last.
    ///
    /// Kept beside the map only so eviction has a deterministic tie-break;
    /// the victim is chosen by distance from the viewport (see
    /// [`evict_victim`]), and two equally distant pages resolve to the older
    /// one.
    order: Vec<usize>,
}

impl Default for ThumbnailCache {
    /// **Previews on, with no time limit** — hand-written because the derive
    /// cannot express the first.
    fn default() -> Self {
        Self {
            ready: HashMap::new(),
            unavailable: HashMap::new(),
            key: None,
            built_at: HashMap::new(),
            synced: PageEpochs::default(),
            structure: 0,
            requested: None,
            skipped: None,
            on: true,
            budget: PAGE_BUDGET_DEFAULT,
            grade: HashMap::new(),
            fine_width: None,
            fine_refused: HashMap::new(),
            visible: Vec::new(),
            order: Vec::new(),
        }
    }
}

impl std::fmt::Debug for ThumbnailCache {
    /// Hand-written: `PageTexture`'s own `Debug` is a render key and a
    /// diagnostics report per entry, and this type appears in a trace to
    /// answer *"how full is it"*, never *"what is in it"*.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ThumbnailCache")
            .field("ready", &self.ready.len())
            .field("unavailable", &self.unavailable.len())
            .field("skipped", &self.skipped)
            .field("on", &self.on)
            .field("budget", &self.budget)
            .finish()
    }
}

impl ThumbnailCache {
    /// Bring the cache up to this frame's revisions and display density.
    ///
    /// A density or structure change drops everything: every picture is then
    /// at the wrong resolution or of the wrong sheet. A content edit drops only
    /// the edited pages' refusals (so they get another attempt); their pictures
    /// stay, as [`TileState::Stale`], until the new ones land.
    pub fn sync(&mut self, epochs: &PageEpochs, pixels_per_point: f32) {
        let density = pixels_per_point.to_bits();
        if self.key != Some(density) || self.structure != epochs.structure() {
            self.key = Some(density);
            self.structure = epochs.structure();
            self.ready.clear();
            self.unavailable.clear();
            self.order.clear();
            self.built_at.clear();
            self.grade.clear();
            self.fine_refused.clear();
            self.requested = None;
        }
        // An undated refusal is treated as stale: the cost is one render.
        let dated = |p: &usize| self.built_at.get(p) == Some(&epochs.get(*p));
        let refused: Vec<usize> = self
            .unavailable
            .keys()
            .copied()
            .filter(|p| !dated(p))
            .collect();
        for page in refused {
            self.unavailable.remove(&page);
            if !self.ready.contains_key(&page) {
                self.built_at.remove(&page);
            }
        }
        self.synced = epochs.clone();
    }

    /// Whether the held picture of `page_index` shows its current revision.
    /// An undated picture is never fresh.
    fn fresh(&self, page_index: usize) -> bool {
        self.built_at.get(&page_index) == Some(&self.synced.get(page_index))
    }

    /// Whether a page would be drawn if one were asked for.
    #[must_use]
    pub fn previews_on(&self) -> bool {
        self.on
    }

    /// The most recent page the budget abandoned, for the panel's note.
    #[must_use]
    pub fn skipped(&self) -> Option<SkippedPage> {
        if !self.on {
            return None;
        }
        self.skipped
    }

    /// Record the operator's own instruction about previews.
    pub fn force_on(&mut self, on: bool) {
        self.on = on;
    }

    /// The operator's per-page time limit, or `None` for *never give up*.
    #[must_use]
    pub fn budget(&self) -> Option<Duration> {
        self.budget
    }

    /// **Set the per-page time limit, and give the skipped pages another go.**
    pub fn set_budget(&mut self, budget: Option<Duration>) {
        let budget = budget.map(|d| d.clamp(MIN_PAGE_BUDGET, MAX_PAGE_BUDGET));
        if budget == self.budget {
            return;
        }
        self.budget = budget;
        let retry: Vec<usize> = self
            .unavailable
            .iter()
            .filter(|(_, u)| matches!(u, Unavailable::Abandoned))
            .map(|(p, _)| *p)
            .collect();
        for page in retry {
            self.unavailable.remove(&page);
            self.built_at.remove(&page);
        }
        self.skipped = None;
        self.fine_refused.clear();
    }

    /// The width tiles are drawn at this frame, in points. Fine pictures are
    /// wanted when it is wider than the draft.
    pub fn set_tile_width(&mut self, width_pts: f32) {
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "a positive tile width of at most a window's width in points" // ui-text-exempt: clippy lint justification, never displayed
        )]
        let stepped = ((width_pts / FINE_STEP_PTS).ceil() * FINE_STEP_PTS) as u32;
        self.fine_width =
            (width_pts.is_finite() && width_pts > THUMBNAIL_WIDTH_PTS).then_some(stepped);
    }

    /// The resolution of `page_index`'s picture, if it has one.
    #[must_use]
    pub fn grade(&self, page_index: usize) -> Option<Grade> {
        self.ready
            .contains_key(&page_index)
            .then(|| self.grade.get(&page_index).copied().unwrap_or(Grade::Draft))
    }

    /// The last fine render refused for `page_index` at its current revision.
    fn refused_at(&self, page_index: usize) -> Option<u32> {
        self.fine_refused
            .get(&page_index)
            .filter(|(epoch, _)| *epoch == self.synced.get(page_index))
            .map(|(_, width)| *width)
    }

    /// Whether `page_index`'s picture should be redrawn finer.
    fn finer(&self, page_index: usize) -> bool {
        self.grade(page_index)
            .is_some_and(|held| wants_finer(held, self.fine_width, self.refused_at(page_index)))
    }

    /// **Which visible picture to redraw finer**, once nothing visible is
    /// waiting for a first or a current picture. The current page first.
    #[must_use]
    pub fn next_finer(&self, visible: &[usize], current: usize) -> Option<usize> {
        if !self.previews_on() {
            return None;
        }
        let candidate = |p: &usize| self.state(*p) == TileState::Ready && self.finer(*p);
        if visible.contains(&current) && candidate(&current) {
            return Some(current);
        }
        visible.iter().copied().find(|p| candidate(p))
    }

    /// The grade to draw a page `next_to_render` picked at: fine when it
    /// holds an older picture and fine ones are wanted, so a redrawn tile does
    /// not drop to the draft and climb back.
    fn pending_grade(&self, page_index: usize) -> Grade {
        match self.fine_width {
            Some(w)
                if self.ready.contains_key(&page_index)
                    && self.refused_at(page_index).is_none_or(|r| w < r) =>
            {
                Grade::Fine(w)
            }
            _ => Grade::Draft,
        }
    }

    /// What tile `page_index` should draw.
    #[must_use]
    pub fn state(&self, page_index: usize) -> TileState {
        if self.ready.contains_key(&page_index) {
            return match (self.fresh(page_index), self.previews_on()) {
                (true, _) => TileState::Ready,
                (false, true) => TileState::Stale,
                // An old picture nobody is going to replace is not shown.
                (false, false) => TileState::PreviewsOff,
            };
        }
        match self.unavailable.get(&page_index) {
            Some(Unavailable::Abandoned) => TileState::Abandoned,
            Some(Unavailable::Failed(_)) => TileState::Failed,
            None if self.previews_on() => TileState::NotDrawnYet,
            None => TileState::PreviewsOff,
        }
    }

    /// The texture for `page_index`, if one exists.
    #[must_use]
    pub fn texture(&self, page_index: usize) -> Option<&egui::TextureHandle> {
        self.ready.get(&page_index).map(|t| &t.texture)
    }

    /// **Which page to draw next, out of the ones the operator can see.**
    #[must_use]
    pub fn next_to_render(&self, visible: &[usize], current: usize) -> Option<usize> {
        if !self.previews_on() {
            return None;
        }
        let pending =
            |p: &usize| matches!(self.state(*p), TileState::NotDrawnYet | TileState::Stale);
        if visible.contains(&current) && pending(&current) {
            return Some(current);
        }
        visible.iter().copied().find(|p| pending(p))
    }

    /// **Collect a finished preview, enforce the time limit, and start the
    /// next one.** Never waits for a render; called once per frame after
    /// [`Self::sync`]. `may_start` is false while an edit is settling.
    ///
    /// Returns the page whose render landed this frame, with its cost.
    pub fn drive(
        &mut self,
        ctx: &egui::Context,
        doc: &OpenDoc,
        visible: &[usize],
        current: usize,
        may_start: bool,
    ) -> Option<(usize, Duration)> {
        let worker = &doc.render_worker;
        self.visible.clear();
        self.visible.extend_from_slice(visible);
        let landed = match worker.poll_background() {
            Some(result) => self.land(ctx, result, viewport_centre(visible, current)),
            None => {
                self.watch_budget(worker);
                None
            }
        };
        let next = self
            .next_to_render(visible, current)
            .map(|p| (p, self.pending_grade(p)))
            .or_else(|| {
                let w = self.fine_width?;
                self.next_finer(visible, current)
                    .map(|p| (p, Grade::Fine(w)))
            });
        if may_start
            && self.requested.is_none()
            && let Some((page_index, grade)) = next
            && let Some(page) = doc.pages.get(page_index)
        {
            #[allow(
                clippy::cast_precision_loss,
                reason = "a width in points of at most a few thousand is exact in f32" // ui-text-exempt: clippy lint justification, never displayed
            )]
            let width = grade.width() as f32;
            let scale = raster_scale_at(page, width, ctx.pixels_per_point());
            worker.spawn_background(thumbnail_request(doc, page_index, page, scale));
            self.requested = Some(Requested {
                page: page_index,
                epoch: self.synced.get(page_index),
                structure: self.structure,
                grade,
            });
        }
        if self.requested.is_some() {
            // Woken to poll; a finished render does not repaint the window.
            ctx.request_repaint_after(POLL_EVERY);
        }
        landed
    }

    /// Keep a finished render if it is still a picture of a sheet this cache
    /// holds, stamped with the revision it was requested at.
    fn land(
        &mut self,
        ctx: &egui::Context,
        result: BackgroundResult,
        viewport_centre: usize,
    ) -> Option<(usize, Duration)> {
        let requested = self.requested.take()?;
        let page_index = result.key.page();
        if requested.page != page_index || requested.structure != self.structure {
            return None;
        }
        match result.outcome? {
            Ok(pixels) => {
                let texture = texture_from_pixels(ctx, Surface::Thumbnail, &pixels);
                self.insert(page_index, texture, viewport_centre, requested.epoch);
                self.grade.insert(page_index, requested.grade);
                self.evict_offscreen_fine(viewport_centre);
            }
            // A finer picture refused: the one on the tile stays.
            Err(_) if self.keeps_picture(&requested) => {
                self.fine_refused
                    .insert(page_index, (requested.epoch, requested.grade.width()));
            }
            Err(error) => {
                self.unavailable
                    .insert(page_index, Unavailable::Failed(refusal_text(&error.reason)));
                self.built_at.insert(page_index, requested.epoch);
            }
        }
        Some((page_index, result.elapsed))
    }

    /// Abandon the running preview once it has had the operator's time limit,
    /// or forget it when something else (an edit) cancelled it.
    ///
    /// Abandoning records the page as [`Unavailable::Abandoned`] and discloses
    /// it through [`Self::skipped`]; it never touches [`Self::previews_on`].
    fn watch_budget(&mut self, worker: &crate::render::worker::RenderWorker) {
        let Some(requested) = self.requested else {
            return;
        };
        let Some((_, running_for)) = worker.background_in_flight() else {
            self.requested = None;
            return;
        };
        let Some(budget) = self.budget.filter(|b| running_for >= *b) else {
            return;
        };
        worker.cancel_background();
        self.requested = None;
        if self.keeps_picture(&requested) {
            self.fine_refused
                .insert(requested.page, (requested.epoch, requested.grade.width()));
            return;
        }
        self.unavailable
            .insert(requested.page, Unavailable::Abandoned);
        self.built_at.insert(requested.page, requested.epoch);
        self.skipped = Some(SkippedPage {
            page_index: requested.page,
            millis: budget.as_millis(),
        });
    }

    /// Whether a refused render leaves a picture on the tile: a fine render of
    /// a page that already has one.
    fn keeps_picture(&self, requested: &Requested) -> bool {
        matches!(requested.grade, Grade::Fine(_)) && self.ready.contains_key(&requested.page)
    }

    /// Drop the furthest off-screen fine pictures beyond
    /// [`MAX_OFFSCREEN_FINE`]. The page comes back as a draft when scrolled to.
    fn evict_offscreen_fine(&mut self, viewport_centre: usize) {
        let mut offscreen: Vec<usize> = self
            .grade
            .iter()
            .filter(|(p, g)| matches!(g, Grade::Fine(_)) && !self.visible.contains(p))
            .map(|(p, _)| *p)
            .collect();
        if offscreen.len() <= MAX_OFFSCREEN_FINE {
            return;
        }
        offscreen.sort_by_key(|p| std::cmp::Reverse(p.abs_diff(viewport_centre)));
        let excess = offscreen.len() - MAX_OFFSCREEN_FINE;
        for page in offscreen.into_iter().take(excess) {
            self.ready.remove(&page);
            self.grade.remove(&page);
            self.built_at.remove(&page);
            self.order.retain(|p| *p != page);
        }
    }

    /// Add a texture, evicting if the cache is full.
    fn insert(
        &mut self,
        page_index: usize,
        texture: PageTexture,
        viewport_centre: usize,
        epoch: u64,
    ) {
        if self.ready.len() >= MAX_CACHED_THUMBNAILS
            && let Some(victim) = evict_victim(&self.order, viewport_centre, page_index)
        {
            self.ready.remove(&victim);
            self.grade.remove(&victim);
            self.order.retain(|p| *p != victim);
        }
        self.ready.insert(page_index, texture);
        self.built_at.insert(page_index, epoch);
        self.order.retain(|p| *p != page_index);
        self.order.push(page_index);
    }

    /// How many pictures are held. For the trace and for tests.
    #[must_use]
    pub fn ready_count(&self) -> usize {
        self.ready.len()
    }

    /// How the visible tiles stand, for the `pages-tiles` trace.
    #[must_use]
    pub fn census(&self, visible: &[usize]) -> Census {
        let mut census = Census::default();
        for &page in visible {
            match self.state(page) {
                TileState::Ready => {
                    census.ready += 1;
                    if matches!(self.grade(page), Some(Grade::Fine(_))) {
                        census.fine += 1;
                    }
                }
                TileState::Stale => {
                    census.stale += 1;
                    census.pending += 1;
                }
                TileState::NotDrawnYet => {
                    census.pending += 1;
                    if self.texture(page).is_none() {
                        census.blank += 1;
                    }
                }
                TileState::PreviewsOff | TileState::Abandoned | TileState::Failed => {}
            }
        }
        census
    }
}

/// The visible tiles counted by state. `blank` is a tile drawn with no picture
/// while previews are on; `stale` is one showing a picture older than its page;
/// `fine` is a ready one drawn finer than the draft.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Census {
    pub ready: usize,
    pub fine: usize,
    pub stale: usize,
    pub pending: usize,
    pub blank: usize,
}

/// The page and revision a background render was started for.
#[derive(Debug, Clone, Copy)]
struct Requested {
    page: usize,
    epoch: u64,
    structure: u64,
    grade: Grade,
}

/// How often the rail looks for a finished preview while one is running.
const POLL_EVERY: Duration = Duration::from_millis(30);

/// A preview's render request. Annotations are always drawn, the document's
/// own layer configuration is obeyed, strokes keep their real widths and no
/// detail is culled: a thumbnail answers "which sheet is this?", and no View
/// toggle may change that answer.
fn thumbnail_request(doc: &OpenDoc, page_index: usize, page: &Page, scale: f32) -> RenderRequest {
    RenderRequest {
        session: std::sync::Arc::clone(&doc.session),
        page: page.clone(),
        page_index,
        raster_scale: scale,
        annotations: true,
        stroke_display: pdfcer_render::font::StrokeDisplay::Actual,
        subpixel_culling: false,
        settings: doc.settings.clone(),
        layers: None,
        layers_generation: 0,
        region: None,
    }
}

/// The renderer's own sentence for a refusal, or the refusal's name.
fn refusal_text(reason: &crate::render::worker::RefusalReason) -> String {
    match reason {
        crate::render::worker::RefusalReason::Engine(sentence) => sentence.clone(),
        other => format!("{other:?}"),
    }
}

/// The raster scale a draft thumbnail of `page` is drawn at.
#[must_use]
pub fn raster_scale_for(page: &Page, pixels_per_point: f32) -> f32 {
    raster_scale_at(page, THUMBNAIL_WIDTH_PTS, pixels_per_point)
}

/// The raster scale that draws `page` `width_pts` wide on screen.
#[must_use]
pub fn raster_scale_at(page: &Page, width_pts: f32, pixels_per_point: f32) -> f32 {
    use crate::app::prefs::RenderQuality;
    let (width, _) = crate::viewer::page_extent_pts(page);
    if width > 0.0 {
        crate::viewer::raster_scale(width_pts / width, pixels_per_point, RenderQuality::Normal)
    } else {
        crate::viewer::raster_scale(1.0, pixels_per_point, RenderQuality::Normal)
    }
}

/// **Which cached page to drop to make room for `incoming`.**
#[must_use]
pub fn evict_victim(order: &[usize], viewport_centre: usize, incoming: usize) -> Option<usize> {
    let distance = |p: usize| p.abs_diff(viewport_centre);
    order
        .iter()
        .copied()
        .filter(|p| *p != incoming)
        // `max_by_key` on a distance, over an oldest-first list. Rust's
        // `max_by_key` returns the LAST maximum, which would be the newest of
        // an equidistant set — the opposite of what the tie-break should be —
        // so the comparison carries the reversed position as a tie-break
        // rather than relying on the iterator's choice.
        .enumerate()
        .max_by_key(|(position, page)| (distance(*page), usize::MAX - position))
        .map(|(_, page)| page)
}

#[cfg(test)]
mod tests;
