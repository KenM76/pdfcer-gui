//! # `panels::pages::thumbnails` — what gets drawn, when, and what is kept
//!
//! The Pages panel's **rendering and caching policy**, separated from the
//! panel body so the decisions are readable and testable without an
//! `egui::Context`. The body asks three questions of this module — *what is
//! the state of tile N?*, *what should I draw next?*, *draw it* — and this
//! module owns every answer.
//!
//! ---
//!
//! # ★ Small does not mean fast. The whole design follows from that.
//!
//! The obvious mental model of a thumbnail grid — *200 pages, 200 cheap
//! little pictures* — is **wrong on this application's flagship document**,
//! and `BENCHMARK.md` measured how wrong:
//!
//! | Render | Pixels | Cost |
//! |---|---:|---:|
//! | full page, scale 1 | 1,002,822 | 877 ms |
//! | a 400 × 300 pt region | 120,701 | 699 ms |
//! | **a 1 × 1 POINT region** | **2** | **691 ms** |
//!
//! **A two-pixel render costs 691 ms.** On the benchmark CAD drawing ~99 % of
//! the cost is *resolution-independent*: 148,517 content-stream operators
//! walked through a sequential state machine at ~5 µs each, and the pixel
//! fill is the small remainder. Scaling a thumbnail down to a postage stamp
//! saves the fill and nothing else.
//!
//! So a 200-page drawing set is not 200 cheap thumbnails. It is potentially
//! **two and a half minutes** of interpretation, and any design that starts
//! it eagerly has hung the application.
//!
//! Two further measurements shape what is *not* done here:
//!
//! - **Tiling is cancelled**, not deferred. `BENCHMARK.md`'s ceiling section
//!   records a 3 × 3 ring of regions as a **9× regression**, for the reason
//!   in the table above.
//! - **A display list is the real fix** and is not this panel's to build. It
//!   would make the second and every later render of a page nearly free, and
//!   it lands in `pdfcer-render`.
//!
//! ---
//!
//! # The policy, stated
//!
//! | Question | Answer |
//! |---|---|
//! | **What is rendered?** | one page, at [`THUMBNAIL_WIDTH_PTS`] wide, with annotations painted and no layer override — the reader's view of the sheet |
//! | **When?** | at most **one page per frame**, and only for a tile the operator can actually see |
//! | **In what order?** | the **current page first** if it is on screen, then the visible tiles in reading order |
//! | **What is cached?** | the uploaded texture, keyed by the page's [`RenderKey`], up to [`MAX_CACHED_THUMBNAILS`] of them |
//! | **What is evicted?** | the cached page **furthest from the middle of what is on screen** |
//! | **What does an undrawn tile show?** | *words* — see [`TileState`] and [`crate::text::pages`] |
//! | **When does a page get skipped?** | when it exceeds the operator's own per-page time limit — [`ThumbnailCache::budget`], default [`PAGE_BUDGET_DEFAULT`]. That page alone is abandoned; the grid carries on with the next one |
//! | **When does the feature switch itself off?** | **never** — see "the skipping rule" below |
//!
//! ## What that policy does on real documents — measured, by driving the
//! binary
//!
//! `PDFCER_DIAG=1 pdfcer-gui <file>`, release build, reading the
//! `pages-thumbnail` and `pages-panel` trace lines:
//!
//! | Document | What happened |
//! |---|---|
//! | `SW41177.pdf` — 36 SolidWorks sheets | 12 tiles visible, 12 drawn, one per frame in 61 · 222 · 48 · 52 · 33 · 31 · 31 · 31 · 32 · 32 · 33 · 33 ms. Then `drawn=12` and **nothing further scheduled** — the other 24 pages were never touched. |
//! | `ncored-benchmark-cad-drawing.pdf` — 1 sheet | page 1 drew in **921 ms**, tripped the then-400 ms `SLOW_PAGE`, and the panel reported `previews=0` on the same frame. |
//!
//! ⚠ **The second row's outcome no longer happens, and the measurement is
//! kept because the number is still true.** 921 ms is what that page
//! costs; what changed on 2026-09-08 is what pdfcer does about it —
//! nothing, because 921 ms is inside the default budget. The row is left
//! standing rather than deleted so the next person to raise
//! [`PAGE_BUDGET_DEFAULT`]'s justification has the evidence in front of
//! them; see "the skipping rule" below for why the global stop went.
//!
//! ## ★ Why this renders on the UI thread, when a cancellable off-thread
//! worker already exists
//!
//! `crate::render::worker::RenderWorker` is the right tool and this module
//! does not use it. That is a finding, not an oversight, and it is recorded
//! here because the next person to look at this file will reach for the
//! worker within a minute.
//!
//! A [`crate::render::worker::RenderRequest`] carries
//! `session: Arc<EditSession>`, and the worker thread holds that clone **for
//! the whole render** — which is the point: it is what lets the borrow cross
//! the thread boundary. `crate::app::state::OpenDoc::session`'s own docs spell
//! out the consequence:
//!
//! > every future mutation must go through a path that first calls
//! > `RenderWorker::cancel_and_wait` — `Arc::get_mut` fails while a render is
//! > running
//!
//! This build has exactly two such paths — `crate::app::actions`' private
//! `vector_edit` and `crate::panels::forms::edit::apply` — and **both cancel
//! only `OpenDoc::render_worker`.** A *second* worker, owned by
//! [`crate::panels::PanelsState`], is invisible to both. So a thumbnail in
//! flight when the operator pressed Delete, dragged an object, or typed into
//! a form field would make `Arc::get_mut` return `None`, and the edit would
//! be **traced as `reason=session-borrowed` and silently declined**.
//!
//! On the benchmark drawing that hazard window is not a millisecond. It is
//! ~0.74 s per page for every page the operator scrolls past — tens of
//! seconds of a document that quietly refuses to be edited, on the exact
//! documents this application exists for.
//!
//! Rendering inline instead means the `session.view()` borrow lives and dies
//! inside one function call on the UI thread. **No `Arc` clone ever escapes
//! the frame**, so the mutation choke point is untouched and the hazard
//! cannot arise. The price is a frame hitch, which is what
//! [`ThumbnailCache::budget`] exists to bound — and, since 2026-09-08, what
//! the operator rather than this module gets to put a number on.
//!
//! **What would close this properly** is one of:
//!
//! 1. moving a second `RenderWorker` onto `OpenDoc` beside `render_worker`,
//!    so the existing `cancel_and_wait` calls can reach it (`app/state.rs`,
//!    `app/actions.rs`, `panels/forms/edit.rs`); or
//! 2. `BENCHMARK.md`'s own recommendation 2 — a **thread pool** for
//!    thumbnails and adjacent-page prerender — which needs the same
//!    cancellation reach and would then fill the grid in proportion to core
//!    count, because pages are independent of each other.
//!
//! Neither is this panel's to build, and doing half of either would ship the
//! silent-refusal defect.
//!
//! ## Why one page per frame rather than the old shell's two
//!
//! The old shell used `THUMBNAILS_PER_FRAME = 2` (`main.rs:392`) with no time
//! bound at all. Two is twice the worst-case hitch for the same throughput,
//! and throughput is not what a thumbnail grid is short of — an operator
//! reads a rail one screenful at a time. One page per frame means the window
//! repaints, the scroll responds, and the operator can turn previews off,
//! *between* every page.
//!
//! ## ★★★ The skipping rule — per page, and the checkbox NEVER moves itself
//!
//! **This replaced an automatic global stop on 2026-09-08, at the operator's
//! instruction** (`OPERATOR_REQUESTS.md` O151):
//!
//! > *"the drawing page previews checkbox should never automatically turn
//! > off. You can add a box next to the checkbox to enter a timeout value."*
//!
//! What was there before: a page costing more than a hard-coded 400 ms set
//! `ThumbnailCache::slow`, **the checkbox went from ticked to unticked**, and
//! no further page was drawn until the operator ticked it again. The
//! reasoning was sound as far as it went — the operator cannot know in
//! advance which of their documents is the expensive kind, pdfcer can after
//! one page, and it said so rather than going quiet.
//!
//! ⇒ **What that reasoning missed is that it wrote its conclusion into the
//! operator's own control.** A checkbox is a record of an instruction. When
//! pdfcer clears it, the operator's next glance at the panel reads *"I must
//! have turned that off"*, and there is no state left that distinguishes
//! *they chose this* from *pdfcer chose this for them*. The old
//! `forced: Option<bool>` existed entirely to paper over that collision, and
//! its own doc comment admitted the shape of the problem — three states for a
//! two-state control.
//!
//! The rule now:
//!
//! - **The tick is the operator's, exclusively.** Nothing in this module
//!   writes it. [`ThumbnailCache::previews_on`] is a plain field read.
//! - **The cost is bounded per page, not per document.** Each render is armed
//!   with a [`RenderCancel`] at [`ThumbnailCache::budget`]; a page that
//!   exceeds it is abandoned, shows [`TileState::Abandoned`], and **the grid
//!   moves on to the next page**. One expensive sheet in a set of thirty-six
//!   no longer costs the other thirty-five their pictures — which is strictly
//!   better than the old rule even before the operator touches anything.
//! - **The budget is the operator's number**, typed into the box beside the
//!   checkbox, clamped to [`MIN_PAGE_BUDGET`]..=[`MAX_PAGE_BUDGET`]. Their
//!   machine, their choice — what pdfcer owes them is the measurement, and
//!   [`crate::text::pages::previews_budget_tooltip`] carries it.
//! - **Raising the budget retries what the old one skipped**
//!   ([`ThumbnailCache::set_budget`] drops every [`Unavailable::Abandoned`]
//!   entry). A dial that could only ever remove pictures would be a trap: the
//!   operator would raise it, see no change, and conclude it did nothing.
//! - **Stated, not silent** — [`crate::text::pages::previews_skipped_note`]
//!   names the page, what it was given, and the box that changes it.
//!
//! ⚠ The price, stated plainly because it is the operator's to pay now: on a
//! document like the benchmark drawing, twelve visible tiles at ~0.92 s each
//! is ~11 s of UI-thread work, spread one page per frame with the window
//! repainting between each. It is a slow grid, not a frozen one, and it was
//! **precisely the thing the old rule bought by unticking the box**. The box
//! is how the operator buys it back.

