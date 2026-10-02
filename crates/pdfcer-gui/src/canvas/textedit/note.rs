//! # `canvas::textedit::note` — what the last keystroke changed on its way into
//! the draft, for the status bar
//!
//! Raised by [`super::edits`], shown by `app::status::disclosure` while the
//! draft is open, replaced by the next edit and dropped with the draft. Each
//! raise is traced as `text-edit-note`.

use pdfcer_gui_base::text::draftnote::DraftNote;

const KEY: &str = "textedit-draft-note"; // ui-text-exempt: a memory key, never displayed.

fn id() -> egui::Id {
    egui::Id::new(KEY)
}

/// Show `note` until the next edit.
pub fn raise(ctx: &egui::Context, note: DraftNote) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!("text-edit-note note={}", note.token())
    });
    ctx.data_mut(|d| d.insert_temp(id(), note));
}

/// Drop the note: the edit it described is no longer the last one.
pub fn forget(ctx: &egui::Context) {
    ctx.data_mut(|d| d.remove::<DraftNote>(id()));
}

/// The note for the open draft, if any.
#[must_use]
pub fn live(ctx: &egui::Context) -> Option<DraftNote> {
    super::read(ctx)?;
    ctx.data(|d| d.get_temp::<DraftNote>(id()))
}
