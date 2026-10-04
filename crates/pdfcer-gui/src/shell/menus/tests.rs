//! # `shell::menus::tests` — the sweeps that keep the menu document honest
//!
//!
//! ## The seam, and why it is a subject rather than a cut
//!
//! [`super`] is a **document**: one function returning the menus pdfcer
//! defines, plus the prose arguing every row of every one of them. It changes
//! when a menu changes.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/shell/menus/tests.md`.

// The marker `tools/gates/check-ui-strings.sh` reads, and it is the FILE that
// has to carry it rather than the `mod tests;` in the parent: that scanner is
// awk over one file at a time, so it cannot see the `#[cfg(test)]` next door,
// and without this line all 26 assertion messages below are reported as
// operator-facing copy. `canvas::selection::tests` carries the same line for
// the same reason, and the gate's own header explains why the marker is the
// attribute rather than the filename — the property that earns the exemption is
// *not in the shipped binary*, and a filename is a restatement of that which
// goes stale the moment a third such module is written.
#![cfg(test)]

use super::*;
use crate::shell::{commands, manifest};
use egui_shell::manifest::Item;
use std::collections::BTreeSet;

/// The shipped shell and a fully populated registry, built the way the
/// application builds them.
fn shell_and_registry() -> (Shell, CommandRegistry) {
    let mut registry = CommandRegistry::new();
    commands::register(&mut registry);
    (manifest::built_in(), registry)
}

/// Conditions for a document that is open, has pages, and has something
/// selected — the state in which every menu here is at its liveliest.
fn everything_open() -> ConditionSet {
    ConditionSet::new()
        .with("doc.open")
        .with("doc.pages")
        .with(manifest::SELECTION_ANY)
        // 2026-08-28. Without it `canvas.object` stopped opening, and
        // the failure was correct: `format.delete` and `format.properties`
        // moved to the wider `selection.actionable` when a form field
        // became something they can act on, and this fixture's name
        // promises *"everything open"* while naming conditions one at a
        // time.
        //
        // ⇒ A hand-listed "liveliest state" fixture goes stale the moment a
        // command's predicate changes, and it fails on the menu that lost
        // its last enabled item rather than on the condition that moved —
        // which is a true failure pointing at the wrong file. Adding the
        // name here is the whole repair; the alternative, deriving the set
        // from the registry, would make the test assert that the registry
        // agrees with itself.
        .with(manifest::SELECTION_ACTIONABLE)
}

/// **Every command every menu names is registered.**
#[test]
fn every_command_every_menu_names_is_registered() {
    let (shell, registry) = shell_and_registry();
    let menus = shell
        .menus
        .as_ref()
        .expect("the built-in shell must carry its menus");
    menus.validate_against(&registry).expect(
        "every command a context menu names must be registered — an unregistered id \
         is silently dropped at render time, so nothing else would report this",
    );
}

/// …and the manifest really carries them, rather than the menus existing
/// only as a function nothing calls.
#[test]
fn the_shipped_shell_carries_the_menu_document() {
    let shell = manifest::built_in();
    let menus = shell
        .menus
        .as_ref()
        .expect("`manifest::built_in` must set the `menus` field");
    assert_eq!(
        menus.len(),
        built_in().len(),
        "the shell carries a different menu document from the one this module defines"
    );
    for context in CONTEXTS {
        assert!(
            menus.get(context).is_some(),
            "the shipped shell has no menu for `{context}`"
        );
    }
}

/// **The catalog and the constant list are the same set.**
#[test]
fn the_catalog_defines_exactly_the_documented_contexts() {
    let menus = built_in();
    let declared: BTreeSet<&str> = CONTEXTS.iter().copied().collect();
    assert_eq!(
        declared.len(),
        CONTEXTS.len(),
        "CONTEXTS lists a context id twice"
    );
    let defined: BTreeSet<&str> = menus.iter().map(|m| m.context.as_str()).collect();
    assert_eq!(
        defined, declared,
        "the menu document and CONTEXTS disagree; every sweep in this file is scoped \
         by CONTEXTS, so the extra or missing entry is untested"
    );
}