use std::collections::HashMap;
use std::sync::mpsc::{RecvTimeoutError, channel};
use std::time::{Duration, Instant};

use pdfcer_core::page_tree::Page;
use pdfcer_render::cancel::RenderCancel;

use crate::app::state::OpenDoc;
use crate::render::raster::{PageTexture, texture_from_pixels};
use crate::render::worker::{RenderKey, RenderedPixels};

/// How wide a thumbnail is rasterized, in PDF points.
///
/// Carried over from the old shell's `raster::THUMBNAIL_WIDTH_PTS`, and the
/// number is a *raster* width rather than a *layout* width. The grid draws
/// tiles at whatever size the dock gives it and lets the texture scale
/// (`LINEAR`, via [`texture_from_pixels`]), so **resizing the dock does not
/// re-rasterize anything**.
///
/// That distinction is worth more here than it was there. Re-rendering on
/// resize would mean a drag of the dock splitter costing ~0.74 s per visible
/// tile per frame on a dense drawing — a resize gesture that freezes the
/// application, from a panel whose entire design is about not doing that.
///
/// 140 pt at a typical 1.5–2× `pixels_per_point` is a 210–280 px picture,
/// which is enough to recognise a title block and a sheet layout by. Larger
/// costs fill time (the *only* part of the render that scales) and memory;
/// smaller stops being recognisable, which is the one job a thumbnail has.
pub const THUMBNAIL_WIDTH_PTS: f32 = 140.0;

