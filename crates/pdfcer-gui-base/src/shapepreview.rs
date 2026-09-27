//! # `shapepreview` — The live geometry preview a gesture draws, in page space, and the hold that keeps it on screen until the raster catches up.

use pdfcer_core::vector::{PaintStyle, Subpath};

/// One object's geometry, in **page space**, ready to be mapped and painted.
#[derive(Debug, Clone, PartialEq)]
pub struct PreviewShape {
    /// Page-space subpaths, already transformed by whatever the gesture is
    /// doing.
    pub subpaths: Vec<Subpath>,
    /// Fill/stroke disposition at paint time (§8.5.3 Table 60).
    pub style: PaintStyle,
    /// Stroke width in **page-space** units.
    pub line_width: f64,
}

/// Everything one gesture is about to change, as geometry.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ShapePreview {
    /// The shapes, in paint order, **at their new position**.
    pub shapes: Vec<PreviewShape>,
    /// The same shapes **where they still are** — the footprint to erase.
    ///
    /// # Why an erase list exists at all
    ///
    /// The page raster underneath is stale: it still shows the object where it
    /// was, and it cannot be re-rendered in under ~0.7 s on the operator's own
    /// drawing (`BENCHMARK.md` — a *two-pixel* region render costs 691 ms
    /// because ~99 % of render cost is content-stream interpretation, not fill).
    ///
    /// So without this the operator sees the object **twice**: once where it
    /// was, painted into the raster, and once where their pointer is. That is
    /// worse than the bounding box this feature replaced.
    ///
    /// # The footprint, not the bounding box — and that is the whole
    /// difference between acceptable and not
    ///
    /// **Ken, 2026-08-30:** *"yeah do both"*, accepting that erasing the old
    /// position would take whatever was underneath with it.
    ///
    /// It takes much less than he agreed to. Because the shell has the real
    /// geometry, the erase is the object's **own outline** — stroked at its own
    /// width, filled where it was filled — rather than a rectangle over it. On a
    /// CAD sheet a bounding box would blank a title-block cell; a stroked
    /// polyline blanks a line's own width.
    ///
    /// ⇒ What is still a lie, stated plainly: anything drawn *underneath the
    /// object's own footprint* disappears for as long as the stale raster is up,
    /// and so does anything drawn *on top* of it there. Bounded to the object's
    /// own ink, transitional, and it ends when the raster lands.
    pub erase: Vec<PreviewShape>,
    /// Whether a cap above stopped this being the whole selection.
    ///
    /// Carried rather than dropped so the painter can decide what to do about
    /// it, and so a check can assert that a big selection produced a *bounded*
    /// preview rather than no preview.
    pub capped: bool,
}
impl ShapePreview {
    /// Whether there is anything to draw.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.shapes.is_empty()
    }

    /// How many segments this preview will paint — the cost, published.
    #[must_use]
    pub fn segment_count(&self) -> usize {
        self.shapes
            .iter()
            .map(|s| s.subpaths.iter().map(|p| p.segments.len()).sum::<usize>())
            .sum()
    }
}
/// A live shape preview kept on screen while the page raster catches up.
///
/// See `OpenDoc::held_preview` for why this exists at all.
pub struct HeldPreview {
    /// The geometry, exactly as the gesture last drew it.
    pub shape: ShapePreview,
    /// `edit_epoch` at the moment the gesture released — **before** the commit.
    ///
    /// The liveness test compares against this rather than against the epoch
    /// the commit produced, because the commit has not happened yet when this is
    /// stored: actions are drained *after* the frame that raised them.
    pub captured_at_epoch: u64,
    /// When it was captured, for the backstop.
    pub since: std::time::Instant,
}
