//! # `app::tests` — split out under R2 on 2026-08-28
//!
//! ★★ **The inner `#![cfg(test)]` is load-bearing and is not a duplicate of
//! the outer `#[cfg(test)] mod tests;`.** Without it, `tools/gates/check-ui-strings.sh`
//! walks this file as ordinary source and reports every assertion message as a
//! user-visible string that should live in `ui_text` — exclusion 2b in that
//! gate. It is the same line every other split test file in this crate carries.
#![cfg(test)]

use super::*;
// ★ Named here rather than at the top of the module: `Action` is used only
// by these tests since the three drawing surfaces moved to
// `crate::app::surfaces` on 2026-08-20, and a `use` at module scope that
// only the test module needs is a `use` that fails the workspace's
// `-D warnings` clippy gate in a release build.
use crate::app::actions::{Action, VectorAction};
use crate::canvas::selection::{ClickHit, SelectionLevel};
use crate::canvas::target::TargetId;
use crate::panels::objects::test_support::engine_fixture;

/// An application with a four-page fixture open, and nothing selected.
pub(crate) fn opened() -> PdfcerApp {
    let mut app = PdfcerApp::new();
    app.open_path(engine_fixture("pageops/four-pages.pdf"));
    assert!(matches!(app.status, Status::Open(_)), "the fixture opens");
    app
}

/// Select whole object `index` on the current page, the way a canvas click
/// would — through [`crate::canvas::selection::SelectionState::click`], so
/// the state under test is the one a gesture really produces.
///
/// `shift` adds to the selection rather than replacing it, exactly as a
/// Shift+click does.
pub(crate) fn select_object(app: &mut PdfcerApp, index: u64, shift: bool) {
    let Status::Open(doc) = &mut app.status else {
        panic!("no document open") // ui-text-exempt: test panic, never displayed
    };
    let page = doc.view.page_index;
    doc.selection.click(
        page,
        ClickHit {
            object: Some(TargetId::Object(index)),
            ..ClickHit::default()
        },
        shift,
        false,
    );
}

/// The handler token the ribbon would raise for `id`.
fn token_for(app: &PdfcerApp, id: &str) -> egui_shell::commands::HandlerToken {
    app.commands
        .get(id)
        .unwrap_or_else(|| panic!("`{id}` must be registered")) // ui-text-exempt: test panic, never displayed
        .handler
}

/// ★ **`selection.any` is published, and only when something is
/// selected.**
///
/// The condition powers two surfaces the manifest has been carrying
/// unwired: the contextual Format tab's *appearance*, and the enable state
/// of the Delete inside it. It could not be published while the selection
/// lived in `egui::Memory` — [`PdfcerApp::conditions`] has no
/// `egui::Context` — so this asserts the consequence of the move rather
/// than a new policy.
///
/// Both directions matter. Publishing it when nothing is selected would
/// arm a **destructive** command over an empty operand list, which is
/// defect D1's shape with the worst possible verb behind it.
/// ★ **The hand tool and the armed region zoom report a pressed state.**
///
/// The two controls that had none. Both halves are asserted: unarmed must
/// be *unset*, armed must be set. Asserting only the armed half would pass
/// on a condition wired to a constant, which is precisely how a toggle
/// comes to render pressed forever.
#[test]
fn the_memory_backed_toggles_report_their_pressed_state() {
    let app = PdfcerApp::new();
    let ctx = egui::Context::default();

    let hand = egui_shell::ribbon::selected_condition("view.tool_hand");
    let region = egui_shell::ribbon::selected_condition("view.zoom_region");

    assert!(
        !app.conditions(&ctx).is_set(&hand),
        "the select tool is the default, so Hand must not read as pressed"
    );
    assert!(!app.conditions(&ctx).is_set(&region));

    crate::canvas::tool::select(&ctx, crate::canvas::tool::CanvasTool::Hand);
    crate::canvas::zoom::arm_region_zoom(&ctx);

    assert!(app.conditions(&ctx).is_set(&hand), "Hand is armed");
    assert!(
        app.conditions(&ctx).is_set(&region),
        "the region zoom is armed"
    );
}

