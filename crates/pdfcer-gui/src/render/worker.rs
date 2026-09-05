//! # `render::worker` — rasterization on a background thread
//!
//! **Salvaged from `D:\Dev\pdfce\crates\pdfce-gui\src\render_worker.rs`**
//! (Class A, `SALVAGE.md`: *"Generation counter + between-operator
//! cancellation. **Measured**: six rapid zoom steps start six generations
//! and complete one. Do not touch the design."*). The header and every
//! explanatory comment below are carried across; the measured numbers in
//! them are the evidence that justifies the whole module and must not be
//! lost to a paraphrase.
//!
//! ---
//!
//! One job: keep a slow page from freezing the application. This module
//! owns the worker thread, the channel, the cancellation token and the
//! generation counter that `raster.rs` named when it documented itself
//! as the seam where off-thread rendering would happen.
//!
//! ## Why this exists, and what it is NOT
//!
//! **It does not make anything faster.** A page that took 10 s still
//! takes 10 s. What changes is that the 10 s is spent on a thread the
//! operator is not waiting on, so the window keeps repainting, the
//! zoom keeps responding, and the render can be abandoned.
//!
//! The evidence that justified building it: a real CAD sheet measured
//! **~10 s at 1× and ~58 s at 2×**, rasterized inline on the UI thread.
//! At those numbers the application does not render slowly — it stops
//! answering. `raster.rs` predicted exactly this and deferred the work
//! until "a real corpus produces pages slow enough to drop frames".
//!
//! ## The three things that make it correct
//!
//! **A generation counter.** A worker that finishes after its request
//! was superseded must have its result *discarded*, not painted. Every
//! spawn takes the next generation; a reply whose generation is not the
//! current one is dropped. Without this, releasing a zoom gesture would
//! paint whichever render happened to finish last rather than the one
//! that matches the screen.
//!
//! **Cancellation that stops work.** [`RenderCancel`] is polled between
//! content-stream operators, so a superseded render abandons the page
//! rather than running to completion and having its output thrown away.
//! At 58 s a discarded result still occupies a core and still delays
//! whatever the operator asked for next. Measured: **28.9 ms** from
//! `cancel()` to thread exit mid-render, against **10,367 ms** to let
//! one finish.
//!
//! **A bounded in-frame wait.** See [`RenderWorker::spawn`] — this is
//! what keeps a fast page indistinguishable from the synchronous
//! behaviour it replaces.
//!
//! ## What this module does not decide
//!
//! Whether, and how, the canvas discloses that it is showing a stale
//! picture. That is a shell question and it lives in the shell, not here.
//! This module only reports, via [`RenderWorker::in_flight_since`], how
//! long the current render has been outstanding, so the shell can decide.
//!
//! ---
//!
//! ## Salvage note: the staleness keys, and which one is still deferred
//!
//! The original [`RenderKey`] compared **five** inputs. Three were absent at
//! S0, and their absence was a decision rather than an oversight:
//!
//! | key | what it invalidates | state |
//! |---|---|---|
//! | `annotations` | the annotation-visibility toggle (§12.5 `/AP` `/N`) | **landed, S4** |
//! | `layers_generation` | the optional-content layer overrides (§8.11.4.3) | **landed, S4** |
//! | `font_env_generation` | operator-supplied font folders | still deferred |
//!
//! Each was added to the original because **without it the cached texture
//! does not invalidate and the control silently does nothing** — a real,
//! separately-diagnosed defect in all three cases. That is the failure
//! mode to expect: not a crash, a control that appears inert. So the rule
//! for every one of them is: *the key lands in the same commit as the
//! surface that varies it, never later.* Carrying a key with no surface able
//! to change it would put a constant in the request and an untriggerable
//! branch in the comparison — which is the "no state a surface can reach"
//! invariant broken from the other side.
//!
//! ### Why two landed at S4 and the third did not
//!
//! Both of the two are **inputs an operator-facing control now varies**, and
//! both of those controls are `RIBBON_IA.md`'s rather than this module's
//! invention:
//!
//! - `view.show_annotations` is a View ▸ Display control that is already
//!   drawn, already enabled whenever a document has pages, and — until this
//!   key existed — could not have changed a pixel if it had been wired up.
//! - The Layers panel was built **without its visibility checkbox
//!   specifically because this key did not carry `layers_generation`**;
//!   `crate::panels::layers`' own header names that as the false one of its
//!   three preconditions.
//!
//! `font_env_generation` has no such control: nothing in this build lets an
//! operator name a font folder, so the bundled [`pdfcer_render::FontEnvironment`]
//! is the only environment any render can use and a generation counter over
//! it would count to one and stop. It lands with the font-folder surface,
//! under the same rule.
//!
//! ### The other half of the invalidation, which is NOT in this module
//!
//! A key on the *request* only stops the worker from de-duplicating two
//! genuinely different renders. It does not, on its own, make the shell ask
//! for the second one: the shell decides "the cached texture is stale" by
//! comparing the texture's own key against the one it wants. That comparison
//! lives in [`crate::app::state::PdfcerApp::settle_and_rasterize`], and it
//! reads **this same [`RenderKey`]**, recorded on
//! [`crate::render::raster::PageTexture`] when the pixels were uploaded.
//!
//! That is deliberate and it is the structural half of the fix. Before S4 the
//! shell kept its own two-field comparison (page index, raster scale) beside
//! this type's two-field one, and a third key added to one and not the other
//! would compile, run, and produce exactly the inert control this table
//! warns about. There is now one key type, constructed by one function
//! ([`RenderKey::new`]), and adding a field to it changes both sides at once.
//!
//! The original also carried `cmyk_intent`, `fonts` and
//! `view_magnification` on the request. All three have correct defaults in
//! [`pdfcer_render::RenderOptions`] (the operator-ruled `NeutralBlack`
//! intent, the bundled font environment, and `None` = the print-correct
//! `/D`-initial optional-content state), and this build has no surface that
//! varies any of them, so they are left to that default and travel on the
//! request when a settings surface exists to move them.
//!
//! **`view_magnification` deserves one extra sentence**, because it looks
//! adjacent to `layers_generation` and is not. §8.11.4.4's usage
//! applications recompute a layer's state from the zoom, and §8.11.4.5
//! forbids a print or aggregate path from applying them at all (core API
//! trap T-12.8). Leaving it `None` is therefore the *print-correct* answer
//! rather than a gap — and if a viewer ever opts in, it needs no new key of
//! its own, because it is a pure function of `raster_scale`, which is
//! already compared.

use std::sync::Arc;
use std::sync::mpsc::{Receiver, RecvTimeoutError, SyncSender, sync_channel};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use pdfcer_core::edit::EditSession;
use pdfcer_core::page_tree::Page;
use pdfcer_render::cancel::RenderCancel;
use pdfcer_render::{Diagnostics, tiny_skia::Pixmap};

/// How long [`RenderWorker::spawn`] will wait, on the UI thread, for a
/// render it just started.
///
/// # Why a blocking wait is the right answer here
///
/// The requirement is that a page rasterizing in milliseconds behaves
/// exactly as it did when rendering was synchronous — no flash, no
/// spinner, no frame of stale content. Handing every render to a worker
/// and collecting it next frame would cost such a page one frame of
/// staleness for no benefit.
///
/// So the spawn waits briefly and collects the result inline when it
/// arrives. One frame at 60 Hz is ~16.7 ms; this is deliberately under
/// that, so even in the worst case the wait cannot itself drop a frame.
/// A page that beats the deadline never touches the asynchronous path
/// at all, and a page that misses it hands control back to the event
/// loop after a delay the operator cannot perceive.
///
/// This is the one place the UI thread blocks on rendering, it is
/// bounded by a constant, and the bound is the whole point.
const IN_FRAME_BUDGET: Duration = Duration::from_millis(12);

