//! # `render::worker::key` — **what a render is OF**, as one comparable value
//!
//! Split out of [`super`] on 2026-09-11 under **R2**, when wiring
//! `pdfcer-render`'s `RasterizerLimit` refusal took `worker.rs` to 1,541 lines.
//!
//! ## ★ Why this is the seam, and not "move the tests out"
//!
//! The obvious way to get a file under the ceiling is to move its `#[cfg(test)]`
//! modules to a sibling, and it would have worked here — there are 375 lines of
//! them. It was rejected because it answers the gate without answering the
//! rule: R2 exists so that a reader can hold a file's subject in their head,
//! and a file whose tests live elsewhere has exactly the same subject it had
//! before, only harder to read.
//!
//! [`super`] has two subjects and they change for different reasons:
//!
//! * **the worker** — a thread, a channel, a cancellation token, one in-flight
//!   slot, and the mapping from `pdfcer-render`'s result to an [`super::Outcome`].
//!   It changes when the rasterisation contract does, which this week meant a
//!   new refusal variant.
//! * **the key** — which inputs make two pictures different. It changes when a
//!   new *control* is added: a layer override, a stroke-display mode, a region.
//!
//! Nothing in this file mentions a thread, and nothing in [`super`]'s worker
//! half decides what makes a picture stale. Two subjects, two rates of change,
//! which is this project's test for a seam.
//!
//! ## What did NOT move
//!
//! [`RenderKey`]'s tests. They stay in [`super`]'s test module beside the
//! worker's, because several of them assert the **pairing** — that the key the
//! worker spawns with is the key the texture is stamped with — and splitting
//! an assertion from one of its two subjects is how the pairing stops being
//! tested by either.

use super::RenderRequest;

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
    ///
    /// `pub(crate)` rather than private since 2026-09-10, for one caller:
    /// `render::settle`'s `OpenDoc::rasterize` stamps
    /// `OpenDoc::render_in_flight` with this before the inline wait, so a
    /// refusal that arrives *inside* the frame budget still knows which
    /// request it is about. See `OpenDoc::render_refused` for why that
    /// mattered - a panicked worker drops its channel and reports
    /// `Disconnected` immediately, so the fast path is the one every panic
    /// takes.
    pub(crate) fn of(request: &RenderRequest) -> Self {
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
