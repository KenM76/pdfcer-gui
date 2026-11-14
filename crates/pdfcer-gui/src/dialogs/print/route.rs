//! **How each sheet travels to the printer: vector graphics or an image.**
//!
//! A plain page goes as the Enhanced Metafile `pdfcer_emf::print_export_with`
//! makes of it, unless the operator ticked Print as image or the engine
//! declined the page; a declined page goes as pixels and its reason is
//! reported after the job. [`VectorRoute`] decides whether gradients and
//! transparency may stay vector; with them off, a page holding either is
//! declined. Sheets whose pixels pdfcer itself changes or composes (grey and
//! black-only colour, n-up and booklet sheets, poster tiles) are always
//! images, and the dialog says so under the switch.

use pdfcer_gui_base::prefs::print_impose::ImposeMode;
use pdfcer_gui_base::prefs::print_route::VectorRoute;
use pdfcer_gui_base::printregions as regions;
use pdfcer_print::output_color::OutputColor;

use super::PrintDialog;
use crate::dialogs::print::spooler::{PageBitmap, PagePlan, VectorTally};
use crate::text::print::route as t;

impl PrintDialog {
    /// The vector route a plain page of this job is offered, if any.
    pub(super) fn vector_offered(&self) -> Option<VectorRoute> {
        (!self.print_as_image && self.output_color == OutputColor::Color && !self.icc.managed())
            .then_some(self.vector_route)
    }

    /// The sentence saying which sheets of this job are images whatever the
    /// switch says, or `None` when the switch governs them all.
    fn forced_images(&self) -> Option<&'static str> {
        if self.icc.managed() {
            Some(crate::text::print::icc::forced_images())
        } else if self.output_color != OutputColor::Color {
            Some(t::forced_colour())
        } else if self.impose.prefs.mode != ImposeMode::Off {
            Some(t::forced_impose())
        } else if self.poster.on {
            Some(t::forced_poster())
        } else {
            None
        }
    }
}

/// The Print as image switch, the vector route's own switches under it while
/// it is unticked, and the forced-image sentence.
pub(super) fn controls(ui: &mut egui::Ui, dialog: &mut PrintDialog) {
    let switch = ui
        .checkbox(&mut dialog.print_as_image, t::as_image())
        .on_hover_text(t::as_image_hover());
    crate::diag::ui_rect_visible(regions::REGION_PRINT_AS_IMAGE, switch.rect, ui.clip_rect());
    if !dialog.print_as_image {
        ui.indent("print-vector-route", |ui| {
            vector_switches(ui, &mut dialog.vector_route);
        });
    }
    if let Some(said) = dialog.forced_images() {
        ui.label(
            egui::RichText::new(said)
                .small()
                .color(egui_shell::theme::Theme::of(ui.ctx()).palette.text_muted),
        );
    }
}

/// The gradient and transparency switches, and the patch-size switch while
/// transparency is on.
fn vector_switches(ui: &mut egui::Ui, route: &mut VectorRoute) {
    let rows: [(&mut bool, &str, &str, &str); 2] = [
        (
            &mut route.gradients,
            t::vector_gradients(),
            t::vector_gradients_hover(),
            regions::REGION_PRINT_VECTOR_GRADIENTS,
        ),
        (
            &mut route.transparency,
            t::vector_transparency(),
            t::vector_transparency_hover(),
            regions::REGION_PRINT_VECTOR_TRANSPARENCY,
        ),
    ];
    for (on, label, hover, region) in rows {
        let row = ui.checkbox(on, label).on_hover_text(hover);
        crate::diag::ui_rect_visible(region, row.rect, ui.clip_rect());
    }
    if route.transparency {
        ui.indent("print-tight-patches", |ui| {
            let row = ui
                .checkbox(&mut route.tight_patches, t::tight_patches())
                .on_hover_text(t::tight_patches_hover());
            crate::diag::ui_rect_visible(
                regions::REGION_PRINT_TIGHT_PATCHES,
                row.rect,
                ui.clip_rect(),
            );
        });
    }
}

/// The engine's metafile options for `route` at `raster_dpi`.
fn emf_options(route: VectorRoute, raster_dpi: f32) -> pdfcer_emf::EmfOptions {
    pdfcer_emf::EmfOptions::default()
        .with_raster_dpi(raster_dpi)
        .with_vector_shadings(route.gradients)
        .with_vector_transparency(route.transparency)
        .with_tight_mask_patches(route.transparency && route.tight_patches)
}

/// What the engine's outcome says the vector route did on one page.
fn tally(outcome: &pdfcer_emf::EmfOutcome) -> VectorTally {
    VectorTally {
        gradients: outcome.shadings.vector,
        transparent: outcome.vectorised.ops,
        patches: outcome.patches.cases(),
        profiles_unused: usize::from(outcome.diagnostics.icc_profiles_unmodelled > 0),
    }
}

/// One plain page as a sheet: its metafile when a vector route is offered
/// and the engine accepts the page, otherwise its pixels at the plan's scale.
pub(super) fn plain_sheet(
    view: &pdfcer_render::DocumentView<'_>,
    page: &pdfcer_core::page_tree::Page,
    plan: &PagePlan,
    size: (f64, f64),
    options: &pdfcer_render::RenderOptions,
    vector: Option<VectorRoute>,
) -> Result<PageBitmap, String> {
    // The metafile is drawn at the raster's own pixel scale, so a fixed line
    // width set in pixels by `lines::apply` means the same on both routes.
    let raster_dpi = plan.render_scale as f32 * 72.0;
    let image_reason = match vector {
        Some(route) => {
            let emf = emf_options(route, raster_dpi);
            match pdfcer_emf::print_export_with(view, page, options, &emf) {
                Ok(export) => {
                    let tally = tally(&export.outcome);
                    trace_page(plan.index, route, tally);
                    return Ok(PageBitmap {
                        width: 0,
                        height: 0,
                        rgba: Vec::new(),
                        placement: plan.placement,
                        page_pt: size,
                        metafile: Some(export.emf),
                        image_reason: None,
                        tally,
                    });
                }
                Err(pdfcer_emf::PrintMetafileError::Raster(why)) => Some(why.as_str().to_owned()),
                Err(error) => Some(error.to_string()),
            }
        }
        None => None,
    };
    let rendered =
        pdfcer_render::render_page_with_view(view, page, plan.render_scale as f32, options)
            .map_err(|e| e.to_string())?;
    super::commit::trace_ink(&rendered.diagnostics);
    Ok(PageBitmap {
        width: rendered.pixmap.width(),
        height: rendered.pixmap.height(),
        // Premultiplied RGBA8, handed over unchanged: the engine's contract.
        rgba: rendered.pixmap.data().to_vec(),
        placement: plan.placement,
        page_pt: size,
        metafile: None,
        image_reason,
        tally: VectorTally {
            profiles_unused: usize::from(rendered.diagnostics.icc_profiles_unmodelled > 0),
            ..VectorTally::default()
        },
    })
}

/// The `print-vector` line: one vector page's switches and what they kept.
fn trace_page(index: usize, route: VectorRoute, tally: VectorTally) {
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "print-vector page={index} gradients_on={} transparency_on={} tight={} \
             gradients={} transparent={} patches={} profiles_unused={}",
            route.gradients,
            route.transparency,
            route.tight_patches,
            tally.gradients,
            tally.transparent,
            tally.patches,
            tally.profiles_unused,
        )
    });
}
