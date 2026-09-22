//! # viewer — the page-view state machine and its geometry
//!
//! **Salvaged whole from the old GUI's `viewer.rs`** — its documentation and
//! its entire test suite carried across rather than lifted as snippets,
//! because a snippet leaves the reasoning behind and the next engineer
//! re-derives a decision that was already paid for. What came across is the
//! zoom ladder with provable reversibility, the fit modes re-derived per
//! frame, and the per-page raster ceiling that accounts for
//! `pixels_per_point`; each is argued below.
//!
//! Two of the geometry functions here carry `#[allow(dead_code, reason = …)]`
//! because their first consumer has not landed yet. They are kept rather than
//! deleted because each is the *pair* of a live function, and **a bridge with
//! only one direction implemented is how the two ends drift apart.**
//!
//! **A page *range* is not a field here, and its absence is the design.** A
//! range is not something a view *holds*: which pages are on screen falls out
//! of where the pages are laid out and where the viewport is. So [`strip`]
//! computes it and [`ViewState`] keeps exactly one index, meaning *"the page
//! the operator is looking at"* — read off the scroll position under a
//! continuous mode, set by navigation under a paged one. See
//! [`strip::Strip::page_at_view`].
//!
//! The view's fourth axis is [`ViewState::display`]: which of the four
//! arrangements is active. It is orthogonal to the three below and is
//! documented in [`display`].
//!
//! **Anchoring a zoom is not this module's question.** It is a question about
//! the *scroll offset* rather than about the ladder, so the rule and the solve
//! live in [`crate::canvas::zoom`] and [`crate::canvas::geometry`]. Two things
//! here are reused by them rather than reimplemented, and that reuse is the
//! point:
//!
//! * [`fit_scale`] under [`FitMode::Page`] computes the scale that frames a
//!   **region** for zoom-to-selection and marquee-zoom, exactly as it computes
//!   the scale that frames a page. One derivation, so a region zoom and a page
//!   fit cannot disagree about what "fits" means;
//! * [`max_zoom_for_page`] and [`clamp_zoom`] apply the per-page raster
//!   ceiling to a framing zoom. A marquee dragged around a bolt head asks for
//!   a scale no page-sized pixmap can supply, and the answer is the same
//!   answer the zoom buttons give — stop at the ceiling, and let the status
//!   bar's readout state the scale that was actually pinned.
//!
//! ---
//!
//! Everything about "which page am I looking at, and how big is it on
//! screen" lives here, deliberately separated from the egui widget code.
//! The split exists for one concrete reason: **this module is unit-testable
//! and the widget code is not.** A windowed UI cannot be exercised
//! headlessly on a CI runner, but zoom-ladder arithmetic, fit-scale
//! derivation, page-index clamping and the raster-size ceiling are exactly
//! the parts where an off-by-one or a divide-by-zero would show up as a
//! user-visible bug — so they are pure functions with tests, and the widget
//! code is reduced to wiring.
//!
//! ## The view model
//!
//! [`ViewState`] carries three things:
//!
//! - `page_index` — 0-based into the flattened page vector from
//!   [`pdfcer_core::page_tree::pages`]. The UI displays it 1-based; the
//!   conversion happens once, in the string catalog.
//! - `zoom` — the **effective** scale in device pixels per PDF user-space
//!   unit, which is precisely the `scale` argument
//!   [`pdfcer_render::render_page`] takes. `1.0` is 72 DPI, i.e. "actual
//!   size" on a nominal 72-point-per-inch display.
//! - `fit` — whether `zoom` is a value the operator pinned
//!   ([`FitMode::None`]) or one derived from the viewport each frame
//!   ([`FitMode::Page`] / [`FitMode::Width`]). This is a *mode*, not a
//!   one-shot action: "Fit page" that stops fitting the moment the window
//!   is resized is the behaviour every viewer gets right and would be
//!   conspicuous to get wrong.
//!
//! ## Why the zoom ladder is a table, not a multiplier
//!
//! Repeatedly multiplying by, say, √2 produces zoom levels like 141%,
//! 199%, 281% — technically fine, but the operator can never get back to
//! a round number, and two different click sequences that "should" land
//! on 100% land on 99.6% and 100.4% instead. A fixed ladder of familiar
//! percentages ([`ZOOM_LADDER`]) makes zoom-in/zoom-out exactly
//! reversible and always lands somewhere nameable. Zoom values *off* the
//! ladder (from ctrl+scroll, or from a fit mode) are handled by taking
//! the next rung strictly above/below the current value, so the ladder
//! also acts as a "snap back to sanity" mechanism.
//!
//! ## The raster-size ceiling is a real constraint, not a formality
//!
//! `pdfcer-render` refuses to allocate a pixmap with an edge over
//! [`pdfcer_render::MAX_PIXMAP_EDGE`] (16,384 px — the allocation guard). A
//! letter page never comes close, but ISO 32000-1 Annex C permits pages up
//! to 14,400 units on an edge, and such a page hits the ceiling at about
//! 1.1× zoom. Rather than let the operator zoom into an error message,
//! [`max_zoom_for_page`] lowers the ceiling per page and [`ViewState`]
//! clamps against it — the zoom buttons simply stop, which is
//! self-explanatory in a way that "requested raster size 115200x86400 is
//! empty or exceeds MAX_PIXMAP_EDGE" is not.

// Which of the four page-display arrangements is active, the spread rule the
// facing ones use, and the per-mode default that makes Read continuous.
pub mod display;
// Where the per-document choice is written down. Beside the type it persists,
// so the enum and its on-disk spelling cannot drift — see that module's header
// for why it is a third file rather than a field in `layout.ron` or
// `recent.txt`.
pub mod ceiling;
pub mod deep;
/// **Where the view is, when the scroll offset can no longer say** —
/// O24 step 2.
///
/// A scroll offset is `f32` into a content space of `page × zoom`, and one
/// unit of that space is one screen pixel — so at 1,000,000 % the offset can
/// only address every other pixel, and at 10,000,000 % it moves in
/// **sixteen-pixel jumps**. Its header carries the measured table.
///
/// `DeepAnchor` replaces it with a page point in `f64` plus where on screen
/// that point sits, which is a statement whose precision does not decay with
/// the zoom.
/// **How far this page can actually be zoomed** — the three limits that
/// bind at three different depths, reconciled in one place.
///
/// Its header carries which is which: the raster ceiling stops mattering
/// once the region tier engages, the `f32` scroll offset's is what the shell
/// can honestly offer today, and the operator's setting is the third.
/// **The zoom levels the `+` and `−` buttons step through**, and the rule
/// for what happens past the last named rung.
///
/// Split out under R2. Its header carries the one property that matters: the
/// two steps must be exact inverses, **above** the ladder as well as on it —
/// and above it is the half that is easy to get wrong (O24g).
pub mod ladder;
// How the zoom is decided from the viewport: the three fitting modes, the
// ratio each takes, and which axes each one PLACES the view on. Split out
// under R2, this project's 1,500-line-per-file ceiling.
pub mod fit;
pub mod remembered;
// Where every page sits, in one coordinate space. The answer to Phase 4.1's
// "a page range rather than a single index", expressed as geometry.
pub mod strip;

pub use display::PageDisplay;
// Re-exported so every existing `viewer::max_zoom_for_page` /
// `viewer::zoom_ceiling` call site is untouched by the R2 split — the
// module boundary is about file size, not about the vocabulary callers use.
pub use ceiling::{deep_position_needed, max_zoom_with_regions, zoom_ceiling};
// Re-exported for the same reason `ceiling`'s are: the split is about file
// size, not about the vocabulary callers use.
pub use ladder::{ZOOM_LADDER, ladder_step_down, ladder_step_up};
// Re-exported for the same reason `ceiling`'s and `ladder`'s are: the split is
// about file size, not about the vocabulary callers use. Every
// `viewer::FitMode` and `viewer::fit_scale` in the crate is untouched by it.
pub use fit::{FitMode, fit_scale};

use egui::{Pos2, Rect};
use pdfcer_core::page_tree::Page;
use pdfcer_render::tiny_skia::{Point, Transform};

/// Lowest zoom the UI offers: 10%, enough to see a poster-sized page
/// whole.
pub const MIN_ZOOM: f32 = 0.10;

/// Highest zoom the UI offers, before the per-page raster ceiling is
/// applied: 800%, past which a screen shows a few glyphs at a time and
/// the pixmap is enormous.
pub const MAX_ZOOM: f32 = 8.0;