/// A finished rasterization, ready for the shell to upload as a texture.
///
/// The worker produces pixels; it does not touch egui. Texture upload
/// needs an `egui::Context` and belongs on the UI thread, which is also
/// what keeps this module free of any GUI type beyond the ones the
/// shell hands back.
pub struct RenderedPixels {
    /// The rasterized page.
    pub pixmap: Pixmap,
    /// Render-time findings for the diagnostics surface.
    ///
    /// Carried even though S0 has no status bar to show them in: they are
    /// the renderer's honesty report (which glyphs were substituted, which
    /// features were skipped), and a render that produced them and threw
    /// them away would have to be re-run to get them back. The surface
    /// that displays them lands at stage S2.
    pub diagnostics: Diagnostics,
    /// Everything this render was *of*, so the shell can key its texture.
    ///
    /// One field rather than a copy of each input: the texture's staleness is
    /// decided by comparing this against the key the shell currently wants,
    /// and a hand-copied subset is how an input silently stops being
    /// compared. See [`RenderKey`].
    pub key: RenderKey,
    /// **How long the rasterization itself took.**
    ///
    /// Measured around the `pdfcer_render` call and nothing else — not around
    /// the spawn, not around the channel, not around the texture upload — so
    /// the number answers *"how expensive is this page to draw?"* rather than
    /// *"how busy was the machine?"*. Those are different questions, and the
    /// second one already has an answer in the trace (`render-async-done ms=`
    /// includes the queueing).
    ///
    /// Carried on the result rather than left in the trace because
    /// `tools.render_diagnostics` shows it to the operator, and a number a
    /// surface displays cannot come from a diagnostic line nothing parses.
    /// `HANDOFF.md` §10 already records the reason this matters on this
    /// project's documents: ~99 % of render cost is resolution-independent on
    /// dense CAD, so *how long* and *at what scale* only mean something
    /// together — which is why the two travel on one struct.
    pub elapsed: std::time::Duration,
}

/// What a worker sends back: pixels, a failure, or nothing at all.
enum Outcome {
    Done(Box<RenderedPixels>),
    Failed(String),
    /// The render observed its cancellation token and stopped early.
    /// Distinguished from a failure so the shell does not report a
    /// deliberate abandonment as a render error.
    Cancelled,
}

/// What a render is *of* — the staleness keys, as one comparable value.
///
/// # Why this is load-bearing rather than bookkeeping
///
/// The shell decides "the texture is stale" by comparing these keys
/// against the cached texture, and re-runs that decision every frame.
/// While a background render is in flight the texture has NOT been
/// replaced yet, so the decision keeps coming out the same way. Without
/// a way to recognise that the render already running is *for the very
/// request being asked for again*, each frame would cancel the previous
/// render and start an identical one — and a page slower than one frame
/// would never finish. Not a slow render: a render that can never
/// complete, on a page that used to merely be slow.
///
/// `raster_scale` is compared by bit pattern rather than by `==`
/// because it comes from the same arithmetic each frame; an exact float
/// comparison is right here and a tolerance would be wrong, since any
/// difference at all means the shell wants a different picture.
///
/// # ★ It is also the SHELL's staleness key, and that is the point
///
/// This type is public and is recorded on
/// [`crate::render::raster::PageTexture`] because the same comparison has to
/// be made in two places for a control to work:
///
/// 1. **"Is the render already running the one I want?"** — here, in
///    [`RenderWorker::spawn`], or a slow page never finishes.
/// 2. **"Is the picture on screen still a picture of what I am looking
///    at?"** — in [`crate::app::state::PdfcerApp::settle_and_rasterize`], or
///    nothing ever *asks* for the second render.
///
/// Those were two independent field lists until S4, and the failure mode of
/// letting them drift is the one the module docs describe: a control that
/// ticks and changes nothing. One type, one constructor
/// ([`Self::new`]), and a field added to it is compared on both sides
/// or on neither.
///
/// # The two categories of input, and why the split is here
///
/// [`Self::discrete_inputs`] and [`Self::scale_bits`] between them cover
/// every field, and the division is a **policy**, not a convenience:
///
/// - A **discrete** input (page, annotation visibility, layer override) is
///   changed by a command or a click. There is no gesture in flight, no
///   intermediate value on the way to it, and no stale picture worth
///   showing, so it re-rasterizes at once.
/// - The **scale** is changed by a wheel gesture that emits dozens of values
///   on the way to the one that was wanted, so it is debounced
///   (`crate::app::state::ZOOM_SETTLE`) and the existing texture is drawn
///   scaled in the meantime.
///
/// Stating it as two methods rather than as a comment means the shell reads
/// the categories off the key instead of re-deriving them, and a new key
/// added to neither accessor fails
/// [`tests::every_render_input_is_either_discrete_or_the_scale`].
///
/// See the module docs for the one further key this will grow
/// (`font_env_generation`) and the rule that it lands with the surface that
/// varies it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct RenderKey {
    /// Which page (0-based).
    page_index: usize,
    /// The raster scale, by bit pattern — see the type docs.
    raster_scale_bits: u32,
    /// Whether annotation appearances (`/AP` `/N`) are painted over the page
    /// content (§12.5). The `bool` **is** the key: there is no generation to
    /// count, because there is exactly one bit of state.
    annotations: bool,
    /// How many times the operator's optional-content override has changed.
    ///
    /// A **counter, not the set**. The set is a `BTreeSet<ObjId>` that the
    /// staleness check would otherwise compare element-by-element on every
    /// frame, for a value that changes only when a control is clicked. A
    /// monotonic counter answers the same question — *is this a different
    /// override from the one that texture was drawn with?* — in one `u64`
    /// comparison, and it answers it correctly for the case a set comparison
    /// would get wrong nowhere and slower everywhere.
    ///
    /// It counts *changes*, so `0` means "no override at all — obey the
    /// document's own default configuration", which
    /// [`pdfcer_render::LayerVisibility`]'s replace-not-merge contract makes a
    /// genuinely distinct state from "an override that hides nothing" (core
    /// API trap T-12.9).
    layers_generation: u64,
    /// ★★★ **Whether strokes were drawn at their declared widths or capped at
    /// one device pixel** — `view.line_weights`, `OPERATOR_REQUESTS.md` O137.
    ///
    /// # Why this HAD to join the key, and what breaks without it
    ///
    /// It is the first View ▸ Display toggle that changes the **raster** rather
    /// than what the canvas paints over one. Rulers, grid, guides and
    /// show-points are all overlay marks; a cached texture is equally correct
    /// under any of them. A texture drawn under `Actual` is simply a
    /// **different picture** from one drawn under `Hairline`.
    ///
    /// Leave it out and the failure is silent and exactly the operator's
    /// complaint: he presses the button, the cache reports a hit, the old
    /// picture is served, and *"the button never worked"* — the sentence O137
    /// exists to answer — is true again for a second reason. Nothing errors,
    /// no test that only checks the plumbing goes red, and the control looks
    /// inert. `the_render_key_moves_when_line_weights_are_turned_off` is what
    /// makes that a build failure instead.
    ///
    /// # ★ Why the engine's enum and not a `bool`
    ///
    /// `StrokeDisplay` derives `Eq` and `Hash` (checked at
    /// `pdfcer-render/src/font/mod.rs:943`), so it is a key component as it
    /// stands. It is `#[non_exhaustive]` with room for a third variant — the
    /// **opposite** convention, Acrobat's *enhance thin lines* — and a `bool`
    /// here would silently collapse that third state onto one of these two the
    /// day it arrives, which is a stale-raster bug that would look like a
    /// rendering fault.
    ///
    /// A **discrete** input ([`Self::discrete_inputs`]): it is changed by
    /// pressing a button, so there is no gesture in flight and nothing to
    /// debounce.
    stroke_display: pdfcer_render::font::StrokeDisplay,
    /// The page-space rectangle this raster covers, by bit pattern, or
    /// `None` for a whole-page raster.
    ///
    /// ★★ **Part of the key, and it has to be.** O24's region tier
    /// rasterizes the viewport rather than the page, so two rasters of the
    /// same page at the same scale can now show *different parts of it*.
    /// Without this field the cache would serve the first one for every
    /// position — the operator pans, the picture does not move, and nothing
    /// reports an error because from the cache's side the request was a hit.
    ///
    /// ★ Bit patterns rather than the floats, for the same reason
    /// `raster_scale_bits` is: `f64` is not `Eq` or `Hash`, and a key that
    /// compared approximately would make "the same view" a matter of
    /// tolerance. Two requests for one view produce identical bits because
    /// `render::strategy::overscanned` is a pure function of the visible
    /// rect — which is exactly the property its own test pins.
    region_bits: Option<[u64; 4]>,
}

