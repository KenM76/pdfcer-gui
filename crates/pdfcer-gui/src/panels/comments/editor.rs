//! # `panels::comments::editor` — everything on a row that WRITES
//!
//!
//! ## Why this is the seam, and not "split the rows from the strip"
//!
//! R2's rule is *find the seam*, and the file genuinely comes apart here.
//! [`super::body`], [`super::row`], [`super::delete_control`] and
//! [`super::filter_strip`] are all **the list**: they decide what is shown and
//! in what order, and every one of them is a pure function of the document plus
//! a filter. What is in this file is the only part of the panel that holds the
//! operator's own half-finished work — a [`super::note::NoteDraft`] — and the
//! only part whose output is a **verb**.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/comments/editor.md`.

use pdfcer_core::object::ObjId;

use super::note::{DraftTarget, NoteDraft};
use super::{
    CommentRow, Note, REGION_BOX, REGION_EDIT, REGION_POST, REGION_REMOVE, REGION_REPLY,
    REGION_SAVE, Relation, RowSink,
};
use crate::app::actions::annot::AnnotAction;
use crate::text::panels::comments as t;

/// **The note editor for one row, and the control that opens it.**
pub(super) fn note_controls(
    ui: &mut egui::Ui,
    comment: &CommentRow,
    draft: &mut NoteDraft,
    epoch: u64,
    sink: &mut RowSink<'_>,
) {
    // **Nothing to type into in a reading stance.** Same finding, same
    // frame and same argument as the Delete control — see `RowSink::deletable`
    // at its assignment. `Add note` and `Edit note` both **write** to the
    // document, so a mode that does not author markup is offered neither.
    //
    // ⚠ Deliberately BEFORE the ce-dimension branch below, and the order is
    // load-bearing: that branch draws an explanatory sentence about why a ce
    // dimension's note is not editable *here*, which in Read would answer a
    // question the operator cannot have asked, about a control that is not on
    // screen. R9's rule is that an unavailable capability renders **nothing** —
    // and a sentence is something.
    if !sink.deletable_stance {
        return;
    }
    if comment.is_ce_dimension {
        ui.label(
            egui::RichText::new(t::comment_row_note_not_editable_ce_dimension())
                .small()
                .weak(),
        );
        return;
    }
    let Some(id) = comment.id else {
        ui.label(
            egui::RichText::new(t::comment_row_note_no_handle())
                .small()
                .weak(),
        );
        return;
    };

    if draft.editing(id, epoch) {
        editor(ui, comment, id, draft, sink);
        return;
    }

    let existing = note_text(&comment.note);
    let label = if existing.is_empty() {
        t::comment_row_add_note()
    } else {
        t::comment_row_edit_note()
    };
    ui.horizontal(|ui| {
        let button = ui.button(label);
        // `ui_rect_visible`, not `ui_rect`: these rows live in a `ScrollArea`,
        // and a control scrolled out of view still reports a rect. A harness
        // clicking a coordinate that is behind the scroll edge clicks whatever
        // IS there, which fails as something else entirely.
        *sink.writing_controls_drawn += 1;
        if !*sink.published {
            crate::diag::ui_rect_visible(REGION_EDIT, button.rect, ui.clip_rect());
            *sink.published = true;
        }
        if button.clicked() {
            draft.begin(id, epoch, existing);
        }
        reply_control(ui, id, draft, epoch, sink);
    });
}

/// **Answer this comment** — `EditSession::add_reply`, `Pass 253.0`.
fn reply_control(
    ui: &mut egui::Ui,
    id: ObjId,
    draft: &mut NoteDraft,
    epoch: u64,
    sink: &mut RowSink<'_>,
) {
    let button = ui
        .button(t::comment_row_reply())
        .on_hover_text(t::comment_row_reply_tooltip());
    *sink.writing_controls_drawn += 1;
    if !*sink.reply_published {
        crate::diag::ui_rect_visible(REGION_REPLY, button.rect, ui.clip_rect());
        *sink.reply_published = true;
    }
    if button.clicked() {
        draft.begin_reply(id, epoch);
    }
}

