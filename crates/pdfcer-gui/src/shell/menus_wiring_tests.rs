//! Tests for `crate::shell::menus_wiring`, kept in the gui because they reach gui modules.

use crate::shell::{commands, menus};
use egui_shell::menu::plan::{IconSlot, icon_slot};
use egui_shell::{CommandRegistry, manifest::Item};

/// Every command id a menu names, per menu, in display order.
fn rows() -> Vec<(String, Vec<String>)> {
    menus::built_in()
        .iter()
        .map(|menu| {
            (
                menu.context.clone(),
                menu.items()
                    .iter()
                    .filter_map(Item::command_id)
                    .map(str::to_owned)
                    .collect(),
            )
        })
        .collect()
}

fn registry() -> CommandRegistry {
    let mut registry = CommandRegistry::new();
    commands::register(&mut registry);
    registry
}

/// **How many menu rows this wiring actually lit up.**
#[test]
fn the_icon_column_lights_up_the_rows_whose_commands_already_name_a_glyph() {
    let registry = registry();
    let mut glyph = 0usize;
    let mut blank = 0usize;
    let mut absent = 0usize;
    let mut reserving_menus = 0usize;
    let mut report = String::new();

    for (context, ids) in rows() {
        // Resolved against the registry, because an id no build
        // registers draws no row at all — the shell drops it before
        // the column is decided (`plan::resolve`, rule 1).
        let keys: Vec<bool> = ids
            .iter()
            .filter_map(|id| registry.get(id))
            .map(|command| command.icon.is_some())
            .collect();
        let reserved = keys.iter().any(|has| *has);
        if reserved {
            reserving_menus += 1;
        }
        let (mut g, mut b) = (0usize, 0usize);
        for has in &keys {
            match icon_slot(reserved, *has) {
                IconSlot::Glyph => g += 1,
                IconSlot::Blank => b += 1,
                IconSlot::Absent => absent += 1,
            }
        }
        glyph += g;
        blank += b;
        report.push_str(&format!("{context}: {g} glyph, {b} blank\n"));
    }

    //
    //
    // `blank` stays at 1 and `absent` stays at 0, and both are the
    // load-bearing halves of this tuple. A new row that had refused a
    // glyph would have moved `blank` to 2 — which is legal, argued at the
    // registration, and would have to be argued here too.
    // 35 → 36: `format.merge_text_runs` on the canvas object menu.
    // 36 → 37: `markup.flatten` on the canvas markup menu.
    // 37 → 41: the canvas dimension menu, four rows.
    // 41 → 43: Show area and Show perimeter on the dimension menu.
    assert_eq!(
        (glyph, blank, absent),
        (43, 1, 0),
        "menu rows by icon slot state; per-menu breakdown:\n{report}"
    );
    assert_eq!(
        reserving_menus, 11,
        "menus that reserve an icon column, of 11:\n{report}"
    );
}

/// **No menu is an icon column that is mostly empty.**
#[test]
fn a_reserved_icon_column_is_never_mostly_empty() {
    let registry = registry();
    for (context, ids) in rows() {
        let keys: Vec<bool> = ids
            .iter()
            .filter_map(|id| registry.get(id))
            .map(|command| command.icon.is_some())
            .collect();
        if !keys.iter().any(|has| *has) {
            continue; // no column at all; nothing to be empty.
        }
        let glyphs = keys.iter().filter(|has| **has).count();
        let blanks = keys.len() - glyphs;
        assert!(
            glyphs > blanks,
            "menu `{context}` would reserve an icon column for {glyphs} glyph(s) \
             against {blanks} blank row(s); a column that is half empty reads worse \
             than none, so either the bare rows want art or this menu wants an argument"
        );
    }
}
