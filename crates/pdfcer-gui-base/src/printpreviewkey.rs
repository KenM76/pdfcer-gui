//! # `printpreviewkey` — what a print-preview bitmap is a picture of
//!
//! The preview cache key, the overhang verdict it is paired with, and the
//! fixed-line-width adjustment the key carries into a render.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/printpreviewkey.md`.

/// What the shown sheet's overhang turned out to contain — the fact the hatch
/// is drawn from, lifted out so the CAPTION can be drawn from the same one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overhang {
    /// The placement reported no clip. The page fits the printable area.
    Fits,
    /// The placement reported a clip **and the overhanging band carries ink**,
    /// so something really will be cropped. Hatched.
    Losing,
    /// The placement reported a clip and the band is **blank paper** — the
    /// 1:1 CAD drawing O113 is about. Nothing hatched, and the caption says so
    /// rather than leaving the operator to wonder why the warning has no
    /// picture.
    BlankBand,
    /// The placement reported a clip and there is **no raster to ask** — the
    /// degraded state [`texture_for`] documents, where the page did not render
    /// and the preview shows a flat fill. The whole band is hatched, because a
    /// failed render must not be able to switch a warning off.
    Unknown,
}

/// What a cached preview bitmap is a picture OF.
#[derive(Debug, Clone, PartialEq)]
pub struct PreviewKey {
    /// Which document page (0-based).
    page: usize,
    /// Which annotation classes are painted.
    scope: pdfcer_render::AnnotationScope,
    /// The operator's configuration, whole — see the type's own docs on why it
    /// is not the rendering fields spelled out.
    settings: pdfcer_core::settings::Settings,
    /// The fixed line width on paper and the page's placement scale (O233);
    /// `None` when the document's own weights print.
    lines: Option<(f64, f64)>,
}

impl PreviewKey {
    /// Build the key for one page.
    pub fn new(
        page: usize,
        scope: pdfcer_render::AnnotationScope,
        settings: &pdfcer_core::settings::Settings,
        lines: Option<(f64, f64)>,
    ) -> Self {
        Self {
            page,
            scope,
            settings: settings.clone(),
            lines,
        }
    }

    /// Set the fixed line width, if any, on a render at `scale` pixels per
    /// page point.
    pub fn apply_lines(&self, options: &mut pdfcer_render::RenderOptions, scale: f64) {
        if let Some((pt, placement)) = self.lines {
            apply_fixed_lines(options, Some(pt), scale, placement);
        }
    }
}

/// Draw every stroke of a render at `paper_pt`. `scale` is the render's pixels
/// per page point, `placement` the page's paper points per page point. Leaves
/// `options` alone when `paper_pt` is `None`.
pub fn apply_fixed_lines(
    options: &mut pdfcer_render::RenderOptions,
    paper_pt: Option<f64>,
    scale: f64,
    placement: f64,
) {
    let Some(pt) = paper_pt else {
        return;
    };
    if !(placement > 0.0 && scale > 0.0) {
        return;
    }
    let device_px = (pt / placement * scale) as f32;
    options.stroke_display = pdfcer_render::font::StrokeDisplay::Fixed { device_px };
}