/// **The default per-page time limit** — what [`ThumbnailCache::budget`]
/// holds until the operator types a different number.
///
/// Two seconds, and the number is the old `RENDER_CEILING`'s, unchanged,
/// because its justification survived the 2026-09-08 rewrite intact even
/// though the rule around it did not.
///
/// The measurements it was chosen against are this panel's own —
/// [`tests::thumbnail_cost_on_the_benchmark_documents`] re-runs them.
/// **Release build, 280 px-wide thumbnails, one core:**
///
/// | Document | Page | Thumbnail |
/// |---|---|---:|
/// | `ncored-benchmark-cad-drawing.pdf` | 1 | **918 ms** |
/// | `SW41177.pdf` (36-sheet SolidWorks set) | 2 | 238 ms |
/// | `SW41177.pdf` | 1, 3, 4 | 58–72 ms |
/// | `fixtures/a1-titleblock.pdf` | 1 | 9 ms |
/// | `pageops/four-pages.pdf` | 1–4 | < 1 ms |
///
/// ⇒ **Two seconds is deliberately well clear of the worst real page.** A
/// default that abandoned a render which would have finished converts a slow
/// picture into no picture, and that is a worse answer than the wait — so on
/// every document measured here, the default budget draws **everything**.
/// What it bounds is the page worse than anything measured: `BENCHMARK.md`
/// records ~10 s at 1× and ~58 s at 2× for a full-size CAD raster, nothing in
/// the format bounds a thumbnail's cost, and a page with ten times the
/// operator count would otherwise freeze the application for the better part
/// of a minute.
///
/// ★ The 400 ms that used to live beside this — `SLOW_PAGE` — was a
/// *different kind of number*: the point at which pdfcer stopped and asked.
/// It is gone with the rule it served, and the empty measured band it sat in
/// (72 ms … 238 ms of real work, then 918 ms) is preserved above for whoever
/// next argues about the default.
///
/// The mechanism, which is real rather than nominal: every render is armed
/// with a [`RenderCancel`] and a one-shot watchdog thread that trips at the
/// budget. `pdfcer-render` polls the token **between content-stream
/// operators**, and its own docs put the worst-case latency at one operation
/// — ~360 µs for the most expensive kind measured.
///
/// A render that trips it is [`Unavailable::Abandoned`], which is *not* a
/// failure: nothing is wrong with the page, and the tile says so in those
/// terms.
pub const PAGE_BUDGET_DEFAULT: Duration = Duration::from_secs(2);

/// The smallest budget the operator may set.
///
/// A tenth of a second. Below this the dial stops being a time limit and
/// becomes an off switch wearing a number: `SW41177.pdf`'s *cheapest* sheets
/// cost 58–72 ms, so 50 ms would abandon an ordinary drawing-office sheet
/// set wholesale while the checkbox still read "on" — which is the exact
/// confusion between *pdfcer decided* and *the operator decided* that this
/// whole rewrite exists to remove. An operator who wants no previews has a
/// checkbox for it, one control to the left.
pub const MIN_PAGE_BUDGET: Duration = Duration::from_millis(100);

/// The largest budget the operator may set.
///
/// A minute. Not a performance judgement — it is the point past which a
/// *single* frame hitch stops being distinguishable from a hang, and an
/// application that appears hung is one the operator kills. Anyone who
/// genuinely wants an unbounded render has the canvas, which is where a
/// full-size raster of that page belongs anyway.
pub const MAX_PAGE_BUDGET: Duration = Duration::from_secs(60);

