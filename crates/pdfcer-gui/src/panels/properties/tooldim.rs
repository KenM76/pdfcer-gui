//! # `panels::properties::tooldim` — the dimension tools' group and direction
//!
//! Which group the next ce dimension joins, a new group made from here, and
//! the linear tool's Aligned / Horizontal / Vertical choice.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/properties/tooldim.md`.

use egui::Ui;
use pdfcer_core::dimension::GroupId;
use pdfcer_core::vector::AxisConstraint;

use crate::app::actions::Action;
use crate::app::actions::dimensions::DimensionAction;
use crate::app::state::OpenDoc;
use crate::canvas::measure::MeasureKind;
use crate::text::dimension_groups as gt;
use crate::text::tool as t;

/// The group combo.
pub const REGION_GROUP_COMBO: &str = "properties.tool.dim.group.combo"; // ui-text-exempt: trace region name, never displayed
/// One entry in the open group combo, suffixed with the group id or `new`.
pub const REGION_GROUP_OPTION_PREFIX: &str = "properties.tool.dim.group.option."; // ui-text-exempt: trace region name, never displayed
/// The new group's name field.
pub const REGION_NEW_NAME: &str = "properties.tool.dim.new_name"; // ui-text-exempt: trace region name, never displayed
/// The button that creates the typed group.
pub const REGION_CREATE: &str = "properties.tool.dim.create"; // ui-text-exempt: trace region name, never displayed
/// One direction choice, suffixed with `aligned`, `horizontal` or `vertical`.
pub const REGION_DIRECTION_PREFIX: &str = "properties.tool.dim.direction."; // ui-text-exempt: trace region name, never displayed

/// Where the new-group draft lives; `Some` while its row is open.
const DRAFT_KEY: &str = "pdfcer-tooldim-new-group"; // ui-text-exempt: an `egui::Id` source string, never displayed

/// The three directions, in the order they are offered.
const DIRECTIONS: [(AxisConstraint, &str); 3] = [
    (AxisConstraint::Aligned, "aligned"), // ui-text-exempt: trace token
    (AxisConstraint::Horizontal, "horizontal"), // ui-text-exempt: trace token
    (AxisConstraint::Vertical, "vertical"), // ui-text-exempt: trace token
];

/// Draw the group row for any dimension tool, and the direction row for the
/// linear one.
pub(super) fn section(ui: &mut Ui, doc: &OpenDoc, kind: MeasureKind, actions: &mut Vec<Action>) {
    let ctx = ui.ctx().clone();
    ui.label(t::measure_heading());
    group_row(ui, &ctx, doc, actions);
    if kind == MeasureKind::Linear {
        direction_row(ui, &ctx);
    }
}

/// The group the next ce dimension joins, and the way to make a new one.
fn group_row(ui: &mut Ui, ctx: &egui::Context, doc: &OpenDoc, actions: &mut Vec<Action>) {
    let model = doc.session.dimension_model();
    let default = pdfcer_core::dimension::DEFAULT_GROUP_ID;
    let mut active = crate::canvas::measure::active_group(ctx).unwrap_or(default);
    // A group deleted from under the tool falls back to the one that always
    // exists, so the combo never names a group the commit cannot reach.
    if model.group(active).is_none() {
        active = default;
        crate::canvas::measure::set_active_group(ctx, active);
    }
    let draft_id = egui::Id::new(DRAFT_KEY);
    let mut draft: Option<String> = ctx.data_mut(|d| d.get_temp(draft_id)).flatten();
    let mut chosen: Option<GroupId> = None;

    ui.horizontal_wrapped(|ui| {
        ui.label(gt::tool_group_label());
        let current = model.group(active).map_or("", |g| g.name.as_str());
        let combo = egui::ComboBox::from_id_salt("properties-tool-dim-group")
            .selected_text(current)
            .show_ui(ui, |ui| {
                for group in model.groups() {
                    let r = ui.selectable_label(group.id == active, &group.name);
                    crate::diag::ui_rect_visible(
                        // ui-text-exempt: trace region name, never displayed
                        &format!("{REGION_GROUP_OPTION_PREFIX}{}", group.id.0),
                        r.rect,
                        ui.clip_rect(),
                    );
                    if r.clicked() {
                        chosen = Some(group.id);
                    }
                }
                let r = ui.selectable_label(draft.is_some(), gt::tool_new_group());
                // ui-text-exempt: trace region name, never displayed
                crate::diag::ui_rect_visible(
                    &format!("{REGION_GROUP_OPTION_PREFIX}new"),
                    r.rect,
                    ui.clip_rect(),
                );
                if r.clicked() {
                    draft = Some(String::new());
                }
            });
        crate::diag::ui_rect_visible(REGION_GROUP_COMBO, combo.response.rect, ui.clip_rect());
    });
    if let Some(group) = chosen.filter(|g| *g != active) {
        crate::canvas::measure::set_active_group(ctx, group);
        draft = None;
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!("dimension-authoring-group id={} via=tool", group.0)
        });
    }
    if let Some(name) = draft.as_mut()
        && new_group_row(ui, name, model.group(active), actions)
    {
        draft = None;
    }
    ctx.data_mut(|d| d.insert_temp(draft_id, draft));
}

