//! # `panels::comments::editor` — everything on a row that WRITES
//!
//! The *Add note* / *Edit note* control, the *Reply* control, and the one text
//! box both of them open. Split out of [`super`] under **R2** on 2026-09-06,
//! when the reply affordance took that file to 1,757 lines against a ceiling
//! of 1,500.
//!
//! ## ★★ Why this is the seam, and not "split the rows from the strip"
//!
//! R2's rule is *find the seam*, and the file genuinely comes apart here.
//! [`super::body`], [`super::row`], [`super::delete_control`] and
//! [`super::filter_strip`] are all **the list**: they decide what is shown and
//! in what order, and every one of them is a pure function of the document plus
//! a filter. What is in this file is the only part of the panel that holds the
//! operator's own half-finished work — a [`super::note::NoteDraft`] — and the
//! only part whose output is a **verb**.
//!
//! ⇒ Which is why the split survives the next feature rather than needing to be
//! redone: a control that writes will land here, a caption that describes will
//! land next door, and the question *"which one is this?"* has an answer that
//! does not depend on how many lines are left in either file.
//!
//! ★ The same seam `super::tests` took the day before, one step further along:
//! that file is *what is asserted about the panel*, this one is *what the panel
//! can change*, and [`super`] is left as the list itself.
//!
//! ## ★★★ TWO destinations, ONE box — the thing this file exists to keep true
//!
//! A note edit and a reply are the same text box pointed at different engine
//! verbs: `set_markup_note` edits a dictionary that already exists, `add_reply`
//! **creates an annotation**. Everything they share — the box, its trace
//! region, the Escape route, the stale-draft rule — is written once in
//! [`editor`], and [`super::note::DraftTarget`] is asked exactly once, at the
//! point where the two genuinely differ.
//!
//! ⚠ The failure this shape is defending against is specific and silent: a box
//! captioned *Post reply* whose commit writes `/Contents`, which puts a
//! reviewer's answer **over the comment they were answering** and looks, on
//! screen, exactly like the reply having worked. Nothing but reading the file
//! afterwards distinguishes the two, which is why the destination lives on the
//! draft's stamp and is compared rather than inferred.
//!
//! ## What is deliberately NOT here
//!
//! **Delete.** `super::delete_control` writes to the document too, and it stays
//! next door because it is drawn on the row's *navigation* line beside *Go to*
//! rather than in the editor block, and because it holds no draft. The line
//! this file draws is not "does it change the document" — it is "does it hold
//! the operator's unfinished words".

use pdfcer_core::object::ObjId;

use super::note::{DraftTarget, NoteDraft};
use super::{
    CommentRow, Note, REGION_BOX, REGION_EDIT, REGION_POST, REGION_REMOVE, REGION_REPLY,
    REGION_SAVE, Relation, RowSink,
};
use crate::app::actions::annot::AnnotAction;
use crate::text::panels::comments as t;