/// How many uploaded thumbnails are kept at once.
///
/// The arithmetic, because a texture cache with no stated size is a leak
/// waiting to be discovered: a 140 pt tile at `pixels_per_point` 2 is 280 px
/// wide, so a portrait A4 is 280 × 396 px = 443 KB of RGBA and a landscape
/// A1 is 280 × 198 px = 222 KB. Sixty-four of the larger kind is ~28 MB of
/// GPU memory, which is a reasonable standing cost for a panel that is
/// usually open and is dwarfed by a single full-size page raster at high
/// zoom.
///
/// It also bounds something less obvious. Every entry here holds an
/// `egui::TextureHandle`, and a cache that grew without limit over a
/// 900-page document would hold 900 live textures — a number egui will
/// accept and a driver may not.
pub const MAX_CACHED_THUMBNAILS: usize = 64;

/// Why a page has no picture, when the reason is not "not yet".
///
/// Recorded rather than retried, and the distinction between the two
/// variants is what a tile says. A failure is deterministic — same bytes,
/// same code — so retrying it every frame would peg a core to produce the
/// same error sixty times a second.
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
///
/// Returned by [`ThumbnailCache::state`] so the body has one `match` over a
/// closed set rather than three `Option` lookups whose combinations it would
/// have to reason about. Every variant that is not [`Self::Ready`] carries a
/// **sentence** in [`crate::text::pages`] — see this module's header on why a
/// blank rectangle is not an option.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileState {
    /// A picture exists; draw it.
    Ready,
    /// Queued, and previews are on: a picture is coming.
    NotDrawnYet,
    /// Queued, and previews are off: a picture is *not* coming until the
    /// operator says so. Deliberately distinct from [`Self::NotDrawnYet`] —
    /// waiting for something that will never arrive is being misled by a
    /// word.
    PreviewsOff,
    /// The render hit [`ThumbnailCache::budget`] and was abandoned.
    ///
    /// ★ Not a failure and not a stop. The next page is drawn normally;
    /// only this one has no picture, and raising the budget brings it
    /// back ([`ThumbnailCache::set_budget`]).
    Abandoned,
    /// The renderer refused the page.
    Failed,
}

/// **The most recent page the budget skipped**, and how long it was given.
///
/// ⚠ Renamed in meaning on 2026-09-08 without changing shape, so read the
/// field docs rather than the type name: `millis` used to be *how long the
/// page took* (a completed render, measured after the fact) and is now *how
/// long it was allowed* before being abandoned. The page's true cost is
/// unknown by construction — pdfcer stopped it precisely so as not to find
/// out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkippedPage {
    /// Which page (0-based).
    pub page_index: usize,
    /// The budget it exceeded, in milliseconds.
    pub millis: u128,
}

/// Every thumbnail this panel has, and the policy that fills it.
///
/// Lives on [`crate::panels::PanelsState`], which is application-scoped, so
/// it outlives a document — and is therefore **forgotten** rather than keyed:
/// `PanelsState::forget_document` runs `*self = Self::default()` from the one
/// place a document is opened. Within one document, [`Self::sync`] drops
/// everything when the edit revision or the display density changes. Between
/// those two there is no identity comparison anywhere, which is the same
/// posture `PanelsState`'s own header argues for and the same reason the old
/// `DocKey` was deleted rather than repaired.
pub struct ThumbnailCache {
    /// The uploaded pictures, by 0-based page index.
    ready: HashMap<usize, PageTexture>,
    /// The pages that have no picture and are not waiting for one.
    unavailable: HashMap<usize, Unavailable>,
    /// The **pixels-per-point bits** everything above describes, or `None`
    /// before the first frame.
    ///
    /// ★★★ **The edit epoch LEFT this key on 2026-08-31** —
    /// `OPERATOR_REQUESTS.md` O74, the operator: *"all of the page previews
    /// get re-rendered instead of just the one that is being changed."* It was
    /// a **document-wide** counter used as the invalidation key for a cache
    /// holding one entry **per page**, so an edit to sheet 12 threw away the
    /// pictures of the other thirty-five. Measured on his own 36-sheet
    /// SolidWorks set: twelve visible tiles, **666 ms of UI-thread work per
    /// edit**, worst frame 282 ms — all of it between his click and its result.
    /// The per-page answer now lives in [`Self::built_at`], compared against
    /// [`crate::app::state::pageepoch::PageEpochs`].
    ///
    /// **The page index was never in it, and still is not.** A page change
    /// moves the highlight ring; it changes no picture, and dropping the cache
    /// on it would re-rasterize the whole visible grid every time the operator
    /// pressed Page Down.
    ///
    /// `pixels_per_point` **stays**, alone, and stays document-wide — because
    /// it genuinely is. It is a factor of the raster scale, so dragging the
    /// window to a monitor with a different density leaves **every** texture at
    /// the wrong resolution, and the symptom is a grid that is soft or aliased
    /// with nothing to say why. A per-page density does not exist.
    key: Option<u32>,
    /// ★ **The page epoch each held entry was built at**, keyed the same way
    /// [`Self::ready`] and [`Self::unavailable`] are.
    ///
    /// One entry per held picture *and* per held refusal — a page that failed
    /// to render is as much a claim about a revision as one that succeeded, and
    /// leaving the refusals unkeyed would mean an edit never got a second
    /// attempt at a page that had failed.
    built_at: HashMap<usize, u64>,
    /// The revisions [`Self::sync`] was last given, so [`Self::insert`] can
    /// stamp a new entry without the render path having to be handed the
    /// document.
    ///
    /// ★ Correct precisely because `sync` runs **once per frame before any
    /// tile is drawn**, which is a contract `sync`'s own docs already state
    /// and which several other things here already depend on. A picture
    /// rendered later in the same frame is a picture of the revision this
    /// snapshot names.
    synced: crate::app::state::pageepoch::PageEpochs,
    /// The most recent page the budget abandoned, if any.
    ///
    /// **A disclosure, not a decision.** Nothing reads this to decide whether
    /// to draw; it exists so the panel can say which page has no picture and
    /// why, per rule 4's *report separately*. Cleared by
    /// [`Self::set_budget`], because a verdict measured against a limit the
    /// operator has since changed is a stale claim.
    skipped: Option<SkippedPage>,
    /// ★★★ **Whether the operator wants previews. Nothing but the operator
    /// writes this.**
    ///
    /// A plain `bool`, and the plainness is the feature. It was
    /// `Option<bool>` until 2026-09-08 — three states for a two-state control
    /// — for one reason: pdfcer also wrote it, so the type had to record
    /// *who last decided*. With the automatic rule gone there is one party,
    /// so there are two states, and "is the box ticked" is the whole of the
    /// question.
    ///
    /// ⚠ If a future change makes this module write this field, the
    /// three-state problem comes straight back and so does the defect the
    /// operator reported. Skip the page, not the feature.
    on: bool,
    /// **The operator's per-page time limit**, from the box beside the
    /// checkbox.
    ///
    /// Held here rather than in `Settings` deliberately, and that is a
    /// limitation worth stating rather than hiding: it is application-scoped
    /// but **not persisted**, so it returns to [`PAGE_BUDGET_DEFAULT`] on the
    /// next launch, exactly as the checkbox does. Persisting it means a
    /// `Settings` field, which is engine territory (`pdfcer-core`), and this
    /// project does not write to the engine — see `PROJECT_PLAN.md`. Filed
    /// rather than smuggled.
    budget: Duration,
    /// The page indices in [`Self::ready`], newest last.
    ///
    /// Kept beside the map only so eviction has a deterministic tie-break;
    /// the victim is chosen by distance from the viewport (see
    /// [`evict_victim`]), and two equally distant pages resolve to the older
    /// one.
    order: Vec<usize>,
}