/// The document is structurally valid on its own.
#[test]
fn the_built_in_menu_document_is_valid() {
    built_in()
        .validate()
        .expect("the built-in menu layer must satisfy every structural rule");
}

/// **No menu names a command that does not exist — stated as the
/// no-placeholders rule, by name.**
#[test]
fn no_menu_offers_a_command_this_build_does_not_have() {
    let planned: BTreeSet<&str> = manifest::PLANNED.iter().map(|(id, _)| *id).collect();
    for menu in built_in().iter() {
        for id in menu.command_ids() {
            assert!(
                !planned.contains(id),
                "menu `{}` offers `{id}`, which `manifest::PLANNED` records as absent \
                 from this build. P3: an unavailable capability renders NOTHING — not \
                 a greyed row, which is a promise the build cannot keep.",
                menu.context
            );
        }
    }
    //
    // The paragraph above reads *"asserted against `PLANNED` rather than
    // against a hand-written list of four ids, so a clipboard command that
    // lands … stops being forbidden here automatically instead of failing
    // a test that had gone stale."* The sweep above does exactly that. And
    // underneath it sat the list anyway, forbidding `edit.cut`,
    // `edit.copy`, `edit.paste` and `edit.paste_in_place` by name.
    //
    //
    // ⇒ Deleted rather than updated, because updating it would restore the
    // exact mechanism the doc comment argues against. `PLANNED` is the one
    // list, and a command that lands leaves it.
}

/// **Every menu opens when the application is at its liveliest.**
#[test]
fn every_menu_offers_something_when_a_document_is_open_and_selected() {
    let (shell, registry) = shell_and_registry();
    let conditions = everything_open();
    let host = MenuHost::new(&shell, &registry, &conditions);
    for context in CONTEXTS {
        assert!(
            host.would_open(context),
            "`{context}` offers nothing even with a document open, pages present and \
             something selected — so right-clicking that surface does nothing, ever"
        );
    }
}

/// **The field menu opens on a field selection ALONE.**
#[test]
fn the_field_menu_opens_with_a_field_selected_and_nothing_else() {
    let (shell, registry) = shell_and_registry();
    let field_only = ConditionSet::new()
        .with("doc.open")
        .with("doc.pages")
        .with(manifest::SELECTION_ACTIONABLE);
    let host = MenuHost::new(&shell, &registry, &field_only);
    assert!(
        host.would_open(CANVAS_FIELD),
        "a selected form field offers no menu, so right-clicking one does nothing"
    );
    // And the object menu opens here TOO, which is correct and is worth
    // asserting rather than leaving as a surprise: both its items can act
    // on a field, so the menus differ by their CONTEXT ID rather than by
    // what is enabled. `canvas::menus::attach` picks Field first when a
    // field is in play, which is where the distinction is made.
    assert!(host.would_open(CANVAS_OBJECT));
}

/// **…and an empty menu never opens.**
#[test]
fn a_menu_with_nothing_to_offer_does_not_open() {
    let (shell, registry) = shell_and_registry();

    // 1. No such context.
    let live = everything_open();
    let host = MenuHost::new(&shell, &registry, &live);
    assert!(
        !host.would_open("canvas.nothing-here"),
        "an unknown context must resolve to no menu, not to an empty one"
    );

    // 2. Every command disabled — nothing is selected, so `format.delete`
    //    is greyed and it is the menu's only item.
    let nothing_selected = ConditionSet::new().with("doc.open").with("doc.pages");
    let host = MenuHost::new(&shell, &registry, &nothing_selected);
    assert!(
        !host.would_open(CANVAS_OBJECT),
        "a menu of nothing but greyed rows is strictly worse than no menu: it costs a \
         click to dismiss and teaches the operator that right-clicking here is useless"
    );
    assert!(
        host.would_open(CANVAS_EMPTY),
        "…while the view menu is still live, which is what makes the canvas's choice \
         of context id the thing that matters"
    );

    // 3. Every command unregistered — the compiled-out build.
    let empty_registry = CommandRegistry::new();
    let host = MenuHost::new(&shell, &empty_registry, &live);
    for context in CONTEXTS {
        assert!(
            !host.would_open(context),
            "`{context}` opened against a registry holding no commands at all"
        );
    }
}

