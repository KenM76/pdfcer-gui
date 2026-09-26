//! Tests for `crate::shell::manifest::rail`, kept in the gui because they reach gui modules.

use crate::shell::manifest::rail::*;
use egui_shell::manifest::{Item, RailFold};

/// The shipped registry. Built here rather than borrowed from
/// `shell::commands`' own test module, which is private to that module.
fn catalog() -> egui_shell::CommandRegistry {
    let mut reg = egui_shell::CommandRegistry::new();
    crate::shell::commands::register(&mut reg);
    reg
}

/// Every id in the rail is a registered command.
#[test]
fn every_rail_id_is_a_registered_command() {
    let registry = catalog();
    for group in groups() {
        for item in &group.items {
            if let Item::Command { id, .. } = item {
                assert!(
                    registry.get(id).is_some(),
                    "rail group `{}` names unregistered command `{id}`",
                    group.id
                );
            }
        }
    }
}

/// Every id in the rail names an icon.
#[test]
fn every_rail_id_names_an_icon() {
    let registry = catalog();
    for group in groups() {
        for item in &group.items {
            if let Item::Command { id, .. } = item {
                let command = registry.get(id).expect("registered");
                assert!(
                    command.icon.is_some(),
                    "rail group `{}` command `{id}` has no icon, and a rail row \
                     with neither picture nor word is a blank",
                    group.id
                );
            }
        }
    }
}

/// The panel-tab group never folds, and it is the only one that does not.
#[test]
fn only_the_panel_tabs_are_marked_never_folding() {
    let groups = groups();
    assert_eq!(groups[0].id, "tabs");
    assert_eq!(groups[0].fold, RailFold::Never);
    assert_eq!(
        groups[0].items.len(),
        6,
        "all six panels, one click away — Comments joined 2026-09-05, on his report"
    );
    for group in &groups[1..] {
        assert_ne!(
            group.fold,
            RailFold::Never,
            "group `{}` claims the floor, and there is only one floor",
            group.id
        );
    }
}

/// ⚠ **Rotate is in the rail with no mode gate**, which is what lets Read
/// dirty a document. His call — O126. Pinned so that removing the gate's
/// absence is a deliberate act rather than a drive-by.
#[test]
fn rotate_is_ungated_and_therefore_reachable_in_read() {
    let group = groups()
        .into_iter()
        .find(|g| g.id == "rotate")
        .expect("the rotate group");
    for item in &group.items {
        assert_eq!(
            item.visible_condition(),
            None,
            "rotate is available in every mode including Read — O126"
        );
    }
}

/// The smart selector is on the rail, in `navigate`, **last**.
#[test]
fn the_two_selection_toggles_are_the_last_rows_of_navigate() {
    let group = groups()
        .into_iter()
        .find(|g| g.id == "navigate")
        .expect("the navigate group");
    let ids: Vec<&str> = group
        .items
        .iter()
        .filter_map(|i| match i {
            Item::Command { id, .. } => Some(id.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(
        ids,
        [
            "view.tool_select",
            "view.tool_node",
            "view.tool_text",
            "view.tool_hand",
            "view.smart_select",
            "view.text_chunks",
        ],
        "the rail's Navigate group mirrors View ▸ Navigate row for row"
    );
    let smart = group
        .items
        .iter()
        .find(|i| matches!(i, Item::Command { id, .. } if id == "view.smart_select"))
        .expect("the smart selector");
    assert_eq!(
        smart.visible_condition(),
        Some("mode.edit_content"),
        "the command is `enabled_when(\"mode.edit_content\")`, so an ungated rail row \
         would be a permanently greyed control on a permanent surface in Read"
    );
    let chunks = group
        .items
        .iter()
        .find(|i| matches!(i, Item::Command { id, .. } if id == "view.text_chunks"))
        .expect("the chunk boxes");
    assert_eq!(
        chunks.visible_condition(),
        Some("mode.edit_content"),
        "O215's toggle carries the identical gate, for the identical reason"
    );
}

/// **At the fold the pin goes to the TOOL, never to the toggle** —
/// even when both are `selected`.
#[test]
fn at_the_floor_the_pinned_row_is_the_armed_tool_and_the_toggle_is_merely_folded() {
    use egui_shell::commands::ConditionSet;
    use egui_shell::dock::rail::{self, RailRow, Rung};

    let rail: egui_shell::manifest::Rail = groups().into_iter().collect();
    // Edit mode, the arrow armed, and the smart selector ON — the state in
    // which the two candidates collide.
    let conditions = ConditionSet::default()
        .with("mode.edit_content")
        .with(egui_shell::ribbon::selected_condition("view.tool_select"))
        .with(egui_shell::ribbon::selected_condition("view.smart_select"));

    let plan = rail::build(&rail, &conditions, Rung::Cramped);
    let pinned: Vec<&str> = plan
        .rows
        .iter()
        .filter_map(|r| match r {
            RailRow::Entry {
                id, pinned: true, ..
            } => Some(id.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(
        pinned,
        ["view.tool_select"],
        "the pinned row answers `what does a drag do?`, and a toggle cannot answer it"
    );
    assert!(
        plan.folded.iter().any(|id| id == "view.smart_select"),
        "the toggle folds behind the chevron rather than vanishing; folded = {:?}",
        plan.folded
    );
}

/// The Points tool is withheld outside Edit, on the rail as on the ribbon.
#[test]
fn the_points_tool_is_withheld_outside_edit() {
    let group = groups()
        .into_iter()
        .find(|g| g.id == "navigate")
        .expect("the navigate group");
    let node = group
        .items
        .iter()
        .find(|i| matches!(i, Item::Command { id, .. } if id == "view.tool_node"))
        .expect("the points tool");
    assert_eq!(node.visible_condition(), Some("mode.edit_content"));
}

/// The lasso is absent, and this test is the tripwire for the day it is
/// added: R9 forbids drawing a capability the build does not have, so a
/// `edit.lasso` id appearing here before the command is registered would
/// otherwise be caught only by `Shell::validate` at start-up.
#[test]
fn the_lasso_is_absent_until_the_command_exists() {
    let registry = catalog();
    let in_rail = groups().iter().any(|g| {
        g.items
            .iter()
            .any(|i| matches!(i, Item::Command { id, .. } if id == "edit.lasso"))
    });
    assert_eq!(
        in_rail,
        registry.get("edit.lasso").is_some(),
        "the rail draws the lasso exactly when the lasso exists"
    );
}