/// **Whether there is anything to post** — the shell's own R83 guard, and
/// the reason it cannot be delegated.
#[must_use]
pub(crate) fn reply_is_postable(text: &str) -> bool {
    !text.trim().is_empty()
}

/// **Whether this annotation already carries a byline that is not ours to
/// move** — the one decision in this panel with a consequence in the file.
pub(super) fn keeps_author(comment: &CommentRow) -> bool {
    keeps_author_name(comment.author.as_deref())
}

/// [`keeps_author`] over the name alone — **the one spelling of the rule**.
#[must_use]
pub(crate) fn keeps_author_name(author: Option<&str>) -> bool {
    author.is_some_and(|author| !author.trim().is_empty())
}

/// The words already on the annotation — what seeds an editor, and what a
/// commit is compared against.
fn note_text(note: &Note) -> &str {
    match note {
        Note::Text(text) | Note::Description(text) => text.as_str(),
        Note::Absent => "",
    }
}

/// **What Escape writes** — the draft, not nothing.
pub(super) fn escape_commits(
    comment: &CommentRow,
    id: ObjId,
    draft: &NoteDraft,
) -> Option<AnnotAction> {
    if draft.target() == Some(DraftTarget::Reply) {
        return reply_is_postable(draft.text()).then(|| AnnotAction::Reply {
            parent: id,
            text: draft.text().to_owned(),
        });
    }
    (draft.text() != note_text(&comment.note)).then(|| AnnotAction::SetNote {
        id,
        text: draft.text().to_owned(),
        keep_author: keeps_author(comment),
    })
}

/// The open editor: the box, the hint, the signature disclosure and the three
/// controls.
fn editor(
    ui: &mut egui::Ui,
    comment: &CommentRow,
    id: ObjId,
    draft: &mut NoteDraft,
    sink: &mut RowSink<'_>,
) {
    //
    // `CommentsUi::writing_controls_drawn` was incremented only by the
    // *buttons* that open an editor, never by the editor itself. So on a row
    // whose editor was already open the tally was blind to a live `TextEdit`
    // and a live *Save note* — and the panel's own R9 test, which drives the
    // real `body` in Read and asserts the tally is zero, **could not see the
    // editor path at all**.
    //
    *sink.writing_controls_drawn += 1;
    let response = ui.add(
        // escape-disposition: commits — `escape_commits` decides the
        // destination and writes it; nothing is raised for an unchanged draft.
        egui::TextEdit::multiline(draft.text_mut())
            .desired_rows(3)
            .desired_width(f32::INFINITY),
    );
    crate::diag::ui_rect_visible(REGION_BOX, response.rect, ui.clip_rect());
    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
        if let Some(verb) = escape_commits(comment, id, draft) {
            *sink.verb = Some(verb);
        }
        draft.close();
        return;
    }

    //
    // It said, in un-`.weak()` type above the keyboard hint, that the words
    // printed in a text box could not be changed once it was placed and that
    // saving would leave the page reading what it did before. That was true and
    // measured when it was written: `EditSession::set_markup_note` committed
    // the annotation dictionary and not the `/AP` stream that paints the words.
    //
    // `pdfcer-core` `95a936e` closed it the same afternoon — `set_markup_note`
    // now re-bakes the appearance itself, in the same command and the same undo
    // entry — so the sentence became false while still reading as caution,
    // which is the kind of lie nobody notices. Deleted rather than reworded.
    //
    // And it could not be reworded to cover what survives. The one case
    // still owed a sentence is a text box whose appearance **another program**
    // drew, which pdfcer preserves rather than replaces — and that is decided
    // by baking the words and comparing bytes, *inside* `set_markup_note`.
    // Nothing drawn before the call can know it, so the surviving disclosure is
    // necessarily an after-the-fact one, on the status line, gated on
    // `MarkupNoteChange::appearance_rebaked`
    // (`crate::app::actions::annots::set_note`). The whole table is at
    // `crate::text::textannot`'s edit-time banner.
    //
    // R8b is unchanged and still met: the report is off-canvas, and the box
    // renders exactly as it will save.

    // **The destination decides everything below this line**, and it is
    // asked once. `NoteDraft::target` is `None` only when nothing is open,
    // which cannot be the case here — the caller reached this function through
    // `draft.editing(id, epoch)` — so the fallback is the note case rather
    // than a branch that draws nothing. See [`DraftTarget`].
    if draft.target() == Some(DraftTarget::Reply) {
        reply_editor(ui, comment, id, draft, sink);
        return;
    }

    ui.label(
        egui::RichText::new(t::comment_row_note_hint())
            .small()
            .weak(),
    );

    // Whose name ends up on it. The disclosure and the action's flag come from
    // ONE function on purpose: a sentence that could disagree with the edit it
    // describes is worse than no sentence.
    let keep_author = keeps_author(comment);
    let signature = match comment.author.as_deref() {
        Some(author) if keep_author => t::comment_row_note_signature_kept(author.trim()),
        _ => t::comment_row_note_signature().to_owned(),
    };
    ui.label(egui::RichText::new(signature).small().weak());

    let had_note = !matches!(comment.note, Note::Absent);
    ui.horizontal(|ui| {
        *sink.writing_controls_drawn += 1;
        let save = ui.button(t::comment_row_note_save());
        crate::diag::ui_rect_visible(REGION_SAVE, save.rect, ui.clip_rect());
        if save.clicked() {
            *sink.verb = Some(AnnotAction::SetNote {
                id,
                text: draft.text().to_owned(),
                keep_author,
            });
        }
        if ui.button(t::comment_row_note_cancel()).clicked() {
            draft.close();
        }
        // Only when there is something to remove. `clear_markup_note` on an
        // annotation with no note is a call whose entire effect is an undo
        // entry, and R9's rule about a control that cannot do anything applies
        // to a control that can only do nothing.
        if had_note {
            *sink.writing_controls_drawn += 1;
            let remove = ui
                .button(t::comment_row_note_remove())
                .on_hover_text(t::comment_row_note_remove_tooltip());
            crate::diag::ui_rect_visible(REGION_REMOVE, remove.rect, ui.clip_rect());
            if remove.clicked() {
                *sink.verb = Some(AnnotAction::ClearNote { id });
            }
        }
    });
}