/// Where the pointer was over the page when a Ctrl+wheel arrived.
///
/// Lives here rather than on `crate::app::state` — where it was declared
/// until the rulers landed — because it is a fact about **zoom**, and this
/// module already owns [`ViewState::zoom`], [`FitMode`], [`ZOOM_LADDER`],
/// [`MAX_ZOOM`] and [`raster_scale`]. `app::state` re-exports it, so
/// `canvas::zoom` still names it by its old path and the move cost that
/// module nothing. See `app::state`'s re-export for the R2 argument that
/// prompted it.
///
/// Recorded on the frame the wheel is seen, consumed on the next one, so
/// the scroll offset can be moved to keep that point still. See
/// [`crate::canvas::geometry::zoom_anchor_offset`].
///
/// **It has to span two frames**, and that is not an implementation
/// detail: the new zoom is not known when the wheel is seen. The zoom is an
/// [`crate::app::actions::Action`] applied after the UI is built, and it
/// *clamps* — so the only honest source of "how big is the page now" is the
/// next frame's own display size. Recording the *inputs* and solving later
/// avoids predicting a clamp we do not control.
#[derive(Debug, Clone, Copy)]
pub struct ZoomAnchor {
    /// The pointer's position as a fraction of the page's drawn size.
    pub frac: (f32, f32),
    /// The scroll offset before the zoom step.
    pub offset_before: (f32, f32),
    /// The page's drawn size before the zoom step.
    pub display_before: (f32, f32),
    /// The scroll viewport, needed for the centring-margin term.
    pub viewport: (f32, f32),
    /// **Which page every other field in this struct is about** — the page
    /// that was being acted on when the anchor was armed.
    ///
    /// # Why an anchor has to name its page (`OPERATOR_REQUESTS.md` O26d)
    ///
    /// `frac`, `offset_before` and `display_before` are all measured against
    /// **one page**: the anchor says *"the point at this fraction of THAT
    /// page was at that offset when the page was that size"*. Under a
    /// continuous mode the canvas then converts the solve's answer back into
    /// a strip offset by adding the page's origin within the strip — and it
    /// used to add **whichever page happened to be current on the frame the
    /// anchor was consumed**.
    ///
    /// Those are two different frames and they can name two different pages:
    /// the anchor is armed on frame N (during `show`, when the wheel is seen)
    /// and solved on frame N+1 (once the zoom has landed), and the current
    /// page tracks the scroll in between. When they differ, the answer is
    /// wrong by whole page pitches — at a million percent that is 10⁷ points,
    /// the offset clamps to the end of its range, and the page lands in a
    /// corner of the screen with the rest of the drawing off it.
    ///
    /// Under [`PageDisplay::Single`] there is one page at the origin and
    /// this field is always the current one, so nothing about that path
    /// changes. It is the strip that made "which page" a question.
    pub page: usize,
}