impl RenderKey {
    /// The key for a render of `page_index` at `raster_scale`, with these
    /// annotation and layer settings.
    ///
    /// **The one place a key is computed from parts.** The shell calls it to
    /// ask what it wants; [`Self::of`] calls it to say what a request is.
    /// Two constructors doing the same arithmetic is how the two sides of the
    /// staleness comparison drift.
    #[must_use]
    ///
    /// ★★ `stroke_display` is a **positional parameter and not a builder**,
    /// unlike [`Self::with_region`], and the difference is deliberate. A
    /// builder may be omitted, and an omission here would silently mean
    /// `Actual` — which is the stale-raster bug this field exists to prevent,
    /// wearing the shape of a call site that simply forgot. As a parameter,
    /// every one of the five sites that computes a key has to answer the
    /// question, and the compiler asks it.
    pub fn new(
        page_index: usize,
        raster_scale: f32,
        annotations: bool,
        layers_generation: u64,
        stroke_display: pdfcer_render::font::StrokeDisplay,
    ) -> Self {
        Self {
            page_index,
            raster_scale_bits: raster_scale.to_bits(),
            annotations,
            layers_generation,
            stroke_display,
            region_bits: None,
        }
    }

    /// Narrow this key to a **region** of the page.
    ///
    /// A builder rather than a second constructor, deliberately: this type's
    /// own note warns that *"two constructors doing the same arithmetic is
    /// how the two sides of the staleness comparison drift"*, and a builder
    /// adds a field without repeating any of it. [`Self::new`] stays the one
    /// place the base key is computed.
    #[must_use]
    pub fn with_region(mut self, region: Option<pdfcer_core::page_tree::Rect>) -> Self {
        self.region_bits = region.map(|r| {
            [
                r.llx.to_bits(),
                r.lly.to_bits(),
                r.urx.to_bits(),
                r.ury.to_bits(),
            ]
        });
        self
    }

    /// Whether two keys describe the **same part of the page**.
    ///
    /// # ★★★ Why this had to become its own question
    ///
    /// `OPERATOR_REQUESTS.md` **O25**, 2026-08-23:
    ///
    /// > *"if I pan to far to one side when I am beyond 800% zoom it doesn't
    /// > always render the new exposed area, and the same thing happens
    /// > usually when I zoom out."*
    ///
    /// The staleness test in `render::settle` asked two things — has a
    /// **discrete input** changed (page, annotations, layers), and has the
    /// **scale** changed — and the region was in the key without being in
    /// either. So a pan that changed nothing but *which part of the page is on
    /// screen* was not stale by any measure, and **no render was ever
    /// requested**. The picture the operator had kept being drawn correctly at
    /// its own region and simply slid off, leaving the newly exposed area
    /// blank for as long as they cared to look at it.
    ///
    /// ★ The zoom-out half is the same fault arriving by a different route. A
    /// zoom does change the scale, so a render *is* requested — but the
    /// request is built from whatever region was current when it spawned, and
    /// by the time it lands the gesture has moved on. Once the scale settles,
    /// nothing notices the region it arrived with is the wrong one. Both
    /// symptoms are one missing comparison.
    ///
    /// Compares the **stored bits**, not the reconstructed rectangles: `f64`
    /// is not `Eq`, and a comparison with a tolerance would make "the same
    /// view" a matter of degree in the one place that must answer yes or no.
    #[must_use]
    pub fn same_region(&self, other: &Self) -> bool {
        self.region_bits == other.region_bits
    }

    /// The page-space rectangle this raster actually covers, or `None` if it
    /// is a whole-page raster.
    ///
    /// # ★★★ Why a texture must be placed by ITS OWN region
    ///
    /// `OPERATOR_REQUESTS.md` **O24c**, reported 2026-08-22:
    ///
    /// > *"As I drag using the middle mouse button the pan will follow and
    /// > work, but if I pan a little too far it jumps back in the opposite
    /// > direction I was moving … if I pan the other direction and cross the
    /// > same area where I experienced the jump the pan location jumps back
    /// > to being correct."*
    ///
    /// The current page's texture is served from its slot **without a
    /// staleness check** — deliberately, so a zoom or a pan shows the last
    /// good picture instead of blank paper while the next one renders. That
    /// is the behaviour the operator asked for by name: *"I don't want the
    /// affect that other readers have where you always have to wait for
    /// detail to render after panning to a new area."*
    ///
    /// But the destination rectangle was computed from the region the shell
    /// **now wants**, and `render::strategy::region_for` quantises that to a
    /// half-viewport grid. So the instant a pan crossed a grid line the
    /// destination jumped a whole grid step while the pixels were still the
    /// previous cell's — the picture lurched backwards, held there until the
    /// new raster landed, and snapped right again when the operator panned
    /// back over the same line. Every detail of the report follows from that,
    /// including *"it isn't exactly in the same place as it started"* (the
    /// step is the grid, not the drag) and the page occasionally leaving the
    /// screen entirely (two grid steps at once, at a zoom where the grid is
    /// most of the window).
    ///
    /// ★ The fix is **not** to reject the stale texture. That would blank the
    /// page on every grid crossing — the exact behaviour he ruled out. It is
    /// to draw the stale pixels *where they belong*, so they slide off
    /// naturally as the pan continues and the new raster replaces them in
    /// place. This accessor is what makes that possible: the key already
    /// carried the region, and nothing ever read it back.
    ///
    /// The round-trip through [`f64::to_bits`] is exact, so the rectangle
    /// returned is bit-identical to the one the request was built from — a
    /// placement derived from it cannot disagree with the render by a
    /// rounding step.
    #[must_use]
    pub fn region(&self) -> Option<pdfcer_core::page_tree::Rect> {
        self.region_bits.map(|b| pdfcer_core::page_tree::Rect {
            llx: f64::from_bits(b[0]),
            lly: f64::from_bits(b[1]),
            urx: f64::from_bits(b[2]),
            ury: f64::from_bits(b[3]),
        })
    }

    /// Which page this is a render of.
    ///
    /// Added at Phase 4, and it is what makes a strip's routing correct: a
    /// finished render is labelled with the key it was run from, so
    /// `crate::render::settle` can file it against the page it is *of* rather
    /// than against whatever slot asked for it. Those differ exactly when the
    /// operator scrolled while it was running, which under a continuous mode
    /// is the common case rather than the rare one.
    ///
    /// It is deliberately a separate accessor from [`Self::discrete_inputs`]
    /// even though that tuple's first element is the same number: that method
    /// is a *staleness category* and its shape belongs to the debounce policy,
    /// while this is an identity. A caller that reached for `.0` would be
    /// reading a policy decision as a fact.
    #[must_use]
    pub fn page(&self) -> usize {
        self.page_index
    }

