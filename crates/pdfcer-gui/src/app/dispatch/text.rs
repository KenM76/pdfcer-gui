//! # `app::dispatch::text` — the caret, and the commands whose operand it is
//!
//! ## What is here
//!
//! | id | what it does |
//! |---|---|
//! | `edit.text`, `edit.add_text` | **arm** the caret |
//! | `edit.reflow_block` | **act** on the paragraph the caret is in |
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/dispatch/text.md`.

use crate::app::actions::Action;
use crate::app::actions::text::TextAction;
use crate::app::state::Status;

/// Whether this file owns `id`.
#[must_use]
pub(crate) fn handles(id: &str) -> bool {
    // ui-text-exempt: registered command ids, never displayed.
    matches!(id, "edit.text" | "edit.add_text" | "edit.reflow_block")
}

/// Route one caret command.
pub(crate) fn dispatch(
    app: &mut crate::app::PdfcerApp,
    ctx: &egui::Context,
    id: &str,
    actions: &mut Vec<Action>,
) {
    match id {
        // ui-text-exempt: registered command id, never displayed.
        "edit.reflow_block" => reflow(ctx, &app.status, actions),
        _ => arm(app, ctx, id),
    }
}

/// Arm the caret, or decline because this stance cannot author content.
fn arm(app: &mut crate::app::PdfcerApp, ctx: &egui::Context, id: &str) {
    match id {
        "edit.text" | "edit.add_text" => {
            let kind = if id == "edit.add_text" {
                crate::canvas::textedit::TextEditKind::Add
            } else {
                crate::canvas::textedit::TextEditKind::Edit
            };
            if app.capabilities().edit_content {
                // Both ids arm the caret directly — two doors into one
                // room. `edit.text` is the one an operator finds on the Edit
                // tab; `view.tool_text` (T) is the one they find in the tool
                // row. **The CLICK decides edit-versus-add**, so the `kind`
                // here is a starting bias rather than a mode:
                // `textedit::click` turns an `Edit` that lands on no run
                // into an origin.
                let _ = crate::canvas::tool::arm_text_edit(ctx, kind);
            } else {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    format!("command-declined id={id} reason=mode-cannot-edit-content")
                });
            }
        }

        _ => {}
    }
}

/// Re-wrap the paragraph the caret is in, or say why not.
fn reflow(ctx: &egui::Context, status: &Status, actions: &mut Vec<Action>) {
    use crate::text::textedit::ReflowRefusal;

    let Status::Open(doc) = status else {
        // Unreachable in practice — the control is `enabled_when("doc.pages")`
        // — and still not a bare `return`. A dispatch arm that can leave without
        // a word is the shape this whole function exists against, and *"there is
        // no document"* is a cause like any other.
        decline("no-document", ReflowRefusal::NeedsCaret);
        return;
    };
    let Some(draft) = crate::canvas::textedit::read(ctx) else {
        decline("no-caret", ReflowRefusal::NeedsCaret);
        return;
    };
    let crate::canvas::textedit::Anchor::Run { run, .. } = draft.anchor else {
        decline("caret-not-on-a-run", ReflowRefusal::NeedsExistingText);
        return;
    };
    let Some(block) = crate::canvas::textedit::reflow::block_of_run(doc, draft.page, run) else {
        decline("run-not-in-a-block", ReflowRefusal::NoBlock);
        return;
    };
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "reflow-resolved page={} run={run} block={block}",
            draft.page
        )
    });
    actions.push(Action::Text(TextAction::Reflow {
        page: draft.page,
        block,
    }));
}

/// Say why, in the status line and in the trace, and change nothing.
fn decline(reason: &str, why: crate::text::textedit::ReflowRefusal) {
    crate::app::status::decline::record_reflow(why);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!("reflow-declined reason={reason}")
    });
}