/// Which page is shown, at what scale, how that scale is chosen, and in what
/// arrangement.
///
/// ## `PartialEq` is here for one test, and it is the right one
///
/// Derived for [`crate::app::prefs::Prefs::seed_view`], whose contract is
/// *"seeding from the shipped preferences leaves a freshly opened view
/// untouched"*. That property is only assertable as **whole-struct
/// equality**: checking the fields the seeder writes would pass while a fifth
/// field was silently clobbered, and checking the fields it does not write
/// requires listing them, which is the same restatement drifting in a second
/// place.
///
/// Deriving it over an `f32` is deliberate rather than overlooked. This struct
/// is a *record of choices* — a zoom that was set, not a zoom that was computed
/// — so two states that arrived at 1.0 by different routes genuinely are the
/// same state. The float-comparison caution applies to accumulated arithmetic,
/// and there is none here.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewState {
    /// **The page the operator is looking at**, 0-based into the flattened
    /// page vector.
    ///
    /// # What this means under a continuous mode, and who writes it
    ///
    /// Under [`PageDisplay::Single`] and [`PageDisplay::Facing`] it is the
    /// page (or the spread) being *shown*, and navigation is the only thing
    /// that changes it — exactly as before Phase 4.
    ///
    /// Under a continuous mode the strip shows several pages at once, so
    /// "which page" is no longer a choice the view makes; it is a **reading of
    /// where the operator has scrolled to**. [`crate::canvas::show`] therefore
    /// writes this field from [`strip::Strip::page_at_view`] on every frame,
    /// as a fourth item of documented per-frame view bookkeeping beside
    /// `last_scroll_offset`, `zoom_anchor` and `selection` — see that module's
    /// header for the whole argument, including why a scroll cannot be
    /// deferred into an [`crate::app::actions::Action`].
    ///
    /// It stays a single index rather than becoming a range because
    /// **everything downstream wants exactly one page**: the decomposition
    /// cache, the selection, the Objects panel, the Properties row, the status
    /// bar's page box and the `objects n=` trace all describe *a* page. A
    /// range would have made every one of them ask "which of these do you
    /// mean?", and the answer would have been this index anyway.
    pub page_index: usize,
    /// Effective device pixels per PDF user-space unit — the exact
    /// value handed to [`pdfcer_render::render_page`].
    pub zoom: f32,
    /// Whether `zoom` is pinned or derived from the viewport.
    pub fit: FitMode,
    /// **Which of the four page-display arrangements is active.**
    ///
    /// A view stance, not a document property: it changes what is on screen
    /// and nothing a save would write, which is why it lives here beside
    /// `zoom` and `fit` rather than anywhere near `EditSession`. It is
    /// nevertheless remembered **per document** — see
    /// [`remembered`] — because a sheet set and a report want different
    /// answers and the operator should only have to say so once per document.
    ///
    /// Changed through [`crate::app::actions::Action::SetPageDisplay`], like
    /// every other view stance the ribbon can reach.
    pub display: PageDisplay,
    /// **Whether the ruler gutters are drawn along the canvas edges.**
    ///
    /// `RIBBON_IA.md` §5.2's View ▸ Display row. It is here rather than in
    /// `egui::Memory` for the reason `view.tool_hand` is *not*: the pressed
    /// state of a toggle has to be published from
    /// `crate::app::PdfcerApp::conditions`, which is handed `&self` and no
    /// `egui::Context` — so a toggle whose state lives in `Memory` cannot
    /// render pressed, which is exactly the gap the hand tool and the region
    /// zoom still carry. Living on the view state closes it for all three of
    /// these with no second mechanism.
    ///
    /// **This is what the ruler costs, and it is a constant.** Switching it on
    /// takes [`crate::canvas::rulers::THICKNESS_PTS`] off each of two edges of
    /// the viewport that [`Self::apply_fit`] divides by. A *variable*
    /// reservation there would be rule R128's fit-to-viewport feedback loop;
    /// see that module's header §3.
    pub rulers: bool,
    /// **Whether a drawing grid is drawn over each page.**
    ///
    /// Per page and in page space — [`crate::canvas::rulers`]' header §2
    /// carries the argument, which is the same one that makes a guide belong
    /// to a page.
    pub grid: bool,
    /// **Whether the operator's guides are drawn, and draggable.**
    ///
    /// The guides *themselves* live on [`crate::app::state::OpenDoc::guides`]
    /// and persist per document; this is only whether they are shown. The
    /// split is deliberate and [`crate::canvas::guides`]' header §2 argues it:
    /// a switch is cheap to flick again and a placed guide is work, so one is
    /// remembered and the other is not — and a document that *has* remembered
    /// guides opens with this already `true`, because the presence of the work
    /// is the preference.
    pub guides: bool,
    /// **`view.show_points` — draw an object's anchors without descending into
    /// it.**
    ///
    /// What it shows is `canvas::overlay::draw_anchors` — the same mark the
    /// multi-node move draws on, so the anchors an operator asks to see and the
    /// anchors a drag acts on cannot disagree.
    ///
    /// **A recorded blocker outlives the thing that justified it.** This
    /// command's read *"there is nothing for it to show — this build draws no
    /// anchor mark at any rung"*, which is a claim about the build, and a claim
    /// about the build is re-derived against the build before it is believed.
    /// A reason that is merely written down, and copied, goes on refusing a
    /// command long after the refusal stopped being true.
    ///
    /// Default **off**, with the other three: it is a drafting aid, and an
    /// operator who has not asked for hollow squares over their drawing should
    /// not get them. `crate::app::prefs` can make that an opening preference
    /// the day somebody wants one.
    pub show_points: bool,
    /// **`view.line_weights` — are strokes drawn at the widths the file
    /// declares, or every one of them at one device pixel?**
    ///
    /// `OPERATOR_REQUESTS.md` **O137**, in his words:
    ///
    /// > *"awhile ago you told me you removed the button to show all lines
    /// > without their thickness — thin lines or something like cad has. The
    /// > button never worked but I do want that display option!"*
    ///
    /// # Which convention this is, because the two are opposites
    ///
    /// * **This, turned OFF** — every stroke drawn at **one device pixel**,
    ///   whatever the file declares. The precedent is AutoCAD's `LWDISPLAY`
    ///   switched off.
    /// * ***Not* this** — sub-pixel strokes bumped **up** to one pixel so they
    ///   do not vanish. That is Acrobat's *enhance thin lines*.
    ///
    /// **One makes thick things thin. The other makes thin things thick.** He
    /// said *"without their thickness"* and named CAD, so this is the first.
    /// Shipping the second would be worse than shipping nothing, because it
    /// would look like the feature working while doing the opposite.
    ///
    /// # Why the field is named for the WEIGHTS and `true` is the default
    ///
    /// Every other toggle in this group is `false` by default and means *draw
    /// something extra*. This one is `true` and means *keep drawing the
    /// document faithfully* — so a fresh `ViewState` renders exactly what
    /// every build before this one rendered, and the operator's gesture is
    /// **turning something off**.
    ///
    /// That is also both precedents' spelling: Acrobat's menu item is
    /// **View ▸ Line Weights**, checked by default; AutoCAD's system variable
    /// is `LWDISPLAY`, and *off* is what a draughtsman asks for. Naming it
    /// `hairline: bool` would have made the pressed state mean *"the document
    /// is not being drawn faithfully"*, which is the harder sentence to read
    /// off a ribbon button.
    ///
    /// # It is the ONLY member of this group that changes the RASTER
    ///
    /// Rulers, grid, guides and show-points are all drawn by the canvas
    /// **over** a finished page texture; none of them can make a cached raster
    /// wrong. This one is a [`pdfcer_render::RenderOptions`] field
    /// (`stroke_display`), so a texture drawn while it was on is a *different
    /// picture* from one drawn while it was off — and a cache that served the
    /// old one would make the toggle look inert, which is precisely the defect
    /// O137 reports about its predecessor.
    ///
    /// It is therefore a **staleness key**:
    /// [`crate::app::state::OpenDoc::render_key_for`] feeds
    /// [`Self::stroke_display`] into [`crate::render::worker::RenderKey::new`],
    /// and `the_render_key_moves_when_line_weights_are_turned_off` is what
    /// stops that being forgotten.
    ///
    /// # Canvas only — the constraint decided in writing before the engine
    /// field existed
    ///
    /// Print, print preview and **every** export — PDF, DXF, PNG, JPEG, SVG,
    /// EMF, form data, text — render the document's **real** widths. This field
    /// is read by exactly one place, `crate::render::worker::render_on_worker`,
    /// and `crate::app::settings::tests::only_the_canvas_worker_sets_stroke_display`
    /// parses every file in the crate to keep it that way.
    ///
    /// > **The one thing worse than not having this feature is having it follow
    /// > him into a file he sends a client.**
    ///
    /// The engine holds the same line from its side and says so on its own
    /// backlog row: there is deliberately **no CLI flag**, because a hairline
    /// export would be an unfaithful file.
    ///
    /// # Fills are untouched
    ///
    /// The width is resolved at one place in the engine,
    /// `Interpreter::stroke_params`, which every stroking operator goes through
    /// (`S`, `s`, `B`, `B*`, `b`, `b*`, and stroked text render modes) and
    /// which **no fill reaches**. So a hatch built out of thin *fills* cannot
    /// thin out or vanish. Said here because an operator whose hatching is
    /// fill-based would otherwise expect it to follow the toggle.
    ///
    /// # Per document, not global
    ///
    /// It lives here, beside `zoom` and `rulers`, so two open drawings can
    /// disagree: comparing a hairline read of a dense sheet against a faithful
    /// read of the sheet beside it is the actual job. There is deliberately no
    /// persisted default in `crate::app::prefs` — see
    /// `crate::text::commands::view_line_weights` for that decision and where a
    /// preference would go if he asks for one.
    pub line_weights: bool,
    /// **`view.off_page` — may the canvas show, and reach, the marks
    /// that sit outside the sheet?**
    ///
    /// A CAD export often carries geometry beyond its own `/MediaBox` — a
    /// title block dragged off the sheet, a detail parked in the margin.
    /// pdfcer can rasterize and hit-test that material, and when it does the
    /// canvas grows a band of pasteboard wide enough to hold it.
    ///
    /// **`false` is not "hide it" — it is "do not grow for it".** The band
    /// is the cost: an operator who is only reading pays for it in scroll
    /// distance and in the grey gap it opens between one sheet and the next.
    /// So the flag gates exactly two things, both in [`crate::canvas::tier`]:
    /// the pasteboard **overhang** — off means no band and no gap, a layout
    /// byte-for-byte identical to one with no off-page support at all — and
    /// the **halo raster tier**, off meaning nothing outside the sheet is
    /// drawn or reachable. Nothing else in the shell reads it.
    ///
    /// **Default `false` — but the mode decides.** Read opens with it off,
    /// Review and Edit with it on, and the operator's own answer is
    /// remembered per mode. All of that lives in
    /// [`crate::app::prefs::offpage`], with the argument for the split.
    pub off_page: bool,
    /// **`view.ocr_layer` — the X-ray over a scanned page's invisible text,
    /// and how far it is slid.**
    ///
    /// `OPERATOR_REQUESTS.md` **O226**, in his words: *"a slider for
    /// transparency — when slid all the way to the left only the pdf shows […]
    /// when slid all the way to the right only the text layer is visible."*
    ///
    /// # Why `Option<f32>` and not `f32`
    ///
    /// Because **off and fully-left are different states and look identical**.
    /// At the left stop the operator is *in* the mode, reading the scan, one
    /// drag away from the text; with the mode off there is no slider at all and
    /// nothing is being withheld. One `f32` would have to pick a sentinel for
    /// the second, and `0.0` is already spoken for by the first.
    ///
    /// So `None` is *the mode is off* — the ribbon button unpressed, the veil
    /// not painted, no text drawn, nothing extracted — and `Some(s)` is *the
    /// mode is on with the slider at `s`*. `ViewChrome::OcrLayer` reads
    /// `.is_some()` and writes [`OCR_OVERLAY_DEFAULT`], so the ribbon toggle
    /// and the slider are two handles on one field rather than two fields that
    /// can disagree.
    ///
    /// # What `s` means, in both directions at once
    ///
    /// | `s` | the page raster | the text |
    /// |---|---|---|
    /// | `0.0` | as saved | not drawn |
    /// | `0.5` | half veiled | half opaque |
    /// | `1.0` | **blank** | full |
    ///
    /// At `1.0` the field behind the text is blank paper, not the scan at one
    /// percent: he asked to read the OCR output *without the paper arguing with
    /// it*. [`crate::canvas::ocrlayer`] owns both halves and the reason the
    /// raster is veiled rather than re-rasterized.
    ///
    /// # R8b
    ///
    /// This is an operator-thrown X-ray switch over content already in the
    /// file — the class Acrobat's *Show OCR text* belongs to — not pdfcer
    /// marking its own uncertainty. Nothing it draws reaches a saved byte, and
    /// the mode is named off-canvas the whole time it is on. `DESIGNS.md`
    /// carries the settlement so it is not re-argued.
    ///
    /// # Per view
    ///
    /// Here, beside `zoom` and `grid`, because two open drawings must be able
    /// to disagree — and because the second pane of O226's split is a second
    /// `ViewState`, which is what lets one pane show the scan while the other
    /// shows the text.
    pub ocr_overlay: Option<f32>,
}

/// Where the OCR slider lands when the ribbon toggle turns the mode **on**.
///
/// Not `1.0` and not `0.0`. Either stop shows exactly one of the two things
/// the mode exists to let an operator compare, so arriving at one would make
/// the first gesture *find the slider* rather than *read the page*. Two thirds
/// puts the text clearly on top with the scan still legible beneath it, which
/// is the position an operator checking a recognition against the paper
/// actually wants.
pub const OCR_OVERLAY_DEFAULT: f32 = 0.65;

/// A slider position brought into `0.0..=1.0`, with a non-finite one refused.
///
/// ★ The guard is `is_finite` **before** the clamp, not after: `f32::clamp`
/// propagates a NaN rather than rejecting it, so a NaN that reached the veil's
/// alpha would paint an undefined rectangle over the page. The same ordering
/// `crate::app::prefs::normalise_ui_scale` uses, for the same reason.
#[must_use]
pub fn normalise_ocr_overlay(raw: f32) -> f32 {
    if raw.is_finite() {
        raw.clamp(0.0, 1.0)
    } else {
        OCR_OVERLAY_DEFAULT
    }
}

