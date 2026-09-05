//! # viewer — the page-view state machine and its geometry
//!
//! **Salvaged from `D:\Dev\pdfce\crates\pdfce-gui\src\viewer.rs`** (Class A,
//! `SALVAGE.md`: 509 code lines, 413 test lines, *"zoom ladder with provable
//! reversibility, fit modes re-derived per frame, per-page raster ceiling
//! accounting for `pixels_per_point`. Well tested."*). Carried across with
//! its documentation and its entire test suite intact, per the salvage
//! procedure's rule that a snippet leaves the reasoning behind and the next
//! engineer re-derives a decision that was already paid for.
//!
//! Changes made during salvage, and only these:
//!
//! - `use eframe::egui::…` became `use egui::…` — this crate names `egui`
//!   directly (see `Cargo.toml`), so the re-export hop is gone.
//! - Two functions that have no consumer until stage S4 gained an explicit
//!   `#[allow(dead_code, reason = …)]`, in the same shape the original
//!   already used for its own not-yet-called geometry bridges. They are
//!   kept rather than deleted because they are the *pair* of a live
//!   function, and a bridge with only one direction implemented is how the
//!   two ends drift apart.
//! - Nothing else. No arithmetic changed, no test was weakened.
//!
//! **Phase 4 landed the page *range*, and it is not a field here.** This
//! module's own known-future-work note used to read *"a page range rather than
//! a single `page_index` (`GUI_ROADMAP` Phase 4.1, the continuous-scroll
//! prerequisite)"*, and the answer turned out to be that a range is not
//! something a view *holds*. Which pages are on screen falls out of where the
//! pages are laid out and where the viewport is, so [`strip`] computes it and
//! [`ViewState`] keeps exactly one index — now meaning *"the page the operator
//! is looking at"*, derived from the scroll position under a continuous mode
//! and set by navigation under a paged one. See [`strip::Strip::page_at_view`].
//!
//! What [`ViewState`] did gain is [`ViewState::display`]: which of the four
//! arrangements is active. It is a fourth axis of the view, orthogonal to the
//! three below, and it is documented in [`display`].
//!
//! **Phase 3.1 is done and it needed nothing from this module.** Anchoring a
//! zoom is a question about the *scroll offset*, not about the ladder, so the
//! rule and the solve live in [`crate::canvas::zoom`] and
//! [`crate::canvas::geometry`]. Two things here are reused rather than
//! reimplemented, and that reuse is the point:
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
/// ★★★ **Where the view is, when the scroll offset can no longer say** —
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
/// ★★ **How far this page can actually be zoomed** — the three limits that
/// bind at three different depths, reconciled in one place.
///
/// Its header carries which is which: the raster ceiling stops mattering
/// once the region tier engages, the `f32` scroll offset's is what the shell
/// can honestly offer today, and the operator's setting is the third.
/// ★ **The zoom levels the `+` and `−` buttons step through**, and the rule
/// for what happens past the last named rung.
///
/// Split out under R2. Its header carries the one property that matters: the
/// two steps must be exact inverses, above the ladder as well as on it —
/// O24g was the half that was not.
pub mod ladder;
// How the zoom is decided from the viewport: the three fitting modes, the
// ratio each takes, and which axes each one PLACES the view on. Split out
// under R2 on 2026-08-24, when O28 and O29 took this file past 1,500 lines.
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
    /// # ★★★ Why an anchor has to name its page (`OPERATOR_REQUESTS.md` O26d)
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
    /// ★ Under [`PageDisplay::Single`] there is one page at the origin and
    /// this field is always the current one, so nothing about that path
    /// changes. It is the strip that made "which page" a question.
    pub page: usize,
}

