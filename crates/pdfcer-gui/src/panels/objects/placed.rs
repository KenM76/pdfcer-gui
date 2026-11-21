//! # `panels::objects::placed` — where each drawing on this page is drawn
//!
//! For every form XObject this page places at top level, one line from the
//! engine's `invocation_map`: how many places on how many pages, said as
//! "at least" when a page's scan was incomplete. The map walks every page, so
//! it is built once per document revision and kept in egui temp data.

use crate::app::state::OpenDoc;
use crate::text::panels::placed as t;
use pdfcer_core::vector::{ImageSource, VectorObject};
use std::collections::BTreeMap;
use std::sync::Arc;

/// The block of placed-drawing lines, published only when one is drawn.
pub const REGION: &str = "panel.objects.placed"; // ui-text-exempt: trace region name, never displayed

/// One form's reach: `(places, pages, lower_bound)`.
type Reach = (usize, usize, bool);

/// Every form's reach in the document, once per revision.
fn reach(ui: &egui::Ui, doc: &OpenDoc) -> Arc<BTreeMap<u32, Reach>> {
    let id = egui::Id::new(("objects-placed", doc.path.as_path(), doc.edit_epoch));
    if let Some(cached) = ui
        .ctx()
        .data(|d| d.get_temp::<Arc<BTreeMap<u32, Reach>>>(id))
    {
        return cached;
    }
    let map: BTreeMap<u32, Reach> =
        pdfcer_core::text_edit::forms::invocation_map(&doc.session.view())
            .into_values()
            .map(|set| {
                (
                    set.object,
                    (set.count(), set.pages.len(), set.is_lower_bound()),
                )
            })
            .collect();
    let map = Arc::new(map);
    ui.ctx().data_mut(|d| d.insert_temp(id, Arc::clone(&map)));
    map
}

/// The form object numbers `objects` places, in first-placed order.
fn forms_on(objects: &[VectorObject]) -> Vec<u32> {
    let mut seen = Vec::new();
    for object in objects {
        if let VectorObject::Image(image) = object
            && image.source == ImageSource::Form
            && let Some(id) = image.xobject
            && !seen.contains(&id.num)
        {
            seen.push(id.num);
        }
    }
    seen
}

/// Draws one line per form `objects` places, and traces each form's reach.
pub(super) fn lines(ui: &mut egui::Ui, doc: &OpenDoc, page: usize, objects: &[VectorObject]) {
    let forms = forms_on(objects);
    if forms.is_empty() {
        return;
    }
    let reach = reach(ui, doc);
    let said: Vec<(u32, Reach)> = forms
        .into_iter()
        .map(|form| (form, reach.get(&form).copied().unwrap_or((1, 1, true))))
        .collect();
    for &(form, (places, pages, lower)) in &said {
        crate::diag::trace(|| {
            // ui-text-exempt: trace line, never displayed
            format!(
                "objects-placed page={page} form={form} places={places} pages={pages} lower_bound={}",
                u8::from(lower)
            )
        });
    }
    let block = ui
        .vertical(|ui| {
            for (form, (places, pages, lower)) in said {
                ui.label(
                    egui::RichText::new(t::objects_dock_placed(form, places, pages, lower)).small(),
                );
            }
        })
        .response
        .on_hover_text(t::objects_dock_placed_hover())
        .rect;
    crate::diag::ui_rect_visible(REGION, block, ui.clip_rect());
}