    /// The inputs whose change must re-rasterize **immediately**.
    ///
    /// See the type docs: none of these has a gesture behind it, so waiting
    /// out the zoom debounce would make a click feel unresponsive for no
    /// benefit — and for a page change there is not even a stale picture
    /// worth showing, because it is a picture of a different page.
    #[must_use]
    pub fn discrete_inputs(&self) -> (usize, bool, u64, pdfcer_render::font::StrokeDisplay) {
        (
            self.page_index,
            self.annotations,
            self.layers_generation,
            // ★ Discrete, not debounced: `view.line_weights` is a button press,
            // so there is no intermediate value on the way to the one the
            // operator wanted and nothing to wait out. Waiting would make the
            // toggle feel broken for `ZOOM_SETTLE` milliseconds, which for a
            // control whose whole complaint history is "it never worked" is the
            // worst available latency.
            self.stroke_display,
        )
    }

    /// The one input that is **debounced** rather than committed at once.
    #[must_use]
    pub fn scale_bits(&self) -> u32 {
        self.raster_scale_bits
    }

    /// The raster scale this key names, as the number it was built from.
    ///
    /// The exact inverse of [`Self::new`]'s `to_bits`, and here rather than at
    /// the one call site (`tools.render_diagnostics`) because a bit pattern
    /// reinterpreted by hand is the kind of arithmetic that is right once and
    /// then copied. Staleness still compares [`Self::scale_bits`]: a bit
    /// comparison is total where `f32` equality is not, which is the whole
    /// reason the field is stored as bits.
    ///
    /// **Device pixels per PDF user-space unit** — the operator's zoom already
    /// multiplied by the display's `pixels_per_point`, per this type's own
    /// docs — so it is not the percentage the status bar shows.
    #[must_use]
    pub fn raster_scale(&self) -> f32 {
        f32::from_bits(self.raster_scale_bits)
    }

    /// The key `request` describes.
    fn of(request: &RenderRequest) -> Self {
        Self::new(
            request.page_index,
            request.raster_scale,
            request.annotations,
            request.layers_generation,
            request.stroke_display,
        )
        .with_region(request.region)
    }
}

/// A render currently running on a worker thread.
struct InFlight {
    rx: Receiver<Outcome>,
    cancel: RenderCancel,
    handle: Option<JoinHandle<()>>,
    key: RenderKey,
    generation: u64,
    started: Instant,
}

/// Owns at most one in-flight rasterization.
///
/// Deliberately single-slot: the canvas shows one page at one scale, so
/// a second concurrent render is always a superseded first one. Keeping
/// a queue would mean deciding which of several stale results to paint,
/// which is a question with no good answer.
#[derive(Default)]
pub struct RenderWorker {
    in_flight: Option<InFlight>,
    next_generation: u64,
}

impl std::fmt::Debug for RenderWorker {
    // Hand-written: `Receiver` and `JoinHandle` are not `Debug`, and the
    // useful state is whether something is running and which request it
    // belongs to — not the channel internals.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RenderWorker")
            .field(
                "in_flight_generation",
                &self.in_flight.as_ref().map(|f| f.generation),
            )
            .field("next_generation", &self.next_generation)
            .finish()
    }
}

/// Everything a render needs, owned, so it can cross a thread boundary.
///
/// `DocumentView<'a>` borrows its graph, so the worker cannot be handed
/// one — it is handed the `Arc<EditSession>` and calls `view()` on the
/// far side, where the borrow stays local to the closure. That is the
/// whole reason the open document's session is an `Arc`, and the reason
/// `ObjectGraph` had to gain `Send + Sync` in the engine.
pub struct RenderRequest {
    /// The edit session to render. Rendered through `session.view()` **on
    /// the worker**, never `session.document()` — the view composes the
    /// overlay and the staging buffer, so unsaved edits are what gets
    /// drawn. S0 makes no edits, but the rule is structural: the canvas
    /// renders the *edited* state, and a base read here is how every
    /// editing feature becomes invisible at once.
    pub session: Arc<EditSession>,
    /// The page to draw. Cloned out of the page vector by the caller so
    /// the worker owns it.
    pub page: Page,
    /// Which page (0-based) — a staleness key.
    pub page_index: usize,
    /// Device pixels per PDF user-space unit — the operator's zoom already
    /// multiplied by `pixels_per_point` ([`crate::viewer::raster_scale`]).
    /// The second staleness key.
    pub raster_scale: f32,
    /// Whether annotation appearances are painted over the page content
    /// (§12.5, [`pdfcer_render::RenderOptions::annotations`]).
    ///
    /// The third staleness key, and the first one this build actually varies.
    /// `true` is what a reader does with a file it was handed; `false`
    /// reproduces the content-only raster, which is what View ▸ Display's
    /// `view.show_annotations` exists to ask for.
    pub annotations: bool,
    /// ★★★ **Whether to draw strokes at the widths the file declares, or to cap
    /// every one of them at one device pixel** —
    /// [`pdfcer_render::RenderOptions::stroke_display`], engine `Pass 254.0`.
    ///
    /// `view.line_weights`, `OPERATOR_REQUESTS.md` **O137**, in his words:
    /// *"the button to show all lines without their thickness — thin lines or
    /// something like cad has … I do want that display option!"*
    ///
    /// # ★★★ This is the ONLY place in the crate that carries it, and that is
    /// the whole export guarantee
    ///
    /// The rule the request was built around: **canvas only.** Print, print
    /// preview and every export — PDF, DXF, PNG, JPEG, SVG, EMF, form data,
    /// text — render the document's real widths. *The one thing worse than not
    /// having this feature is having it follow him into a file he sends a
    /// client.* The engine holds the same line from its side and shipped
    /// deliberately **without** a CLI flag for the same reason.
    ///
    /// Here that rule is not prose. `crate::app::settings`' funnel builds every
    /// `RenderOptions` in the crate and never touches this field;
    /// `render_on_worker` — which serves the canvas and nothing else — is the
    /// one function that assigns it, from this request; and
    /// `crate::app::settings::tests::only_the_canvas_worker_sets_stroke_display`
    /// parses every `.rs` in the crate with `syn` and fails the build if a
    /// second site appears. A grep would not do, for the reason that test's
    /// neighbour already gives: the identifier appears in a dozen doc comments,
    /// including this one, and a syntax tree contains no comments at all.
    ///
    /// ★ It IS a staleness key, unlike [`Self::settings`] beside it, and the
    /// difference is that a settings change drops every cached raster
    /// explicitly while this changes several times a minute during ordinary
    /// reading. See [`RenderKey::stroke_display`].
    ///
    /// A snapshot, like everything else here: the worker may finish after the
    /// operator has flipped the toggle again, and what it must report is the
    /// picture it actually drew.
    pub stroke_display: pdfcer_render::font::StrokeDisplay,
    /// ★ The operator's configuration, as of the frame this request was built.
    ///
    /// **Five of the thirteen settings change what a rasterization looks
    /// like** — the CMYK intent, the mask resampling filter, the minification
    /// filter, the CMYK JPEG polarity, and what is drawn for an annotation
    /// with no stated appearance state — and until 2026-08-17 not one of them
    /// reached this worker. The old shell had the same hole: every setting in
    /// that group was persisted, shown in a window, edited by the operator,
    /// and then discarded here by a bare `RenderOptions::default()`.
    ///
    /// It is **not** a staleness key, and that is a decision. Adding it to
    /// [`RenderKey`] would mean deriving `Hash`/`Eq` over a struct that is
    /// `#[non_exhaustive]` in another crate, and the invalidation it would buy
    /// already happens explicitly: `app::settings_window` drops every cached
    /// raster the moment a Save is adopted, which is both more direct and
    /// visible in one place rather than emergent from a key comparison.
    ///
    /// Carried by value so the worker thread owns it. See
    /// `OpenDoc::render_request_for` for why it is cloned rather than shared.
    pub settings: pdfcer_core::settings::Settings,
    /// The operator's optional-content override, or `None` to obey the
    /// document's own default configuration (§8.11.4.3).
    ///
    /// **A complete answer, never a patch.** `pdfcer-render` uses this
    /// *instead of* the document's `/D` configuration rather than merging
    /// with it (core API trap T-12.9), so the caller computes the whole
    /// hidden set — starting from
    /// `pdfcer_core::annot::optional_content_default_off` — and hands it in.
    /// `None` and `Some(empty)` are therefore different renders: the first
    /// obeys the document, the second shows every layer.
    ///
    /// Not itself a staleness key — [`Self::layers_generation`] is, and its
    /// own docs say why a counter beats comparing the set.
    pub layers: Option<pdfcer_render::LayerVisibility>,
    /// How many times the override above has changed — the fourth staleness
    /// key. See [`RenderKey::layers_generation`].
    pub layers_generation: u64,
    /// ★★ **The page-space rectangle to rasterize, or `None` for the whole
    /// page.** O24.
    ///
    /// `None` is today's path and is what every caller asks for at every
    /// zoom the shell currently offers — `render::strategy::for_page` only
    /// answers `Region` above the pixmap ceiling, which `viewer::MAX_ZOOM`
    /// currently stops the operator reaching. So this field is **dormant**
    /// until that ceiling is raised, and wiring it changes nothing today.
    ///
    /// ★ That dormancy is the point of landing it separately: the region
    /// path can be built, keyed and reviewed while it is provably unreachable,
    /// rather than arriving in the same change as the thing that makes it
    /// reachable.
    pub region: Option<pdfcer_core::page_tree::Rect>,
}

