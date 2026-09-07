//! # `canvas::notepopup::controls` — everything in the window that CHANGES
//! # something
//!
//! *Add note* / *Edit note*, *Save*, *Cancel*, *Remove note*, *Delete
//! comment*, and **Open by default** — plus the three sentences that stand in
//! their place when the operator's stance or the file forbids them. Split out
//! of [`super`] under **R2** on 2026-09-06, when the `/Open` write-back took
//! that file to 1,639 lines against a ceiling of 1,500.
//!
//! ## ★★ Why this is the seam
//!
//! It is the same one `crate::panels::comments::editor` took the same day, and
//! for the same reason: the rest of the pop-up **reads** — a heading, a byline,
//! the words, the thread, a hover tooltip, a placement calculation — and this
//! is the only part of it that produces an `Action`. `super::body` decides what
//! the window *says*; this decides what it can *do*.
//!
//! ⇒ That line survives the next feature. A Reply control on this surface would
//! land here; a caption about a group subordinate would land next door; and the
//! question *"which file?"* has an answer that does not depend on how many
//! lines are left in either.
//!
//! ## ★★★ TWO controls write, and they write to different things
//!
//! This is the distinction the whole file is arranged around, and it is the one
//! an operator can most easily get wrong:
//!
//! | control | what it changes | undo entries |
//! |---|---|---|
//! | *Save note*, *Remove note*, *Delete comment* | the annotation | one each |
//! | **Open by default** | the annotation's `/Open` **and its `/Popup`'s**, as one command | one |
//! | the window's ✕ (in [`super::body`]) | `super::open`'s per-document override — the screen | **none** |
//!
//! The last row is the interesting one and [`open_default`]'s doc comment
//! carries its argument in full: reading a marked-up drawing *is* opening and
//! closing bubbles, so wiring that gesture to the document would fill a
//! reviewer's undo stack with entries that say *"looked at a comment"*.

use super::model::{self, NoteView};
use super::{Ctx, REGION_DELETE, REGION_EDIT, REGION_OPEN_DEFAULT, REGION_SAVE, open, t};
use crate::app::actions::Action;
use crate::app::actions::annot::AnnotAction;
use crate::panels::comments::note::NoteDraft;

/// The controls under the note: edit, save, remove, delete — or the sentence
/// that says why there are none.
///
/// # ★★★ Four states, and each is a fact about the document or the mode rather
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
                // ★★★ `keep_author` comes from the SAME function the Comments
                // panel uses, and that is not tidiness. `pdfcer-core` named
                // this mistake when it shipped the verb: *"an implementation
                // writing all three keys unconditionally would silently strip
                // the author and date on every correction, leaving a review
                // comment from nobody, dated never."* Two editors for one
                // note, two spellings of the rule, and one of them eventually
                // gets it wrong — so there is one spelling.
                actions.push(Action::Annot(AnnotAction::SetNote {
                    id: note.id,
                    text: draft.text().to_owned(),
                    keep_author: crate::panels::comments::keeps_author_name(note.author.as_deref()),
                }));
                draft.close();
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