/// **A corrected condition changes the answer.**
#[test]
fn correcting_the_selection_condition_is_what_opens_the_object_menu() {
    let (shell, registry) = shell_and_registry();
    // The snapshot the frame was composed with: nothing was selected
    // when the ribbon was drawn.
    let stale = ConditionSet::new().with("doc.open").with("doc.pages");
    let host = MenuHost::new(&shell, &registry, &stale);
    assert!(!host.would_open(CANVAS_OBJECT));

    // The canvas has since selected the object under the pointer.
    //
    // BOTH conditions, because `attach` corrects both — see
    // `MenuHost::with_conditions`. Correcting only `selection.any` here
    // would have this test passing on a build where `attach` forgot the
    // second, which is the exact hazard the test exists for one level up.
    let corrected = host.with_conditions(&[
        (manifest::SELECTION_ANY, true),
        (manifest::SELECTION_ACTIONABLE, true),
    ]);
    assert!(
        host.would_open_with(CANVAS_OBJECT, &corrected),
        "the right-click selected an object and the menu still refused to open"
    );

    // …and the correction goes both ways, so a menu cannot be opened by
    // a condition the caller has just found to be false.
    //
    // BOTH have to be cleared, and the reason is worth a sentence
    // because the first version of this line cleared only `selection.any`
    // and the assertion failed. `canvas.object`'s two items now take
    // `selection.actionable`, so clearing the narrower condition alone
    // leaves them enabled and the menu opens — correctly.
    //
    // ⇒ A "goes both ways" assertion has to clear **every** condition the
    // forward direction set, or it is asserting about a state the forward
    // direction never produces.
    let cleared = MenuHost::new(&shell, &registry, &corrected).with_conditions(&[
        (manifest::SELECTION_ANY, false),
        (manifest::SELECTION_ACTIONABLE, false),
    ]);
    assert!(!host.would_open_with(CANVAS_OBJECT, &cleared));
}

/// A command may appear in several menus, and on a tab as well.
#[test]
fn every_menu_command_is_also_reachable_from_the_ribbon() {
    let shell = manifest::built_in();
    let on_a_surface: BTreeSet<String> = shell
        .command_references()
        .into_iter()
        .map(|(_, id)| id)
        .collect();
    // The exemption register, and the assertion below consults it rather
    // than being weakened. See `manifest::TAB_SCOPED`.
    let tab_scoped: BTreeSet<&str> = manifest::TAB_SCOPED.iter().map(|(id, _)| *id).collect();
    for menu in built_in().iter() {
        for id in menu.command_ids() {
            if tab_scoped.contains(id) {
                continue;
            }
            assert!(
                on_a_surface.contains(id),
                "menu `{}` is the ONLY route to `{id}`. A context menu is a third \
                 surface carrying commands that already have a home, not a home of \
                 its own — a command reachable by right-click alone is undiscoverable.",
                menu.context
            );
        }
    }
}