impl Default for ViewState {
    /// First-open defaults: page 1, fit-page, single page.
    ///
    /// Fit-page rather than 100% is a deliberate choice. Opening at a
    /// raw 100% produces a wildly different first impression depending
    /// on the page size — a business card fills a thumb's worth of the
    /// window, an A0 poster overflows it — and both read as a bug even
    /// though nothing is wrong. Fit-page always shows the operator the
    /// thing they just opened.
    ///
    /// **Single page rather than continuous**, for the reason
    /// [`display`]'s header states at length: continuous is an option, not a
    /// replacement, and paging one sheet at a time is the right model for
    /// drafting review. Read mode's continuous default is applied by the open
    /// path (which knows the mode and the document), not by this `Default` —
    /// so a `ViewState` built with no context is the conservative one.
    ///
    /// **All three View ▸ Display toggles start off**, and that is not
    /// timidity. A ruler, a grid and a set of guides are all chrome drawn over
    /// or beside the drawing, and pdfcer's first duty on opening a sheet is to
    /// show the sheet. Defaulting the rulers on would also take
    /// `THICKNESS_PTS` off two edges of every canvas for every operator who
    /// never wanted them, which is the one default that has a measurable cost
    /// (see the field's own docs and rule R128).
    ///
    /// `guides` is the one that is *overridden* at open — by
    /// [`crate::app::state::OpenDoc::new`], when the document turns out to
    /// have remembered guides. That override lives there rather than here for
    /// the same reason Read mode's continuous default does: a `ViewState`
    /// built with no context is the conservative one, and the path that knows
    /// the document is the path that may know better.
    fn default() -> Self {
        Self {
            page_index: 0,
            zoom: 1.0,
            fit: FitMode::Page,
            display: PageDisplay::Single,
            rulers: false,
            grid: false,
            guides: false,
            // Off, with the other three. See the field's own note: an
            // operator who has not asked for hollow squares over their drawing
            // should not get them.
            show_points: false,
            // **ON**, and it is the one member of this group whose default
            // is not `false` — because `true` here means *draw the document as
            // it says it should be drawn*, not *draw something extra*. A fresh
            // view therefore rasterizes byte for byte what every build before
            // O137 rasterized, and the operator's gesture is turning line
            // weights OFF. See the field's own docs for why the toggle is
            // named for the weights rather than for the hairline.
            line_weights: true,
            // Off, and this is the one default that is routinely
            // *overridden* on the way in: `crate::app::prefs::offpage`
            // answers per ribbon mode (Read off, Review and Edit on) and
            // remembers the operator's own answer for each. Off here for
            // the same reason `display` is `Single` here — the path that
            // knows the mode is the path that may know better.
            off_page: false,
            // `None` — the mode is OFF, which is not the same state as the
            // slider at its left stop. See the field's own note. A document
            // opens showing what it says it shows; an X-ray is something the
            // operator asks for.
            ocr_overlay: None,
        }
    }
}

impl ViewState {
    /// **[`Self::line_weights`] as the engine spells it** — the one place
    /// this shell's `bool` becomes a [`pdfcer_render::font::StrokeDisplay`].
    ///
    /// # Why the conversion is a named function and not an `if` at the call
    /// site
    ///
    /// There are two call sites and they must not be able to disagree: the
    /// **render key** ([`crate::app::state::OpenDoc::render_key_for`]) says
    /// *what picture I want*, and the **render request** (built next to it, read
    /// by `crate::render::worker::render_on_worker`) says *what picture this
    /// is*. Two hand-written `if`s is exactly how a cache comes to serve a
    /// raster drawn under the opposite answer — the failure mode that makes a
    /// toggle look inert, which is the defect O137 reports about the button
    /// this replaces.
    ///
    /// # Why the return type is the engine's ENUM and not a `bool`
    ///
    /// `StrokeDisplay` is `#[non_exhaustive]` with two variants today —
    /// `Actual` and `Hairline` — and the engine made it an enum deliberately so
    /// that Acrobat's *enhance thin lines* (the **opposite** convention: thin
    /// things made thick) can arrive as a third variant. A `hairline: bool`
    /// anywhere in this shell would, that day, come to mean *"one of the two"*.
    /// So the boolean stops here and the engine's vocabulary starts here.
    ///
    /// `Hairline` is the **off** position. `true` means faithful widths; see
    /// the field.
    #[must_use]
    pub const fn stroke_display(&self) -> pdfcer_render::font::StrokeDisplay {
        if self.line_weights {
            pdfcer_render::font::StrokeDisplay::Actual
        } else {
            pdfcer_render::font::StrokeDisplay::Hairline
        }
    }

    /// Move to `index`, clamped into `0..page_count`.
    ///
    /// Clamping rather than erroring is right for a *view*: the only
    /// ways to get an out-of-range index are a keyboard repeat past the
    /// end and a page count that shrank, and in both cases the operator
    /// wants the nearest valid page, not a message.
    pub fn go_to_page(&mut self, index: usize, page_count: usize) {
        self.page_index = clamp_page_index(index, page_count);
    }

    /// Step one page toward the end, stopping at the last page.
    ///
    /// Saturating rather than wrapping: wrap-around page navigation
    /// silently teleports an operator from page 400 to page 1, which is
    /// disorienting and is not what any document reader does.
    pub fn next_page(&mut self, page_count: usize) {
        self.go_to_page(self.page_index.saturating_add(1), page_count);
    }

    /// Step one page toward the start, stopping at the first page.
    pub fn prev_page(&mut self, page_count: usize) {
        self.go_to_page(self.page_index.saturating_sub(1), page_count);
    }

    /// Pin the zoom to an explicit value, clamped to `[MIN_ZOOM, max]`,
    /// and drop out of any fit mode.
    ///
    /// `max` is the per-page ceiling from [`max_zoom_for_page`], passed
    /// in rather than recomputed so this stays a pure state transition
    /// with no page argument.
    pub fn set_zoom(&mut self, zoom: f32, max: f32) {
        self.zoom = clamp_zoom(zoom, max);
        self.fit = FitMode::None;
    }

    /// Multiply the current zoom (the ctrl+scroll path), clamped, and
    /// drop out of any fit mode.
    pub fn zoom_by(&mut self, factor: f32, max: f32) {
        self.set_zoom(self.zoom * factor, max);
    }

    /// Step to the next ladder rung above the current zoom.
    pub fn zoom_in(&mut self, max: f32) {
        self.set_zoom(ladder_step_up(self.zoom), max);
    }

    /// Step to the next ladder rung below the current zoom.
    pub fn zoom_out(&mut self, max: f32) {
        self.set_zoom(ladder_step_down(self.zoom), max);
    }

    /// Enter a fit mode. The zoom itself is recomputed by
    /// [`ViewState::apply_fit`] once the viewport size is known, which
    /// in immediate mode is not until the frame is being laid out.
    pub fn set_fit(&mut self, fit: FitMode) {
        self.fit = fit;
    }

    /// If a fit mode is active, recompute `zoom` from the viewport.
    /// A no-op under [`FitMode::None`], so it is safe (and intended) to
    /// call unconditionally every frame.
    pub fn apply_fit(&mut self, page_pts: (f32, f32), viewport: (f32, f32), max: f32) {
        if self.fit == FitMode::None {
            return;
        }
        self.zoom = clamp_zoom(fit_scale(page_pts, viewport, self.fit), max);
    }

    /// The zoom as a whole percentage, for the toolbar readout.
    ///
    /// Rounds rather than truncates so a fit scale of 0.99997 reads as
    /// `100%`, not `99%`.
    #[must_use]
    #[allow(
        dead_code,
        reason = "the zoom readout is a status-bar control and lands at stage S2; kept with the ladder it reports on so the rounding rule cannot be re-derived differently" // ui-text-exempt: clippy lint justification, never displayed
    )]
    pub fn zoom_percent(&self) -> f64 {
        // `f64`, not `u32` — `OPERATOR_REQUESTS.md` O24j.
        //
        // A saturating `as u32` cast clamps at 4,294,967,295, so at a trillion
        // percent the status bar states **4294967295%** — `u32::MAX` presented
        // as a measurement, which is a number an operator quite reasonably
        // reads as a crash. Three digits were enough while `MAX_ZOOM` was 8.0;
        // O24 raised the reachable ceiling to 10¹².
        //
        // **A limit lifted in one place leaves every narrower type downstream
        // enforcing the old one silently**, and a readout is where that
        // surfaces as a plausible-looking number rather than as a refusal.
        f64::from(self.zoom * 100.0).max(0.0)
    }
}

/// Clamp a page index into `0..page_count`, mapping the empty-document
/// case to `0`.
///
/// Returning `0` for an empty document rather than panicking keeps the
/// "no pages" condition a *presentation* decision (the canvas shows
/// [`crate::text::canvas_no_pages`]) instead of a crash, which matters
/// because a valid PDF really can have `/Count 0`.
#[must_use]
pub fn clamp_page_index(index: usize, page_count: usize) -> usize {
    index.min(page_count.saturating_sub(1))
}