/// ★★★ **Record this comment's window state in the FILE** —
/// `EditSession::set_annotation_open`, `pdfcer-core` `Pass 253.3`.
///
/// # ★★★ THE UNDO DECISION, and it is the whole reason this is a separate
/// control
///
/// `/Open` is a key in the document, so writing it is a document change and
/// lands in the undo log like every other edit — that part was never in
/// question. What was in question is **which gesture writes it**, and the
/// choice made here is *only an explicit act*:
///
/// | gesture | what moves | undo entries |
/// |---|---|---|
/// | clicking a note on the page | [`open`]'s per-document override — interface state | none |
/// | pressing this window's ✕ | the same override | none |
/// | ticking **Open by default** | the document's `/Open`, on the note and on its `/Popup`, in one command | **one** |
///
/// ## Why not "write on every open and close"
///
/// Because reading a marked-up drawing *is* opening and closing bubbles, dozens
/// of times, and none of it is an edit the operator would recognise as one. Wire
/// it and a reviewer who glanced at six comments ends the session with six undo
/// entries between `Ctrl+Z` and the last thing they actually changed, and a
/// document that reports itself modified after a sitting in which they altered
/// nothing. The Save dialog would then ask them to write a file they did not
/// mean to change — and they would say yes, because they have no way to know
/// what the six entries were.
///
/// ## Why not coalesce
///
/// It was the third candidate and it is the worst of the three, because it
/// makes the undo log's contents depend on **timing**. Two presses inside the
/// window are one entry and two presses either side of it are two, so `Ctrl+Z`
/// stops meaning anything an operator can predict — and the failure is
/// invisible: nothing on screen says which side of the boundary a press landed
/// on. A rule an operator cannot see is a rule they cannot rely on.
///
/// ## Why not "accept the entries as honest"
///
/// It is the most defensible of the three rejected options and it was rejected
/// on a measurement rather than a preference: the entries would be honest and
/// they would be **overwhelming**. The undo stack's job is to let an operator
/// walk back their own work, and a log in which nineteen of twenty entries are
/// *"looked at a comment"* does not do that job however true each line is.
///
/// ⇒ `crate::canvas::markup::swatch`'s header argues the same shape from the
/// other side and reaches the opposite conclusion for its own case — a pen
/// colour *"has no undo, it raises no `Action`"* because it touches no
/// document. The rule both obey: **the undo log records changes to the
/// document, and only the ones the operator meant as changes.** A pen colour
/// fails the first test; opening a bubble to read it fails the second.
///
/// # ★★ Why the on-screen state is pinned when the document's is written
///
/// The override is set to the window's **current** state — open, because this
/// control is only drawn inside an open window — at the same moment the
/// document is written. Without that, unticking *Open by default* would make
/// `authored_open` false, the override would still be unset, and the window the
/// operator is looking at would **vanish under their hand** on the next frame.
/// Recording a preference for the next reader is not a request to close the
/// thing you are reading.
///
/// # R83: not drawn when there is nowhere to write it
///
/// [`model::can_record_open_state`] carries the table. The short form: Table
/// 169 gives `/Open` to `/Text` and to `/Popup` and to nothing else, and the
/// engine will not manufacture a companion, so a `/Square` with no `/Popup` in
/// the file has no window state to express. The engine reports that case as a
/// **no-op rather than a refusal** — deliberately, so a caller over a mixed
/// selection need not filter by subtype — which makes the affordance this
/// shell's to withhold. `crate::text::annotpopup::open_state_written` words it
/// anyway, for the case where this gate and the engine ever disagree.
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
        // ★ Pin the window open FIRST, before the action is queued. The action
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
///
/// # ★★★ The Comments panel's *"this build has no Delete"* was true and is not
///
/// That paragraph was written on 2026-08-14 and its stated reason —
/// *"`crate::app::actions::Action` has no variant that could carry the
/// intent"* — stopped being true when `AnnotAction::Delete` landed. It is
/// corrected in place, dated, in that module's own header, along with the
/// `/TrapNet` reasoning that depended on it. **A limitation sentence is a
/// citation with an hours-long shelf life**, and this project has now paid for
/// that lesson six times.
///
/// # R83: the control is omitted when the engine would refuse
///
/// `EditSession::annotation_deletion_refusal` answers *"would
/// `delete_annotation` refuse right now?"* for the two document-wide reasons —
/// encryption and a certification signature — and asking it is what lets this
/// draw nothing instead of offering a button whose only outcome is a worded
/// decline. The per-annotation refusals (a locked annotation, a ce dimension)
/// are handled above by the same rule.
///
/// ⚠ It is **not a perfect oracle**, and `docs/core-api/03-capabilities.md`
/// §3.4 says so: the real call can still refuse. That is why the funnel's
/// worded decline stays the answer of record and this is only a filter on the
/// affordance.
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