/// Menus survive a round trip through RON, which is what makes them
/// customizable.
#[test]
fn the_menu_document_round_trips_through_ron() {
    let original = built_in();
    let text = original.to_ron_pretty().expect("serializes");
    assert_eq!(
        Menus::from_ron(&text).expect("the pretty form parses"),
        original
    );
    // And the shapes an operator would search for are legible in it.
    //
    // The command spelling is checked on the COMPACT form. RON's pretty
    // printer breaks a struct variant across three lines, and `Item::Command`
    // became one when `ItemSize` landed — so a `contains` for the one-line
    // spelling fails on a pretty document that is perfectly correct. The
    // context id is still checked on the pretty form, because that is the
    // string an operator scrolling the file actually looks for.
    assert!(text.contains(CANVAS_OBJECT), "{text}");
    let compact = original.to_ron().expect("serializes");
    //
    // `canvas.object`'s gained `selection.delete_permitted` with the
    // annotation half of R83; `canvas.field`'s gained the same name with
    // the form half, which is what this assertion's previous bare spelling
    // was silently attesting was still missing. A `contains` for
    // `Command(id:"format.delete")` matched only because no gate was
    // written on that menu at all.
    //
    // ⇒ Asserting the gated spelling rather than deleting the assertion:
    // the point of the check is that an operator scrolling the compact
    // document can find the command, and the visible-condition is the half
    // that decides whether the row is drawn — which is exactly what such an
    // operator is looking for it to say.
    assert!(
        compact
            .contains("Command(id:\"format.delete\",visible_when:\"selection.delete_permitted\")"),
        "{compact}"
    );
    assert!(
        !compact.contains("Command(id:\"format.delete\")"),
        "an UNGATED `format.delete` is back on some menu. Both of them are \
         gated on `selection.delete_permitted`, because a Delete drawn where \
         the engine refuses it is silently inert — and on `canvas.field` that \
         press also cleared the selection, blanking the Properties panel \
         sentence that explained the refusal: {compact}"
    );
}

/// Each menu holds the items this module's header claims it holds.
#[test]
fn each_menu_holds_exactly_the_documented_items() {
    let menus = built_in();
    for (context, expected) in [
        (
            CANVAS_OBJECT,
            &[
                "view.zoom_selection",
                "format.properties",
                "format.select_text_line",
                "format.select_form",
                "format.unshare_form",
                "format.merge_text_runs",
                "format.split_text_lines",
                "edit.redact_selection",
                "format.move_to_layer",
                "format.delete",
            ][..],
        ),
        (
            CANVAS_EMPTY,
            &[
                "view.zoom_fit_page",
                "view.zoom_fit_width",
                "view.zoom_fit_height",
                "view.zoom_actual",
            ][..],
        ),
        (
            DOCK_TAB,
            &[
                "view.panel_float",
                "view.panel_dock",
                "view.panel_close",
                "view.reset_layout",
            ][..],
        ),
        (
            CANVAS_DIMENSION,
            &[
                "format.properties",
                "format.dimension_diameter",
                "format.dimension_radius",
                "format.dimension_area",
                "format.dimension_perimeter",
                "format.move_to_layer",
                "format.delete",
            ][..],
        ),
        (
            CANVAS_MARKUP,
            &[
                "format.properties",
                "markup.add_node",
                "markup.remove_node",
                "edit.cut",
                "edit.copy",
                "edit.paste",
                "markup.flatten",
                "format.move_to_layer",
                "format.delete",
            ][..],
        ),
        (OBJECTS_ROW, &["file.properties"][..]),
    ] {
        let menu = menus.get(context).expect("defined");
        let ids: Vec<&str> = menu.command_ids().collect();
        assert_eq!(
            ids, expected,
            "menu `{context}` no longer matches the table in this module's header"
        );
        //
        // The invariant's own stated reason is the test: the sweeps walk
        // `command_ids()`, so an item carrying a command id they cannot
        // see is a hole in them. `Item::Separator` carries no id, refers
        // to no capability and cannot be a route to anything — there is
        // nothing for a sweep to miss — whereas `Item::Custom` carries a
        // *kind* the application draws, which can be a control that
        // invokes something, and `manifest::COLOUR_SWATCH`'s own note
        // records a custom kind that no renderer ever matched going
        // unreported for a whole release.
        //
        // ⇒ So the assertion names the thing it was protecting against
        // rather than everything that is not a command. Widening it back
        // would forbid a separator in every menu in the program to guard
        // against a case a separator cannot produce.
        assert!(
            !menu
                .items()
                .iter()
                .any(|i| matches!(i, Item::Custom { .. })),
            "menu `{context}` holds a non-command item; the sweeps in this file walk \
             `command_ids()` and would not see it"
        );
    }
}