impl RenderWorker {
    /// Start rendering `request`, abandoning whatever was running.
    ///
    /// Returns `Some` when the render finished inside
    /// [`IN_FRAME_BUDGET`] — the fast path, which behaves exactly as
    /// the previous synchronous code did. Returns `None` when it is
    /// still running, in which case the shell should keep drawing the
    /// previous texture and call [`Self::poll`] on later frames.
    ///
    /// Cancels the previous render *before* spawning rather than after:
    /// two rasterizations of a CAD page competing for cores make both
    /// slower, and the old one's output is already known to be unwanted.
    pub fn spawn(&mut self, request: RenderRequest) -> Option<Result<RenderedPixels, String>> {
        let key = RenderKey::of(&request);

        // Already rendering exactly this? Leave it alone. See `RenderKey`
        // — without this the per-frame staleness check would cancel and
        // restart the same render forever, and any page slower than one
        // frame would never appear at all.
        if self.in_flight.as_ref().is_some_and(|f| f.key == key) {
            return None;
        }

        self.cancel_in_flight();

        self.next_generation = self.next_generation.wrapping_add(1);
        let generation = self.next_generation;
        let cancel = RenderCancel::new();

        // Capacity 1: the worker sends exactly one message and exits.
        // A bounded channel makes that a compile-time-ish guarantee
        // rather than an unbounded buffer nobody drains.
        let (tx, rx): (SyncSender<Outcome>, Receiver<Outcome>) = sync_channel(1);
        let worker_cancel = cancel.clone();

        // Traced BEFORE the move, because `request` is consumed by the
        // closure. Every generation that starts gets a line, which is what
        // makes the "six rapid zoom steps start six generations and complete
        // one" observation checkable from outside the process rather than
        // being a claim about code that has to be believed.
        //
        // The page and scale ride along because they are the whole
        // `RenderKey`: a trace that says a render started without saying
        // what OF cannot distinguish a legitimate new request from the
        // restart-the-same-render livelock the key exists to prevent.
        let (traced_page, traced_scale) = (request.page_index, request.raster_scale);
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        crate::diag::trace(|| {
            format!("render-spawn gen={generation} page={traced_page} scale={traced_scale}")
        });

        let handle = std::thread::spawn(move || {
            let outcome = render_on_worker(&request, &worker_cancel);
            // A send failure means the shell dropped the receiver — the
            // document was closed, or a later render superseded this
            // one and the slot was replaced. Both are ordinary; there is
            // nobody left to tell.
            let _ = tx.send(outcome);
        });

        let started = Instant::now();

        // The bounded in-frame wait. See IN_FRAME_BUDGET.
        match rx.recv_timeout(IN_FRAME_BUDGET) {
            Ok(outcome) => {
                // Finished inside the budget: join immediately so no
                // thread outlives the call, and return inline.
                let _ = handle.join();
                let elapsed_ms = started.elapsed().as_millis();
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                crate::diag::trace(|| {
                    format!("render-inline gen={generation} ms={elapsed_ms} async=0")
                });
                Self::outcome_to_result(outcome)
            }
            Err(RecvTimeoutError::Timeout) => {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                crate::diag::trace(|| {
                    format!(
                        "render-async-started gen={generation} budget_ms={}",
                        IN_FRAME_BUDGET.as_millis()
                    )
                });
                self.in_flight = Some(InFlight {
                    rx,
                    cancel,
                    handle: Some(handle),
                    key,
                    generation,
                    started,
                });
                None
            }
            Err(RecvTimeoutError::Disconnected) => {
                // The worker panicked without sending. Surface it as a
                // render failure rather than hanging forever waiting for
                // a message that will never arrive.
                let _ = handle.join();
                Some(Err(crate::text::canvas_render_worker_stopped().to_owned()))
            }
        }
    }

    /// Collect a finished render, if one is ready. Never blocks.
    ///
    /// Returns `None` both when nothing is running and when the render
    /// is still going — the shell's action is the same either way.
    pub fn poll(&mut self) -> Option<Result<RenderedPixels, String>> {
        let flight = self.in_flight.as_mut()?;
        match flight.rx.try_recv() {
            Ok(outcome) => {
                let mut flight = self.in_flight.take()?;
                if let Some(handle) = flight.handle.take() {
                    let _ = handle.join();
                }
                let elapsed_ms = flight.started.elapsed().as_millis();
                let generation = flight.generation;
                let kind = match &outcome {
                    Outcome::Done(_) => "done",
                    Outcome::Cancelled => "cancelled",
                    Outcome::Failed(_) => "failed",
                };
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                crate::diag::trace(|| {
                    format!("render-async-done gen={generation} ms={elapsed_ms} outcome={kind}")
                });
                Self::outcome_to_result(outcome)
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => None,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                let mut flight = self.in_flight.take()?;
                if let Some(handle) = flight.handle.take() {
                    let _ = handle.join();
                }
                Some(Err(crate::text::canvas_render_worker_stopped().to_owned()))
            }
        }
    }

