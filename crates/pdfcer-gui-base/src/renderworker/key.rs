//! # `render::worker::key` — **what a render is OF**, as one comparable value
//!
//!
//! ## Why this is the seam, and not "move the tests out"
//!
//! The obvious way to get a file under the ceiling is to move its `#[cfg(test)]`
//! modules to a sibling, and it would have worked here — there are 375 lines of
//! them. It was rejected because it answers the gate without answering the
//! rule: R2 exists so that a reader can hold a file's subject in their head,
//! and a file whose tests live elsewhere has exactly the same subject it had
//! before, only harder to read.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/renderworker/key.md`.

use super::RenderRequest;

/// What a render is *of* — the staleness keys, as one comparable value.
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
    /// **Whether strokes were drawn at their declared widths or capped at
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
    /// # Why the engine's enum and not a `bool`
    ///
    /// `StrokeDisplay` derives `Eq` and `Hash` — checked on the enum's own
    /// `#[derive]` in `pdfcer-render` — so it is a key component as it stands. It is `#[non_exhaustive]` with room for a third variant — the
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
    /// **Part of the key, and it has to be.** O24's region tier
    /// rasterizes the viewport rather than the page, so two rasters of the
    /// same page at the same scale can now show *different parts of it*.
    /// Without this field the cache would serve the first one for every
    /// position — the operator pans, the picture does not move, and nothing
    /// reports an error because from the cache's side the request was a hit.
    ///
    /// Bit patterns rather than the floats, for the same reason
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
    #[must_use]
    pub fn same_region(&self, other: &Self) -> bool {
        self.region_bits == other.region_bits
    }

    /// The page-space rectangle this raster actually covers, or `None` if it
    /// is a whole-page raster.
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
    #[must_use]
    pub fn page(&self) -> usize {
        self.page_index
    }

    /// The inputs whose change must re-rasterize **immediately**.
    #[must_use]
    pub fn discrete_inputs(&self) -> (usize, bool, u64, pdfcer_render::font::StrokeDisplay) {
        (
            self.page_index,
            self.annotations,
            self.layers_generation,
            // Discrete, not debounced: `view.line_weights` is a button press,
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
    #[must_use]
    pub fn raster_scale(&self) -> f32 {
        f32::from_bits(self.raster_scale_bits)
    }

    /// The key `request` describes.
    pub fn of(request: &RenderRequest) -> Self {
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