/// **The reply editor** — the same box, pointed at `add_reply`.
fn reply_editor(
    ui: &mut egui::Ui,
    comment: &CommentRow,
    parent: ObjId,
    draft: &mut NoteDraft,
    sink: &mut RowSink<'_>,
) {
    ui.label(
        egui::RichText::new(t::comment_row_reply_hint())
            .small()
            .weak(),
    );
    ui.label(
        egui::RichText::new(t::comment_row_reply_signature())
            .small()
            .weak(),
    );
    if matches!(comment.relation, Some(Relation::Reply)) {
        ui.label(
            egui::RichText::new(t::comment_row_reply_to_a_reply())
                .small()
                .weak(),
        );
    }

    // Asked BEFORE the horizontal layout borrows the draft, because the
    // control's very existence depends on the answer and a borrow taken for
    // the button would outlive the question.
    let postable = reply_is_postable(draft.text());
    ui.horizontal(|ui| {
        // R83 at its smallest: a blank reply is one this shell has decided not
        // to author, so the control that would author it is **not drawn**
        // rather than drawn and refused. Not greyed either — greying is for
        // temporarily unavailable, and this genuinely is temporary, but the
        // remedy is "type something" and the box is directly above with a
        // caret in it. A greyed button beside an empty box explains nothing
        // the box does not already say.
        if postable {
            *sink.writing_controls_drawn += 1;
            let post = ui.button(t::comment_row_reply_save());
            crate::diag::ui_rect_visible(REGION_POST, post.rect, ui.clip_rect());
            if post.clicked() {
                *sink.verb = Some(AnnotAction::Reply {
                    parent,
                    text: draft.text().to_owned(),
                });
            }
        }
        // Cancel is drawn either way, because an operator who opened the
        // editor by accident must be able to close it without typing into it
        // first, and because it is the **only** discard: Escape posts what is
        // there, per [`escape_commits`].
        if ui.button(t::comment_row_note_cancel()).clicked() {
            draft.close();
        }
    });
}
