//! Typed X and Y for several selected items: the box around them all moves,
//! every item by the same distance, as one undo step through Align's
//! [`shift`](crate::panels::align::shift).

use egui::{Pos2, Ui};
use pdfcer_core::page_tree::Page;
use pdfcer_gui_base::alignlayout::Bx;

use super::{APPLY_REGION, Bounds, GeometryDraft, POINTS, REGION, Subject, X_REGION, field};
use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::panels::align::{Measured, shift};
use crate::text::panels::properties as t;
use crate::viewer::{canvas_to_pdf_space, pdf_space_to_canvas};

/// The `MoveMixed` gesture a typed position commits under.
const GESTURE: &str = "typed"; // ui-text-exempt: a trace value, never displayed

/// Draw X and Y over `m`'s union box, returning whether anything drew.
pub(super) fn section(
    ui: &mut Ui,
    doc: &OpenDoc,
    m: &Measured,
    draft: &mut GeometryDraft,
    actions: &mut Vec<Action>,
) -> bool {
    let Some(sheet) = doc.pages.get(m.page) else {
        return false;
    };
    let Some(union) = m.boxes.iter().copied().reduce(Bx::union) else {
        return false;
    };
    let Some(bounds) = page_bounds(union, sheet) else {
        return false;
    };
    let n = m.boxes.len();
    draft.sync(
        m.page,
        Subject::Several(key(n, union)),
        doc.edit_epoch,
        bounds,
        None,
    );
    ui.label(t::geometry_heading());
    ui.label(egui::RichText::new(t::geometry_several(n)).small().weak());
    ui.label(egui::RichText::new(t::geometry_units_note()).small().weak());
    field(
        ui,
        t::geometry_x(),
        &mut draft.x,
        Some(X_REGION),
        None,
        POINTS,
    );
    field(ui, t::geometry_y(), &mut draft.y, None, None, POINTS);
    let (dx, dy) = (draft.x - bounds.x0, draft.y - bounds.y0);
    let changed = dx.abs() >= 0.1 || dy.abs() >= 0.1;
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "geometry-several n={n} x={:.2} y={:.2} bx={:.2} by={:.2} bw={:.2} bh={:.2} \
             changed={changed}",
            draft.x,
            draft.y,
            bounds.x0,
            bounds.y0,
            bounds.w(),
            bounds.h()
        )
    });
    let response = ui
        .add_enabled(changed, egui::Button::new(t::geometry_apply()))
        .on_disabled_hover_text(t::geometry_nothing_typed());
    crate::diag::ui_rect_visible(APPLY_REGION, response.rect, ui.clip_rect());
    if response.clicked()
        && let Some(canvas) = canvas_delta(sheet, bounds, (dx, dy))
        && let Some(action) = shift(doc, m, canvas, GESTURE)
    {
        actions.push(action);
    }
    ui.separator();
    crate::diag::ui_rect_visible(REGION, ui.min_rect(), ui.clip_rect());
    true
}

/// A canvas box as page-space bounds: both corners through the page's
/// transform, which may flip or turn them.
fn page_bounds(b: Bx, sheet: &Page) -> Option<Bounds> {
    #[allow(clippy::cast_possible_truncation)]
    let corner = |x: f64, y: f64| canvas_to_pdf_space(Pos2::new(x as f32, y as f32), sheet);
    let (a, c) = (corner(b.x0, b.y0)?, corner(b.x1, b.y1)?);
    Some(Bounds {
        x0: f64::from(a.x.min(c.x)),
        y0: f64::from(a.y.min(c.y)),
        x1: f64::from(a.x.max(c.x)),
        y1: f64::from(a.y.max(c.y)),
    })
}

/// A page-space displacement of the box's corner as a canvas delta.
fn canvas_delta(sheet: &Page, b: Bounds, (dx, dy): (f64, f64)) -> Option<(f64, f64)> {
    #[allow(clippy::cast_possible_truncation)]
    let at = |x: f64, y: f64| pdf_space_to_canvas(Pos2::new(x as f32, y as f32), sheet);
    let (from, to) = (at(b.x0, b.y0)?, at(b.x0 + dx, b.y0 + dy)?);
    Some((f64::from(to.x - from.x), f64::from(to.y - from.y)))
}

/// What the draft describes: the count and the union, so a different set
/// with a different box reseeds.
fn key(n: usize, b: Bx) -> u64 {
    [b.x0, b.y0, b.x1, b.y1]
        .iter()
        .fold(n as u64, |h, v| h.rotate_left(13) ^ v.to_bits())
}
