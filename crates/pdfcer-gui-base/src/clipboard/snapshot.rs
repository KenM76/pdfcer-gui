//! # `clipboard::snapshot` — the snapshot box, copied out cut to the box
//!
//! Has the engine cut the region under the box out of the page, with the
//! viewer's annotation and layer state, then places that one-page drawing
//! through [`super::place`] at the operator's snapshot resolution, in the
//! formats and order a page copy uses.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/clipboard/snapshot.md`.

use pdfcer_core::document::Document;
use pdfcer_core::page_tree::{Page, Rect};
use pdfcer_core::pageops::{RegionExport, RegionReport, extract_region};
use pdfcer_render::RenderOptions;
use pdfcer_render::tiny_skia::Pixmap;

use super::CopyPayload;
use super::place::{
    Refusal, emf_options, place, place_withheld, raster_at, render_options, svg_options,
};

/// The largest picture a snapshot makes, in pixels.
pub const MAX_PIXELS: f64 = 50_000_000.0;

/// What a snapshot copy put on the clipboard.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotCopy {
    /// The format names, in placement order.
    pub formats: Vec<&'static str>,
    /// The resolution the picture was made at.
    pub dpi: u32,
    /// The resolution the operator set; above [`Self::dpi`] when the box was
    /// too large to hold it.
    pub asked_dpi: u32,
    /// The picture's width in pixels.
    pub width_px: u32,
    /// The picture's height in pixels.
    pub height_px: u32,
    /// Whether the vector forms were placed, and what the operator is owed
    /// about them.
    pub vectors: Vectors,
}

/// What became of the vector forms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Vectors {
    /// Placed, cut to the box: nothing outside it is in any format.
    Cut {
        /// Characters crossing the box's edge, left out whole.
        glyphs_removed: u64,
        /// The engine's sentences about anything it removed or could not
        /// flatten.
        notes: Vec<String>,
    },
    /// Withheld, the picture placed alone: drawing outside the box could not
    /// be cut away, so a vector form would have carried it.
    Withheld {
        /// The engine's sentences naming what survived.
        notes: Vec<String>,
    },
    /// Withheld, the picture placed alone: the engine refused the region.
    Refused(String),
}

impl Vectors {
    /// The trace token.
    #[must_use]
    pub const fn token(&self) -> &'static str {
        match self {
            Self::Cut { .. } => "cut", // ui-text-exempt: a trace token, never displayed
            Self::Withheld { .. } => "withheld", // ui-text-exempt: a trace token, never displayed
            Self::Refused(_) => "refused", // ui-text-exempt: a trace token, never displayed
        }
    }
}

/// **Copy the document's snapshot box to the clipboard.** `NoPage` when there
/// is no box, or it lies off its page.
pub fn copy_snapshot(doc: &crate::opendoc::OpenDoc) -> Result<SnapshotCopy, Refusal> {
    let laid = doc.snapshot.ok_or(Refusal::NoPage)?;
    let page = doc.pages.get(laid.page).ok_or(Refusal::NoPage)?;
    let region = cropped(page, laid.rect).ok_or(Refusal::NoPage)?;
    let asked_dpi = doc.prefs.snapshot.dpi();
    let dpi = fitted_dpi(&region.crop_box, asked_dpi);
    #[allow(clippy::cast_precision_loss)]
    // ui-text-exempt: a clippy lint name, never displayed
    let dpi_f = dpi as f32;
    let options = render_options(doc);
    let mut state = RegionExport::new().with_annotations(options.annotations);
    if let Some(hidden) = &doc.layers.hidden {
        state = state.with_hidden_layers(hidden.iter().copied());
    }
    // The view, so unsaved edits are what is copied — as `place::page_payload`.
    let view = doc.session.view();
    let (payload, vectors) = match extract_region(&view, laid.page, region.crop_box, &state) {
        Ok((bytes, report)) => cut_payload(bytes, &report, &options, dpi_f)?,
        Err(error) => {
            let rendered = pdfcer_render::render_page_with_view(
                &view,
                &region,
                crate::imageexport::scale_for(dpi_f),
                &options,
            )
            .map_err(|error| Refusal::Render(error.to_string()))?;
            let picture = picture(rendered.pixmap, dpi_f)?;
            (picture, Vectors::Refused(error.to_string()))
        }
    };
    let (width_px, height_px) = payload
        .pixmap
        .as_ref()
        .map_or((0, 0), |p| (p.width(), p.height()));
    let formats = match vectors {
        Vectors::Cut { .. } => place(&payload)?,
        _ => place_withheld(&payload)?,
    };
    Ok(SnapshotCopy {
        formats,
        dpi,
        asked_dpi,
        width_px,
        height_px,
        vectors,
    })
}

