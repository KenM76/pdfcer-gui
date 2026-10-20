//! # `canvas::textedit::restyled` — a draft stays open through Bold, Italic
//! and the font pickers
//!
//! Formatting a selection inside a draft on one run commits the draft and
//! sends the restyle as an action, which closes the draft. [`arm`] records
//! the run; once the restyle has landed (the document's edit epoch has moved)
//! and left the same characters selected on that run, [`resume`] opens the
//! draft again with them selected, so typing and further formatting carry on
//! as in a word processor.
//!
//! A restyle that is refused, or that leaves the selection on another run,
//! reopens nothing: the selection stays as a sweep, as before.

use super::{Anchor, Draft, TextEditKind, read, store};
use crate::app::state::OpenDoc;

const KEY: &str = "textedit-restyled-reopen"; // ui-text-exempt: a memory key, never displayed.

/// How many passes an armed reopen waits for its restyle to land before it is
/// dropped.
const PATIENCE: u64 = 30;

#[derive(Clone, Copy, Debug)]
struct Armed {
    page: usize,
    run: usize,
    /// The document's edit epoch when the restyle was sent.
    epoch: u64,
    /// `ctx.cumulative_pass_nr()` then.
    pass: u64,
    /// Whether the caret was at the selection's end.
    caret_last: bool,
}

/// Record that the draft `draft` on `doc` is closing for a restyle of its
/// selection, so [`resume`] reopens it. Only a draft on a run is reopened.
pub fn arm(ctx: &egui::Context, doc: &OpenDoc, draft: &Draft) {
    let Anchor::Run { run, .. } = draft.anchor else {
        return;
    };
    let armed = Armed {
        page: draft.page,
        run,
        epoch: doc.edit_epoch,
        pass: ctx.cumulative_pass_nr(),
        caret_last: draft.mark.is_none_or(|m| draft.caret >= m),
    };
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(KEY), armed));
}

/// Reopen the draft [`arm`] closed, once its restyle has landed. Answers
/// whether it did.
pub fn resume(ctx: &egui::Context, doc: &OpenDoc) -> bool {
    let Some(armed) = ctx.data(|d| d.get_temp::<Armed>(egui::Id::new(KEY))) else {
        return false;
    };
    let waited = ctx.cumulative_pass_nr().saturating_sub(armed.pass);
    if doc.edit_epoch == armed.epoch && waited <= PATIENCE && read(ctx).is_none() {
        return false;
    }
    ctx.data_mut(|d| d.remove::<Armed>(egui::Id::new(KEY)));
    if doc.edit_epoch == armed.epoch || read(ctx).is_some() {
        return false;
    }
    let Some(draft) = reopened(doc, armed) else {
        return false;
    };
    let (page, run) = (armed.page, armed.run);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!("text-edit-reopened-after-style page={page} run={run}")
    });
    store(ctx, draft);
    true
}

/// The draft over the live selection, when it lies inside the armed run.
fn reopened(doc: &OpenDoc, armed: Armed) -> Option<Draft> {
    let sel = doc
        .text_selection
        .as_ref()
        .filter(|s| s.live(doc.edit_epoch) && s.page == armed.page)?;
    let (from, to) = pdfcer_gui_base::textselection::ordered(sel.anchor(), sel.focus());
    if from.run != armed.run || to.run != armed.run {
        return None;
    }
    let text = doc.provenance_page_text(armed.page)?;
    let original = text.runs.get(armed.run)?.text.clone();
    let chars = |byte: usize| original.get(..byte).map(|s| s.chars().count());
    let (a, b) = (chars(from.byte_offset)?, chars(to.byte_offset)?);
    let (mark, caret) = if armed.caret_last { (a, b) } else { (b, a) };
    Some(Draft {
        page: armed.page,
        kind: TextEditKind::Edit,
        anchor: Anchor::Run {
            run: armed.run,
            original: original.clone(),
        },
        text: original,
        caret,
        mark: (a != b).then_some(mark),
        seeded: true,
    })
}