/// …and they keep reporting it with **no document open**.
///
/// Deliberate, and the opposite of the other conditions in this function:
/// the armed tool survives closing a document, so a ribbon that forgot
/// which tool you were in the moment you closed a file would be reporting
/// something untrue about its own state. The commands are gated on
/// `doc.pages` separately, so the control is greyed *and* pressed — which
/// is exactly "this is the tool you are in, and there is nothing to use it
/// on".
#[test]
fn an_armed_tool_stays_pressed_with_nothing_open() {
    let app = PdfcerApp::new();
    let ctx = egui::Context::default();
    crate::canvas::tool::select(&ctx, crate::canvas::tool::CanvasTool::Hand);

    assert!(matches!(app.status, Status::Empty), "nothing is open");
    assert!(
        app.conditions(&ctx)
            .is_set(&egui_shell::ribbon::selected_condition("view.tool_hand")),
    );
}

/// An application with the engine's page-sized-form fixture open — one page
/// object (the form) and three squares painted from inside it.
fn opened_with_a_form() -> PdfcerApp {
    let mut app = PdfcerApp::new();
    app.open_path(engine_fixture("forms-xobject/page-sized-form.pdf"));
    assert!(matches!(app.status, Status::Open(_)), "the fixture opens");
    app
}

/// Select the form-interior leaf at `index`, the way a canvas click on a
/// square inside the form now would.
fn select_leaf(app: &mut PdfcerApp, index: u64) {
    let Status::Open(doc) = &mut app.status else {
        panic!("no document open") // ui-text-exempt: test panic, never displayed
    };
    let page = doc.view.page_index;
    doc.selection.click(
        page,
        ClickHit {
            object: Some(TargetId::Leaf(index)),
            ..ClickHit::default()
        },
        false,
        false,
    );
}

/// ★★ `selection.in_form` is set for a form-interior selection and for
/// nothing else — which is what greys `format.select_form` correctly.
///
/// The two negatives are the load-bearing half. A condition that were
/// merely a synonym for `selection.any` would light the control on every
/// selection in every document, and the operator would meet a button that
/// declines more often than it works.
#[test]
fn the_in_form_condition_is_set_only_for_a_form_interior_selection() {
    let mut app = opened_with_a_form();
    let ctx = egui::Context::default();
    assert!(
        !app.conditions(&ctx).is_set("selection.in_form"),
        "a freshly opened document has nothing selected"
    );

    // The form itself is an ordinary page object and is NOT "in a form".
    select_object(&mut app, 0, false);
    assert!(app.conditions(&ctx).is_set("selection.any"));
    assert!(
        !app.conditions(&ctx).is_set("selection.in_form"),
        "the container is not inside itself"
    );

    select_leaf(&mut app, 1);
    assert!(app.conditions(&ctx).is_set("selection.any"));
    assert!(app.conditions(&ctx).is_set("selection.in_form"));
}

/// ★★★ **`format.select_form` selects the container**, and what it lands on
/// is an edit operand — which is the whole point of offering it.
///
/// Before this command the operator could reach an object inside a form and
/// could reach nothing else: the deep hit test excludes forms outright, so
/// the container had no route on the canvas at all. This is that route, and
/// the assertion that matters is the last one — after pressing it, Delete
/// has something to delete.
#[test]
fn select_the_form_lands_on_the_container_and_it_is_deletable() {
    let mut app = opened_with_a_form();
    let ctx = egui::Context::default();
    select_leaf(&mut app, 1);

    {
        let Status::Open(doc) = &app.status else {
            unreachable!()
        };
        assert!(
            doc.selection.deletable_objects_on(0).is_empty(),
            "a form-interior object is not an operand for any paint-order verb"
        );
    }

    let mut actions = Vec::new();
    app.dispatch_command(&ctx, "format.select_form", &mut actions);

    let Status::Open(doc) = &app.status else {
        unreachable!()
    };
    assert_eq!(
        doc.selection.targets_on(0),
        vec![TargetId::Object(0)],
        "the outermost enclosing form, in the page's own index space"
    );
    assert_eq!(
        doc.selection.deletable_objects_on(0),
        vec![0],
        "and NOW there is something a verb can act on"
    );
    assert!(
        doc.selection.leaf_indices_on(0).is_empty(),
        "the leaf selection was replaced, not added to"
    );
}

/// Pressing it with nothing selected says so rather than doing nothing.
///
/// ★ `enabled_when` greys the ribbon item and enforces nothing — every
/// other route reaches the dispatcher unchecked — so the arm asks again,
/// and the arm's answer is a sentence rather than silence.
#[test]
fn select_the_form_with_no_form_selected_says_why() {
    let mut app = opened_with_a_form();
    let ctx = egui::Context::default();
    select_object(&mut app, 0, false);

    let mut actions = Vec::new();
    app.dispatch_command(&ctx, "format.select_form", &mut actions);

    assert_eq!(
        crate::app::status::decline::recorded_for_test(),
        Some(crate::app::status::decline::Declined::InsideForm),
        "the operator pressed something that did nothing; it owes them a reason"
    );
}