/// Clamp a zoom value into `[MIN_ZOOM, max]`, mapping NaN to `1.0`.
///
/// NaN is reachable in practice: a degenerate page whose CropBox has
/// zero width makes `viewport_width / page_width` infinite or NaN, and
/// an unclamped NaN would propagate into the render scale and then into
/// a pixmap size, where it becomes a much less obvious failure. Mapping
/// it to actual size fails visibly and harmlessly.
#[must_use]
pub fn clamp_zoom(zoom: f32, max: f32) -> f32 {
    if !zoom.is_finite() {
        return 1.0;
    }
    // `max` can legitimately fall below MIN_ZOOM for an absurdly large
    // page, in which case the ceiling must win — hence clamping to the
    // top first and the bottom second would be wrong; take the ceiling
    // last.
    zoom.max(MIN_ZOOM).min(max.max(f32::MIN_POSITIVE))
}

/// The highest zoom at which this page still rasterizes within
/// [`pdfcer_render::MAX_PIXMAP_EDGE`], capped at [`MAX_ZOOM`].
///
/// See the module docs for why this exists. Two subtleties:
///
/// - **The whole of [`raster_density`] is part of the calculation**, not just
///   the display density. The zoom the operator sees is a *logical* scale
///   (points per PDF unit); the raster is made at `zoom × raster_density` (see
///   [`raster_scale`]). On a 2× display, or with View ▸ Render ▸ Quality on
///   Sharper, every page therefore hits the pixmap ceiling at a lower zoom than
///   it otherwise would — omitting either factor is how a guard like this
///   passes its tests and still fails on the one machine that matters.
/// - **A one-pixel guard band is subtracted** before dividing, because
///   the renderer computes its pixmap edge with `ceil()`: a scale that
///   divides out to exactly the limit rounds *up* past it and is refused.
///
/// [`raster_scale`]: crate::viewer::raster_scale
#[must_use]
pub fn max_zoom_for_page(
    page_pts: (f32, f32),
    pixels_per_point: f32,
    quality: crate::app::prefs::RenderQuality,
) -> f32 {
    let longest = page_pts.0.max(page_pts.1);
    let density = raster_density(pixels_per_point, quality);
    if !longest.is_finite() || longest <= 0.0 {
        return MAX_ZOOM;
    }
    #[allow(
        clippy::cast_precision_loss,
        reason = "MAX_PIXMAP_EDGE is 16384; f32 is exact to 2^24" // ui-text-exempt: clippy lint justification, never displayed
    )]
    let ceiling = (pdfcer_render::MAX_PIXMAP_EDGE - 1) as f32 / (longest * density);
    // `clamp` is safe here (MIN_ZOOM < MAX_ZOOM, neither is NaN, and
    // `ceiling` is finite because `longest` was checked above and
    // `raster_density` is finite and positive by construction) — so clippy's
    // `manual_clamp` suggestion is taken rather than suppressed.
    ceiling.clamp(MIN_ZOOM, MAX_ZOOM)
}

/// The display density to actually use, given whatever egui reported.
///
/// Returns `pixels_per_point` when it is a usable density and `1.0` otherwise.
///
/// # Why this is a named function rather than a `.max()` at each site
///
/// Four places divide or multiply by the display density — [`raster_scale`],
/// [`ceiling::zoom_ceiling`]'s learned clause, `render::settle`'s
/// `learn_raster_ceiling`, and `app::status::rasterstop` — and they are not free
/// to guard it differently, because they are three readings of *one* number
/// (`crate::render::ceiling::RasterCeiling`'s stored raster scale) and a
/// disagreement between them is a shell that clamps at one zoom and explains
/// itself at another.
///
/// The tempting spelling is `pixels_per_point.max(f32::MIN_POSITIVE)`, and it
/// is **wrong in the one case that matters**. `f32::max` returns the *other*
/// operand when one is `NaN`, so a `NaN` density becomes `f32::MIN_POSITIVE` —
/// and a division by it produces infinity, which is the most destructive
/// possible answer rather than a conservative one. The consequence is not
/// abstract: the sentence that explains a zoom limit would be switched off
/// permanently and silently, in exactly the state it exists for. The `NaN` row
/// of this function's unit test is what holds the guard to it.
///
/// `1.0` is the right fallback because it is the *identity*: a scale and a zoom
/// are the same number at unit density, so a caller that cannot learn the
/// density falls back to treating the two as interchangeable, which is what the
/// shell did for its whole life before HiDPI was handled at all.
#[must_use]
pub fn sane_pixels_per_point(pixels_per_point: f32) -> f32 {
    if pixels_per_point.is_finite() && pixels_per_point > 0.0 {
        pixels_per_point
    } else {
        1.0
    }
}

/// The device-pixel scale to rasterize at for a given logical `zoom`.
///
/// `zoom` is points per PDF user-space unit — what the operator sees as
/// a percentage and what fit modes compute. The raster has to be made in
/// *pixels*, so it is multiplied by the display's `pixels_per_point`.
/// Getting this wrong is not a crash; it is a viewer that looks
/// permanently slightly blurry on every HiDPI laptop and perfectly sharp
/// on the developer's external monitor.
#[must_use]
pub fn raster_scale(
    zoom: f32,
    pixels_per_point: f32,
    quality: crate::app::prefs::RenderQuality,
) -> f32 {
    zoom * raster_density(pixels_per_point, quality)
}

/// The factor between a **zoom** and a **raster scale**, in device pixels per
/// logical unit.
///
/// # ★★★ Why this is a function and not two multiplications
///
/// A raster scale is `zoom × pixels_per_point × quality.multiplier()`, and four
/// places in the shell need to run that conversion **backwards**:
/// [`max_zoom_for_page`], [`ceiling::zoom_ceiling`]'s learned clause,
/// `render::settle::absorb`'s `learn_raster_ceiling`, and `app::status::rasterstop`.
/// Every one of them divided by the density alone, and the quality factor was
/// simply absent — so on View ▸ Render ▸ Quality ≥ Normal the derived ceiling
/// asked the engine for a pixmap over [`pdfcer_render::MAX_PIXMAP_EDGE`], the
/// engine refused, and the clamp that exists to rescue the operator landed by
/// the same factor too high and refused again. O218.
///
/// Routing both directions through this one function is what makes
/// [`zoom_for_raster_scale`] the *exact* inverse of [`raster_scale`] rather than
/// a second reading of the same rule that has to be kept in step by hand.
///
/// # What is in it
///
/// * `pixels_per_point`, through [`sane_pixels_per_point`], so the raster stays
///   sharp on a HiDPI display.
/// * `quality.multiplier()`. `Normal` is `1.0`: one raster pixel per device
///   pixel, which is exactly `zoom × ppp` and is therefore what a build whose
///   operator never opens the Settings window gets, byte for byte. The knob
///   multiplies that, so the setting can only ever be a deliberate departure
///   from the default — there is no compiled-in quality constant anywhere else
///   for it to disagree with.
///
/// The result is finite and strictly positive without a guard, because
/// `sane_pixels_per_point` guarantees that of its half and
/// [`crate::app::prefs::RenderQuality::multiplier`] is a `const fn` over a
/// closed enum whose three values are 0.75, 1.0 and 1.5.
#[must_use]
pub fn raster_density(pixels_per_point: f32, quality: crate::app::prefs::RenderQuality) -> f32 {
    sane_pixels_per_point(pixels_per_point) * quality.multiplier()
}

/// The logical zoom that rasterizes at `scale` — the inverse of
/// [`raster_scale`].
///
/// [`crate::render::ceiling::RasterCeiling`] stores what the engine refused as a
/// **raster scale**, deliberately: a ceiling kept as a zoom would be wrong by the
/// density ratio on a window dragged between two monitors, silently, and only on
/// the machine it was not measured on. Every reader therefore has to convert,
/// and this is the conversion — the whole of [`raster_density`], not the display
/// density alone.
#[must_use]
pub fn zoom_for_raster_scale(
    scale: f32,
    pixels_per_point: f32,
    quality: crate::app::prefs::RenderQuality,
) -> f32 {
    scale / raster_density(pixels_per_point, quality)
}

/// A page's on-screen extent in PDF user-space units, with `/Rotate`
/// already applied (a 90°-rotated portrait page is landscape on screen).
///
/// # One definition of how big a page is
///
/// Delegates to [`crate::render::region::PageFrame::extent_pts`] rather than
/// reading `page.crop_box` directly. That is the point, and it has not
/// changed: a fit-page computed from an un-rotated `CropBox` against a rotated
/// raster is the classic version of this bug, so the rotation table lives in
/// exactly one place — the same place that holds the canvas↔user conversion
/// this extent has to agree with.
///
/// # Why NOT [`pdfcer_render::page_device_geometry`]'s pixmap dimensions
///
/// Those are `u32` and therefore **ceiled**, which makes them the wrong
/// measure of a page for a layout that translates in points. A page measuring
/// 2383.937 × 1683.78 pt lays out as 2384 × 1684 — a canvas space 0.22 pt
/// taller than the page whose coordinates it carries, because
/// `PageFrame::user_to_canvas` puts that page's bottom edge at 1683.78.
///
/// **A rounding error in a layout is multiplied by the zoom.** At 100 % that
/// gap is a fifth of a pixel and invisible; at 1040 %, against a page 17,509 pt
/// tall on screen, it puts `render::region::region_on_screen`'s region raster
/// **2.3 pt** away from where the page's own rect says it belongs. Measured by
/// `ui-verify`'s `panning_at_deep_zoom_stays_where_it_was_put`, the only
/// instrument in the project that compares a raster's *painted* rect against a
/// rect recomputed independently from the page.
///
/// The full argument, including why the pixmap still being a fraction of a
/// pixel larger than the page is harmless and why the ceiled extent's version
/// of the same error was not, is on
/// [`crate::render::region::PageFrame::extent_pts`].
#[must_use]
pub fn page_extent_pts(page: &Page) -> (f32, f32) {
    crate::render::region::PageFrame::of(page).extent_pts()
}

