//! The selected ce dimension's anchored points: what each follows, whether
//! it solved, and *Detach*.

use egui::Ui;
use pdfcer_core::dimension::{AnchorState, DimensionModel, DimensionRecord};

use crate::app::actions::Action;
use crate::app::actions::dimensions::DimensionAction;
use crate::app::state::OpenDoc;
use crate::text::panels::dimension as t;

/// The first anchored row's *Detach* button; later rows add `.<n>`.
pub const REGION_DETACH: &str = "properties.dimension.anchor.detach"; // ui-text-exempt: trace region name, never displayed

/// Draw one row per anchored point of `record`; nothing when it has none.
pub(super) fn show(
    ui: &mut Ui,
    doc: &OpenDoc,
    model: &DimensionModel,
    record: &DimensionRecord,
    actions: &mut Vec<Action>,
) {
    let anchors = doc.session.dimension_anchors(record.id);
    if anchors.is_empty() {
        return;
    }
    ui.label(t::anchors_heading());
    for (row, (point, anchor, state)) in anchors.into_iter().enumerate() {
        let target = model.dimension(anchor.target).map(|r| &r.kind);
        ui.horizontal(|ui| {
            ui.label(t::anchor_point(&record.kind, point));
            ui.label(t::anchor_target(anchor.role, target));
            let r = ui
                .button(t::anchor_detach())
                .on_hover_text(t::anchor_detach_hover());
            let region = if row == 0 {
                REGION_DETACH.to_owned()
            } else {
                format!("{REGION_DETACH}.{row}")
            };
            crate::diag::ui_rect(&region, r.rect);
            if r.clicked() {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed
                    format!("dimension-anchor-detach id={} point={point}", record.id.0)
                });
                actions.push(Action::Dimension(DimensionAction::SetAnchor {
                    dimension: record.id,
                    point,
                    anchor: None,
                }));
            }
        });
        if let AnchorState::Unsolvable(why) = state {
            ui.weak(t::anchor_unsolved(why));
        }
    }
}
