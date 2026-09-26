//! # `app::actions::tests` — the vocabulary's own assertions
//!
//! [`super`] is the **vocabulary** — one enum, and the argument for every
//! variant in it. This is what is asserted *about* that vocabulary. A reader
//! looking up what a variant means never needs this file, and a reader asking
//! whether the dispatch reaches it never needs the prose.
//!
//! A separate file rather than an inline `#[cfg(test)]` block, because
//! `super`'s content is nearly all doc comments and a test block at the bottom
//! of it would be a hundred lines of *code* at the end of a document.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/tests.md`.

use super::*;

/// **`edit.undo` and `edit.redo` raise actions rather than falling
/// through to `command-unimplemented`.**
#[test]
fn the_history_commands_raise_actions() {
    let ctx = egui::Context::default();
    let mut app = crate::app::tests::opened();

    for (id, expected) in [("edit.undo", Action::Undo), ("edit.redo", Action::Redo)] {
        let token = app
            .commands
            .get(id)
            .unwrap_or_else(|| panic!("`{id}` must be registered")) // ui-text-exempt: test panic
            .handler;
        let mut actions = Vec::new();
        app.dispatch_token(&ctx, token, &mut actions);
        assert_eq!(
            actions,
            vec![expected],
            "`{id}` must raise its action rather than falling through to \
                 `command-unimplemented`, which is what it did for the whole life of the project"
        );
    }
}

/// **The push button arms its tool, like every other kind, and does not also
/// decline.**
#[test]
fn the_push_button_arms_its_tool_like_every_other_kind() {
    let ctx = egui::Context::default();
    let mut app = crate::app::tests::opened();
    // Reached through the dispatcher rather than by writing the field, so the
    // mode is entered exactly as an operator's Ctrl+3 enters it.
    app.dispatch_command(&ctx, "mode.edit", &mut Vec::new());
    crate::app::status::decline::retire();

    let mut actions = Vec::new();
    app.dispatch_command(&ctx, "edit.form_push_button", &mut actions);

    assert!(
        matches!(
            crate::canvas::tool::active(&ctx),
            crate::canvas::tool::CanvasTool::Form(
                crate::canvas::formfield::FormFieldKind::PushButton
            )
        ),
        "the button tool did not arm. The likely cause is one line: `app::conditions` sets \
         `forms.push_button_runnable` beside `doc.pages`, and `edit.form_push_button` is \
         `enabled_when` it." // ui-text-exempt: test assertion message
    );
    assert_eq!(
        crate::app::status::decline::recorded_for_test(),
        None,
        "…and it must not ALSO decline. A command that arms and complains is worse than one \
         that does either — the operator has a live tool and a sentence saying they have not."
    );
}

/// **Every `FormFieldKind` that is useful once placed arms its tool.**
#[test]
fn the_four_useful_form_commands_still_arm() {
    use crate::canvas::formfield::FormFieldKind;
    let ctx = egui::Context::default();
    let mut app = crate::app::tests::opened();
    app.dispatch_command(&ctx, "mode.edit", &mut Vec::new());

    for kind in FormFieldKind::ALL {
        if !kind.is_useful_once_placed() {
            continue;
        }
        let mut actions = Vec::new();
        app.dispatch_command(&ctx, kind.command_id(), &mut actions);
        assert_eq!(
            crate::canvas::tool::active(&ctx),
            crate::canvas::tool::CanvasTool::Form(kind),
            "{} must arm its tool", // ui-text-exempt: test assertion message
            kind.command_id()
        );
    }
}