// ---------------------------------------------------------------------------
// Canvas-interaction geometry
// ---------------------------------------------------------------------------
//
// Two distinct coordinate spaces, named here because the substrate and the
// authoring APIs live in different ones and a future stage that conflated
// them would author geometry in the wrong frame:
//
// - **Canvas space** — page-device points at zoom 1.0: Y-**down**, origin
//   top-left, `/Rotate` already resolved into a possibly-swapped
//   width/height. This is the space [`page_extent_pts`] measures and the
//   space the on-screen raster is drawn in. `screen_to_page`/`page_to_screen`
//   convert between the screen and this space; they carry NO rotation logic
//   (rotation is already baked into the `extent` they are handed — see
//   [`page_extent_pts`]).
// - **PDF user space** — Y-**up**, origin at the *un-rotated*
//   MediaBox/CropBox lower-left, exactly what an annotation `/Rect`, a
//   content-stream operand, or the object model expresses. The second
//   bridge, `canvas_to_pdf_space`/`pdf_space_to_canvas`, converts between
//   canvas space and this one by reusing — and inverting — the SAME device
//   transform [`pdfcer_render::page_device_geometry`] computes to rasterize
//   the page, so the interaction geometry and the render agree by
//   construction rather than by two hand-derived rotation formulas quietly
//   drifting apart.

/// Map a screen point to **canvas space**.
///
/// `image_rect` is the canvas Response's own `.rect` for this frame
/// (the rect the page raster occupies on screen); `extent` is
/// [`page_extent_pts`] for the current page (the rotated device
/// width/height); `zoom` is [`ViewState::zoom`]. The page raster is drawn
/// at `image_rect.min` scaled by `zoom`, so undoing that — subtract the
/// origin, divide by the zoom — is the whole of the arithmetic.
///
/// **No rotation branch lives here on purpose.** Rotation-correctness comes
/// entirely from `extent` already carrying the rotated width/height (see
/// [`page_extent_pts`]); adding a rotation-aware branch here as well would
/// double-apply it. The `extent` argument is consulted only to reject a
/// degenerate page (per the contract below) — the mapping itself is a pure
/// affine undo of the draw.
///
/// Returns [`Pos2::ZERO`] for a degenerate page or zoom (zero/negative/
/// non-finite `extent` or `zoom`), mirroring [`fit_scale`]/[`clamp_zoom`]'s
/// "fail to a finite, harmless value, never a NaN/panic" discipline: there
/// is no sensible canvas coordinate for a page with no area.
#[must_use]
pub fn screen_to_page(pos: Pos2, image_rect: Rect, extent: (f32, f32), zoom: f32) -> Pos2 {
    if !geometry_inputs_ok(extent, zoom) {
        return Pos2::ZERO;
    }
    Pos2::new(
        (pos.x - image_rect.min.x) / zoom,
        (pos.y - image_rect.min.y) / zoom,
    )
}

/// The exact inverse of [`screen_to_page`]: **canvas space** → screen.
///
/// Needed every frame by any live-preview overlay (a stored canvas-space
/// geometry must be projected back to the screen to be drawn) and, from
/// stage S4, to draw a hit-tested object's selection outline. Same
/// degenerate-input contract as [`screen_to_page`].
#[must_use]
#[allow(
    dead_code,
    reason = "the inverse half of a bridge whose forward half IS live (screen_to_page, used by the canvas pointer trace); its first drawing consumer is S4's selection outline. Kept because a bridge with one direction implemented is exactly how the two ends drift apart." // ui-text-exempt: clippy lint justification, never displayed
)]
pub fn page_to_screen(page_pt: Pos2, image_rect: Rect, extent: (f32, f32), zoom: f32) -> Pos2 {
    if !geometry_inputs_ok(extent, zoom) {
        return Pos2::ZERO;
    }
    Pos2::new(
        page_pt.x * zoom + image_rect.min.x,
        page_pt.y * zoom + image_rect.min.y,
    )
}

/// Whether the geometry inputs describe a real, finite page at a real
/// zoom — the shared degenerate-input guard for the screen⟷canvas bridge.
#[must_use]
fn geometry_inputs_ok(extent: (f32, f32), zoom: f32) -> bool {
    zoom.is_finite() && zoom > 0.0 && extent.0.is_finite() && extent.0 > 0.0 && extent.1 > 0.0
}

/// Convert a **canvas-space** point into genuine **PDF user space** — the
/// frame every `pdfcer-core` authoring API consumes.
///
/// Implemented by inverting the SAME transform
/// [`pdfcer_render::page_device_geometry`] computes to rasterize this page
/// at scale 1.0 (its third tuple element, a
/// [`pdfcer_render::tiny_skia::Transform`]). Canvas space *is* that
/// transform's output space at scale 1.0, so its inverse is exactly the
/// canvas→user map, rotation and Y-flip included, with no second formula to
/// keep in sync (the geometry analogue of "reuse the renderer's own walk so
/// they agree by construction").
///
/// Returns `None` only for a genuinely non-invertible page transform (a
/// degenerate page). Callers decline the commit rather than author garbage
/// geometry.
#[must_use]
pub fn canvas_to_pdf_space(point: Pos2, page: &Page) -> Option<Pos2> {
    let (_, _, ctm) = pdfcer_render::page_device_geometry(page, 1.0);
    let inverse = ctm.invert()?;
    Some(apply_transform(&inverse, point))
}

/// The exact inverse of [`canvas_to_pdf_space`]: **PDF user space** →
/// **canvas space**.
///
/// Needed by any consumer that receives geometry already in PDF space — the
/// primary case being the object-model provider handing back a hit-tested
/// object's bounds in PDF space, which the selection overlay must project to
/// the screen via `page_to_screen(pdf_space_to_canvas(bounds, page), ..)`.
/// Returns `None` under the same non-invertible-page condition as
/// [`canvas_to_pdf_space`], so the two bridges decline together.
#[must_use]
#[allow(
    dead_code,
    reason = "built and tested at S0; first live consumer is S4's selection-outline projection" // ui-text-exempt: clippy lint justification, never displayed
)]
pub fn pdf_space_to_canvas(point: Pos2, page: &Page) -> Option<Pos2> {
    let (_, _, ctm) = pdfcer_render::page_device_geometry(page, 1.0);
    // Guard on invertibility so the two directions accept/decline the same
    // pages; the forward map itself does not need the inverse, but a page
    // whose transform cannot round-trip has no well-defined canvas point.
    ctm.invert()?;
    Some(apply_transform(&ctm, point))
}

/// Apply a `tiny_skia` [`Transform`] to a single egui [`Pos2`].
///
/// One place the `Pos2` ⟷ `tiny_skia::Point` marshalling lives, so the two
/// bridge directions cannot marshal inconsistently.
#[must_use]
fn apply_transform(transform: &Transform, point: Pos2) -> Pos2 {
    let mut mapped = [Point::from_xy(point.x, point.y)];
    transform.map_points(&mut mapped);
    Pos2::new(mapped[0].x, mapped[0].y)
}

#[cfg(test)]
#[allow(
    clippy::float_cmp,
    reason = "clamps and raster ceilings are exact f32 values"
)] // ui-text-exempt: clippy lint justification, never displayed
mod tests {
    use super::*;

    // ---- page-index clamping -------------------------------------

    #[test]
    fn clamping_keeps_indices_inside_the_document() {
        assert_eq!(clamp_page_index(0, 5), 0);
        assert_eq!(clamp_page_index(4, 5), 4);
        assert_eq!(clamp_page_index(5, 5), 4);
        assert_eq!(clamp_page_index(usize::MAX, 5), 4);
        // A page-less document must clamp to 0, not underflow.
        assert_eq!(clamp_page_index(3, 0), 0);
    }