impl Default for ThumbnailCache {
    /// **Previews on, at the default budget** — hand-written because the
    /// derive cannot express either.
    ///
    /// `#[derive(Default)]` would give `on: false` and `budget: 0 s`, which
    /// is a build that draws nothing and blames a time limit for it. Both are
    /// the kind of default that is only ever discovered by an operator.
    fn default() -> Self {
        Self {
            ready: HashMap::new(),
            unavailable: HashMap::new(),
            key: None,
            built_at: HashMap::new(),
            synced: crate::app::state::pageepoch::PageEpochs::default(),
            skipped: None,
            on: true,
            budget: PAGE_BUDGET_DEFAULT,
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
    /// Drop every entry that no longer describes **its own page's** revision,
    /// or this display.
    ///
    /// Called once per frame before any tile is drawn, so no two tiles can
    /// disagree about which revision they are pictures of.
    ///
    /// **The operator's tick, their budget, and the skip note all survive.**
    /// The first two for the same reason: they are instructions, and an
    /// instruction that evaporates on the next edit was not honoured. The
    /// note survives because an edit to sheet 12 does not make sheet 4 cheap,
    /// and re-deriving that would cost a whole abandoned render to reach a
    /// sentence already on screen.
    ///
    /// ⚠ The pictures themselves do NOT survive their own page's edit — that
    /// is this function's entire job, and the two rules are independent.
    pub fn sync(
        &mut self,
        epochs: &crate::app::state::pageepoch::PageEpochs,
        pixels_per_point: f32,
    ) {
        // 1. The density, which really is document-wide. A change here leaves
        //    every texture at the wrong resolution, so everything goes.
        let density = pixels_per_point.to_bits();
        if self.key != Some(density) {
            self.key = Some(density);
            self.ready.clear();
            self.unavailable.clear();
            self.order.clear();
            self.built_at.clear();
        }

        // 2. The per-page revisions, which are the O74 fix. An entry is kept
        //    only while the page it describes has not moved — so an edit on
        //    sheet 12 drops sheet 12's tile and leaves the rest alone.
        //
        //    ★ An entry with no `built_at` is dropped rather than kept. That is
        //    unreachable today (every insertion stamps one) and it is written
        //    this way round deliberately: the failure mode of "keep what you
        //    cannot date" is showing the operator a picture of content he has
        //    already changed, which rule 4 forbids, and the failure mode of
        //    "drop what you cannot date" is one extra render.
        let stale: Vec<usize> = self
            .built_at
            .keys()
            .copied()
            .chain(self.ready.keys().copied())
            .chain(self.unavailable.keys().copied())
            .filter(|p| self.built_at.get(p) != Some(&epochs.get(*p)))
            .collect();
        for page in stale {
            self.ready.remove(&page);
            self.unavailable.remove(&page);
            self.built_at.remove(&page);
            self.order.retain(|p| *p != page);
        }

        // 3. Remember the revisions this frame is drawing, so an insertion
        //    later in the same frame stamps the right number.
        self.synced = epochs.clone();
    }

    /// Whether a page would be drawn if one were asked for.
    ///
    /// One field read, so "is the control ticked" and "will anything be
    /// drawn" cannot come apart — the panel seeds its checkbox from this and
    /// writes the answer straight back through [`Self::force_on`].
    #[must_use]
    pub fn previews_on(&self) -> bool {
        self.on
    }

    /// The most recent page the budget abandoned, for the panel's note.
    ///
    /// `None` while previews are off, because the sentence it feeds
    /// ([`crate::text::pages::previews_skipped_note`]) names a page that has
    /// no picture *for a specific reason*, and with previews off no page has
    /// one for a much simpler reason the operator already knows. Two
    /// explanations for the same blank tile is one too many.
    #[must_use]
    pub fn skipped(&self) -> Option<SkippedPage> {
        if !self.on {
            return None;
        }
        self.skipped
    }

    /// Record the operator's own instruction about previews.
    ///
    /// The only writer of [`Self::on`], and it is called from exactly one
    /// place — the checkbox.
    pub fn force_on(&mut self, on: bool) {
        self.on = on;
    }

    /// The operator's per-page time limit.
    #[must_use]
    pub fn budget(&self) -> Duration {
        self.budget
    }

    /// **Set the per-page time limit, and give the skipped pages another go.**
    ///
    /// Three things happen, and the second and third are the ones that matter:
    ///
    /// 1. The value is clamped to [`MIN_PAGE_BUDGET`]..=[`MAX_PAGE_BUDGET`].
    ///    The control clamps too, but a control narrower than what the value
    ///    may legally hold silently rewrites it, so the clamp lives on the
    ///    value as well.
    /// 2. **Every [`Unavailable::Abandoned`] entry is dropped**, so the pages
    ///    the *old* budget gave up on are queued again. Without this, raising
    ///    the limit would visibly do nothing — the operator's whole reason for
    ///    raising it is the tile that says "Not finished", and a dial that
    ///    can only ever remove pictures is a trap. `Unavailable::Failed` is
    ///    deliberately left alone: a page the renderer *refused* will be
    ///    refused again, and retrying it every keystroke would cost a render
    ///    per digit typed.
    /// 3. The skip note is cleared, because it quotes a limit that is no
    ///    longer in force.
    ///
    /// ★ Idempotent by design — the panel calls this from a `DragValue` that
    /// reports a change on every pixel of a drag, so an unchanged value must
    /// cost nothing.
    pub fn set_budget(&mut self, budget: Duration) {
        let budget = budget.clamp(MIN_PAGE_BUDGET, MAX_PAGE_BUDGET);
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
    }

    /// What tile `page_index` should draw.
    #[must_use]
    pub fn state(&self, page_index: usize) -> TileState {
        if self.ready.contains_key(&page_index) {
            return TileState::Ready;
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
    ///
    /// The scheduling rule, isolated so it can be asserted without a window.
    /// `visible` is in reading order — the order the tiles are laid out —
    /// and `current` is the page the canvas is showing.
    ///
    /// The current page wins when it is on screen, and the reason is not
    /// politeness. It is the tile carrying the highlight ring, so it is the
    /// one the operator is using to answer *"where am I?"*; a ring around a
    /// tile that says "not drawn yet" answers that question with the page
    /// number alone, which is what they already knew. Everything else fills
    /// in reading order, because a grid that filled in some other order would
    /// look like it was choosing at random.
    ///
    /// Returns `None` when everything visible is settled — which is the
    /// steady state, and the reason this is cheap to call every frame.
    #[must_use]
    pub fn next_to_render(&self, visible: &[usize], current: usize) -> Option<usize> {
        if !self.previews_on() {
            return None;
        }
        let pending = |p: &usize| matches!(self.state(*p), TileState::NotDrawnYet);
        if visible.contains(&current) && pending(&current) {
            return Some(current);
        }
        visible.iter().copied().find(|p| pending(p))
    }

    /// **Rasterize one page and keep the result.**
    ///
    /// The one place this panel renders anything. Runs on the **UI thread**
    /// — see the module header for the `Arc<EditSession>` argument that put
    /// it there — and holds the frame for as long as the page takes, bounded
    /// by [`RENDER_CEILING`].
    ///
    /// Four steps, in this order:
    ///
    /// 1. **Arm the watchdog** at [`Self::budget`] — the operator's number,
    ///    read at the start of each render so a change takes effect on the
    ///    next page rather than on the next document. A one-shot thread that
    ///    cancels the render at the deadline and exits the moment the render
    ///    returns, so no thread outlives the call. `recv_timeout`
    ///    distinguishes *the deadline passed* from *the sender was dropped*,
    ///    which is what makes the disarm free rather than a second message.
    /// 2. **Render**, through `session.view()` — never `session.document()`.
    ///    The view composes the edit overlay, so a thumbnail shows the file
    ///    as *edited*. The old shell shipped the other read for a while and
    ///    recorded what it cost: *"the page rail showed the file AS OPENED
    ///    while the canvas beside it showed the file as EDITED. Two pictures
    ///    of the same page, disagreeing, is worse than the original defect:
    ///    it invites the operator to trust the wrong one."*
    /// 3. **Record the outcome** — a texture, or a reason there is none.
    ///    A render the watchdog cancelled is [`Unavailable::Abandoned`] and
    ///    is also noted in [`Self::skipped`] for the panel's sentence.
    ///
    /// Returns how long the render took, for the caller's trace.
    pub fn render(
        &mut self,
        ctx: &egui::Context,
        doc: &OpenDoc,
        page_index: usize,
        page: &Page,
        pixels_per_point: f32,
        viewport_centre: usize,
    ) -> Duration {
        let scale = raster_scale_for(page, pixels_per_point);

        // ★ Through the funnel, not `RenderOptions::default()`.
        //
        // A thumbnail is a small picture of the same page the canvas draws, so
        // it must obey the same five rendering settings — otherwise an operator
        // who sets "black ink is black" gets a black drawing and a grey rail of
        // thumbnails of it, which reads as a rendering bug rather than as a
        // setting only half applied.
        //
        // What the funnel does NOT decide is on the next two lines, and that is
        // the point of the split: annotations and layers are *this surface's*
        // answer to what a thumbnail is for, and no setting may override them.
        use crate::app::settings::SettingsExt;
        let mut options = doc.settings.render_options();
        // Annotations always on, and the layer override deliberately absent
        // — the two decisions that make a thumbnail a *fixed overview* rather
        // than a second copy of the canvas.
        //
        // A thumbnail answers "which sheet is this?", and the answer must not
        // change because a View ▸ Display toggle was flipped or a layer was
        // hidden to work on something. `None` layers is "obey the document's
        // own default configuration" (core API trap T-12.9), which is what a
        // reader who was handed the file sees, and it is also why these two
        // inputs are absent from the invalidation key in [`Self::sync`]:
        // nothing this panel does can vary them.
        options.annotations = true;
        options.layers = None;
        // ★★ …and `stroke_display` is deliberately NOT set, which leaves the
        // funnel's `StrokeDisplay::Actual` — O137's `view.line_weights` does not
        // reach the rail.
        //
        // Same argument as the two lines above, and one of its own. A thumbnail
        // is a fixed overview, so a View ▸ Display toggle must not change it.
        // And at thumbnail scale the point is moot in the operator's favour:
        // every stroke on a page drawn 90 points wide is already under a device
        // pixel, so the engine's §8.4.3.2 floor has put it at one pixel before
        // the hairline ceiling could. Turning it on here would cost a whole
        // second raster of every visible page for a picture nobody could tell
        // apart.
        //
        // ⚠ It follows that the rail is NOT a place to check what the toggle
        // does. The canvas is.

        let cancel = RenderCancel::new();
        // ★ Copied out before the thread is spawned: `self` is borrowed
        // mutably for the whole of this function, so the closure cannot read
        // the field, and a `Duration` is `Copy`.
        let budget = self.budget;
        // 1. The watchdog. `tx` stays here; dropping it at the end of this
        //    function disconnects the channel, which wakes the thread with
        //    `Disconnected` and exits it without cancelling.
        let (tx, rx) = channel::<()>();
        let watchdog = cancel.clone();
        let guard = std::thread::spawn(move || {
            if matches!(rx.recv_timeout(budget), Err(RecvTimeoutError::Timeout)) {
                watchdog.cancel();
            }
        });
        options.cancel = Some(cancel.clone());

        // 2. The render. The `view()` borrow lives and dies inside this
        //    statement, so no `Arc<EditSession>` clone escapes the frame.
        let started = Instant::now();
        let outcome = {
            let view = doc.session.view();
            pdfcer_render::render_page_with_view(&view, page, scale, &options)
        };
        let elapsed = started.elapsed();
        drop(tx);
        let _ = guard.join();

        // 3. The outcome.
        match outcome {
            Ok(rendered) => {
                let pixels = RenderedPixels {
                    pixmap: rendered.pixmap,
                    diagnostics: rendered.diagnostics,
                    // Built from the inputs this render actually used, through
                    // the same key type the canvas's textures carry, so a
                    // thumbnail is labelled with what produced it rather than
                    // with what was intended. `annotations: true` and
                    // `layers_generation: 0` are the fixed reader defaults set
                    // above; stating them here keeps the key honest rather than
                    // convenient. `StrokeDisplay::Actual` joins them for the
                    // same reason and under the same rule: it is what this
                    // render actually used, stated rather than defaulted into.
                    key: RenderKey::new(
                        page_index,
                        scale,
                        true,
                        0,
                        pdfcer_render::font::StrokeDisplay::Actual,
                    ),
                    // The same measurement this function was already taking for
                    // its own trace, now carried on the value rather than only
                    // written out — so a thumbnail and a canvas raster report
                    // their cost through one field. Nothing reads a thumbnail's
                    // copy today: `tools.render_diagnostics` is about the page
                    // on the canvas, and a strip of twelve pages would need a
                    // surface that says *which* twelve.
                    elapsed,
                };
                self.insert(
                    page_index,
                    texture_from_pixels(ctx, &pixels),
                    viewport_centre,
                );
            }
            // ★ A refusal is stamped too (O74). It is as much a claim about a
            // revision as a picture is, and an unstamped refusal would be
            // dropped by `sync` on every frame — or, worse if the polarity were
            // reversed, would survive the edit that fixed it and the page would
            // never get a second attempt.
            Err(_) if cancel.is_cancelled() => {
                self.unavailable.insert(page_index, Unavailable::Abandoned);
                self.built_at
                    .insert(page_index, self.synced.get(page_index));
                // ★ The disclosure, and the ONLY thing this arm does beyond
                // recording the tile's state. It does not touch `self.on`.
                // That is the whole of O151: a page the budget could not
                // afford is a fact about that page, and the operator's
                // instruction about the feature is none of its business.
                //
                // `budget`, not `elapsed`, because the page's real cost is
                // unknown — pdfcer stopped it precisely so as not to spend it.
                self.skipped = Some(SkippedPage {
                    page_index,
                    millis: budget.as_millis(),
                });
            }
            Err(error) => {
                self.unavailable
                    .insert(page_index, Unavailable::Failed(error.to_string()));
                self.built_at
                    .insert(page_index, self.synced.get(page_index));
            }
        }

        elapsed
    }

    /// Add a texture, evicting if the cache is full.
    fn insert(&mut self, page_index: usize, texture: PageTexture, viewport_centre: usize) {
        if self.ready.len() >= MAX_CACHED_THUMBNAILS
            && let Some(victim) = evict_victim(&self.order, viewport_centre, page_index)
        {
            self.ready.remove(&victim);
            self.order.retain(|p| *p != victim);
        }
        self.ready.insert(page_index, texture);
        // ★ Stamped with the revision `sync` recorded at the top of this frame
        // (O74). Not `epochs.get()` re-read here: this function has no document
        // and giving it one would put a document borrow inside the eviction
        // path for a number that cannot have changed since the frame began.
        self.built_at
            .insert(page_index, self.synced.get(page_index));
        self.order.retain(|p| *p != page_index);
        self.order.push(page_index);
    }

    /// How many pictures are held. For the trace and for tests.
    #[must_use]
    pub fn ready_count(&self) -> usize {
        self.ready.len()
    }
}

/// The raster scale a thumbnail of `page` is drawn at.
///
/// [`THUMBNAIL_WIDTH_PTS`] divided by the page's own width, put through
/// `crate::viewer::raster_scale` so the display's `pixels_per_point` is
/// applied by the same function the canvas uses — one definition of "device
/// pixels per user-space unit" rather than two that can drift.
///
/// A degenerate page — a zero or negative `/CropBox` width, which real files
/// do contain — would otherwise divide to infinity and produce a raster the
/// size guard refuses. Falling back to 1.0 draws the page at its natural size
/// instead, which is wrong-looking and *present*, and the tile is scaled into
/// its box anyway.
///
/// # ★ The rail is pinned to `Normal` quality, and that is a decision
///
/// Every other raster in the application follows the operator's
/// `RenderQuality`. This one does not, and the reason is the same one that
/// keeps annotations and layers out of the thumbnail's invalidation key: **a
/// thumbnail answers "which sheet is this?"**, and that answer must not change
/// with a setting made for a different surface.
///
/// Concretely, both directions are wrong here. `Sharper` multiplies the whole
/// rail — forty tiles, not one page — by 2.25× in pixels, to add detail to an
/// image already scaled down to a hundred points wide, where none of it is
/// resolvable. `Faster` saves almost nothing, because a thumbnail is already
/// the cheapest raster in the program, and buys that nothing at the cost of the
/// one thing a rail has to do: be recognisable at a glance.
///
/// This is not the module second-guessing the operator. It is the same
/// distinction the settings window itself draws between a preference and a
/// property of a surface — and it is stated here rather than being an omission
/// somebody later reads as a missed call site.
#[must_use]
pub fn raster_scale_for(page: &Page, pixels_per_point: f32) -> f32 {
    use crate::app::prefs::RenderQuality;
    let (width, _) = crate::viewer::page_extent_pts(page);
    if width > 0.0 {
        crate::viewer::raster_scale(
            THUMBNAIL_WIDTH_PTS / width,
            pixels_per_point,
            RenderQuality::Normal,
        )
    } else {
        crate::viewer::raster_scale(1.0, pixels_per_point, RenderQuality::Normal)
    }
}

/// **Which cached page to drop to make room for `incoming`.**
///
/// The one eviction rule, as a pure function over the held page indices.
///
/// # Why distance from the viewport rather than least-recently-used
///
/// Both are one line; they differ on the gesture that matters. Scrolling a
/// rail *back up* re-draws tiles that were cached and then dropped — and
/// under LRU those tiles are exactly the ones evicted first, because they
/// were touched longest ago. The operator scrolls down a 200-page set and
/// back, and every tile on the way home is re-rendered at ~0.74 s.
///
/// Distance from the middle of what is on screen keeps the neighbourhood the
/// operator is working in, in both directions, which is the property a rail
/// actually needs.
///
/// Ties break toward the **older** entry (`order` is oldest-first), so an
/// equidistant pair does not evict at random.
///
/// Returns `None` when there is nothing to evict, and never returns
/// `incoming` — evicting the page about to be inserted would be a cache that
/// is permanently full and permanently empty.
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