/// Which page is shown, at what scale, how that scale is chosen, and in what
/// arrangement.
///
/// ## ★ `PartialEq` is here for one test, and it is the right one
///
/// Added 2026-08-17 with [`crate::app::prefs::Prefs::seed_view`], whose
/// contract is *"seeding from the shipped preferences leaves a freshly opened
/// view untouched"*. That property is only assertable as **whole-struct
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
    /// # ★ What this means under a continuous mode, and who writes it
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
    /// Wired 2026-08-28, after the blocker on that command was re-derived and
    /// found stale. The recorded reason was *"there is nothing for it to
    /// show — this build draws no anchor mark at any rung"*, which was true
    /// when it was written and stopped being true on 2026-08-19 when the
    /// multi-node move landed with `canvas::overlay::draw_anchors`. The dead
    /// sentence had **three copies** and `FEATURES.md` contradicted it twelve
    /// lines from one of them.
    ///
    /// ★ Default **off**, with the other three: it is a drafting aid, and an
    /// operator who has not asked for hollow squares over their drawing should
    /// not get them. `crate::app::prefs` can make that an opening preference
    /// the day somebody wants one.
    pub show_points: bool,
    /// ★★★ **`view.line_weights` — are strokes drawn at the widths the file
    /// declares, or every one of them at one device pixel?**
    ///
    /// `OPERATOR_REQUESTS.md` **O137**, in his words, 2026-09-05:
    ///
    /// > *"awhile ago you told me you removed the button to show all lines
    /// > without their thickness — thin lines or something like cad has. The
    /// > button never worked but I do want that display option!"*
    ///
    /// # ★★★ Which convention this is, because the two are opposites
    ///
    /// | | | precedent |
    /// |---|---|---|
    /// | **this, turned OFF** | every stroke drawn at **one device pixel**, whatever the file declares | AutoCAD `LWDISPLAY` off |
    /// | *not* this | sub-pixel strokes bumped **up** to one pixel so they do not vanish | Acrobat's *enhance thin lines* |
    ///
    /// **One makes thick things thin. The other makes thin things thick.** He
    /// said *"without their thickness"* and named CAD, so this is the first.
    /// Shipping the second would be worse than shipping nothing, because it
    /// would look like the feature working while doing the opposite.
    ///
    /// # ★★ Why the field is named for the WEIGHTS and `true` is the default
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
    /// # ★★★ It is the ONLY member of this group that changes the RASTER
    ///
    /// Rulers, grid, guides and show-points are all drawn by the canvas
    /// **over** a finished page texture; none of them can make a cached raster
    /// wrong. This one is a [`pdfcer_render::RenderOptions`] field
    /// (`stroke_display`, engine `Pass 254.0`), so a texture drawn while it was
    /// on is a *different picture* from one drawn while it was off — and a
    /// cache that served the old one would make the toggle look inert, which is
    /// precisely the defect O137 reports about its predecessor.
    ///
    /// ⇒ It is therefore a **staleness key**:
    /// [`crate::app::state::OpenDoc::render_key_for`] feeds
    /// [`Self::stroke_display`] into [`crate::render::worker::RenderKey::new`],
    /// and `the_render_key_moves_when_line_weights_are_turned_off` is what
    /// stops that being forgotten.
    ///
    /// # ★★★ Canvas only — the constraint decided in writing before the engine
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
    /// # ★ Fills are untouched
    ///
    /// Only `S`/`s`/`B`/`B*`-painted strokes reach the engine's `stroke_params`
    /// (`pdfcer-render/src/interpret.rs:8806-8812`), so a hatch built out of
    /// thin *fills* cannot vanish. Said here because an operator whose hatching
    /// is fill-based would otherwise expect it to thin out with everything
    /// else.
    ///
    /// # ★ Per document, not global
    ///
    /// It lives here, beside `zoom` and `rulers`, so two open drawings can
    /// disagree: comparing a hairline read of a dense sheet against a faithful
    /// read of the sheet beside it is the actual job. There is deliberately no
    /// persisted default in `crate::app::prefs` — see
    /// `crate::text::commands::view_line_weights` for that decision and where a
    /// preference would go if he asks for one.
    pub line_weights: bool,
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
            // ★ Off, with the other three. See the field's own note: an
            // operator who has not asked for hollow squares over their drawing
            // should not get them.
            show_points: false,
            // ★★★ **ON**, and it is the one member of this group whose default
            // is not `false` — because `true` here means *draw the document as
            // it says it should be drawn*, not *draw something extra*. A fresh
            // view therefore rasterizes byte for byte what every build before
            // O137 rasterized, and the operator's gesture is turning line
            // weights OFF. See the field's own docs for why the toggle is
            // named for the weights rather than for the hairline.
            line_weights: true,
        }
    }
}