/// ★★ **Delete on a form-interior selection explains itself.**
///
/// The state this closes: the operator has an outline round the thing they
/// want gone, presses Delete, and nothing at all happens. From where they
/// sit, Delete is broken. It is not — no paint-order verb can address a
/// leaf — but a program that cannot say so has, for practical purposes,
/// the defect anyway.
///
/// And the negative: at the Part or Node rung the operand list is empty for
/// a completely different reason, one the operator can see and put
/// themselves in, and that case stays silent. A bar that narrates the
/// obvious stops being read.
#[test]
fn delete_on_a_form_interior_selection_explains_itself() {
    let mut app = opened_with_a_form();
    let ctx = egui::Context::default();
    select_leaf(&mut app, 1);

    let mut actions = Vec::new();
    app.dispatch_command(&ctx, "format.delete", &mut actions);
    assert!(
        actions.is_empty(),
        "nothing may be raised: there is no operand"
    );
    assert_eq!(
        crate::app::status::decline::recorded_for_test(),
        Some(crate::app::status::decline::Declined::InsideForm),
    );

    // …and an ordinary object still deletes, with no sentence.
    select_object(&mut app, 0, false);
    let mut actions = Vec::new();
    app.dispatch_command(&ctx, "format.delete", &mut actions);
    assert_eq!(actions.len(), 1, "the form itself is perfectly deletable");
    assert_eq!(
        crate::app::status::decline::recorded_for_test(),
        None,
        "a command that ran retires the sentence rather than adding to it"
    );
}

#[test]
fn the_selection_condition_follows_the_selection() {
    let mut app = PdfcerApp::new();
    assert!(
        !app.conditions(&egui::Context::default())
            .is_set("selection.any"),
        "nothing is open, so nothing can be selected"
    );

    app = opened();
    assert!(
        app.conditions(&egui::Context::default())
            .is_set("doc.pages")
    );
    assert!(
        !app.conditions(&egui::Context::default())
            .is_set("selection.any"),
        "a freshly opened document has nothing selected"
    );

    select_object(&mut app, 1, false);
    assert!(
        app.conditions(&egui::Context::default())
            .is_set("selection.any")
    );

    // Escape at the Object rung clears, and the condition follows it back
    // down — a tab that stayed visible over an empty selection would offer
    // a Delete with nothing to delete.
    let Status::Open(doc) = &mut app.status else {
        unreachable!()
    };
    doc.selection.escape();
    assert!(
        !app.conditions(&egui::Context::default())
            .is_set("selection.any")
    );
}

/// ★ **The ribbon's Delete raises the same action the Delete key does.**
///
/// `format.delete` was drawn and enabled from the moment the Format tab
/// landed, and did nothing — the live instance of D1's shape that this
/// stage is accountable for. It became wirable when the selection moved
/// onto `OpenDoc`, because [`PdfcerApp::dispatch_token`] has no
/// `egui::Context` and therefore had no route to a selection in
/// `egui::Memory`.
///
/// Asserted through the real token lookup rather than by calling the arm
/// directly: the dispatch resolves a token back to an id, so a test that
/// skipped that step would pass even if the command were never registered.
#[test]
fn the_ribbon_delete_raises_the_delete_action() {
    // A bare context: these tests exercise the dispatcher, not a
    // frame. `dispatch_command` needs one because three navigation arms
    // write the armed tool and the zoom anchor into egui memory, which
    // is where per-frame UI state lives.
    let ctx = egui::Context::default();
    let mut app = opened();
    let delete = token_for(&app, "format.delete");

    // Nothing selected: nothing raised. An empty batch would be an action
    // the engine has to refuse, reported as a failure the operator caused.
    let mut actions = Vec::new();
    app.dispatch_token(&ctx, delete, &mut actions);
    assert!(actions.is_empty());

    select_object(&mut app, 2, false);
    select_object(&mut app, 0, true);
    let mut actions = Vec::new();
    app.dispatch_token(&ctx, delete, &mut actions);
    assert_eq!(
        actions,
        vec![
            VectorAction::DeleteSelection {
                page: 0,
                objects: vec![0, 2],
            }
            .into()
        ],
        "one action carrying the whole batch, ascending — `delete_objects` \
         resolves every index before planning, so a second single-object \
         action would renumber the page between them"
    );
}

