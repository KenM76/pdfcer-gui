//! The selected ce dimension's anchored points: what each follows, whether
//! it solved, *Measure to* for a point on a circle's rim, and *Detach*.

use egui::Ui;
use pdfcer_core::dimension::{
    Anchor, AnchorRole, AnchorState, ArcCondition, DimensionId, DimensionModel, DimensionRecord,
};

use crate::app::actions::Action;
use crate::app::actions::dimensions::DimensionAction;
use crate::app::state::OpenDoc;
use crate::text::panels::dimension as t;

/// The first anchored row's *Detach* button; later rows add `.<n>`.
pub const REGION_DETACH: &str = "properties.dimension.anchor.detach"; // ui-text-exempt: trace region name, never displayed
/// The first rim-anchored row's *Measure to* choice, one region per option
/// named `<this>.<role token>`; later rows insert `.<n>` before the token.
pub const REGION_ARC_CONDITION: &str = "properties.dimension.anchor.arc"; // ui-text-exempt: trace region name, never displayed

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
        // The point and its button share a row; the sentence naming the
        // target wraps under them, so a long one cannot push Detach past
        // the panel's edge.
        ui.horizontal(|ui| {
            ui.label(t::anchor_point(&record.kind, point));
            let r = ui
                .button(t::anchor_detach())
                .on_hover_text(t::anchor_detach_hover());
            crate::diag::ui_rect_visible(&region(REGION_DETACH, row), r.rect, ui.clip_rect());
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
        ui.label(t::anchor_target(anchor.role, target));
        if let AnchorRole::Rim(condition) = anchor.role {
            arc_condition(ui, (record.id, point, row), anchor, condition, actions);
        }
        if let AnchorState::Unsolvable(why) = state {
            ui.weak(t::anchor_unsolved(why));
        }
    }
}

/// The row's region name: the first row bare, later rows `.<n>`.
fn region(base: &str, row: usize) -> String {
    if row == 0 {
        base.to_owned()
    } else {
        format!("{base}.{row}")
    }
}

/// **Measure to** Centre, Near edge or Far edge, for a point following a
/// circle's rim: the same anchor with another arc condition.
fn arc_condition(
    ui: &mut Ui,
    (dimension, point, row): (DimensionId, usize, usize),
    anchor: Anchor,
    current: ArcCondition,
    actions: &mut Vec<Action>,
) {
    let mut chosen = current;
    ui.horizontal_wrapped(|ui| {
        ui.label(t::arc_condition_label());
        let base = region(REGION_ARC_CONDITION, row);
        for c in [ArcCondition::Centre, ArcCondition::Min, ArcCondition::Max] {
            let r = ui
                .radio_value(&mut chosen, c, t::arc_condition(c))
                .on_hover_text(t::arc_condition_hover());
            let name = format!("{base}.{}", AnchorRole::Rim(c).token());
            crate::diag::ui_rect_visible(&name, r.rect, ui.clip_rect());
        }
    });
    if chosen != current {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "dimension-anchor-arc id={} point={point} condition={}",
                dimension.0,
                AnchorRole::Rim(chosen).token()
            )
        });
        actions.push(Action::Dimension(DimensionAction::SetAnchor {
            dimension,
            point,
            anchor: Some(Anchor::new(anchor.target, AnchorRole::Rim(chosen))),
        }));
    }
}