impl ViewState {
    /// ★★★ **[`Self::line_weights`] as the engine spells it** — the one place
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
    /// # ★★ Why the return type is the engine's ENUM and not a `bool`
    ///
    /// `StrokeDisplay` is `#[non_exhaustive]` with two variants today —
    /// `Actual` and `Hairline` — and the engine made it an enum deliberately so
    /// that Acrobat's *enhance thin lines* (the **opposite** convention: thin
    /// things made thick) can arrive as a third variant. A `hairline: bool`
    /// anywhere in this shell would, that day, come to mean *"one of the two"*.
    /// So the boolean stops here and the engine's vocabulary starts here.
    ///
    /// ★ `Hairline` is the **off** position. `true` means faithful widths; see
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
        // ★★★ `f64`, not `u32` — `OPERATOR_REQUESTS.md` O24j.
        //
        // A saturating `as u32` cast clamps at 4,294,967,295, so the status
        // bar read **4294967295%** at a trillion percent — u32::MAX presented
        // as a measurement. Seen in the deep-zoom screenshot gallery, and it
        // is the kind of number an operator quite reasonably reads as a crash.
        //
        // ★ The type was right when `MAX_ZOOM` was 8.0 and every reachable
        // value fitted in three digits. O24 raised the ceiling to 10¹² and did
        // not revisit it — which is the recurring shape of this whole request:
        // a limit lifted in one place while a narrower type downstream keeps
        // enforcing the old one silently.
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
/// - **`pixels_per_point` is part of the calculation.** The zoom the
///   operator sees is a *logical* scale (points per PDF unit); the raster
///   is made at `zoom × pixels_per_point` so it stays sharp on a HiDPI
///   display (see [`raster_scale`]). On a 2× display, therefore, every
///   page hits the pixmap ceiling at half the zoom it otherwise would —
///   omitting this factor is how a guard like this passes its tests and
///   still fails on the one machine that matters.
/// - **A one-pixel guard band is subtracted** before dividing, because
///   the renderer computes its pixmap edge with `ceil()`: a scale that
///   divides out to exactly the limit rounds *up* past it and is refused.
///
/// [`raster_scale`]: crate::viewer::raster_scale
#[must_use]
pub fn max_zoom_for_page(page_pts: (f32, f32), pixels_per_point: f32) -> f32 {
    let longest = page_pts.0.max(page_pts.1);
    let ppp = if pixels_per_point.is_finite() && pixels_per_point > 0.0 {
        pixels_per_point
    } else {
        1.0
    };
    if !longest.is_finite() || longest <= 0.0 {
        return MAX_ZOOM;
    }
    #[allow(
        clippy::cast_precision_loss,
        reason = "MAX_PIXMAP_EDGE is 16384; f32 is exact to 2^24" // ui-text-exempt: clippy lint justification, never displayed
    )]
    let ceiling = (pdfcer_render::MAX_PIXMAP_EDGE - 1) as f32 / (longest * ppp);
    // `clamp` is safe here (MIN_ZOOM < MAX_ZOOM, neither is NaN, and
    // `ceiling` is finite because `longest` and `ppp` were both checked
    // above) — so clippy's `manual_clamp` suggestion is taken rather
    // than suppressed.
    ceiling.clamp(MIN_ZOOM, MAX_ZOOM)
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
    let ppp = if pixels_per_point.is_finite() && pixels_per_point > 0.0 {
        pixels_per_point
    } else {
        1.0
    };
    // ★ The operator's quality multiplier — 2026-08-17.
    //
    // `RIBBON_IA.md` §5.2 commissioned this as View ▸ Render ▸ Quality and
    // `manifest::DIRECTED` carried it as *"partial G — the raster-scale
    // multiplier is a compiled-in constant today. What is new is the knob, not
    // the value."* That description was optimistic: there was no multiplier at
    // all, compiled-in or otherwise. This function was `zoom * ppp` exactly.
    //
    // So the knob and the value arrived together, and `Normal` is `1.0` — one
    // raster pixel per device pixel, which is what the line above produced
    // before and is what a build that never opens the Settings window still
    // gets, byte for byte.
    zoom * ppp * quality.multiplier()
}

