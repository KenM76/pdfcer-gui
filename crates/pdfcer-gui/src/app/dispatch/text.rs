//! # `app::dispatch::text` — the caret, and the commands whose operand it is
//!
//! ## What is here
//!
//! | id | what it does |
//! |---|---|
//! | `edit.text`, `edit.add_text` | **arm** the caret |
//! | `edit.reflow_block` | **act** on the paragraph the caret is in |
//! | `edit.paragraphs` | **switch** whether a click opens a whole paragraph |
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/dispatch/text.md`.

use crate::app::actions::Action;
use crate::app::actions::text::TextAction;
use crate::app::state::Status;

/// Whether this file owns `id`.
#[must_use]
pub(crate) fn handles(id: &str) -> bool {
    // ui-text-exempt: registered command ids, never displayed.
    matches!(
        id,
        "edit.text"
            | "edit.add_text"
            | "edit.reflow_block"
            | "edit.paragraphs"
            | "edit.paragraphs_page"
    )
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
        "edit.paragraphs" => paragraphs(app, ctx),
        "edit.paragraphs_page" => paragraphs_page(app, ctx),
        _ => arm(app, ctx, id),
    }
}

/// Flip whether a click opens a whole paragraph, live and persisted.
fn paragraphs(app: &mut crate::app::PdfcerApp, ctx: &egui::Context) {
    let on = !crate::canvas::textedit::promote::enabled(ctx);
    crate::canvas::textedit::promote::set_enabled(ctx, on);
    app.prefs.open_paragraphs = on;
    let _ = app.prefs.save();
}

/// Flip Paragraphs for the pages the page commands act on (the thumbnails'
/// pick, else the page on screen), to the opposite of the page on screen.
fn paragraphs_page(app: &mut crate::app::PdfcerApp, ctx: &egui::Context) {
    let Some(pages) = app.page_operands() else {
        return;
    };
    let Status::Open(doc) = &app.status else {
        return;
    };
    let promote = crate::canvas::textedit::promote::enabled_on;
    let on = !promote(ctx, doc.serial, doc.view.page_index);
    crate::canvas::textedit::promote::set_pages(ctx, doc.serial, &pages, on);
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
    let run = match draft.anchor {
        crate::canvas::textedit::Anchor::Run { run, .. } => run,
        // The paragraph itself is open. An edited one is re-wrapped by its own
        // commit (`edit_block_text` sets it to its box), so committing it IS
        // the reflow; an unedited one is reflowed by number.
        crate::canvas::textedit::Anchor::Block {
            block,
            ref original,
            ..
        } => {
            if draft.text == *original {
                crate::canvas::textedit::abandon(ctx);
                push(actions, draft.page, block, "paragraph");
            } else {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    format!("reflow-by-commit page={} block={block}", draft.page)
                });
                crate::canvas::textedit::settle(ctx, actions);
            }
            return;
        }
        _ => {
            decline("caret-not-on-a-run", ReflowRefusal::NeedsExistingText);
            return;
        }
    };
    let Some(block) = crate::canvas::textedit::reflow::block_of_run(doc, draft.page, run) else {
        decline("run-not-in-a-block", ReflowRefusal::NoBlock);
        return;
    };
    push(actions, draft.page, block, "run");
}

/// Queue the re-wrap of `block`, recording what the caret was on.
fn push(actions: &mut Vec<Action>, page: usize, block: usize, from: &str) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!("reflow-resolved page={page} block={block} from={from}")
    });
    actions.push(Action::Text(TextAction::Reflow {
        page,
        block,
        width: None,
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