/// **R9, resolved: an absent row, a greyed row and a live row from the
/// same menu definition.**
#[test]
fn the_two_node_rows_are_absent_greyed_and_live_in_the_three_states() {
    use egui_shell::menu::Shortcuts;
    use egui_shell::menu::plan;

    let (shell, registry) = shell_and_registry();
    let shortcuts = Shortcuts::of(&shell);
    let menu = shell
        .menus
        .as_ref()
        .expect("the built-in shell must carry its menus")
        .get(CANVAS_MARKUP)
        .expect("the markup menu is defined");

    // The row for `id`, as `(drawn, pressable)`.
    let row = |conditions: &ConditionSet, id: &str| -> (bool, bool) {
        let slots = plan::resolve(
            menu.items(),
            &registry,
            conditions,
            &shortcuts,
            CANVAS_MARKUP,
        );
        let found = slots.iter().find_map(|slot| match slot {
            plan::Slot::Command {
                command, enabled, ..
            } if command.id == id => Some(*enabled),
            _ => None,
        });
        (found.is_some(), found.unwrap_or(false))
    };

    // 1. A shape with no points at all. Neither node condition is set,
    //    which is what `annotnodes::menu::rows` answers for a `/Square`.
    let no_points = everything_open();
    assert_eq!(
        row(&no_points, "markup.remove_node"),
        (false, false),
        "a shape that will NEVER have points must draw no node row — R9: an \
         unavailable capability renders nothing, and a permanently greyed row is a \
         promise the build cannot keep"
    );
    assert_eq!(row(&no_points, "markup.add_node"), (false, false));
    // …and the menu still opens, on the rows that do apply. This is the half
    // that stops the R9 answer from silently costing the operator the whole
    // context: an `/Ink` stroke still has properties, a clipboard and a Delete.
    let host = MenuHost::new(&shell, &registry, &no_points);
    assert!(
        host.would_open(CANVAS_MARKUP),
        "a markup with no editable points must still offer its other verbs"
    );

    // 2. A three-corner polygon, right-clicked on a corner. The engine
    //    answers `ReshapeWouldBreachVertexFloor`, which is TEMPORARY —
    //    draw another corner and it comes back — so the row is drawn and
    //    greyed, with the command's tooltip naming the floor.
    let at_the_floor = everything_open().with(NODE_REMOVE_OFFERED);
    assert_eq!(
        row(&at_the_floor, "markup.remove_node"),
        (true, false),
        "at the vertex floor the row must be DRAWN and greyed: the refusal stops \
         being true the moment another corner is drawn, and R9 greys exactly that"
    );

    // 3. The same shape with two more corners.
    let live = everything_open()
        .with(NODE_REMOVE_OFFERED)
        .with(NODE_REMOVABLE);
    assert_eq!(row(&live, "markup.remove_node"), (true, true));

    // And the insert row answers the same three ways, from its own pair of
    // conditions — asserted rather than assumed, because the two rows are
    // wired independently and a copy-paste that pointed both at one condition
    // would pass every assertion above.
    assert_eq!(
        row(
            &everything_open()
                .with(NODE_INSERT_OFFERED)
                .with(NODE_INSERTABLE),
            "markup.add_node"
        ),
        (true, true)
    );
    assert_eq!(
        row(
            &everything_open().with(NODE_INSERT_OFFERED),
            "markup.add_node"
        ),
        (true, false)
    );
    assert_eq!(
        row(&live, "markup.add_node"),
        (false, false),
        "the two rows must not share a condition: a right-click on a CORNER offers \
         the removal and must not also offer to split an edge"
    );
}

/// **The menu surface owns no copy of its own — asserted, not
/// assumed.**
#[test]
fn the_menu_surface_owns_no_copy_of_its_own() {
    for menu in built_in().iter() {
        for item in menu.items() {
            match item {
                // A command carries an id; its words are the registry's.
                Item::Command { .. } => {}
                // Punctuation. No words.
                Item::Separator => {}
                Item::Custom { kind, .. } => panic!(
                    // ui-text-exempt: a test panic, read by whoever is looking at
                    // the failure. Never rendered to an operator.
                    "menu `{}` holds a custom row `{kind}`, which the application draws \
                     itself — so it has words, and they belong in `text::menus` rather \
                     than at the call site. This module is empty only while every menu \
                     item is a command reference.",
                    menu.context
                ),
            }
        }
    }
}