/// A page's on-screen extent in PDF user-space units, with `/Rotate`
/// already applied (a 90°-rotated portrait page is landscape on screen).
///
/// Delegates to [`pdfcer_render::page_device_geometry`] at scale `1.0`
/// rather than reading `page.crop_box` directly. That is the point: the
/// GUI's idea of how big a page is and the renderer's idea of how big
/// the pixmap will be come from **one** function, so they cannot drift
/// apart — a fit-page computed from an un-rotated CropBox against a
/// rotated raster is the classic version of this bug.
#[must_use]
pub fn page_extent_pts(page: &Page) -> (f32, f32) {
    let (w, h, _) = pdfcer_render::page_device_geometry(page, 1.0);
    #[allow(
        clippy::cast_precision_loss,
        reason = "page edges are bounded by MAX_PIXMAP_EDGE" // ui-text-exempt: clippy lint justification, never displayed
    )]
    (w as f32, h as f32)
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
#[allow(clippy::float_cmp, reason = "ladder rungs are exact f32 literals")] // ui-text-exempt: clippy lint justification, never displayed
mod tests {
    /// ★★★ **The ladder can actually REACH a configured maximum**, stepping.
    ///
    /// `zoom_ceiling` answering a big number is necessary and not sufficient:
    /// the `+` button walks `ZOOM_LADDER`, which ends at 8.0. If stepping
    /// stopped there the setting would be honoured by every code path except
    /// the one the operator actually uses, which is the same silently-inert
    /// control in a subtler place.
    ///
    /// ★ This is the gap `OPERATOR_REQUESTS.md` O24 predicted in its own
    /// words — *"the buttons stop working exactly where the setting starts
    /// mattering"* — asserted rather than left to be discovered.
    #[test]
    fn the_zoom_ladder_can_climb_to_a_configured_maximum() {
        let ceiling = zoom_ceiling((1584.0, 1224.0), 1.0, 500_000.0);
        let mut zoom = 1.0_f32;
        for _ in 0..200 {
            let mut view = ViewState {
                zoom,
                ..ViewState::default()
            };
            view.zoom_in(ceiling);
            if (view.zoom - zoom).abs() < f32::EPSILON {
                break;
            }
            zoom = view.zoom;
        }
        assert!(
            zoom > 100.0,
            "stepping stalled at {zoom}x against a ceiling of {ceiling}x — the ladder \
             cannot reach the configured maximum, so the setting is inert for the +/- \
             buttons even though `zoom_ceiling` honours it"
        );
    }

