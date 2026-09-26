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
///
/// # Four states, and each is a fact about the document or the mode rather
/// than about what this build can do
///
/// | state | what is drawn |
/// |---|---|
/// | **Read mode** | one sentence naming the mode that can edit. R9's *temporarily* unavailable case — see the module header |
/// | **the file locks it** (§12.5.3 bit 8) | one sentence saying so. R83: the controls are omitted, not offered and refused |
/// | **a ce dimension** | one sentence saying where its text comes from. Rule 15; a note typed over it is regenerated away |
/// | anything else | *Add note* / *Edit note*, the editor when it is open, and *Delete comment* |
///
/// None of the three sentences is a greyed button, and none is temporary in a
/// way the operator cannot see: the first names its own remedy, and the other
/// two are properties of the file.
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
///
/// # Escape saves here, and the labelled buttons still mean what they say
///
/// The operator's rule: *"for adding and editing text when using any tool that
/// has text escape should also save changes to the text. The user can always
/// undo if they want, but it is easy to accidentally press escape and lose a
/// lot of text that has been entered."* The cost is asymmetric — a commit the
/// operator did not want costs one `Ctrl+Z`, and a discard they did not want
/// costs everything they typed, with nothing to undo — so both this function
/// and the Escape arm in [`super::body`] call it.
///
/// ⇒ *Cancel* and the window's ✕ are untouched and still discard. A keystroke
/// is what gets pressed by accident; a labelled button under the pointer is
/// not, and removing the only deliberate way to throw a draft away in order to
/// protect against the accidental one would be a worse trade than the one it
/// fixes.
///
/// # A draft identical to the note is not a write
///
/// `SetNote` with the stored text would be an undo entry whose entire content
/// is *"changed nothing"*, which is the same objection [`open_default`] makes
/// to writing `/Open` on every glance. Escape makes that case common — press it
/// on an editor you opened and did not type in — so the guard is here rather
/// than at either caller.
///
/// # Why the authority test is repeated rather than inherited
///
/// The three early returns in [`controls`] are a **disclosure** ladder: each
/// names a different reason and draws a different sentence. This is the
/// **authority** test, and it has one outcome. [`super::body`]'s Escape arm is
/// outside that ladder, so a note whose editor was opened before the document
/// was signed — or whose annotation was locked under the operator's hand —
/// must not be written by a keypress the ladder never saw.
///
/// `keep_author` comes from the same function the Comments panel uses.
/// `pdfcer-core` named this mistake when it shipped the verb: *"an
/// implementation writing all three keys unconditionally would silently strip
/// the author and date on every correction, leaving a review comment from
/// nobody, dated never."* Two editors for one note, two spellings of the rule,
/// and one of them eventually gets it wrong.
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
