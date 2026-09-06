//! # `shell::menus::tests` — the sweeps that keep the menu document honest
//!
//! Split out of [`super`] under **R2** on 2026-09-06, when the sixth canvas
//! menu — `canvas.markup`, the right-click route to a placed markup shape's
//! nodes — took that file past 1,500 lines.
//!
//! ## ★ The seam, and why it is a subject rather than a cut
//!
//! [`super`] is a **document**: one function returning the menus pdfcer
//! defines, plus the prose arguing every row of every one of them. It changes
//! when a menu changes.
//!
//! This is a **checker**. Every test here is a sweep over *whatever*
//! [`super::built_in`] happens to return — every command registered, every menu
//! non-empty in the state it is opened in, every id also reachable from the
//! ribbon, the whole thing round-tripping through RON. Not one of them names a
//! menu it was written for. They change when the *rules about menus* change,
//! which is a different rate and a different reason.
//!
//! ⇒ That is this project's own test for a seam, applied: two subjects, two
//! rates of change. It is the same cut [`crate::canvas::annotnodes`] and
//! [`crate::shell::commands::reach::guards`] already make, and it is why the
//! file that had to be split is the one that grew — the document grew a menu;
//! the checker did not grow anything.
//!
//! ## What did NOT move, and why
//!
//! [`super::MenuHost`]. It is ~300 lines and looks like the other obvious cut,
//! and it stays because it is not a separate subject: `with_condition`'s
//! frame-ordering argument is about *when a menu is drawn*, which is the same
//! subject as *what is in it*, and a reader following a row from its
//! `visible_when` to the condition that corrects it would have to change files
//! mid-sentence. Splitting the tests off costs the reader nothing, because a
//! sweep is read on its own or not at all.

// ★ The marker `tools/gates/check-ui-strings.sh` reads, and it is the FILE that
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
        // ★★ 2026-08-28. Without it `canvas.object` stopped opening, and
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

/// **★ Every command every menu names is registered.**
///
/// The check nothing else performs. `Shell::validate_against` walks
/// `command_references()`, which covers tab groups, the QAT and the
/// keymap and deliberately **not** the menus — so a menu naming a
/// command this build does not have passes the manifest's own
/// validation, renders as one row fewer, and discloses the omission on
/// a channel nobody reads during development.
///
/// `Menus::validate_against` is the engine's own opt-in answer and it
/// checks both this and the structural rules (no empty context id, no
/// duplicate context, no command listed twice within one menu), so it
/// is asked rather than reimplemented. Its error names the menu **and**
/// the id, which is what makes a failure point at a line rather than at
/// a file.
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
///
/// The failure this catches is a one-line omission with no symptom: drop
/// the `menus` assignment from `manifest::built_in` and every test in
/// this file that builds `built_in()` directly still passes, while every
/// right-click in the running application does nothing.
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
///
/// [`CONTEXTS`] is hand-written and every sweep below is only as
/// complete as it is, which is the classic way a test suite quietly
/// stops covering something. Checked in both directions.
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
///
/// Distinct from the registry check and not implied by it: the built-in
/// layer is what every customization layer patches and what a reset
/// restores, so it has to stand up without an application present —
/// non-empty context ids, no duplicates, no command listed twice in one
/// menu.
#[test]
fn the_built_in_menu_document_is_valid() {
    built_in()
        .validate()
        .expect("the built-in menu layer must satisfy every structural rule");
}

