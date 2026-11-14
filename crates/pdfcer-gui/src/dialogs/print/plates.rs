//! # `dialogs::print::plates` — a plain page as its separation plates
//!
//! Under Colour ▸ *Separations* each plain page (not n-up, booklet or poster)
//! is rendered by `pdfcer_render::plates::render_plates` as one opaque grey
//! sheet per ink, the ink's name drawn at the sheet's foot by the engine, and
//! each plate is spooled as its own sheet with the page's placement. A page
//! the engine cannot separate (its colorant buffer refused for memory) comes
//! back as its composite and prints in colour; the receipt counts it.
//!
//! Plates always go as images: a plate is a raster the engine composed. The
//! caller skips the printer profile and recolouring for plate sheets.

use super::spooler::{PageBitmap, PagePlan, PlateTally, VectorTally};

/// `page`'s sheets under separations, each with whether it is a plate,
/// adding to `tally`.
pub(super) fn sheets(
    view: &pdfcer_render::DocumentView<'_>,
    page: &pdfcer_core::page_tree::Page,
    plan: &PagePlan,
    size: (f64, f64),
    options: &pdfcer_render::RenderOptions,
    tally: &mut PlateTally,
) -> Result<Vec<(PageBitmap, bool)>, String> {
    let sheet = |pixmap: &pdfcer_render::tiny_skia::Pixmap| PageBitmap {
        width: pixmap.width(),
        height: pixmap.height(),
        rgba: pixmap.data().to_vec(),
        placement: plan.placement,
        page_pt: size,
        metafile: None,
        image_reason: None,
        tally: VectorTally::default(),
    };
    let plates =
        pdfcer_render::plates::render_plates(view, page, plan.render_scale as f32, options)
            .map_err(|e| e.to_string())?;
    let out: Vec<(PageBitmap, bool)> = match &plates {
        pdfcer_render::plates::Plates::Separated(plates) => {
            tally.pages += 1;
            tally.plates += plates.len();
            for plate in plates {
                if !tally.inks.contains(&plate.ink) {
                    tally.inks.push(plate.ink.clone());
                }
            }
            plates.iter().map(|p| (sheet(&p.pixmap), true)).collect()
        }
        pdfcer_render::plates::Plates::Composite(pixmap) => {
            tally.refused += 1;
            vec![(sheet(pixmap), false)]
        }
        // A kind of answer this shell does not know yet: the page in colour.
        _ => {
            tally.refused += 1;
            let rendered =
                pdfcer_render::render_page_with_view(view, page, plan.render_scale as f32, options)
                    .map_err(|e| e.to_string())?;
            vec![(sheet(&rendered.pixmap), false)]
        }
    };
    crate::diag::trace(|| {
        let inks: Vec<&str> = match &plates {
            pdfcer_render::plates::Plates::Separated(p) => {
                p.iter().map(|p| p.ink.as_str()).collect()
            }
            _ => Vec::new(),
        };
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "print-plates page={} plates={} inks={} refused={}",
            plan.index,
            inks.len(),
            inks.join("|"),
            !matches!(plates, pdfcer_render::plates::Plates::Separated(_)),
        )
    });
    Ok(out)
}
