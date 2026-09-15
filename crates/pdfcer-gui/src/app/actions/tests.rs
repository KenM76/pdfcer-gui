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

use super::*;

/// **`edit.undo` and `edit.redo` raise actions rather than falling
/// through to `command-unimplemented`.**
///
/// The dispatch link. Written through `PdfcerApp::dispatch_token` with the
/// token the **ribbon** would raise, so a build that renamed the id or
/// reassigned the token fails here rather than shipping a control whose press
/// is traced and discarded. `crate::app::files`'
/// `the_save_copy_command_raises_the_save_action` is the same assertion for
/// the save commands.
///
/// # What it deliberately does not assert
///
/// That the actions *do* anything. Two arms that pushed the wrong variant
/// would pass a test written as "some action was raised", which is why the
/// comparison is against the exact vector — and what each variant does when
/// applied is `crate::app::actions::apply`'s
/// `an_undo_is_an_edit_and_moves_the_epoch_like_one`, on a real fixture with
/// a real edit on the log.
///
/// # Why an EMPTY log is the state under test here
///
/// Because the dispatcher must not consult one. `undo.available` greys the
/// control and the apply arm declines an empty stack in words — both of
/// which are somebody else's job — and an arm that checked the session here
/// would be the second place that question is asked. So the action is raised
/// with nothing to undo, exactly as it would be for a `Ctrl+Z` fired at a
/// freshly opened document, and the decline happens downstream.
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
///
/// `edit.form_push_button` is live: `pdfcer-core`'s
/// `EditSession::set_button_action` gives a placed button an `/A`, so the
/// button pdfcer draws runs something. `app::conditions` sets
/// `forms.push_button_runnable` and the command is `enabled_when` it — one
/// line, and therefore one careless revert away from re-greying the control.
/// Without this test that revert is invisible: the ribbon item goes grey and
/// every other test still passes.
///
/// # Why it also asserts that nothing was declined
///
/// Because **greying is a hint and a sentence is the answer**, and the two are
/// not interchangeable. `egui` refuses a click on a disabled widget and that is
/// the whole of what greying does — a chord, the QAT, a context menu or the
/// `PDFCER_DIAG_INVOKE` seam all reach `dispatch_command` without passing the
/// ribbon at all. So a command that is unavailable must say so in words at the
/// point it is refused, and a blanket refusal at the top of `dispatch_command`
/// is not that repair: it stops the arming and removes the words in one move.
/// `the_history_commands_raise_actions`' header states the same rule from the
/// other side — the dispatcher must not consult the undo log, because the apply
/// arm is what declines an empty stack in words.
///
/// The guard that forces a future author to rebuild a worded decline for a kind
/// that becomes inert is `canvas::formfield::tests::no_kind_is_authorable_but_inert`,
/// whose failure message names both halves of the repair.
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
///
/// The positive control for the test above. Without it, a mistake that declined
/// every form command would leave that test passing and the whole feature dead
/// — the standing rule that a check which cannot fail is not evidence, applied
/// to its own neighbour.
///
/// The `is_useful_once_placed` filter is the enumeration's own answer to *"can
/// pdfcer do anything with this once it is on the page?"*. It currently admits
/// every kind, so the `continue` is dormant; it is kept so that a kind added
/// while it is still inert does not turn this control red for the wrong reason.
/// The name says four and the body says all of them — read the body.
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
