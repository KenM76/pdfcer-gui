//! # `footer` — a panel's controls pinned under its list
//!
//! A list panel that draws anything conditional *above* its rows moves every
//! row when that thing appears, and the commonest trigger is the click that
//! selected a row: the operator's target slides out from under the pointer.
//! A footer grows upward instead — the list's viewport shrinks from the
//! bottom, the scroll offset holds, and no row moves.
//!
//! Two calls, list first:
//!
//! 1. [`list_height`] — the height the list's `ScrollArea` gets: what the
//!    panel has, less the footer's height as measured last frame.
//! 2. [`show`] — the footer, drawn after the list and measured for next frame.
//!
//! The one-frame lag when the footer changes height is closed by a repaint,
//! so it is never visible. The footer caps its collapsible body at half the
//! panel and scrolls it beyond that, so the list is never squeezed to nothing.

/// The height the list above the footer may take, in points.
#[must_use]
pub fn list_height(ui: &egui::Ui, id: &str) -> f32 {
    let footer = ui
        .ctx()
        .data(|d| d.get_temp::<f32>(memory_id(id)))
        .unwrap_or_else(|| ui.spacing().interact_size.y * 2.0);
    (ui.available_height() - footer).max(ui.spacing().interact_size.y * 3.0)
}

/// Draw the footer: `always` unconditionally, then a collapsed-by-default
/// header titled `title` whose body is `tools`, or no header when `tools` is
/// `None`. The header publishes its rect as region `id`, so a driven check can
/// open it.
pub fn show(
    ui: &mut egui::Ui,
    id: &str,
    panel_height: f32,
    title: &str,
    always: impl FnOnce(&mut egui::Ui),
    tools: Option<impl FnOnce(&mut egui::Ui)>,
) {
    let top = ui.cursor().top();
    ui.separator();
    always(ui);
    // What the open body wanted beyond what it was given. The list sizes itself
    // from the stored height, so storing only what fitted lets the list keep the
    // space and squeezes the open body to a sliver, frame after frame.
    let mut shortfall = 0.0_f32;
    if let Some(tools) = tools {
        let header = egui::CollapsingHeader::new(title)
            .id_salt(id)
            .default_open(false)
            .show(ui, |ui| {
                let body = egui::ScrollArea::vertical()
                    .id_salt((id, "body"))
                    .max_height(panel_height * 0.5)
                    .show(ui, tools);
                body.content_size.y.min(panel_height * 0.5) - body.inner_rect.height()
            });
        shortfall = header.body_returned.unwrap_or(0.0).max(0.0);
        crate::diag::ui_rect(id, header.header_response.rect);
        let open = header.openness > 0.99;
        let drawn = ui.cursor().top() - top;
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!(
                "panel-footer id={id} open={open} height={drawn:.1} wants={:.1}",
                drawn + shortfall
            )
        });
    }
    let height = ui.cursor().top() - top + shortfall;
    let key = memory_id(id);
    let before = ui.ctx().data(|d| d.get_temp::<f32>(key));
    if before.is_none_or(|b| (b - height).abs() > 0.5) {
        ui.ctx().data_mut(|d| d.insert_temp(key, height));
        ui.ctx().request_repaint();
    }
}

fn memory_id(id: &str) -> egui::Id {
    egui::Id::new(("panel-footer", id))
}