    /// ★★★ **THE SETTING IS NOT DECORATIVE** — the whole risk of O24.
    ///
    /// `OPERATOR_REQUESTS.md` O24 warned in as many words that shipping the
    /// setting without the mechanism would produce *"a control that is drawn,
    /// accepted, persisted, and quietly overruled downstream"* — the operator
    /// types 100,000 % and the zoom stops near a thousand with nothing said.
    ///
    /// This is that failure, stated as an assertion. `zoom_ceiling` must
    /// answer the operator's configured maximum wherever it is higher than
    /// the whole-page raster limit, on a page large enough that the raster
    /// limit really does bind.
    #[test]
    fn a_configured_maximum_is_honoured_past_the_whole_page_raster_limit() {
        let a1 = (1584.0_f32, 1224.0);
        let whole_page = max_zoom_for_page(a1, 1.0);
        assert!(
            whole_page < 20.0,
            "the premise: an A1 sheet's whole-page ceiling is around 1,000% ({whole_page})"
        );

        // ★ Below the positional cap the configured maximum is honoured
        // exactly. `10_000%` and `100_000%` are both well inside it on an A1
        // sheet, whose cap is around 1,050,000%.
        for percent in [10_000.0_f32, 100_000.0] {
            let ceiling = zoom_ceiling(a1, 1.0, percent);
            assert!(
                (ceiling - percent / 100.0).abs() / (percent / 100.0) < 1e-6,
                "{percent}% was overruled: ceiling {ceiling}, wanted {}",
                percent / 100.0
            );
        }

        // ★★ …and a TRILLION percent is honoured too, since tier 3 wired the
        // `f64` position model. The cap that stood here until then is gone; the
        // same constant now decides when `DeepAnchor` takes over instead of
        // when to refuse.
        // ★★ …and above it the STRIP EXTENT is what binds now, not the raster
        // and not the scroll offset. Asking for a trillion percent yields the
        // deepest zoom the page is confirmed to actually draw at.
        let deep = zoom_ceiling(a1, 1.0, 1e12);
        assert!(
            (deep - 1e10).abs() / 1e10 < 1e-6,
            "a trillion percent must be honoured in full now that nothing caps it: {deep}x"
        );
        assert!(
            deep_position_needed(a1, deep),
            "…and at that zoom the f64 anchor must be the one positioning the view"
        );
        assert!(
            deep > 1_000_000.0,
            "the cap must still be past 100,000,000%: {deep}x"
        );
    }

    /// ★★★ **The default reaches the maximum** — the operator's instruction of
    /// 2026-08-22, *"Also set the default to be able to hit the maximum zoom."*
    ///
    /// This test previously asserted the opposite: that the default reproduced
    /// the old ceiling exactly, so a fresh install was unchanged. That was the
    /// cautious call and he overruled it — a capability you have to find a
    /// preferences file to switch on is one most of its users never have.
    ///
    /// ★ The property is kept, not dropped: **what must not change is the
    /// PANNING**, which is what he actually cares about. That is asserted by
    /// `every_zoom_the_shell_offers_today_still_rasterizes_the_whole_page` in
    /// `render::strategy`, which walks the whole ladder — the ceiling is
    /// permission, and the strategy is behaviour.
    #[test]
    fn the_default_reaches_the_maximum_on_every_page_and_display_scale() {
        for page in [(1584.0_f32, 1224.0), (612.0, 792.0), (306.0, 396.0)] {
            for ppp in [1.0_f32, 1.5, 2.0] {
                let ceiling = zoom_ceiling(page, ppp, crate::app::prefs::DEFAULT_MAX_ZOOM_PERCENT);
                // ★ The default asks for the maximum and now GETS it, on every
                // page and display scale — which is only honest because tier 3
                // positions the view past the point an `f32` offset could.
                // ★ The default asks for the maximum and gets the deepest the
                // strip can still place a page at — which is what the shell can
                // actually deliver, on every page size.
                let wanted = crate::app::prefs::DEFAULT_MAX_ZOOM_PERCENT / 100.0;
                assert!(
                    (ceiling - wanted).abs() / wanted < 1e-6,
                    "page {page:?} at {ppp}x: ceiling {ceiling} should be {wanted}"
                );
                assert!(
                    ceiling > 1_000_000.0,
                    "every page must reach past 100,000,000%: {page:?} got {ceiling}x"
                );
            }
        }
    }