/// The new group's name and Create button; `true` once the group was asked for.
fn new_group_row(
    ui: &mut Ui,
    name: &mut String,
    from: Option<&pdfcer_core::dimension::Group>,
    actions: &mut Vec<Action>,
) -> bool {
    let mut created = false;
    ui.horizontal_wrapped(|ui| {
        ui.label(gt::new_name_label());
        // escape-disposition: keeps-draft — the name lives in this row until
        // Create or another group is chosen.
        let field = ui.add(egui::TextEdit::singleline(name).desired_width(120.0));
        crate::diag::ui_rect_visible(REGION_NEW_NAME, field.rect, ui.clip_rect());
        let trimmed = name.trim().to_owned();
        let button = ui.add_enabled(
            !trimmed.is_empty(),
            egui::Button::new(gt::tool_create_group()),
        );
        crate::diag::ui_rect_visible(REGION_CREATE, button.rect, ui.clip_rect());
        let button = button.on_disabled_hover_text(gt::new_needs_a_name());
        let enter = field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
        if !trimmed.is_empty() && (button.clicked() || enter) {
            let unit = from.map_or(pdfcer_core::dimension::Unit::Millimeter, |g| g.format.unit);
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    "dimension-group-add via=tool unit={unit:?} chars={} scale_from={}",
                    trimmed.len(),
                    from.map_or(-1, |g| i64::from(g.id.0))
                )
            });
            actions.push(Action::Dimension(DimensionAction::AddGroup {
                name: trimmed,
                unit,
                scale_from: from.map(|g| g.id),
                author_into: true,
            }));
            created = true;
        }
    });
    ui.weak(gt::tool_new_group_hint());
    created
}

/// The linear tool's direction: Aligned, Horizontal or Vertical.
fn direction_row(ui: &mut Ui, ctx: &egui::Context) {
    let current = crate::canvas::measure::linear_constraint(ctx);
    let mut picked = current;
    ui.horizontal_wrapped(|ui| {
        ui.label(t::measure_direction_label());
        for (constraint, token) in DIRECTIONS {
            let r = ui.radio_value(
                &mut picked,
                constraint,
                t::measure_direction_name(constraint),
            );
            crate::diag::ui_rect_visible(
                &format!("{REGION_DIRECTION_PREFIX}{token}"),
                r.rect,
                ui.clip_rect(),
            );
        }
    });
    ui.weak(t::measure_direction_hint());
    if picked != current {
        crate::canvas::measure::set_linear_constraint(ctx, picked);
        let token = DIRECTIONS
            .iter()
            .find(|(c, _)| *c == picked)
            .map_or("?", |(_, t)| t);
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!("measure-direction constraint={token}")
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every region sits under the Tool options prefix, so a sweep of
    /// `properties.tool` finds them.
    #[test]
    fn every_region_is_under_the_tool_section() {
        for name in [
            REGION_GROUP_COMBO,
            REGION_GROUP_OPTION_PREFIX,
            REGION_NEW_NAME,
            REGION_CREATE,
            REGION_DIRECTION_PREFIX,
        ] {
            assert!(name.starts_with(super::super::tool::REGION), "{name}");
        }
    }

    /// Each of the engine's three directions is offered once.
    #[test]
    fn every_direction_is_offered_once() {
        let mut seen: Vec<AxisConstraint> = DIRECTIONS.iter().map(|(c, _)| *c).collect();
        seen.dedup();
        assert_eq!(seen.len(), 3);
    }
}