    /// How long the current render has been outstanding, if any.
    ///
    /// The shell uses this to decide whether the canvas has been stale
    /// long enough to say so. Returning the duration rather than a
    /// boolean keeps the threshold — a presentation decision — out of
    /// this module.
    #[allow(
        dead_code,
        reason = "the stale-canvas disclosure is a status-bar sentence and lands at stage S2; kept with the clock it reads, because the threshold is the shell's decision and the measurement is this module's" // ui-text-exempt: clippy lint justification, never displayed
    )]
    pub fn in_flight_since(&self) -> Option<Duration> {
        self.in_flight.as_ref().map(|f| f.started.elapsed())
    }

    /// Whether a render is currently running.
    pub fn is_rendering(&self) -> bool {
        self.in_flight.is_some()
    }

    /// **What the worker is rendering right now**, if anything.
    ///
    /// Two callers need it and both are Phase 4's:
    ///
    /// * a page that is being drawn *now* says so
    ///   ([`crate::render::strip::PageState::Drawing`]) rather than saying it
    ///   is waiting, because those are different promises to the operator;
    /// * a **failure** arrives from the worker as a bare message with no key
    ///   of its own, so the page it is about has to be read from the slot
    ///   before [`Self::poll`] takes it. Without that, a strip page that would
    ///   not draw would be attributed to the current page and blank the whole
    ///   canvas.
    ///
    /// Returns the whole key rather than the page index because the second
    /// caller files the failure under it, and a key rebuilt at that call site
    /// could disagree with the one the render actually ran from.
    #[must_use]
    pub fn rendering_key(&self) -> Option<RenderKey> {
        self.in_flight.as_ref().map(|f| f.key)
    }

    /// Stop any in-flight render and wait for the thread to exit.
    ///
    /// **This is the choke point that makes `Arc<EditSession>`
    /// workable.** A worker holds a clone of the session for as long as
    /// it renders, so `Arc::get_mut` fails while one is running. Every
    /// mutation must go through a path that calls this first — so by the
    /// time any edit touches the session, the render holding the other
    /// reference has exited.
    ///
    /// The alternative rulings were rejected with numbers: blocking the
    /// edit until the render finishes costs up to 58 s, which is the
    /// freeze this whole module exists to remove; snapshotting the
    /// session would need a public deep-copy impl on `EditSession`
    /// (which is not `Clone`) and would copy the document per edit.
    /// Cancel-then-mutate costs the measured **28.9 ms** of teardown.
    ///
    /// S0 makes no edits, so nothing calls this yet outside [`Drop`]. It
    /// is salvaged now, with its argument intact, because the first edit
    /// to arrive without it would reintroduce the 58-second freeze
    /// through a door that had already been closed once.
    #[allow(
        dead_code,
        reason = "the mutation choke point; S0 has no mutations, and the first stage that does (S4) must route through this rather than re-derive it" // ui-text-exempt: clippy lint justification, never displayed
    )]
    pub fn cancel_and_wait(&mut self) {
        self.cancel_in_flight();
    }

    /// Cancel, drain and join. Idempotent.
    fn cancel_in_flight(&mut self) {
        let Some(mut flight) = self.in_flight.take() else {
            return;
        };
        flight.cancel.cancel();
        if let Some(handle) = flight.handle.take() {
            // Join rather than detach: the whole point is that the
            // session's other reference is gone when this returns. A
            // detached thread might still be holding it.
            let _ = handle.join();
        }
    }

    fn outcome_to_result(outcome: Outcome) -> Option<Result<RenderedPixels, String>> {
        match outcome {
            Outcome::Done(pixels) => Some(Ok(*pixels)),
            Outcome::Failed(message) => Some(Err(message)),
            // A cancelled render has no result and is not a failure.
            // The shell keeps whatever it was already showing.
            Outcome::Cancelled => None,
        }
    }
}

impl Drop for RenderWorker {
    /// Closing a document must not leave a 58-second render running
    /// against a session nobody can see.
    fn drop(&mut self) {
        self.cancel_in_flight();
    }
}

