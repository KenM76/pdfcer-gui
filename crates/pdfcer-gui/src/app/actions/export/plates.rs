//! One page of an image export as one EMF per separation plate.

use std::path::{Path, PathBuf};

use crate::app::actions::imageexport::{self, ImagePlan};
use crate::text::export_plates as t;

/// The files one page wrote and the sentences its receipt owes.
pub(super) struct Separated {
    pub files: Vec<PathBuf>,
    pub notes: Vec<String>,
}

/// Where a page's plates go: the name the operator chose and the page's place
/// in the run.
pub(super) struct Target<'a> {
    pub chosen: &'a Path,
    pub page_index: usize,
    pub multi: bool,
}

/// Write `page`'s plates beside `target`; `Err` carries the sentence that
/// stopped the run.
pub(super) fn page(
    view: &pdfcer_render::DocumentView<'_>,
    page: &pdfcer_core::page_tree::Page,
    options: &pdfcer_render::RenderOptions,
    plan: &ImagePlan,
    target: &Target<'_>,
) -> Result<Separated, String> {
    let number = target.page_index.saturating_add(1);
    let emf = pdfcer_emf::EmfOptions::default()
        .with_raster_dpi(plan.dpi)
        .with_background(plan.flatten_colour());
    let plates = pdfcer_emf::export_emf_plates_view(view, page, options, &emf)
        .map_err(|e| crate::text::export_image::render_failed(number, &e.to_string()))?;
    let mut done = Separated {
        files: Vec::new(),
        notes: Vec::new(),
    };
    match plates {
        pdfcer_emf::EmfPlates::Separated(plates) => {
            for plate in &plates {
                let path = imageexport::plate_path(
                    target.chosen,
                    target.page_index,
                    target.multi,
                    &plate.ink,
                );
                write(&path, &plate.emf)?;
                let p = &plate.outcome.patches;
                trace_plate(target.page_index, &plate.ink, &path, plate.outcome.ops, p);
                done.notes
                    .push(t::plate(&plate.ink, plate.outcome.ops, p.rasters));
                done.files.push(path);
            }
            let first = done.files.first().map(|p| p.display().to_string());
            done.notes.insert(
                0,
                t::wrote(number, plates.len(), &first.unwrap_or_default()),
            );
        }
        pdfcer_emf::EmfPlates::Composite(export) => {
            let path = imageexport::output_path(
                target.chosen,
                imageexport::ImageFormat::Emf,
                target.page_index,
                target.multi,
            );
            write(&path, &export.emf)?;
            crate::diag::trace(|| {
                format!("export-image-plates page={} composite=1", target.page_index)
            }); // ui-text-exempt: diagnostic trace
            let counts = imageexport::EmfCounts::from(&export.outcome);
            done.notes.push(crate::text::export_image::wrote_emf(
                &path.display().to_string(),
                number,
                counts.ops,
                counts.rasters_embedded,
            ));
            done.notes.push(t::composite(number));
            done.files.push(path);
        }
        _ => return Err(t::unsupported(number)),
    }
    Ok(done)
}

fn write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    std::fs::write(path, bytes).map_err(|e| crate::text::export_image::write_failed(&e.to_string()))
}

fn trace_plate(
    page_index: usize,
    ink: &str,
    path: &Path,
    ops: usize,
    p: &pdfcer_emf::PlatePatches,
) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        format!(
            "export-image-plate page={page_index} ink={} file={} ops={ops} rasters={} \
             images={} shadings={} soft_masks={} transparency={}",
            ink.replace(' ', "_"),
            path.display().to_string().replace(' ', "_"),
            p.rasters,
            p.images,
            p.shadings,
            p.soft_masks,
            p.transparency,
        )
    });
}
