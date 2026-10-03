//! # `canvas::textedit::history` — the open draft's undo stack, in
//! `egui::Memory`
//!
//! Lives as long as the draft: [`super::abandon`] calls [`forget`]. The
//! stacks themselves are `editmodel::history`.

use pdfcer_gui_base::editmodel::history::{DraftHistory, EditKind, Snap};

use super::Draft;

const KEY: &str = "textedit-draft-history"; // ui-text-exempt: a memory key, never displayed.

fn id() -> egui::Id {
    egui::Id::new(KEY)
}

fn load(ctx: &egui::Context) -> DraftHistory {
    ctx.data(|d| d.get_temp::<DraftHistory>(id()))
        .unwrap_or_default()
}

fn save(ctx: &egui::Context, h: DraftHistory) {
    ctx.data_mut(|d| d.insert_temp(id(), h));
}

/// The draft's undoable state.
#[must_use]
pub fn snap(draft: &Draft) -> Snap {
    Snap {
        text: draft.text.clone(),
        caret: draft.caret,
        mark: draft.mark,
    }
}

/// Record `draft` as it is before an edit of `kind`.
pub fn record(ctx: &egui::Context, draft: &Draft, kind: EditKind) {
    let mut h = load(ctx);
    h.record(snap(draft), kind);
    save(ctx, h);
}

/// Re-base the draft's history into its paragraph; see `DraftHistory::embed`.
pub fn embed(ctx: &egui::Context, before: &str, after: &str) {
    let mut h = load(ctx);
    h.embed(before, after);
    save(ctx, h);
}

/// End the current typing run; the next edit is its own undo entry.
pub fn break_run(ctx: &egui::Context) {
    let mut h = load(ctx);
    h.break_run();
    save(ctx, h);
}

/// Which way a step moves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Undo,
    Redo,
}

/// Step the draft's history in `draft`, answering whether it moved. `false`
/// when the step belongs to the document instead: nothing to undo, or for a
/// redo, nothing to redo in a draft with no edits.
pub fn step(ctx: &egui::Context, draft: &mut Draft, way: Step) -> bool {
    let mut h = load(ctx);
    let to = match way {
        Step::Undo => h.undo(snap(draft)),
        Step::Redo => h.redo(snap(draft)),
    };
    let Some(to) = to else {
        return false;
    };
    draft.text = to.text;
    draft.caret = to.caret;
    draft.mark = to.mark;
    save(ctx, h);
    true
}

/// Whether the draft has edits of its own, so an empty redo stack still keeps
/// `Ctrl+Y` inside the draft.
#[must_use]
pub fn has_edits(ctx: &egui::Context) -> bool {
    !load(ctx).is_empty()
}

/// Drop the history with its draft.
pub fn forget(ctx: &egui::Context) {
    ctx.data_mut(|d| d.remove::<DraftHistory>(id()));
}