/// ★ **The ribbon's Delete obeys the same rung rule as the key.**
///
/// Inside an object the selection names a subpath, and the only wired verb
/// removes whole objects — one measured CAD export holds an entire drawing
/// view as a single path object with 1,194 subpaths. The rule lives once,
/// on `SelectionState::deletable_objects_on`; this asserts that the ribbon
/// path really reads it rather than re-deriving an operand list of its
/// own, which is exactly how two spellings of a destructive rule drift
/// apart.
#[test]
fn the_ribbon_delete_declines_inside_an_object_just_as_the_key_does() {
    // A bare context: these tests exercise the dispatcher, not a
    // frame. `dispatch_command` needs one because three navigation arms
    // write the armed tool and the zoom anchor into egui memory, which
    // is where per-frame UI state lives.
    let ctx = egui::Context::default();
    let mut app = opened();
    select_object(&mut app, 1, false);

    let Status::Open(doc) = &mut app.status else {
        unreachable!()
    };
    // Double-click into part 1 of the selected object.
    doc.selection.click(
        0,
        ClickHit {
            object: Some(TargetId::Object(1)),
            part: Some(1),
            node: None,
        },
        false,
        true,
    );
    assert_eq!(doc.selection.level(), SelectionLevel::Part);

    let delete = token_for(&app, "format.delete");
    let mut actions = Vec::new();
    app.dispatch_token(&ctx, delete, &mut actions);
    assert!(
        actions.is_empty(),
        "the Part rung has no delete verb wired, and the ribbon must not \
         borrow the Object rung's any more than the key may"
    );

    // …and the tab is still visible, which is why the decline has to be
    // handled rather than made unreachable: something IS selected.
    assert!(
        app.conditions(&egui::Context::default())
            .is_set("selection.any")
    );
}

// -----------------------------------------------------------------------
// The two commands that used to dispatch nowhere
// -----------------------------------------------------------------------

/// ★ **`file.properties` puts the Properties panel on screen, from any
/// mode.**
///
/// The command was named by File ▸ Document, named by the `objects.row`
/// context menu, registered in `crate::shell::commands` — and had no arm,
/// so invoking it traced `command-unimplemented` and did nothing. That is
/// D1's shape: a control that looks available and is inert.
///
/// The mode matters, which is why the test walks all three. The
/// application **opens in Read**, and Read's default arrangement mounts no
/// Properties panel at all (`app::modes`' `spec("read")`), so the
/// interesting case — activate fails, mount, activate again — is the
/// *first* one an operator meets rather than an edge case. Review and Edit
/// mount it already and take the cheap path.
///
/// Driven through the real token lookup, so a command that stopped being
/// registered fails here rather than silently taking the `other` arm.
#[test]
fn the_properties_command_puts_the_panel_on_screen_in_every_mode() {
    // A bare context: these tests exercise the dispatcher, not a
    // frame. `dispatch_command` needs one because three navigation arms
    // write the armed tool and the zoom anchor into egui memory, which
    // is where per-frame UI state lives.
    let ctx = egui::Context::default();
    let mut app = opened();
    let properties = egui_shell::dock::PanelId::new(crate::panels::Panel::Properties.command_id());
    let token = token_for(&app, "file.properties");

    for mode in ["read", "review", "edit"] {
        app.modes
            .on_mode_changed(mode, &mut app.dock, &mut app.layout, &app.panel_registry);

        let mut actions = Vec::new();
        app.dispatch_token(&ctx, token, &mut actions);
        assert!(
            app.dock.is_on_screen(&properties),
            "`file.properties` must produce the panel in the `{mode}` arrangement, \
             mounting it if the operator's layout no longer holds it"
        );
        assert!(
            actions.is_empty(),
            "showing a panel is a dock change, not a document action"
        );

        // Idempotent: asking twice is not a toggle. The `objects.row`
        // context menu offers this command to *describe the row just
        // clicked*, and a second invocation that hid the description
        // would be actively hostile.
        app.dispatch_token(&ctx, token, &mut Vec::new());
        assert!(app.dock.is_on_screen(&properties));
    }
}