    #[test]
    fn page_stepping_saturates_at_both_ends() {
        let mut v = ViewState::default();
        v.next_page(3);
        assert_eq!(v.page_index, 1);
        v.next_page(3);
        v.next_page(3);
        v.next_page(3);
        assert_eq!(v.page_index, 2);
        v.prev_page(3);
        assert_eq!(v.page_index, 1);
        v.prev_page(3);
        v.prev_page(3);
        assert_eq!(v.page_index, 0);
    }

    #[test]
    fn stepping_an_empty_document_stays_at_zero() {
        let mut v = ViewState::default();
        v.next_page(0);
        assert_eq!(v.page_index, 0);
        v.prev_page(0);
        assert_eq!(v.page_index, 0);
    }

    // ---- zoom clamping -------------------------------------------

    #[test]
    fn zoom_clamps_to_the_configured_range() {
        assert_eq!(clamp_zoom(0.001, MAX_ZOOM), MIN_ZOOM);
        assert_eq!(clamp_zoom(100.0, MAX_ZOOM), MAX_ZOOM);
        assert_eq!(clamp_zoom(2.0, MAX_ZOOM), 2.0);
    }

    #[test]
    fn a_page_ceiling_below_the_floor_still_wins() {
        // An absurd page can push the raster ceiling under MIN_ZOOM. The
        // ceiling has to win, or the render would be refused at a zoom
        // the UI claims is legal.
        assert_eq!(clamp_zoom(1.0, 0.05), 0.05);
    }

    #[test]
    fn non_finite_zoom_falls_back_to_actual_size() {
        assert_eq!(clamp_zoom(f32::NAN, MAX_ZOOM), 1.0);
        assert_eq!(clamp_zoom(f32::INFINITY, MAX_ZOOM), 1.0);
    }

    #[test]
    fn zoom_percent_rounds_rather_than_truncating() {
        let mut v = ViewState::default();
        v.set_zoom(0.999_97, MAX_ZOOM);
        assert_eq!(crate::text::status::zoom_percent(v.zoom_percent()), "100%");
        v.set_zoom(0.335, MAX_ZOOM);
        assert_eq!(crate::text::status::zoom_percent(v.zoom_percent()), "34%");
    }

    /// O24j — **the readout must survive the whole ceiling the zoom offers.**
    ///
    /// A `u32` return here saturates, and a saturated `as u32` reads as
    /// **4294967295%** on the status bar: `u32::MAX` presented as a
    /// measurement, at a zoom the ladder genuinely reaches.
    ///
    /// Asserted against the FORMATTED string, because that is the artefact an
    /// operator reads. A test of the numeric value passes on a build that
    /// narrows the type further downstream, which is where such a defect
    /// actually lives.
    #[test]
    fn the_readout_survives_the_whole_configured_range() {
        let mut v = ViewState::default();
        for (zoom, want) in [
            (1.0_f32, "100%"),
            (8.0, "800%"),
            (1.0e6, "100000000%"),
            // Not "1000000000000%", and the difference is not a defect.
            // `ViewState::zoom` is an `f32`, so the nearest representable
            // value to 10¹⁰ is 9,999,999,827,968 / 1000 — and the readout
            // shows what the view IS rather than what was asked for. Pinned
            // exactly, so a future change that starts rounding the display
            // instead of reporting it has to be a deliberate one.
            (1.0e10, "999999995904%"),
        ] {
            v.set_zoom(zoom, f32::MAX);
            let shown = crate::text::status::zoom_percent(v.zoom_percent());
            assert_eq!(shown, want, "zoom {zoom} showed {shown}");
            assert!(
                !shown.contains("4294967295"),
                "the readout saturated at u32::MAX"
            );
        }
    }

    // ---- raster-size ceiling -------------------------------------

    #[test]
    fn a_normal_page_is_not_constrained_by_the_raster_ceiling() {
        // US Letter: 16383 / 792 ≈ 20.7, far above MAX_ZOOM.
        assert_eq!(max_zoom_for_page((612.0, 792.0), 1.0, NORMAL), MAX_ZOOM);
    }

    #[test]
    fn an_annex_c_maximum_page_is_constrained() {
        // 14,400 user units is ISO 32000-1 Annex C's largest page edge.
        let max = max_zoom_for_page((14_400.0, 14_400.0), 1.0, NORMAL);
        assert!(max < MAX_ZOOM);
        // And the ceiling must actually keep the raster legal: the
        // renderer ceil()s, so check the rounded-up edge too.
        let edge = (14_400.0_f32 * max).ceil() as u32;
        assert!(edge <= pdfcer_render::MAX_PIXMAP_EDGE);
    }

    #[test]
    fn the_ceiling_is_what_actually_clamps_zoom_in_on_a_huge_page() {
        let page = (14_400.0, 14_400.0);
        let max = max_zoom_for_page(page, 1.0, NORMAL);
        let mut v = ViewState::default();
        for _ in 0..20 {
            v.zoom_in(max);
        }
        assert_eq!(v.zoom, max);
        assert!((page.0 * v.zoom).ceil() as u32 <= pdfcer_render::MAX_PIXMAP_EDGE);
    }

    #[test]
    fn degenerate_page_extent_does_not_produce_a_nonsense_ceiling() {
        assert_eq!(max_zoom_for_page((0.0, 0.0), 1.0, NORMAL), MAX_ZOOM);
        assert_eq!(max_zoom_for_page((f32::NAN, 10.0), 1.0, NORMAL), MAX_ZOOM);
    }

    // ---- HiDPI ----------------------------------------------------

    /// Shorthand for the quality that multiplies by one, so every test below
    /// that is not ABOUT the quality asserts the same number it did before the
    /// factor entered the arithmetic.
    use crate::app::prefs::RenderQuality;
    const NORMAL: RenderQuality = RenderQuality::Normal;

    #[test]
    fn raster_scale_multiplies_zoom_by_the_display_density() {
        assert_eq!(raster_scale(1.5, 2.0, NORMAL), 3.0);
        assert_eq!(raster_scale(1.5, 1.0, NORMAL), 1.5);
    }

    /// ★★★ **[`zoom_for_raster_scale`] is the exact inverse of
    /// [`raster_scale`], at every quality** — O218.
    ///
    /// The defect this pins is not that either function was wrong. Each was
    /// right about what it claimed; they simply did not agree, because the
    /// forward direction multiplied by the quality and all four backward
    /// readings divided by the display density alone. On the operator's own
    /// build — `render_quality = sharper` — every derived ceiling was therefore
    /// half again too high, the engine refused the pixmap, and the clamp that
    /// exists to rescue him landed by the same factor too high and refused
    /// again.
    ///
    /// ★ The repair is structural rather than arithmetic: both directions now
    /// run through [`raster_density`], so this test cannot be made to fail by
    /// changing one of them. That is the point of it — it is here to fail if
    /// somebody re-opens the two into separate expressions.
    #[test]
    fn a_zoom_survives_a_round_trip_through_a_raster_scale() {
        for quality in [
            RenderQuality::Faster,
            RenderQuality::Normal,
            RenderQuality::Sharper,
        ] {
            for ppp in [1.0_f32, 1.25, 1.5, 2.0] {
                for zoom in [0.1_f32, 1.0, 8.0, 5_000.0] {
                    let back =
                        zoom_for_raster_scale(raster_scale(zoom, ppp, quality), ppp, quality);
                    assert!(
                        (back - zoom).abs() <= zoom * 1e-6,
                        "{zoom}x at {ppp}x {quality:?} came back as {back}x"
                    );
                }
            }
        }
    }

    /// ★★ **A nonsense density is the identity in BOTH directions.**
    ///
    /// `sane_pixels_per_point` lives inside [`raster_density`], so the guard is
    /// stated once and both directions inherit it. Were it applied in the
    /// forward direction only, a `NaN` density would rasterize at `zoom` and
    /// convert back through a division by `NaN` — a ceiling of `NaN`, which
    /// compares false against everything and switches a clamp off silently.
    #[test]
    fn a_nonsense_density_is_the_identity_in_both_directions() {
        for bad in [0.0_f32, -1.0, f32::NAN, f32::INFINITY] {
            assert_eq!(raster_scale(2.0, bad, NORMAL), 2.0);
            assert_eq!(zoom_for_raster_scale(2.0, bad, NORMAL), 2.0);
        }
    }