/// Every format from the engine's one-page cut, or the picture alone when the
/// report says something outside the box survived it.
fn cut_payload(
    bytes: Vec<u8>,
    report: &RegionReport,
    options: &RenderOptions,
    dpi: f32,
) -> Result<(CopyPayload, Vectors), Refusal> {
    let render = |error: &dyn std::fmt::Display| Refusal::Render(error.to_string());
    let cut = Document::from_bytes(bytes).map_err(|e| render(&e))?;
    let pages = pdfcer_core::page_tree::pages(&cut).map_err(|e| render(&e))?;
    let page = pages.first().ok_or(Refusal::NoPage)?;
    // The cut has no layers left, and the source's layer ids name nothing in it.
    let mut options = options.clone();
    options.layers = None;
    let scale = crate::imageexport::scale_for(dpi);
    let rendered =
        pdfcer_render::render_page_with(&cut, page, scale, &options).map_err(|e| render(&e))?;
    if report.has_residuals() {
        let notes = report.notes.clone();
        return Ok((picture(rendered.pixmap, dpi)?, Vectors::Withheld { notes }));
    }
    let svg = pdfcer_render::svg::export_svg(&cut, page, &options, &svg_options(dpi))
        .map_err(|e| render(&e))?;
    let emf = pdfcer_render::emf::export_emf(&cut, page, &options, &emf_options(dpi))
        .map_err(|e| render(&e))?;
    let vectors = Vectors::Cut {
        glyphs_removed: report.glyphs_removed,
        notes: report.notes.clone(),
    };
    Ok((raster_at(svg.svg, emf.emf, rendered.pixmap, dpi)?, vectors))
}

/// The picture formats alone, stamped with `dpi`.
fn picture(pixmap: Pixmap, dpi: f32) -> Result<CopyPayload, Refusal> {
    let png = pdfcer_render::export::encode_png(&pixmap, Some(dpi))
        .map_err(|error| Refusal::Render(error.to_string()))?;
    Ok(CopyPayload {
        png: Some(png),
        pixmap: Some(pixmap),
        pixels_per_metre: super::pixels_per_metre(dpi),
        ..CopyPayload::default()
    })
}

/// The page with its crop box narrowed to `rect`; `None` when the box and the
/// page's crop box do not overlap. Its crop box is the region the engine cuts,
/// and the frame of the picture when the cut is refused.
#[must_use]
pub fn cropped(page: &Page, rect: Rect) -> Option<Page> {
    let crop = page.crop_box;
    let inside = Rect {
        llx: rect.llx.max(crop.llx),
        lly: rect.lly.max(crop.lly),
        urx: rect.urx.min(crop.urx),
        ury: rect.ury.min(crop.ury),
    };
    if inside.urx <= inside.llx || inside.ury <= inside.lly {
        return None;
    }
    let mut region = page.clone();
    region.crop_box = inside;
    Some(region)
}