/// **★ No menu names a command that does not exist — stated as the
/// no-placeholders rule, by name.**
///
/// `every_command_every_menu_names_is_registered` proves the positive.
/// This proves the *specific* negative `RIBBON_IA.md` §6 asks for and
/// P3 forbids: §6 wants Cut/Copy/Paste on the selection menu, this build
/// has no object clipboard, and the honest answer is **absence**.
///
/// Asserted against `PLANNED` rather than against a hand-written list of
/// four ids, so a clipboard command that lands — and is therefore
/// removed from `PLANNED` — stops being forbidden here automatically
/// instead of failing a test that had gone stale.
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
    // ★★★ **A HAND-WRITTEN LIST OF FOUR IDS STOOD HERE UNTIL 2026-09-01,
    // AND THIS TEST'S OWN DOC COMMENT SAID IT DID NOT.**
    //
    // The paragraph above reads *"asserted against `PLANNED` rather than
    // against a hand-written list of four ids, so a clipboard command that
    // lands … stops being forbidden here automatically instead of failing
    // a test that had gone stale."* The sweep above does exactly that. And
    // underneath it sat the list anyway, forbidding `edit.cut`,
    // `edit.copy`, `edit.paste` and `edit.paste_in_place` by name.
    //
    // The object clipboard landed on 2026-08-20. `edit.copy` has a
    // registration, a dispatch arm, a driven check and — as of
    // `OPERATOR_REQUESTS.md` O71 — a right-click row on the reader's
    // picture menu, which is what made this fail. **The test was not
    // protecting an invariant; it was pinning a fact that had stopped
    // being true, in a file whose prose already said it should not.**
    //
    // ⇒ Deleted rather than updated, because updating it would restore the
    // exact mechanism the doc comment argues against. `PLANNED` is the one
    // list, and a command that lands leaves it.
}

/// **★ Every menu opens when the application is at its liveliest.**
///
/// The other half of the empty-menu rule, and the half that would
/// otherwise be satisfied by defining no menus at all. A menu that never
/// opens is indistinguishable from a right-click that is not wired, and
/// the operator draws the same conclusion from both.
///
/// `dock.tab` is included: it is not *attached* (see the module header),
/// but the day the `egui-shell` seam lands it must have something to
/// offer, and this is what says so.
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

/// ★★★ **The field menu opens on a field selection ALONE.**
///
/// The state the operator is actually in when they right-click a text box:
/// `doc.selected_field` is set and `SelectionState` is **empty**, because a
/// `/Widget` is deliberately not an annotation selection. Every other canvas
/// menu resolves nothing there.
///
/// ⇒ This is the assertion that would have caught the bug this feature
/// shipped with for ten minutes: `format.delete` and `format.properties`
/// were gated on `selection.any`, which is **false** in exactly this state,
/// so both items resolved disabled, `offers_anything` was false, and the
/// menu never opened. A right-click on a form field would have done nothing
/// at all — `DEFECTS.md` D1's shape, arrived at through a new door.
///
/// ★ `everything_open()` is deliberately not used: it sets both conditions
/// and would pass on a build where the two are confused. The whole point is
/// that only the wider one holds here.
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
    // ★★ And the object menu opens here TOO, which is correct and is worth
    // asserting rather than leaving as a surprise: both its items can act
    // on a field, so the menus differ by their CONTEXT ID rather than by
    // what is enabled. `canvas::menus::attach` picks Field first when a
    // field is in play, which is where the distinction is made.
    assert!(host.would_open(CANVAS_OBJECT));
}

/// **★ …and an empty menu never opens.**
///
/// The engine's rule 2, asserted through the seam this application
/// actually uses rather than against the engine's own unit tests.
/// Three shapes, and all three are reachable:
///
/// 1. **a context with no menu at all** — a right-click site whose id is
///    misspelled, or one wired ahead of its menu;
/// 2. **a menu whose every command is disabled** — `canvas.object` with
///    nothing selected, which is what a right-click on paper would find
///    if the canvas picked the wrong context id;
/// 3. **a menu whose every command is unregistered** — the shape a
///    build with a capability compiled out produces.
///
/// Shape 2 is the one that matters most in daily use, and it is the one
/// a naive wiring gets wrong: `format.delete` is registered, so a
/// `context_menu` closure written by hand would happily draw it greyed
/// and cost a click to dismiss.
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

