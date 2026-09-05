//! # `app::dispatch::text` — the caret, and the commands whose operand it is
//!
//! Split out of [`super`] under **R2** on 2026-08-28, when that file sat at
//! exactly 1,500 lines and paragraph reflow needed an arm.
//!
//! ## What is here
//!
//! | id | what it does |
//! |---|---|
//! | `edit.text`, `edit.add_text` | **arm** the caret |
//! | `edit.reflow_block` | **act** on the paragraph the caret is in |
//!
//! ★ The arming pair came across with the reflow rather than being left behind,
//! and that is the seam rather than a convenience: they are the only three
//! commands in the build whose subject is a text caret, and a reader asking
//! *"what can I do to page text"* now has one file to read. Leaving the pair in
//! [`super`] would have satisfied R2 and left the subject in two places, which
//! is the split that costs more than it saves.
//!
//! ## ★★★ Why a command with no operand in its id needs a module of prose
//!
//! `edit.reflow_block` names a **block**, and nothing in the invocation says
//! which. Every other id on the Edit tab either arms a tool (`edit.text`) or
//! acts on a selection the shell already holds (`edit.cut`). This one has to go
//! and *find* its operand, and the answer is a chain:
//!
//! ```text
//! the draft in egui memory  →  page + Anchor::Run { run }  →  block index
//! ```
//!
//! ⇒ Every link can fail, and **each failure means something different to the
//! operator**. That is the whole content of this module: three refusals, each
//! saying the thing that gets them unstuck, instead of one silence.
//!
//! | what is missing | what they must do |
//! |---|---|
//! | no caret at all | click in the paragraph first |
//! | caret on bare page (`Origin`/`Box`) | the caret is placing NEW text; there is no paragraph yet |
//! | run not in a recognised block | this text is not laid out as a paragraph pdfcer can re-wrap |
//!
//! ## ★★ Why the CARET and not a selection rectangle
//!
//! `use-the-conventional-interaction-never-invent-one`: in every word processor
//! the operator has ever used, a paragraph command acts on the paragraph the
//! **insertion point** is in. Word does not ask you to select a paragraph to
//! change its justification, and neither does this.
//!
//! ★ It also happens to be the only thing available — the shell's other
//! selections are annotations, widgets and vector objects, none of which is a
//! text run — but the convention is the reason, and it would still be the
//! reason if a run-selection existed.
//!
//! ## ★ The mode guard is in the registry, not here
//!
//! `edit.reflow_block` is registered `enabled_when("edit.content")` — a reflow
//! rewrites the page's content stream, so a reading stance must never offer it.
//! [`super`]'s `edit.text` arm re-checks `capabilities().edit_content` because
//! that id is *also* reachable from the tool row; this one is reachable only
//! from the Edit tab, so the registry's own gate is the single gate.

use crate::app::actions::Action;
use crate::app::actions::text::TextAction;
use crate::app::state::Status;

/// Whether this file owns `id`.
///
/// ★ Three ids, and `edit.reflow_block` is the only one [`dispatch`] can refuse
/// — the other two arm a tool, which cannot fail once the mode allows it.
///
/// `pub(crate)` for [`super::routes::handles`]' reason: `shell::commands::reach`'s
/// reachability checker must be able to evaluate every guard arm it finds.
#[must_use]
pub(crate) fn handles(id: &str) -> bool {
    // ui-text-exempt: registered command ids, never displayed.
    matches!(id, "edit.text" | "edit.add_text" | "edit.reflow_block")
}

/// Do whatever this build does about one of the three caret commands.
///
/// ★★ The mode check is here for the arming pair and NOT for the reflow, and
/// the asymmetry is deliberate. `edit.text` is reachable from the tool row as
/// well as the Edit tab — `view.tool_text`, the `T` chord — so it can be
/// invoked in a stance whose ribbon never drew it, and [`super`]'s
/// `view.tool_node` arm declines by name for exactly that reason. Reflow has no
/// such second door: it exists on the Edit tab and in the canvas text menu,
/// both of which are absent outside an editing stance.
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
                // ★ Both ids still arm the caret directly, and both are kept
                // — two doors into one room. `edit.text` is the one an
                // operator finds on the Edit tab; `view.tool_text` (T) is
                // the one they find in the tool row. Since 2026-08-19 the
                // CLICK decides edit-versus-add, so the `kind` here is a
                // starting bias rather than a mode: `textedit::click` turns
                // an `Edit` that lands on no run into an origin.
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
///
/// Takes `&Status` rather than `&mut OpenDoc`: this resolves an operand and
/// raises an [`Action`]. The document is changed by
/// `crate::app::actions::textstyle::reflow`, one funnel later, which is where
/// the *"save and reopen"* refusal lives — that one is about the session's
/// history and this one is about the caret, and they are deliberately not
/// merged.
fn reflow(ctx: &egui::Context, status: &Status, actions: &mut Vec<Action>) {
    use crate::text::textedit::ReflowRefusal;

    let Status::Open(doc) = status else {
        // ★★ Unreachable in practice — the control is `enabled_when("doc.pages")`
        // — and still not a bare `return` as of O127. A dispatch arm that can
        // leave without a word is the shape this whole function exists against,
        // and *"there is no document"* is a cause like any other.
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
///
/// ★ One helper for all of them, so a further refusal cannot be added that
/// traces but does not tell the operator — the asymmetry that makes a feature
/// look broken while the log says it declined politely.
///
/// # ★★★ It writes to the DECLINE slot, and until O127 it wrote to the wrong one
///
/// This function used to call `crate::app::actions::record_note`, which the bar
/// draws under **`⚑ About your last edit:`**. Every sentence it produced was
/// correct, and every one of them arrived labelled as a footnote about an
/// **earlier edit** — for a press that changed nothing. The operator's verdict
/// was *"I haven't seen the reflow option actually work with anything when I
/// press it."*
///
/// He was being answered. `app::status::decline`'s own header had already ruled
/// on this exact swap for two other sentences: *"an operator who reads 'About
/// your last edit' after a gesture that did nothing has been told a small lie
/// confidently."* Nothing happened here either, so the slot that says so is
/// `⊗`, and [`crate::app::status::decline::record_reflow`] is the door to it.
///
/// ⇒ The `epoch` parameter went with the swap. A disclosure is stamped with the
/// edit it describes, so it can go stale and retire itself; a decline describes
/// **no** edit and is retired by the operator's next command instead. Carrying
/// an epoch here was the clearest possible sign the sentence was in the wrong
/// place — it was being dated against an edit that had not happened.
fn decline(reason: &str, why: crate::text::textedit::ReflowRefusal) {
    crate::app::status::decline::record_reflow(why);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!("reflow-declined reason={reason}")
    });
}