/// The highest whole resolution at or below `asked` whose picture of `rect`
/// fits both the renderer's edge limit and [`MAX_PIXELS`]; never below 1.
#[must_use]
pub fn fitted_dpi(rect: &Rect, asked: u32) -> u32 {
    let (w, h) = (rect.urx - rect.llx, rect.ury - rect.lly);
    // One pixel short of the edge limit, so the renderer's own rounding up
    // never tips the picture over it.
    let edge = f64::from(pdfcer_render::MAX_PIXMAP_EDGE - 1);
    let by_edge = edge * 72.0 / w.max(h);
    let by_area = 72.0 * (MAX_PIXELS / (w * h)).sqrt();
    let limit = by_edge.min(by_area).floor();
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    // ui-text-exempt: clippy lint names, never displayed
    let limit = if limit.is_finite() && limit >= 1.0 {
        limit as u32
    } else {
        1
    };
    asked.min(limit).max(1)
}
#[cfg(test)]
mod tests {
    use super::*;

    fn rect(w: f64, h: f64) -> Rect {
        Rect {
            llx: 0.0,
            lly: 0.0,
            urx: w,
            ury: h,
        }
    }

    #[test]
    fn a_small_box_keeps_the_resolution_asked_for() {
        assert_eq!(fitted_dpi(&rect(200.0, 100.0), 300), 300);
    }

    #[test]
    fn a_large_box_is_lowered_to_fit_and_stays_within_both_limits() {
        let r = rect(2384.0, 1684.0);
        let dpi = fitted_dpi(&r, 2400);
        assert!(dpi < 2400);
        let px = |side: f64| (side * f64::from(dpi) / 72.0).ceil();
        assert!(px(2384.0) < f64::from(pdfcer_render::MAX_PIXMAP_EDGE));
        assert!(px(2384.0) * px(1684.0) <= MAX_PIXELS * 1.001);
    }

    #[test]
    fn a_long_thin_box_is_held_by_the_edge_limit() {
        let dpi = fitted_dpi(&rect(10_000.0, 2.0), 2400);
        assert!((10_000.0 * f64::from(dpi) / 72.0).ceil() < 16_384.0);
    }

    #[test]
    fn the_resolution_never_falls_below_one() {
        assert_eq!(fitted_dpi(&rect(1.0e9, 1.0e9), 300), 1);
    }

    /// Paths drawn in an SVG, counted by their opening tags.
    fn paths(svg: &str) -> usize {
        svg.matches("<path").count()
    }

    #[test]
    fn a_box_over_blank_paper_cuts_away_the_drawing_beside_it() {
        let file = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/blank-overhang.pdf");
        let doc = Document::load(&file).expect("the fixture loads");
        let pages = pdfcer_core::page_tree::pages(&doc).expect("the fixture has pages");
        // Blank paper: the fixture's ink lies inside x 60..500, y 400..740.
        let blank = Rect {
            llx: 700.0,
            lly: 80.0,
            urx: 900.0,
            ury: 240.0,
        };
        let region = cropped(&pages[0], blank).expect("the box is on the page");
        let options = RenderOptions::default();
        let clipped = pdfcer_render::svg::export_svg(&doc, &region, &options, &svg_options(72.0))
            .expect("the cropped page exports");
        assert!(
            paths(&clipped.svg) > 0,
            "the uncut page still carries its drawing"
        );
        let (bytes, report) = extract_region(
            &pdfcer_core::view::DocumentView::new(&doc, doc.bytes(), doc.version()),
            0,
            region.crop_box,
            &RegionExport::new(),
        )
        .expect("the region exports");
        let (payload, vectors) =
            cut_payload(bytes, &report, &options, 72.0).expect("the cut places");
        assert!(matches!(vectors, Vectors::Cut { .. }), "{vectors:?}");
        let svg = payload.svg.expect("the vectors are placed");
        assert!(
            paths(&svg) < paths(&clipped.svg),
            "the cut kept drawing from outside the box: {} paths, {} uncut",
            paths(&svg),
            paths(&clipped.svg)
        );
        let pixmap = payload.pixmap.expect("the picture is placed");
        assert_eq!((pixmap.width(), pixmap.height()), (200, 160));
    }
}