/// ★ **`view.reset_layout` restores the active mode's default
/// arrangement.**
///
/// The other command with no arm. `Modes::reset` existed and was tested;
/// nothing invoked it, so View ▸ Window ▸ Reset layout and the `dock.tab`
/// context menu both traced `command-unimplemented`.
///
/// The test asserts the arrangement is *exactly* the mode's default,
/// which is a stronger claim than "the closed panel came back": a reset
/// that produced some third arrangement, or that reset only one dock,
/// would pass the weaker one.
///
/// It resets **before** rearranging as well as after, deliberately. This
/// application loads the operator's persisted layout at start-up, so the
/// arrangement a test inherits is whatever is on the machine running it;
/// the first dispatch is both the assertion that the command works from an
/// arbitrary starting point and the thing that makes the second half
/// deterministic.
///
/// **`ResetScope::All` is the scope this build passes**, and that is a
/// decision recorded in the dispatch arm, not an oversight — with no
/// chooser surface, a control named "Reset layout" that reset half the
/// layout would be the more surprising failure.
#[test]
fn the_reset_layout_command_restores_the_modes_default_arrangement() {
    // A bare context: these tests exercise the dispatcher, not a
    // frame. `dispatch_command` needs one because three navigation arms
    // write the armed tool and the zoom anchor into egui memory, which
    // is where per-frame UI state lives.
    let ctx = egui::Context::default();
    let mut app = opened();
    app.modes
        .on_mode_changed("edit", &mut app.dock, &mut app.layout, &app.panel_registry);
    let default = crate::app::modes::layout_for_build("edit", &app.panel_registry);
    let reset = token_for(&app, "view.reset_layout");

    let mut actions = Vec::new();
    app.dispatch_token(&ctx, reset, &mut actions);
    assert_eq!(
        app.dock.layout(),
        &default,
        "`view.reset_layout` must restore this mode's default, whole"
    );
    assert!(actions.is_empty(), "a layout reset touches no document");

    // Rearrange — closing a panel is the most likely reason an operator
    // reaches for this — and reset again.
    let objects = egui_shell::dock::PanelId::new(crate::panels::Panel::Objects.command_id());
    assert!(app.dock.layout_mut().close(&objects));
    assert_ne!(app.dock.layout(), &default);
    app.dispatch_token(&ctx, reset, &mut Vec::new());
    assert_eq!(app.dock.layout(), &default);
}

/// ★ **A keyboard chord and the control that shares its command do the
/// same thing.**
///
/// The structural half of the two-owner fix. `crate::app::keyboard` no
/// longer knows what `Ctrl+0` *means*; it reads the id out of the manifest
/// keymap and hands it here, so the chord and the ribbon button land in
/// one arm by construction.
///
/// This asserts the consequence: dispatching the id the keymap binds to
/// `Ctrl+0` raises exactly what the ribbon's Actual size raises. It would
/// have failed before the fix — the chord raised `Fit(FitMode::Page)` and
/// the button raised `ZoomTo(1.0)`, which is the defect in one line.
#[test]
fn the_chord_and_the_button_raise_the_same_action() {
    // A bare context: these tests exercise the dispatcher, not a
    // frame. `dispatch_command` needs one because three navigation arms
    // write the armed tool and the zoom anchor into egui memory, which
    // is where per-frame UI state lives.
    let ctx = egui::Context::default();
    let mut app = opened();
    let keymap = app
        .shell
        .as_ref()
        .and_then(|s| s.keymap.as_ref())
        .expect("the built-in manifest binds chords");
    let bound = keymap.get("Ctrl+0").expect("Ctrl+0 is bound").to_owned();

    let mut from_chord = Vec::new();
    app.dispatch_command(&ctx, &bound, &mut from_chord);

    let mut from_button = Vec::new();
    app.dispatch_token(&ctx, token_for(&app, &bound), &mut from_button);

    assert_eq!(from_chord, from_button);
    assert_eq!(from_chord, vec![Action::ZoomTo(1.0)]);
}

/// The mode chords select the mode their tooltips name.
///
/// `MODES_AND_PANELS.md` Part 1 §6 specifies `Ctrl+1`/`Ctrl+2`/`Ctrl+3`,
/// and all three `crate::text::commands::mode_*` tooltips print the chord.
/// Until this arm existed, all three sentences were false: the manifest
/// bound the chords, nothing dispatched them, and `Ctrl+2` was in fact
/// doing fit-width from `keyboard::collect`.
#[test]
fn the_mode_commands_move_the_ribbon_selector() {
    // A bare context: these tests exercise the dispatcher, not a
    // frame. `dispatch_command` needs one because three navigation arms
    // write the armed tool and the zoom anchor into egui memory, which
    // is where per-frame UI state lives.
    let ctx = egui::Context::default();
    let mut app = opened();
    for (command, mode) in [
        ("mode.review", "review"),
        ("mode.edit", "edit"),
        ("mode.read", "read"),
    ] {
        app.dispatch_command(&ctx, command, &mut Vec::new());
        assert_eq!(app.ribbon.mode(), Some(mode));
    }
}