/// **The note editor for one row, and the control that opens it.**
///
/// Three shapes, decided by what the annotation is rather than by what this
/// build can do:
///
/// | the row | what is drawn |
/// |---|---|
/// | a **ce dimension** | a caption saying where its text actually comes from |
/// | an annotation with **no object id** | a caption saying why pdfcer cannot address it |
/// | anything else | *Add note* / *Edit note*, and the editor when it is open |
///
/// # ★★★ R9: neither caption is a greyed button
///
/// *"An unavailable capability renders nothing, not a disabled stub. Greying is
/// reserved for temporarily unavailable."* Neither of these is temporary: a ce
/// dimension's `/Contents` is regenerated from its measurement by
/// `author_dimension`, so a note written over it would be silently thrown away,
/// and a direct-dictionary annotation is a **malformed file** (§12.5.2 Table 164
/// requires the dictionary to be an indirect object) with nothing to name. A
/// greyed *Edit note* would promise that some state of the program would let
/// the operator press it, and none would.
///
/// # ★★ Why a `/Link` is offered the editor
///
/// `/Contents` is dual-purpose (§12.5.2): note text on a subtype that displays
/// text, an accessibility description on one that does not — and
/// [`t::comment_row_description_caption`] already says which this row is.
/// `set_markup_note` accepts both, so withholding the editor would be
/// withholding a capability the engine has, on a guess about the operator's
/// intent. The caption is the honest half; the button is the useful half.
///
/// ★ It is worth knowing what this costs: on a `/Link` with no `/Contents` at
/// all the control still says *Add note*, because `Note::Absent` carries no
/// subtype interpretation to distinguish "nobody wrote a comment" from "nobody
/// wrote a description". Named here rather than left to be found.
pub(super) fn note_controls(
    ui: &mut egui::Ui,
    comment: &CommentRow,
    draft: &mut NoteDraft,
    epoch: u64,
    sink: &mut RowSink<'_>,
) {
    // ★★★ **Nothing to type into in a reading stance.** Same finding, same
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

    // The existing words, which seed the editor. `Note::Description` seeds it
    // too — the operator is editing that string whichever of §12.5.2's two
    // meanings it carries, and an editor that opened empty over a description
    // would invite them to destroy it by typing.
    let existing = match &comment.note {
        Note::Text(text) | Note::Description(text) => text.as_str(),
        Note::Absent => "",
    };
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

/// ★★★ **Answer this comment** — `EditSession::add_reply`, `Pass 253.0`.
///
/// # The gap this closes, in this shell's own words
///
/// 2026-09-05, filed and carried open until the verb landed: *"we can read a
/// comment thread and cannot add to it."* The **read half already worked** —
/// this panel's trace has printed `replies=` since the day it was written, and
/// `crate::canvas::notepopup::thread` draws the conversation inside a comment's
/// window. What was missing was a destination, and this is it.
///
/// # ★★ Beside *Add note*, not under it, and the pairing is the explanation
///
/// The two controls do the two things a reviewer does to a comment: **change
/// what it says** and **say something back**. Putting them on one row makes the
/// choice legible at the moment it is made, which matters here more than it
/// usually would because the two outcomes look almost identical afterwards —
/// new words appear near the same annotation either way, and only the file
/// records which happened. `crate::text::panels::comments::comment_row_reply_tooltip`
/// carries the sentence that says so.
///
/// # Why it is offered on EVERY row that takes a note editor
///
/// Including rows that are themselves replies, and including rows with no
/// `/Contents` at all. §12.5.6.2 puts no constraint on what may be replied to,
/// `add_reply` refuses only a reply to *itself*, and a shape somebody drew
/// without a comment is exactly the thing a second reviewer most often wants
/// to ask about. The one exclusion is inherited rather than added: the caller
/// has already returned for a ce dimension and for a row with no object id, so
/// this is only ever reached for an annotation the engine can name.
///
/// ★ **The parent is the ROW's own annotation**, never the thread root. The
/// file therefore records the real depth of the conversation even though both
/// of this shell's surfaces draw it flat — see [`body`]'s threading paragraph.
/// A shell that rewrote the parent to the root on the way in would be
/// destroying a fact about who answered whom, permanently, to make its own
/// display arithmetic simpler.
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

/// ★★★ **Whether there is anything to post** — the shell's own R83 guard, and
/// the reason it cannot be delegated.
///
/// `add_reply`'s doc comment lists `EditError::MarkupNoteEmpty` among its
/// errors and **that variant does not exist**: measured 2026-09-06 against the
/// pinned engine at `d2ea5de`, the identifier occurs once in the whole crate
/// and the occurrence is that doc line. `MarkupNote::validate`
/// (`edit.rs:4731`) checks the `/M` date's §7.9.4 shape and nothing else, so an
/// empty reply is **authored rather than refused**.
///
/// ⇒ So the decision is this shell's, and it is *not* the one the note editor
/// makes. An empty `/Contents` on a sticky note is ordinary — it is what the
/// operator has just placed and is about to write in — and
/// `AnnotAction::SetNote` permits it by name. An empty **reply** is a new
/// annotation in somebody else's thread that says nothing, that this panel
/// offers no later way to give words to, and that reads to the next reviewer
/// as a defect rather than as a remark.
///
/// ★ **Trimmed**, for [`keeps_author_name`]'s reason one screen down: a reply
/// of `"   "` renders in every surface exactly as an empty one does, and a
/// guard that let it through would be a guard that only stopped the operator
/// who pressed Post with the cursor at position zero.
#[must_use]
pub(crate) fn reply_is_postable(text: &str) -> bool {
    !text.trim().is_empty()
}

/// ★★★ **Whether this annotation already carries a byline that is not ours to
/// move** — the one decision in this panel with a consequence in the file.
///
/// `true` means the `SetNote` action sends **no `/T` at all**, and
/// `pdfcer-core` leaves an omitted key untouched. `false` means the operator's
/// name from Settings > Comments is written, or nothing is if that name is
/// blank, which is a supported choice meaning *comment anonymously*.
///
/// # Why this is a function rather than three words at its call site
///
/// Because it is the mistake the engine warned about **by name** when it
/// shipped the verb, and it is invisible from every other angle:
///
/// > An implementation writing all three keys unconditionally would silently
/// > strip the author and date on every correction, leaving a review comment
/// > from nobody, dated never, looking exactly like a note somebody else had
/// > mangled.
///
/// A `Ui` cannot be driven in a unit test in this crate, so an expression
/// buried in [`editor`] would be reachable only by `tools/ui-verify` — and a
/// driven check can assert that *a* note was written far more easily than it
/// can assert that a `/T` was **not**. Pulled out, the rule has a name, a
/// suite, and one caller that also feeds the sentence the operator reads.
///
/// # ★ Whitespace counts as absent
///
/// A `/T` of `"  "` is a byline nobody wrote — the commonest way for one to
/// exist is a producer writing an empty string — and preserving it would leave
/// a comment credited to a space. Trimmed, so *"has an author"* means the same
/// thing here as it does in the row's own byline, which is drawn by
/// [`t::comment_row_byline`] under the same rule.
pub(super) fn keeps_author(comment: &CommentRow) -> bool {
    keeps_author_name(comment.author.as_deref())
}

/// [`keeps_author`] over the name alone — **the one spelling of the rule**.
///
/// # ★★★ Why this is separate, added 2026-09-05
///
/// Because there are now **two** editors for one note: this panel's, and the
/// canvas pop-up's (`crate::canvas::notepopup`), which is the route that works
/// in Read mode and the answer to the operator's report of that date.
///
/// Two editors writing the same key is exactly the shape in which the mistake
/// `pdfcer-core` named by name gets made in one of them and not the other:
///
/// > An implementation writing all three keys unconditionally would silently
/// > strip the author and date on every correction, leaving a review comment
/// > from nobody, dated never, looking exactly like a note somebody else had
/// > mangled.
///
/// The pop-up has no [`CommentRow`] — it works from
/// `crate::canvas::notepopup::model::NoteView` — so the rule had to be
/// expressible over the name by itself or it would have been re-derived at the
/// second call site. Re-derived is how two surfaces come to disagree, and this
/// one's disagreement would be invisible until somebody read a saved file.
///
/// [`tests::a_note_with_an_author_keeps_it`] and its two siblings are the
/// suite, and they exercise this through [`keeps_author`].
#[must_use]
pub(crate) fn keeps_author_name(author: Option<&str>) -> bool {
    author.is_some_and(|author| !author.trim().is_empty())
}

/// The open editor: the box, the hint, the signature disclosure and the three
/// controls.
///
/// # ★★ The signature line is a rule-4 disclosure, not a caption
///
/// What `/T` will say is **invisible on the page** — a sticky's byline lives in
/// a pop-up window this shell does not draw, and a shape's lives nowhere at all
/// — so an operator has no way to discover what name their comments carry, or
/// that they carry none, or that editing somebody else's comment will leave
/// their name on it. Two sentences, one per case, and the case is decided by
/// the row rather than by a preference this panel cannot see.
///
/// # ★ Escape closes it, and it does so through egui rather than by reading the
/// keyboard
///
/// `TextEdit` surrenders focus on Escape, so `lost_focus()` plus the key is the
/// idiomatic test and — importantly for this codebase — it asks nothing about
/// whether "the operator is typing". A panel that read the raw key would be a
/// second claimant on a key the canvas caret and the tool arming both want, and
/// `tools/gates/check-typing-guard.sh` exists because that class of second
/// claimant has already cost this project the Delete key and the space bar.
fn editor(
    ui: &mut egui::Ui,
    comment: &CommentRow,
    id: ObjId,
    draft: &mut NoteDraft,
    sink: &mut RowSink<'_>,
) {
    // ★★★ **The open box is itself a writing control, and counting it closed a
    // hole in the instrument** — 2026-09-06.
    //
    // `CommentsUi::writing_controls_drawn` was incremented only by the
    // *buttons* that open an editor, never by the editor itself. So on a row
    // whose editor was already open the tally was blind to a live `TextEdit`
    // and a live *Save note* — and the panel's own R9 test, which drives the
    // real `body` in Read and asserts the tally is zero, **could not see the
    // editor path at all**.
    //
    // ⇒ It was found by falsifying the reply-editor test: the stance check was
    // deliberately moved after the draft branch — the exact 2026-09-05 defect
    // shape, a live writing control in Read — and the test **stayed green**.
    // A guard that cannot go red is not a guard. Same lesson as
    // `crate::app::modes`' deleted start-up assertion, one rung down: ★ a
    // negative assertion is vacuous when the thing that would produce the
    // positive is not counted.
    *sink.writing_controls_drawn += 1;
    let response = ui.add(
        egui::TextEdit::multiline(draft.text_mut())
            .desired_rows(3)
            .desired_width(f32::INFINITY),
    );
    crate::diag::ui_rect_visible(REGION_BOX, response.rect, ui.clip_rect());
    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
        draft.close();
        return;
    }

    // ★★★ **A `/FreeText` row's before-the-write warning was DELETED here on
    // 2026-09-06, hours after it was added, and the deletion is the record.**
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
    // ★★ And it could not be reworded to cover what survives. The one case
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

    // ★★★ **The destination decides everything below this line**, and it is
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

/// ★★★ **The reply editor** — the same box, pointed at `add_reply`.
///
/// Reached from [`editor`] when the draft's destination is
/// [`DraftTarget::Reply`], *after* that function has already drawn the text
/// box, published [`REGION_BOX`] and handled Escape. That ordering is the
/// whole reuse: the box, the region name, the abandon key and the
/// stale-draft rule are written once and cannot come to differ between
/// writing a note and answering one.
///
/// # ★★ What differs, and every difference is a fact rather than a style
///
/// | | note editor | this |
/// |---|---|---|
/// | the verb | `set_markup_note` — edits a dictionary that exists | `add_reply` — **creates an annotation** |
/// | the commit's label | *Save note* | *Post reply*, because they are different acts |
/// | the byline | `keep_author` decides; somebody else's `/T` may be preserved | always the operator's; there is no prior byline |
/// | *Remove note* | offered when there is one | never — there is nothing yet to remove |
/// | an empty box | permitted; an empty comment is a comment | **refused**, see [`reply_is_postable`] |
///
/// # ★ The threading disclosure, and why it is here rather than in a document
///
/// A row that is itself a reply gets [`t::comment_row_reply_to_a_reply`]. The
/// `/IRT` this writes is the **row's own** annotation, so the file keeps the
/// true depth of the conversation, while both of this shell's surfaces draw a
/// thread flat. That is a real difference between what is recorded and what is
/// shown, and rule 4 makes stating it mandatory — see [`body`]'s threading
/// paragraph for the argument, which is short enough to live at the code.
///
/// ⇒ Drawn only on a reply row, deliberately: on an ordinary comment the
/// sentence would be answering a question the operator has not raised, which
/// is the noise every other conditional caption in this panel exists to avoid.
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
        // first. Escape does the same thing and the hint above says so; the
        // button is for the hand already on the mouse.
        if ui.button(t::comment_row_note_cancel()).clicked() {
            draft.close();
        }
    });
}