    /// ★★ **…and a LOW setting still lets the pixmap ceiling bind.**
    ///
    /// The half that survives from the test this replaced, and it is the one
    /// that stops the change being dangerous: below `MAX_ZOOM` the whole-page
    /// raster limit is a real constraint — an A1 sheet at 1.5x tops out at
    /// 690 %, not 800 % — and asking past it would demand a raster the engine
    /// refuses.
    #[test]
    fn a_low_setting_does_not_lift_the_whole_page_raster_limit() {
        let a1 = (1584.0_f32, 1224.0);
        let whole_page = max_zoom_for_page(a1, 1.5);
        assert!(
            whole_page < MAX_ZOOM,
            "the premise: {whole_page} < {MAX_ZOOM}"
        );

        // A setting BELOW the pixmap ceiling must not raise it…
        let ceiling = zoom_ceiling(a1, 1.5, 300.0);
        assert!(
            (ceiling - whole_page).abs() < 1e-4,
            "a 300% setting should leave the {whole_page}x pixmap ceiling alone, got {ceiling}"
        );
    }
    /// ★★★ **The position model changes hands exactly where an `f32` offset
    /// stops being able to place the view** — O24 tier 3.
    ///
    /// One unit of content space is one screen pixel, so `2^24` content points
    /// is the last extent at which the offset is exact. Below it the scroll
    /// area is authoritative and the canvas is unchanged; above it
    /// `viewer::deep::DeepAnchor` is.
    ///
    /// ★ Asserted on both sides of the threshold, because a predicate that
    /// answered `true` everywhere would put the whole shell on the deep path —
    /// and that path is the one that has never carried ordinary use.
    #[test]
    fn the_deep_position_model_takes_over_only_past_the_sub_pixel_extent() {
        let letter = (612.0_f32, 792.0);
        let threshold = ceiling::SUB_PIXEL_CONTENT_EXTENT / letter.1;

        assert!(
            !deep_position_needed(letter, threshold * 0.9),
            "below the extent the scroll offset must stay authoritative"
        );
        assert!(
            deep_position_needed(letter, threshold * 1.1),
            "above it the f64 anchor must take over"
        );

        // Every zoom the shell has ever offered stays on the ordinary path.
        for zoom in ZOOM_LADDER {
            assert!(
                !deep_position_needed(letter, *zoom),
                "zoom {zoom} left the ordinary position model"
            );
        }

        // Degenerate input never claims to need the deep path.
        for bad in [f32::NAN, f32::INFINITY, 0.0, -1.0] {
            assert!(!deep_position_needed(letter, bad));
            assert!(!deep_position_needed((bad, bad), 1.0));
        }
    }

    /// ★★★ **The page's size stops mattering once regions are available** —
    /// O24.
    ///
    /// This is the whole point of the region tier stated as an assertion. In
    /// the whole-page tier an A0 sheet hits its ceiling far sooner than a
    /// business card, because the ceiling is a pixmap size and the page is in
    /// it. With regions the pixmap is the window, so both pages reach the same
    /// limit — the operator's.
    #[test]
    fn with_regions_the_page_size_no_longer_caps_the_zoom() {
        let huge = (3370.0_f32, 2384.0); // A0
        let tiny = (180.0_f32, 252.0); // a business card

        // Whole-page tier: the two pages have very different ceilings.
        assert!(
            max_zoom_for_page(tiny, 1.0) > max_zoom_for_page(huge, 1.0),
            "the whole-page ceiling must depend on the page's size"
        );

        // Region tier: neither page enters the arithmetic.
        let limit = 10_000.0_f32;
        assert!((max_zoom_with_regions(limit) - limit).abs() < f32::EPSILON);
    }

    /// A stored limit that is nonsense must not make the document unzoomable.
    #[test]
    fn a_broken_limit_falls_back_to_the_floor_rather_than_to_zero() {
        for bad in [f32::NAN, f32::NEG_INFINITY, -5.0, 0.0, MIN_ZOOM / 2.0] {
            assert!(
                (max_zoom_with_regions(bad) - MIN_ZOOM).abs() < f32::EPSILON,
                "{bad} should fall back to MIN_ZOOM"
            );
        }
    }

    /// ★ **Infinity is not a limit**, and is refused rather than passed
    /// through — an infinite ceiling would propagate into a scroll extent and
    /// blank the canvas, which is the failure `geometry`'s guards exist for.
    #[test]
    fn an_infinite_limit_is_refused() {
        assert!((max_zoom_with_regions(f32::INFINITY) - MIN_ZOOM).abs() < f32::EPSILON);
    }

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