/// **★ A corrected condition changes the answer.**
///
/// [`MenuHost::with_condition`] exists for one frame-ordering hazard,
/// and this is that hazard reduced to two assertions: with the stale
/// snapshot the selection menu does not open, and with the correction
/// the canvas just computed it does.
///
/// Without this the first right-click on an object silently does
/// nothing — the menu is decided before `egui` is asked for a popup, so
/// there is no later frame on which it can recover.
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
    // ★ BOTH conditions, because `attach` corrects both — see
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
    // ★★ BOTH have to be cleared, and the reason is worth a sentence
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
///
/// `RIBBON_IA.md` §5.8: the context menu *"carries the same commands
/// again … that is not duplication in the P1 sense — context menus are
/// not tabs"*. Every id in this document is also on a ribbon tab, which
/// is the point and not an oversight; if a future edit extends the
/// one-command-one-tab rule over menus, this is the test that says no.
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
///
/// The whole value proposition of the shell-as-data design is that an
/// operator can edit this; `crate::shell::ron` asserts the same thing
/// for the manifest as a whole. Asserted here as well, on the menu
/// document alone, because a failure in the shared file says only that
/// *something* stopped round-tripping.
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
    // ★ The command spelling is checked on the COMPACT form. RON's pretty
    // printer breaks a struct variant across three lines, and `Item::Command`
    // became one when `ItemSize` landed — so a `contains` for the one-line
    // spelling fails on a pretty document that is perfectly correct. The
    // context id is still checked on the pretty form, because that is the
    // string an operator scrolling the file actually looks for.
    assert!(text.contains(CANVAS_OBJECT), "{text}");
    let compact = original.to_ron().expect("serializes");
    // ★★ The spelling checked here carries the CONDITION, and it had to
    // change on 2026-08-29: **both** `format.delete` items now do.
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
///
/// A change-detector, and deliberately one: the table in the header is
/// the specification, and a menu that quietly gains an item has a
/// specification that quietly became wrong. The failure message names
/// the menu, so the fix is one line in one of the two places.
#[test]
fn each_menu_holds_exactly_the_documented_items() {
    let menus = built_in();
    for (context, expected) in [
        (
            CANVAS_OBJECT,
            &[
                "view.zoom_selection",
                "format.properties",
                "format.select_form",
                "format.unshare_form",
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
        // ★ The markup menu, 2026-09-06. Listed here for the same reason the
        // four above are — the header claims an Items column and a claim beside
        // the thing it describes decays — and for one more: `markup.add_node`
        // and `markup.remove_node` are `TAB_SCOPED`, so this list is the ONLY
        // written record of where either verb is reachable from. Losing a row
        // here loses the command, and no ribbon test would notice.
        (
            CANVAS_MARKUP,
            &[
                "format.properties",
                "markup.add_node",
                "markup.remove_node",
                "edit.cut",
                "edit.copy",
                "edit.paste",
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
        // ★★ **No CUSTOM item**, which is what the sweeps above would
        // miss. Narrowed from "no non-command item" on 2026-09-04, when
        // `dock.tab` grew a separator.
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

/// **★★★ R9, resolved: an absent row, a greyed row and a live row from the
/// same menu definition.**
///
/// The markup menu's two node rows are the only place in this document where a
/// row can be *absent on one shape and greyed on another*, and the two halves
/// come from two different mechanisms — `visible_when` on the item, and the
/// command's own `enabled_when`. A build that wired one and not the other would
/// look correct in every screenshot of the working case.
///
/// The three cases below are the three the operator meets:
///
/// | conditions | what the row is | the shape it describes |
/// |---|---|---|
/// | neither offered | **absent** | a `/Square`, an `/Ink` stroke — no points, ever |
/// | offered, not enabled | **greyed** | a three-corner polygon, on a corner: the vertex floor |
/// | offered and enabled | **live** | a five-corner polygon, on a corner |
///
/// ★ Falsified three ways, each independently: dropping `shown_when` from the
/// item makes case 1 fail (the row is drawn where it can never work); dropping
/// `enabled_when` from the command makes case 2 fail (a floor-breaching remove
/// is offered as pressable); and setting `enabled` without `offered` — which no
/// caller does, because `RowState::enabled` implies `RowState::shown` — would
/// leave case 3 asserting nothing, which is why case 3 sets both.
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
