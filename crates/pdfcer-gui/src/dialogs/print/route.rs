//! **How each sheet travels to the printer: vector graphics or an image.**
//!
//! A plain page goes as the Enhanced Metafile `pdfcer_render::emf::print_metafile`
//! makes of it, unless the operator ticked Print as image or the engine
//! declined the page; a declined page goes as pixels and its reason is
//! reported after the job. Sheets whose pixels pdfcer itself changes or
//! composes (grey and black-only colour, n-up and booklet sheets, poster
//! tiles) are always images, and the dialog says so under the switch.

use pdfcer_gui_base::prefs::print_impose::ImposeMode;
use pdfcer_print::output_color::OutputColor;

use super::PrintDialog;
use crate::dialogs::print::spooler::{PageBitmap, PagePlan};
use crate::text::print::route as t;

impl PrintDialog {
    /// Whether a plain page of this job is offered the vector route.
    pub(super) fn vector_offered(&self) -> bool {
        !self.print_as_image && self.output_color == OutputColor::Color
    }

    /// The sentence saying which sheets of this job are images whatever the
    /// switch says, or `None` when the switch governs them all.
    fn forced_images(&self) -> Option<&'static str> {
        if self.output_color != OutputColor::Color {
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

/// The Print as image switch and, under it, the forced-image sentence.
pub(super) fn controls(ui: &mut egui::Ui, dialog: &mut PrintDialog) {
    let switch = ui
        .checkbox(&mut dialog.print_as_image, t::as_image())
        .on_hover_text(t::as_image_hover());
    crate::diag::ui_rect_visible(
        pdfcer_gui_base::printregions::REGION_PRINT_AS_IMAGE,
        switch.rect,
        ui.clip_rect(),
    );
    if let Some(said) = dialog.forced_images() {
        ui.label(
            egui::RichText::new(said)
                .small()
                .color(egui_shell::theme::Theme::of(ui.ctx()).palette.text_muted),
        );
    }
}

/// One plain page as a sheet: its metafile when `vector` is offered and the
/// engine accepts the page, otherwise its pixels at the plan's scale.
pub(super) fn plain_sheet(
    view: &pdfcer_render::DocumentView<'_>,
    page: &pdfcer_core::page_tree::Page,
    plan: &PagePlan,
    size: (f64, f64),
    options: &pdfcer_render::RenderOptions,
    vector: bool,
) -> Result<PageBitmap, String> {
    // The metafile is drawn at the raster's own pixel scale, so a fixed line
    // width set in pixels by `lines::apply` means the same on both routes.
    let raster_dpi = plan.render_scale as f32 * 72.0;
    let image_reason = if vector {
        match pdfcer_render::emf::print_metafile(view, page, options, raster_dpi) {
            Ok(emf) => {
                return Ok(PageBitmap {
                    width: 0,
                    height: 0,
                    rgba: Vec::new(),
                    placement: plan.placement,
                    page_pt: size,
                    metafile: Some(emf),
                    image_reason: None,
                });
            }
            Err(pdfcer_render::emf::PrintMetafileError::Raster(why)) => {
                Some(why.as_str().to_owned())
            }
            Err(error) => Some(error.to_string()),
        }
    } else {
        None
    };
    let rendered =
        pdfcer_render::render_page_with_view(view, page, plan.render_scale as f32, options)
            .map_err(|e| e.to_string())?;
    Ok(PageBitmap {
        width: rendered.pixmap.width(),
        height: rendered.pixmap.height(),
        // Premultiplied RGBA8, handed over unchanged: the engine's contract.
        rgba: rendered.pixmap.data().to_vec(),
        placement: plan.placement,
        page_pt: size,
        metafile: None,
        image_reason,
    })
}