    /// ★★★ O24j — **the readout must survive the ceiling it now offers.**
    ///
    /// `zoom_percent` returned a `u32`, and `as u32` saturates — so at a
    /// trillion percent the status bar showed **4294967295%**, `u32::MAX`
    /// presented as a measurement. Found in the deep-zoom screenshot gallery,
    /// which is the only instrument that reads the number an operator reads.
    ///
    /// ★ Asserted against the FORMATTED string, because that is the artefact
    /// with the defect in it. A test of the numeric value would have passed on
    /// a build that formatted it through a narrower type further downstream.
    #[test]
    fn the_readout_survives_the_whole_configured_range() {
        let mut v = ViewState::default();
        for (zoom, want) in [
            (1.0_f32, "100%"),
            (8.0, "800%"),
            (1.0e6, "100000000%"),
            // ★★ Not "1000000000000%", and the difference is not a defect.
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
        assert_eq!(max_zoom_for_page((612.0, 792.0), 1.0), MAX_ZOOM);
    }

    #[test]
    fn an_annex_c_maximum_page_is_constrained() {
        // 14,400 user units is ISO 32000-1 Annex C's largest page edge.
        let max = max_zoom_for_page((14_400.0, 14_400.0), 1.0);
        assert!(max < MAX_ZOOM);
        // And the ceiling must actually keep the raster legal: the
        // renderer ceil()s, so check the rounded-up edge too.
        let edge = (14_400.0_f32 * max).ceil() as u32;
        assert!(edge <= pdfcer_render::MAX_PIXMAP_EDGE);
    }

    #[test]
    fn the_ceiling_is_what_actually_clamps_zoom_in_on_a_huge_page() {
        let page = (14_400.0, 14_400.0);
        let max = max_zoom_for_page(page, 1.0);
        let mut v = ViewState::default();
        for _ in 0..20 {
            v.zoom_in(max);
        }
        assert_eq!(v.zoom, max);
        assert!((page.0 * v.zoom).ceil() as u32 <= pdfcer_render::MAX_PIXMAP_EDGE);
    }

    #[test]
    fn degenerate_page_extent_does_not_produce_a_nonsense_ceiling() {
        assert_eq!(max_zoom_for_page((0.0, 0.0), 1.0), MAX_ZOOM);
        assert_eq!(max_zoom_for_page((f32::NAN, 10.0), 1.0), MAX_ZOOM);
    }

    // ---- HiDPI ----------------------------------------------------

    #[test]
    fn raster_scale_multiplies_zoom_by_the_display_density() {
        assert_eq!(
            raster_scale(1.5, 2.0, crate::app::prefs::RenderQuality::Normal),
            3.0
        );
        assert_eq!(
            raster_scale(1.5, 1.0, crate::app::prefs::RenderQuality::Normal),
            1.5
        );
    }

    #[test]
    fn a_nonsense_pixels_per_point_is_treated_as_one() {
        // egui should never hand us these, but a zero here would render
        // a zero-size pixmap and a NaN would render nothing at all —
        // both far worse than ignoring a bad density.
        assert_eq!(
            raster_scale(2.0, 0.0, crate::app::prefs::RenderQuality::Normal),
            2.0
        );
        assert_eq!(
            raster_scale(2.0, f32::NAN, crate::app::prefs::RenderQuality::Normal),
            2.0
        );
        assert_eq!(
            max_zoom_for_page((14_400.0, 1.0), 0.0),
            max_zoom_for_page((14_400.0, 1.0), 1.0)
        );
    }

    #[test]
    fn the_raster_ceiling_accounts_for_display_density() {
        // The bug this pins: a guard computed in logical points passes
        // on a 1x developer monitor and blows the pixmap limit on a 2x
        // laptop, because the raster is twice as many pixels.
        let page = (14_400.0, 14_400.0);
        let max_1x = max_zoom_for_page(page, 1.0);
        let max_2x = max_zoom_for_page(page, 2.0);
        assert!(max_2x < max_1x);
        let edge = (page.0 * raster_scale(max_2x, 2.0, crate::app::prefs::RenderQuality::Normal))
            .ceil() as u32;
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