/// The worker body. Runs on the spawned thread; touches no GUI type.
fn render_on_worker(request: &RenderRequest, cancel: &RenderCancel) -> Outcome {
    // ★ Through the funnel, never `RenderOptions::default()`.
    //
    // `crate::app::settings::SettingsExt` is the one place that turns the
    // operator's configuration into render options, and a `syn` check in that
    // module fails the build if any other file constructs these itself. The
    // reason is the defect it replaced: a bare `::default()` here is correct
    // in isolation and silently discards five settings, which is exactly how
    // the old shell came to persist nine settings it never read.
    //
    // Everything NOT set below keeps whatever the funnel produced: the bundled
    // font environment (reproducible on any machine) and `None` view
    // magnification (the print-correct answer, T-12.8). Each becomes a request
    // field when a surface exists to vary it, not before.
    use crate::app::settings::SettingsExt;
    let mut options = request.settings.render_options();
    options.cancel = Some(cancel.clone());
    options.annotations = request.annotations;
    // ★★★ **The canvas's stroke-width display convention, and the ONE
    // assignment of this field in the whole crate** — O137,
    // `RenderOptions::stroke_display`, engine `Pass 254.0`.
    //
    // This function draws the interactive canvas and nothing else. Print, print
    // preview and every export build their options through the same funnel and
    // never reach this line, so they render the document's REAL widths — which
    // is the constraint the feature was designed around and the one that makes
    // it safe to ship at all. `Actual` is `RenderOptions::default()`'s value, so
    // an export path does not have to remember to say anything.
    //
    // The rule is enforced rather than asserted: see the request field's docs
    // and `crate::app::settings::tests::only_the_canvas_worker_sets_stroke_display`.
    options.stroke_display = request.stroke_display;
    // Cloned rather than moved because the worker takes the request by
    // reference — a `BTreeSet<ObjId>` per render, against a rasterization
    // measured in seconds. `None` here is not the same as an empty set: it
    // means "obey the document" (T-12.9), and collapsing the two would
    // reveal every layer the document turned off.
    options.layers = request.layers.clone();

    // `session.view()`, NOT `session.document()` — the view composes the
    // overlay and the staging buffer, so unsaved edits are what gets drawn.
    // The borrow lives and dies inside this function, which is why the
    // request can own the `Arc` and still hand `render_page_with_view` a
    // reference.
    let view = request.session.view();
    // ★ The clock starts here and nowhere else. Everything above is option
    // assembly and a borrow; everything below is a `match` on the result. What
    // is timed is therefore the rasterization, which is what
    // `RenderedPixels::elapsed` says it is — and the two lines are adjacent so
    // that a future statement inserted between them is visibly inside the
    // measurement rather than accidentally so.
    let started = Instant::now();
    // ★★ O24: the region tier. `None` is the whole-page path this shell has
    // always taken; `Some` rasterizes only the rectangle asked for, so the
    // pixmap stops scaling with the zoom.
    //
    // ★ The two calls are deliberately adjacent and share everything above
    // them — the same view, the same options, the same scale, the same clock.
    // A second assembly path for the region case is how the five settings that
    // reach this worker would come to reach only one of them.
    let rendered = match request.region {
        None => pdfcer_render::render_page_with_view(
            &view,
            &request.page,
            request.raster_scale,
            &options,
        ),
        Some(region) => pdfcer_render::render_page_region(
            &view,
            &request.page,
            request.raster_scale,
            region,
            &options,
        ),
    };
    match rendered {
        Ok(rendered) => {
            // ★★★ **The compositing space, published per raster.**
            //
            // Added 2026-08-26. The operator reported colours changing with
            // zoom; the cause is that `pdfcer-render` composites a page with
            // transparency in a subtractive CMYK buffer only while that buffer
            // fits under `MAX_CMYK_BUFFER_BYTES`, and falls back to sRGB above.
            // Which side of that a given raster landed on is **invisible in a
            // screenshot** and is the single most useful fact about why two
            // renders of one page disagree.
            //
            // It is traced here rather than inferred from the scale, because
            // the ceiling is on the buffer the renderer actually tried to
            // allocate — which for a REGION render is the region's size, not
            // the page's. Deriving it from the zoom would be a second
            // derivation of a rule this shell cannot even read.
            crate::diag::trace_on_change("raster-blend-space", || {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "cmyk_buffer={} refused={} wrong_space={} scale={:.3}",
                    rendered.diagnostics.cmyk_buffer_engaged,
                    rendered.diagnostics.cmyk_buffer_refused,
                    rendered.diagnostics.blends_in_wrong_space,
                    request.raster_scale
                )
            });
            Outcome::Done(Box::new(RenderedPixels {
                pixmap: rendered.pixmap,
                diagnostics: rendered.diagnostics,
                // The key is derived from the request the render was actually
                // run from, so the texture cannot be labelled with anything but
                // the inputs that produced it.
                key: RenderKey::of(request),
                elapsed: started.elapsed(),
            }))
        }
        Err(e) if cancel.is_cancelled() => {
            // Deliberate abandonment, not a defect. Checking the token
            // rather than matching the error variant keeps this correct
            // if the render gains other early-exit paths.
            let _ = e;
            Outcome::Cancelled
        }
        Err(e) => Outcome::Failed(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfcer_render::font::StrokeDisplay;

    /// A `RenderKey` for the default render of a page at a scale:
    /// annotations on (what a reader shows), no layer override.
    ///
    /// The two-argument shorthand the geometry cases use, so a test about
    /// page and scale is not obscured by two constants it does not vary.
    fn key(page_index: usize, scale: f32) -> RenderKey {
        RenderKey::new(page_index, scale, true, 0, StrokeDisplay::Actual)
    }

    /// Two renders of the same thing must compare EQUAL.
    ///
    /// # Why this is the load-bearing test and not bookkeeping
    ///
    /// The shell re-runs its staleness check every frame, and while a
    /// background render is in flight the cached texture has not been
    /// replaced — so the check keeps saying "stale" and keeps asking
    /// for the same render. `spawn` recognises that request as the one
    /// already running *only* through this equality.
    ///
    /// If it fails, every frame cancels the render the previous frame
    /// started and begins an identical one. A page slower than a single
    /// frame then **never finishes at all** — which is strictly worse
    /// than the freeze this module was written to remove, and it would
    /// look like a hang rather than a bug.
    ///
    /// This was a real defect in the original's first draft: the guard did
    /// not exist, and the livelock was reasoned out before it could be
    /// observed.
    #[test]
    fn the_same_request_twice_is_recognised_as_the_same_render() {
        assert_eq!(key(3, 2.0), key(3, 2.0));
    }

    /// Every staleness key must be part of the comparison.
    ///
    /// The test above cannot distinguish a correct `RenderKey` from one
    /// that compares nothing at all and reports every pair as equal — and
    /// that failure is not hypothetical. A key that ignored a field would
    /// make the guard swallow a *genuine* new request: change the zoom,
    /// and the shell would decline to re-render because it believes the
    /// in-flight job already covers it. The page would stop responding to
    /// zoom entirely.
    ///
    /// So each field is varied one at a time. Dropping any single field
    /// from `RenderKey`'s `PartialEq` fails exactly one of these — and
    /// each key this struct grows (see the module docs) must add its own
    /// line here in the same commit.
    #[test]
    fn changing_any_single_render_input_makes_a_different_key() {
        let base = RenderKey::new(3, 2.0, true, 7, StrokeDisplay::Actual);
        assert_ne!(
            base,
            RenderKey::new(4, 2.0, true, 7, StrokeDisplay::Actual),
            "page index must be compared"
        );
        assert_ne!(
            base,
            RenderKey::new(3, 2.5, true, 7, StrokeDisplay::Actual),
            "raster scale must be compared"
        );
        assert_ne!(
            base,
            RenderKey::new(3, 2.0, false, 7, StrokeDisplay::Actual),
            "annotation visibility must be compared, or View ▸ Display's \
             `view.show_annotations` toggles a bool and redraws nothing"
        );
        assert_ne!(
            base,
            RenderKey::new(3, 2.0, true, 8, StrokeDisplay::Actual),
            "the layer-override generation must be compared, or the Layers \
             panel's visibility control ticks and redraws nothing — which is \
             the exact defect that kept the checkbox out of the build"
        );
    }

    /// ★★★ **TWO REGIONS OF ONE PAGE ARE DIFFERENT KEYS** — O24.
    ///
    /// The region tier rasterizes the viewport rather than the page, so two
    /// rasters of the same page at the same scale can show different parts of
    /// it. If the region were not in the key the cache would serve the first
    /// for every position: **the operator pans and the picture does not move**,
    /// with nothing reporting an error, because from the cache's side every
    /// request was a hit.
    ///
    /// That is the worst shape of defect this project keeps finding — silent,
    /// and indistinguishable from a frozen canvas.
    #[test]
    fn a_region_is_part_of_the_key() {
        use pdfcer_core::page_tree::Rect;
        let whole = RenderKey::new(3, 2.0, true, 7, StrokeDisplay::Actual);
        let left = whole.with_region(Some(Rect::from_corners(0.0, 0.0, 100.0, 100.0)));
        let right = whole.with_region(Some(Rect::from_corners(100.0, 0.0, 200.0, 100.0)));

        assert_ne!(whole, left, "a region raster is not the whole-page raster");
        assert_ne!(
            left, right,
            "two different regions are two different rasters"
        );
        assert_eq!(
            left,
            whole.with_region(Some(Rect::from_corners(0.0, 0.0, 100.0, 100.0))),
            "the same region must be the same key, or nothing ever caches"
        );
        assert_eq!(
            whole,
            left.with_region(None),
            "clearing the region returns the whole-page key"
        );
    }

    /// **★ Every field is in exactly one of the two staleness categories.**
    ///
    /// [`RenderKey::discrete_inputs`] and [`RenderKey::scale_bits`] are how
    /// the shell decides whether a change re-rasterizes **now** or waits out
    /// the zoom debounce. A field that appears in neither is a change the
    /// shell cannot see at all: the key would compare unequal, the worker
    /// would happily run the new render — and nothing would ever ask for it,
    /// because the texture would still look current.
    ///
    /// That is not the same failure as an uncompared field, and it is worse:
    /// the module's own key would be *correct* while the picture stayed
    /// wrong, so the obvious place to look would be the innocent one.
    ///
    /// Each field is varied one at a time and the pair is asserted to move.
    /// A key this struct grows must add its line here in the same commit,
    /// exactly as it must to the test above.
    #[test]
    fn every_render_input_is_either_discrete_or_the_scale() {
        let base = RenderKey::new(3, 2.0, true, 7, StrokeDisplay::Actual);
        let moved = |k: RenderKey| {
            k.discrete_inputs() != base.discrete_inputs() || k.scale_bits() != base.scale_bits()
        };
        assert!(
            moved(RenderKey::new(4, 2.0, true, 7, StrokeDisplay::Actual)),
            "page index"
        );
        assert!(
            moved(RenderKey::new(3, 2.5, true, 7, StrokeDisplay::Actual)),
            "raster scale"
        );
        assert!(
            moved(RenderKey::new(3, 2.0, false, 7, StrokeDisplay::Actual)),
            "annotations"
        );
        assert!(
            moved(RenderKey::new(3, 2.0, true, 8, StrokeDisplay::Actual)),
            "layers generation"
        );
        assert!(
            moved(RenderKey::new(3, 2.0, true, 7, StrokeDisplay::Hairline)),
            "stroke display"
        );
    }

    /// **Turning line weights off makes every cached raster stale** —
    /// `OPERATOR_REQUESTS.md` **O137**, and the assertion without which the
    /// whole feature can ship inert.
    ///
    /// # The vacuous test this replaces, and it is the likeliest mistake here
    ///
    /// A test that `view.line_weights` is *plumbed* — that the request carries
    /// it and the worker assigns it — **passes on a build where the cache
    /// serves the old picture.** The operator presses the button, the strip
    /// reports a hit, the texture drawn under `Actual` is drawn again, and
    /// nothing anywhere reports an error. From his chair that is *"the button
    /// never worked"*, which is the sentence O137 exists to answer, arriving
    /// for a second reason.
    ///
    /// So the property asserted is not "the field reaches the renderer". It is
    /// **the key moves**, in both of the ways the shell compares keys:
    ///
    /// * `RenderKey` equality, which `render::strip` uses to decide whether a
    ///   cached raster may be served at all; and
    /// * [`RenderKey::discrete_inputs`], which `render::settle` uses to decide
    ///   whether to re-rasterize **at once** rather than after `ZOOM_SETTLE`.
    ///
    /// The second matters on its own: a stroke display that landed in the
    /// *scale* category would make the toggle take 150 ms to do anything, on a
    /// control whose entire complaint history is that it did nothing.
    ///
    /// And the reverse, so the test cannot pass by making every key unequal:
    /// two keys that agree about line weights and about everything else must
    /// still be equal.
    #[test]
    fn the_render_key_moves_when_line_weights_are_turned_off() {
        let shown = RenderKey::new(3, 2.0, true, 7, StrokeDisplay::Actual);
        let hairline = RenderKey::new(3, 2.0, true, 7, StrokeDisplay::Hairline);

        assert_ne!(
            shown, hairline,
            "a raster drawn with real widths would be served for a hairline view, so the \
             toggle would look exactly as inert as the dead button it replaces"
        );
        assert_ne!(
            shown.discrete_inputs(),
            hairline.discrete_inputs(),
            "the stroke display is not a DISCRETE input, so the toggle would wait out the \
             zoom debounce before doing anything"
        );
        assert_eq!(
            shown.scale_bits(),
            hairline.scale_bits(),
            "turning line weights off is not a zoom"
        );
        assert_eq!(
            shown,
            RenderKey::new(3, 2.0, true, 7, StrokeDisplay::Actual),
            "two keys agreeing about everything must still be equal — otherwise the assertion \
             above is satisfied by a key that is never equal to anything"
        );
        // The region builder must carry it too, because the region tier is
        // exactly where a dense CAD sheet is read at 200-400 % — which is the
        // zoom this feature exists for.
        let region = pdfcer_core::page_tree::Rect {
            llx: 0.0,
            lly: 0.0,
            urx: 100.0,
            ury: 100.0,
        };
        assert_ne!(
            shown.with_region(Some(region)),
            hairline.with_region(Some(region)),
            "the region tier lost the stroke display, so the toggle would be inert at exactly \
             the zooms the operator asked for it"
        );
    }

    /// **The scale is the ONLY debounced input.**
    ///
    /// The other half of the split, asserted from the other side: if a
    /// discrete input leaked into the scale category it would inherit the
    /// 150 ms zoom debounce, and a click on the annotation toggle would take
    /// a fifth of a second to do anything for no reason an operator could
    /// see. If the scale leaked into the discrete category, every notch of a
    /// wheel gesture would rasterize a CAD sheet — the behaviour
    /// `ZOOM_SETTLE` exists to remove.
    #[test]
    fn only_the_raster_scale_is_debounced() {
        let base = RenderKey::new(3, 2.0, true, 7, StrokeDisplay::Actual);
        // A scale change moves the scale category and NOT the discrete one.
        let rescaled = RenderKey::new(3, 2.5, true, 7, StrokeDisplay::Actual);
        assert_ne!(rescaled.scale_bits(), base.scale_bits());
        assert_eq!(rescaled.discrete_inputs(), base.discrete_inputs());
        // …and each discrete change moves the discrete category and NOT the
        // scale.
        for changed in [
            RenderKey::new(4, 2.0, true, 7, StrokeDisplay::Actual),
            RenderKey::new(3, 2.0, false, 7, StrokeDisplay::Actual),
            RenderKey::new(3, 2.0, true, 8, StrokeDisplay::Actual),
        ] {
            assert_eq!(changed.scale_bits(), base.scale_bits());
            assert_ne!(changed.discrete_inputs(), base.discrete_inputs());
        }
    }

    /// A scale difference far below any perceptible threshold is still a
    /// different render.
    ///
    /// Comparing `f32` by bit pattern rather than by a tolerance is
    /// deliberate. The shell derives `raster_scale` from the same
    /// arithmetic each frame, so an unchanged zoom yields bit-identical
    /// values and the guard holds; but any difference at all means the
    /// shell has asked for a different picture, and a tolerance would
    /// silently serve it the wrong one.
    #[test]
    fn a_one_bit_scale_difference_is_a_different_render() {
        let a = key(0, 1.0);
        let b = key(0, f32::from_bits(1.0f32.to_bits() + 1));
        assert_ne!(a, b);
    }

    /// A fresh worker is idle, and reports no in-flight age.
    ///
    /// Guards the (stage S2) status-bar disclosure against the most
    /// embarrassing failure mode: announcing that the canvas is behind
    /// when nothing is rendering.
    #[test]
    fn an_idle_worker_reports_nothing_in_flight() {
        let worker = RenderWorker::default();
        assert!(!worker.is_rendering());
        assert!(worker.in_flight_since().is_none());
    }

    /// Dropping the worker must not leave a thread running.
    ///
    /// The `Drop` impl exists because closing a document must not leave a
    /// 58-second render running against a session nobody can see. There is
    /// no page to render in a unit test, so what is checked is the weaker
    /// but still meaningful property that the teardown path is reachable
    /// and idempotent on an idle worker — a `cancel_in_flight` that
    /// panicked or blocked on an empty slot would hang every close.
    #[test]
    fn cancelling_an_idle_worker_is_a_harmless_no_op() {
        let mut worker = RenderWorker::default();
        worker.cancel_and_wait();
        worker.cancel_and_wait();
        assert!(!worker.is_rendering());
    }
}

#[cfg(test)]
mod region_accessor_tests {
    use super::RenderKey;
    use pdfcer_core::page_tree::Rect;
    use pdfcer_render::font::StrokeDisplay;

    fn key() -> RenderKey {
        RenderKey::new(0, 1.5, true, 0, StrokeDisplay::Actual)
    }

    /// A whole-page raster has no region, and must say so rather than
    /// inventing one — the caller switches on exactly this to choose between
    /// filling the page's rect and placing a sub-rectangle.
    #[test]
    fn a_whole_page_key_reports_no_region() {
        assert_eq!(key().region(), None);
        assert_eq!(key().with_region(None).region(), None);
    }

    /// ★★ The round-trip must be EXACT, not close.
    ///
    /// The placement is computed from what comes back out, and the render was
    /// run from what went in. A rounding step between them is a rounding step
    /// between the pixels and where they are drawn — which at a high zoom is a
    /// visible offset, and is the class of defect O24c was.
    #[test]
    fn the_region_round_trips_bit_exactly() {
        let awkward = Rect {
            llx: 0.1 + 0.2,
            lly: -1.0 / 3.0,
            urx: 1e12 + 0.5,
            ury: f64::MIN_POSITIVE,
        };
        let back = key().with_region(Some(awkward)).region().expect("a region");
        assert_eq!(back.llx.to_bits(), awkward.llx.to_bits());
        assert_eq!(back.lly.to_bits(), awkward.lly.to_bits());
        assert_eq!(back.urx.to_bits(), awkward.urx.to_bits());
        assert_eq!(back.ury.to_bits(), awkward.ury.to_bits());
    }

    /// ★ Two keys that differ only by region are different keys, and each
    /// reports its own.
    ///
    /// This is what lets a held texture be placed by the region it is a
    /// picture of while the shell is already asking for the next one. If the
    /// accessor returned the shell's wanted region — or if the builder
    /// mutated in place and both keys ended up agreeing — the placement would
    /// silently follow the request instead of the pixels, which is the exact
    /// shape of the defect this pair exists to prevent.
    #[test]
    fn neighbouring_regions_stay_distinguishable() {
        let a = Rect {
            llx: 0.0,
            lly: 0.0,
            urx: 100.0,
            ury: 100.0,
        };
        let b = Rect {
            llx: 50.0,
            lly: 0.0,
            urx: 150.0,
            ury: 100.0,
        };
        let ka = key().with_region(Some(a));
        let kb = key().with_region(Some(b));
        assert_ne!(ka, kb, "two regions of one page must not share a key");
        assert_eq!(ka.region().expect("a").llx, 0.0);
        assert_eq!(kb.region().expect("b").llx, 50.0);
    }
}
