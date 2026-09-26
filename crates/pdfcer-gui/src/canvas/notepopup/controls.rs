//! # `canvas::notepopup::controls` — everything in the window that CHANGES
//! # something
//!
//!
//! ## Why this is the seam
//!
//! It is the same one `crate::panels::comments::editor` took the same day, and
//! for the same reason: the rest of the pop-up **reads** — a heading, a byline,
//! the words, the thread, a hover tooltip, a placement calculation — and this
//! is the only part of it that produces an `Action`. `super::body` decides what
//! the window *says*; this decides what it can *do*.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/notepopup/controls.md`.

use super::model::{self, NoteView};
use super::{Ctx, REGION_DELETE, REGION_EDIT, REGION_OPEN_DEFAULT, REGION_SAVE, open, t};
use crate::app::actions::Action;
use crate::app::actions::annot::AnnotAction;
use crate::panels::comments::note::NoteDraft;

/// The controls under the note: edit, save, remove, delete — or the sentence
/// that says why there are none.
pub(super) fn controls(
    ui: &mut egui::Ui,
    f: &Ctx<'_>,
    note: &NoteView,
    draft: &mut NoteDraft,
    existing: &str,
    actions: &mut Vec<Action>,
) {
    if !f.caps.author_markup {
        ui.label(egui::RichText::new(t::popup_read_only()).small().weak());
        return;
    }
    if note.locked {
        ui.label(egui::RichText::new(t::popup_locked()).small().weak());
        return;
    }
    if f.is_ce_dimension {
        // The caption is already under the body — a ce dimension's text is its
        // measurement — so nothing more is said here. What is withheld is the
        // whole control row, including Delete: `delete_annotation` would
        // remove the `/Line` and leave the `/PieceInfo` sidecar describing a
        // ce dimension that no longer exists. That is the Dimension groups
        // panel's subject, not this window's.
        return;
    }

    if draft.editing(note.id, f.doc.edit_epoch) {
        ui.horizontal(|ui| {
            let save = ui.button(t::popup_save());
            crate::diag::ui_rect_visible(REGION_SAVE, save.rect, f.clip);
            if save.clicked() {
                save_draft(f, note, draft, actions);
            }
            if ui.button(t::popup_cancel()).clicked() {
                draft.close();
            }
            // Only when there is something to remove. `clear_markup_note` on
            // an annotation with no note is a call whose entire effect is an
            // undo entry, and R9's rule about a control that cannot do
            // anything applies to one that can only do nothing.
            if !existing.is_empty()
                && ui
                    .button(t::popup_remove())
                    .on_hover_text(t::popup_remove_tooltip())
                    .clicked()
            {
                actions.push(Action::Annot(AnnotAction::ClearNote { id: note.id }));
                draft.close();
            }
        });
        return;
    }

    ui.horizontal(|ui| {
        let label = if existing.is_empty() {
            t::popup_add()
        } else {
            t::popup_edit()
        };
        let edit = ui.button(label);
        crate::diag::ui_rect_visible(REGION_EDIT, edit.rect, f.clip);
        if edit.clicked() {
            draft.begin(note.id, f.doc.edit_epoch, existing);
        }
        delete(ui, f, note, actions);
    });
    open_default(ui, f, note, actions);
}

/// **Write the open draft to the note and close the editor.**
pub(super) fn save_draft(
    f: &Ctx<'_>,
    note: &NoteView,
    draft: &mut NoteDraft,
    actions: &mut Vec<Action>,
) {
    if !f.caps.author_markup || note.locked || f.is_ce_dimension {
        draft.close();
        return;
    }
    let existing = note.contents.as_deref().unwrap_or_default();
    if draft.text() != existing {
        actions.push(Action::Annot(AnnotAction::SetNote {
            id: note.id,
            text: draft.text().to_owned(),
            keep_author: crate::panels::comments::keeps_author_name(note.author.as_deref()),
        }));
    }
    draft.close();
}

/// **Record this comment's window state in the FILE** —
/// `EditSession::set_annotation_open`, `pdfcer-core` `Pass 253.3`.
fn open_default(ui: &mut egui::Ui, f: &Ctx<'_>, note: &NoteView, actions: &mut Vec<Action>) {
    if !model::can_record_open_state(note) {
        return;
    }
    // The DOCUMENT's value, not what is on screen. The two are different
    // questions and this control answers only the first — see the module
    // header's table. `authored_open` is `read_open`'s folded answer, so an
    // absent `/Open` shows unticked, which is §12.5.6.4 Table 172's stated
    // default value rather than a guess.
    let mut recorded = note.authored_open;
    let response = ui
        .checkbox(&mut recorded, t::popup_open_default())
        .on_hover_text(t::popup_open_default_tooltip());
    crate::diag::ui_rect_visible(REGION_OPEN_DEFAULT, response.rect, f.clip);
    if response.changed() {
        // Pin the window open FIRST, before the action is queued. The action
        // drains after the frame and bumps the edit epoch; the override is
        // read on the very next frame's draw. Setting it here means there is
        // no frame in between on which `authored_open` has changed and nothing
        // is holding the window up.
        open::set(ui.ctx(), &f.doc.path, note.id, true);
        actions.push(Action::Annot(AnnotAction::SetOpen {
            id: note.id,
            open: recorded,
        }));
    }
}

/// *Delete comment*, and the guard that decides whether it is drawn at all.
fn delete(ui: &mut egui::Ui, f: &Ctx<'_>, note: &NoteView, actions: &mut Vec<Action>) {
    if f.doc.session.annotation_deletion_refusal().is_some() {
        return;
    }
    let button = ui
        .button(t::popup_delete())
        .on_hover_text(t::popup_delete_tooltip());
    crate::diag::ui_rect_visible(REGION_DELETE, button.rect, f.clip);
    if button.clicked() {
        // The pop-up is closed first. An open window describing an annotation
        // that no longer exists would draw for one more frame with an empty
        // body, which reads as the delete having failed.
        open::set(ui.ctx(), &f.doc.path, note.id, false);
        actions.push(Action::Annot(AnnotAction::Delete {
            page: f.page_index,
            id: note.id,
        }));
    }
}