    /// ★★★ **The page ceiling moves with the render quality** — O218's other
    /// half, and the assertion that fails on the old arithmetic.
    ///
    /// Sharper rasterizes at 1.5×, so the zoom at which a page fills
    /// [`pdfcer_render::MAX_PIXMAP_EDGE`] is two-thirds of what it is at Normal.
    /// The old [`max_zoom_for_page`] returned the same number for all three
    /// qualities, so on Sharper it offered a zoom whose raster the engine
    /// refuses — which is the sentence the operator reported reading across his
    /// drawing.
    #[test]
    fn the_page_ceiling_moves_with_the_render_quality() {
        let page = (1584.0_f32, 1224.0);
        let sharper = max_zoom_for_page(page, 1.0, RenderQuality::Sharper);
        let normal = max_zoom_for_page(page, 1.0, NORMAL);
        assert!(
            sharper < normal,
            "Sharper rasterizes 1.5x larger, so its ceiling must be lower: \
             {sharper} vs {normal}"
        );
        for quality in [
            RenderQuality::Faster,
            RenderQuality::Normal,
            RenderQuality::Sharper,
        ] {
            let ceiling = max_zoom_for_page(page, 1.0, quality);
            let edge = (page.0 * raster_scale(ceiling, 1.0, quality)).ceil() as u32;
            assert!(
                edge <= pdfcer_render::MAX_PIXMAP_EDGE,
                "{quality:?}: the ceiling {ceiling}x orders a {edge}px edge"
            );
        }
    }

    #[test]
    fn a_nonsense_pixels_per_point_is_treated_as_one() {
        // egui should never hand us these, but a zero here would render
        // a zero-size pixmap and a NaN would render nothing at all —
        // both far worse than ignoring a bad density.
        assert_eq!(raster_scale(2.0, 0.0, NORMAL), 2.0);
        assert_eq!(raster_scale(2.0, f32::NAN, NORMAL), 2.0);
        assert_eq!(
            max_zoom_for_page((14_400.0, 1.0), 0.0, NORMAL),
            max_zoom_for_page((14_400.0, 1.0), 1.0, NORMAL)
        );
    }

    #[test]
    fn the_raster_ceiling_accounts_for_display_density() {
        // The bug this pins: a guard computed in logical points passes
        // on a 1x developer monitor and blows the pixmap limit on a 2x
        // laptop, because the raster is twice as many pixels.
        let page = (14_400.0, 14_400.0);
        let max_1x = max_zoom_for_page(page, 1.0, NORMAL);
        let max_2x = max_zoom_for_page(page, 2.0, NORMAL);
        assert!(max_2x < max_1x);
        let edge = (page.0 * raster_scale(max_2x, 2.0, NORMAL)).ceil() as u32;
        assert!(edge <= pdfcer_render::MAX_PIXMAP_EDGE);
    }

    // ---- canvas-interaction geometry -----------------------------------

    use pdfcer_core::object::{Dict, ObjId};
    use pdfcer_core::page_tree::Rect as PageRect;

    /// A minimal page fixture: a `w`×`h` MediaBox/CropBox at the origin
    /// with the given clockwise `/Rotate`. Enough for the geometry
    /// functions, which read only `crop_box` and `rotate`.
    fn test_page(w: f64, h: f64, rotate: u16) -> Page {
        Page {
            id: ObjId::new(1, 0),
            resources: Dict::new(),
            media_box: PageRect::from_corners(0.0, 0.0, w, h),
            crop_box: PageRect::from_corners(0.0, 0.0, w, h),
            rotate,
            contents: Vec::new(),
            contents_unresolved: 0,
            resources_defaulted: false,
            contents_flattened: 0,
        }
    }

    /// Two `Pos2` are equal within a few `f32` ULPs of accumulated error.
    fn near(a: Pos2, b: Pos2) -> bool {
        (a.x - b.x).abs() <= 1e-3 && (a.y - b.y).abs() <= 1e-3
    }

    #[test]
    fn screen_page_round_trips_at_every_rotation() {
        // Property 1: page_to_screen ∘ screen_to_page == identity, for the
        // extent `page_extent_pts` actually returns at each of the four
        // legal rotations. The four angles test that NOTHING
        // rotation-specific leaks into these functions — they are agnostic
        // to rotation, because `extent` already carries it.
        for &rotate in &[0u16, 90, 180, 270] {
            let page = test_page(200.0, 300.0, rotate);
            let extent = page_extent_pts(&page);
            for &zoom in &[MIN_ZOOM, 0.5, 1.0, 2.5, MAX_ZOOM] {
                let display = egui::vec2(extent.0 * zoom, extent.1 * zoom);
                let rect = Rect::from_min_size(Pos2::new(37.0, 11.0), display);
                for &p in &[
                    Pos2::new(37.0, 11.0),
                    Pos2::new(100.0, 250.0),
                    rect.center(),
                    rect.max,
                ] {
                    let round =
                        page_to_screen(screen_to_page(p, rect, extent, zoom), rect, extent, zoom);
                    // Round-trip within a few ULPs at rotate={0,90,180,270},
                    // zoom across the ladder extremes, for several points.
                    assert!(near(round, p));
                }
            }
        }
    }

    #[test]
    fn screen_to_page_distance_scales_as_one_over_zoom() {
        // Property 2: a fixed SCREEN distance maps to a page-space distance
        // of screen_distance / zoom — the invariance any screen-space snap
        // tolerance relies on.
        let extent = (200.0, 300.0);
        for &zoom in &[MIN_ZOOM, 0.5, 1.0, 3.0, MAX_ZOOM] {
            let rect = Rect::from_min_size(
                Pos2::new(5.0, 9.0),
                egui::vec2(extent.0 * zoom, extent.1 * zoom),
            );
            let a = screen_to_page(Pos2::new(50.0, 50.0), rect, extent, zoom);
            let b = screen_to_page(Pos2::new(90.0, 50.0), rect, extent, zoom);
            let page_dx = (b.x - a.x).abs();
            // A 40px screen span maps to a 40/zoom page span, for every zoom.
            assert!((page_dx - 40.0 / zoom).abs() <= 1e-3);
        }
    }

    #[test]
    fn screen_page_reject_degenerate_inputs_without_panicking() {
        // Property 4: zero/negative/non-finite geometry falls back to a
        // finite, harmless value rather than a NaN or a panic.
        let rect = Rect::from_min_size(Pos2::ZERO, egui::vec2(100.0, 100.0));
        assert_eq!(
            screen_to_page(Pos2::new(5.0, 5.0), rect, (0.0, 100.0), 1.0),
            Pos2::ZERO
        );
        assert_eq!(
            screen_to_page(Pos2::new(5.0, 5.0), rect, (100.0, 100.0), 0.0),
            Pos2::ZERO
        );
        assert_eq!(
            page_to_screen(Pos2::new(5.0, 5.0), rect, (100.0, -1.0), 1.0),
            Pos2::ZERO
        );
        assert_eq!(
            page_to_screen(Pos2::new(5.0, 5.0), rect, (100.0, 100.0), f32::NAN),
            Pos2::ZERO
        );
    }

    #[test]
    fn canvas_pdf_bridge_round_trips_at_every_rotation() {
        // pdf_space_to_canvas ∘ canvas_to_pdf_space is the identity at each
        // rotation.
        for &rotate in &[0u16, 90, 180, 270] {
            let page = test_page(200.0, 300.0, rotate);
            for &p in &[
                Pos2::new(0.0, 0.0),
                Pos2::new(50.0, 80.0),
                Pos2::new(120.0, 240.0),
            ] {
                let user = canvas_to_pdf_space(p, &page).unwrap();
                let back = pdf_space_to_canvas(user, &page).unwrap();
                assert!(near(back, p), "rotate={rotate} p={p:?} back={back:?}"); // ui-text-exempt: test failure message, never displayed
            }
        }
    }

    #[test]
    fn pdf_space_to_canvas_agrees_with_the_renderer_by_construction() {
        // The forward map must equal `page_device_geometry`'s own
        // (already pixel-tested) transform — this is what proves "agrees
        // with the renderer by construction", not merely self-consistent.
        for &rotate in &[0u16, 90, 180, 270] {
            let page = test_page(200.0, 300.0, rotate);
            let (_, _, ctm) = pdfcer_render::page_device_geometry(&page, 1.0);
            for &p in &[
                Pos2::new(0.0, 0.0),
                Pos2::new(200.0, 0.0),
                Pos2::new(0.0, 300.0),
            ] {
                let via_bridge = pdf_space_to_canvas(p, &page).unwrap();
                let via_render = apply_transform(&ctm, p);
                assert!(near(via_bridge, via_render), "rotate={rotate} p={p:?}"); // ui-text-exempt: test failure message, never displayed
            }
        }
    }

    #[test]
    fn pdf_space_bridge_places_the_lower_left_corner_at_the_bottom() {
        // A concrete orientation check, un-rotated: PDF user-space (Y-up)
        // origin (0,0) is the page's lower-left, which in canvas space
        // (Y-down) is the BOTTOM-left — i.e. y == page height.
        let page = test_page(200.0, 300.0, 0);
        let ll = pdf_space_to_canvas(Pos2::new(0.0, 0.0), &page).unwrap();
        assert!(near(ll, Pos2::new(0.0, 300.0)));
        let ul = pdf_space_to_canvas(Pos2::new(0.0, 300.0), &page).unwrap();
        assert!(near(ul, Pos2::new(0.0, 0.0)));
    }
}
