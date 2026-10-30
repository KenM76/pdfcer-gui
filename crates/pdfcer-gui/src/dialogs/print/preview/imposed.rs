//! The preview of an imposed sheet: the printable area, composed exactly as
//! the commit composes it but at preview density, cached on the dialog's
//! imposition state and redrawn only when the sheet or its rendering inputs
//! change. It has no drag target and no detail pass: every page's position is
//! the layout's.

use egui::{Color32, Rect, TextureId, Ui};
use pdfcer_gui_base::imposition::{self as base, Cell, ImposedSheet};
use pdfcer_render::tiny_skia::Pixmap;

use super::Inputs;
use crate::dialogs::print::PrintDialog;

/// Draw `sheet` over `printable`. `None` when nothing could be rendered; the
/// printable frame already drawn is then the picture.
pub(super) fn paint(
    ui: &Ui,
    inputs: &Inputs<'_>,
    dialog: &mut PrintDialog,
    sheet: &ImposedSheet,
    printable: Rect,
) -> Option<TextureId> {
    let printable_pt = inputs.job.device.printable_pt;
    let key = (
        sheet.clone(),
        inputs.context.preview_key(usize::MAX, 1.0),
        dialog.output_color,
    );
    let cached = dialog
        .impose
        .preview
        .as_ref()
        .filter(|(k, _)| *k == key)
        .map(|(_, texture)| texture.id());
    let id = match cached {
        Some(id) => Some(id),
        None => {
            let composed = compose(inputs, dialog, sheet, printable_pt)?;
            let texture = super::upload(ui.ctx(), &composed);
            let id = texture.id();
            dialog.impose.preview = Some((key, texture));
            Some(id)
        }
    };
    if let Some(id) = id {
        ui.painter_at(printable).image(
            id,
            printable,
            Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            // NOT A THEME COLOUR: the pass-through tint for a rendered sheet,
            // as for a single page.
            Color32::WHITE,
        );
    }
    id
}

/// Render the sheet's pages at preview density and compose them.
fn compose(
    inputs: &Inputs<'_>,
    dialog: &PrintDialog,
    sheet: &ImposedSheet,
    printable_pt: (f64, f64),
) -> Option<Pixmap> {
    let px_per_pt = f64::from(super::raster_scale(printable_pt));
    let options = crate::dialogs::print::commit::render_options(dialog.scope, &inputs.doc.settings);
    let view = inputs.doc.session.view();
    let mut pages: Vec<(Pixmap, Cell)> = Vec::with_capacity(sheet.cells.len());
    for cell in &sheet.cells {
        let Some(page) = inputs.doc.pages.get(cell.page) else {
            continue;
        };
        let scale = base::render_scale(&cell.fit, px_per_pt);
        let mut options = options.clone();
        inputs
            .context
            .preview_key(cell.page, cell.fit.scale)
            .apply_lines(&mut options, scale);
        if let Ok(rendered) =
            pdfcer_render::render_page_with_view(&view, page, scale as f32, &options)
        {
            pages.push((rendered.pixmap, *cell));
        }
    }
    let borrowed: Vec<(&Pixmap, Cell)> = pages.iter().map(|(p, c)| (p, *c)).collect();
    let mut composed = base::compose(printable_pt, px_per_pt, &borrowed)?;
    super::super::colour::recolour(composed.data_mut(), dialog.output_color, "preview"); // ui-text-exempt: trace token
    Some(composed)
}
