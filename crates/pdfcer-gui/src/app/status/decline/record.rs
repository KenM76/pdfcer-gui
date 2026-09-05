//! # `app::status::decline::record` — every writer of the decline slot
//!
//! ★★★ **The recording half of [`super`], moved here on 2026-09-05 under R2.**
//! That file stood at 1,497 lines against the 1,500-line ceiling — three lines
//! of headroom — and the clipboard's mode refusal needed a variant, two match
//! arms and their reasons.
//!
//! ## Why this is the seam, and not a size-driven cut anywhere convenient
//!
//! [`super`] answers two questions that share a store and nothing else:
//!
//! | question | where it is now answered |
//! |---|---|
//! | *what is a decline, and how long does it owe its sentence?* | [`super`] — the [`Declined`] enum, `still_true`, `line`, `retire`, `live`, `show` |
//! | *who says one, and from which phase?* | **here** — one constructor per source of truth |
//!
//! The second is a family of twenty-odd functions whose whole content is a
//! one-line write plus the argument for **where it is called from** — dispatch
//! or apply, before the verb or inside it — and that argument is the same
//! argument twenty times over. It reads better as a file about recording than
//! as a tail on a file about retirement.
//!
//! ★ It is the **third** split from [`super`], after `decline/floor.rs` and
//! `decline/textedit.rs`, and both of those are recorders too — so this move
//! makes the arrangement regular rather than inventing one. Those two keep
//! their own files because each carries an argument of its own (the floor's
//! ordering rule; the text caret's channel argument) that would be buried in a
//! file of twenty siblings.
//!
//! ## ★ Re-exported, so no call site moved
//!
//! [`super`] carries `pub(crate) use record::*;`. Every `decline::record_*`
//! call in the crate resolves exactly as it did, which is what makes this a
//! move rather than a refactor: a split that also renames its callers cannot be
//! reviewed as a split.
//!
//! **Written unconditionally, overwriting whatever was live**, everywhere in
//! this file. `LAST` is a slot rather than a queue precisely so the most recent
//! answer is the visible one — see [`super::record_text_style`]'s rule, and
//! `decline/textedit.rs`'s restatement of why it matters for a control an
//! operator presses twice.

use super::{Declined, LAST};
use crate::canvas::zoom::ZoomOutcome;

/// Record what a framing zoom did, so the bar can say so if it declined.
///
/// **Written unconditionally**, including for a grant, which stores `None`.
/// That mirrors `crate::app::actions::record_edit_disclosure`'s discipline:
/// the slot never holds a sentence whose only defence against being shown is a
/// filter somewhere else. A zoom that *worked* silences a decline immediately,
/// on the same call, rather than relying on [`retire`] having been reached.
///
/// Called from `crate::app::dispatch`'s `view.zoom_selection` arm — which is
/// still routing rather than computing: it hands over the value the verb
/// returned and decides nothing about it.
pub(crate) fn record(outcome: ZoomOutcome) {
    let declined = Declined::of(outcome);
    LAST.with_borrow_mut(|slot| *slot = declined);
}

/// Record that `file.save_copy` was given a destination and produced no file.
///
/// Called from `crate::app::save::write_and_report`, which is in the **apply**
/// phase rather than in the dispatcher — the one difference from [`record`]'s
/// call site, and it is why this is a separate entry point rather than a second
/// argument to that one. [`retire`] runs at the top of `dispatch_command`, so a
/// sentence recorded during the apply of the *same* frame survives it: the
/// order is dispatch (retire, raise the action) → apply (write, record) → next
/// frame (the bar draws it) → the operator's next command (retire).
///
/// Unconditional, and there is deliberately no matching "the save worked" call
/// that stores `None`. A successful save-a-copy produces a file at a path the
/// operator typed into a dialog they were looking at, which is the most visible
/// confirmation this application has; adding a sentence for it would narrate
/// what they just did. Two saves in a row, one failing and one succeeding, are
/// still handled — the second press retires the first's sentence through
/// [`retire`] before its own arm runs.
/// Record that a push button was asked for and pdfcer cannot make a working one.
///
/// Called from `crate::app::dispatch`'s form arm, in the **dispatch** phase
/// like [`record`] rather than in apply — the command never reaches a document,
/// so there is no apply to decline in.
///
/// ★ It exists as its own function rather than joining [`record`] for the
/// reason that one's neighbours already show: [`record`] converts a
/// `ZoomOutcome`, and this has no outcome to convert. A constructor per source
/// of truth is what keeps the enum from acquiring a `From` impl for every type
/// in the crate.
/// Record that a restyle of existing text refused, and why.
///
/// Called from `crate::app::actions::textstyle`, in the **apply** phase like
/// [`record_save_failure`] rather than in the dispatcher — the refusal comes
/// from the engine, which is only reached once the action is being applied.
///
/// ★ A constructor of its own rather than a second argument to [`record`], for
/// the reason [`record_push_button_inert`]'s docs give: [`record`] converts a
/// `ZoomOutcome` and this has no outcome to convert. One constructor per source
/// of truth is what keeps this enum from acquiring a `From` impl for every type
/// in the crate.
pub(crate) fn record_text_style(why: crate::text::status::TextStyleRefusal) {
    LAST.with_borrow_mut(|slot| *slot = Some(Declined::TextStyle(why)));
}

/// Record that a flatten was refused by the document's certification.
///
/// Called from the dispatch arm rather than from the apply phase, unlike
/// [`record_save_failure`], because the refusal is knowable **before** the
/// action is raised — `flatten_refusal` is a query — and raising an action the
/// apply arm would then have to refuse would put the same rule in two places.
pub(crate) fn record_flatten_certified() {
    LAST.with_borrow_mut(|slot| *slot = Some(Declined::FlattenCertified));
}

/// Record that a **form-field or widget delete** was refused by the document's
/// structure gate.
///
/// Called from `crate::app::actions::forms::delete`, in the **apply** phase —
/// which is unusual for a refusal that is knowable from a query, and is the
/// honest place for it here. [`record_flatten_certified`] is called from the
/// dispatch arm precisely so that no action is raised for a refusal already
/// known; these two verbs have **four** doors instead of one, every one of
/// which already asks `formfield::refuses_delete` before offering anything.
/// What is left for the verb to say is the residue those four cannot cover — a
/// chord, a stale frame, an engine guard the query does not forecast, and a
/// delete arriving with no field selected — and the verb is the one place all
/// of it passes through.
///
/// ★★★ Without it that residue was a **silence**, because
/// `crate::app::actions::apply::vector_edit`'s `Err` arm wrote a trace line and
/// said nothing to the operator by its own recorded decision. It now words
/// [`Declined::EditRefused`] (O116), which is a floor rather than a
/// replacement: it cannot say *form fields* or *certification*. R83's rule is
/// not *gate the controls*; it is *a refusal must be a sentence*. See
/// [`Declined::FieldDeleteRefused`].
/// Record that a field-group deletion **preview** was refused.
///
/// ★★★ These two replace `record_note` calls, and the swap is the point.
/// `record_note` renders under **`⚑ About your last edit:`**, which
/// `crate::text::status`' own rule forbids for a decline — *"an operator who
/// reads 'About your last edit' after a gesture that did nothing has been told
/// a small lie confidently."* Nothing happened; the slot that says so is this
/// one, and it wears `⊗`.
///
/// The sibling verb in the same commit — `unshare_form` — used the right
/// channel from the start, which is what made the mismatch findable: two verbs
/// shipped together, one wording its refusal as a disclosure and one not.
pub(crate) fn record_field_group_preview_refused() {
    LAST.with_borrow_mut(|slot| *slot = Some(Declined::FieldGroupPreviewRefused));
}

/// Record that a field-group deletion was refused after confirmation.
/// See [`record_field_group_preview_refused`].
pub(crate) fn record_field_group_delete_refused() {
    LAST.with_borrow_mut(|slot| *slot = Some(Declined::FieldGroupDeleteRefused));
}

/// Record that the **Points tool** was asked for in a mode that cannot change
/// page content — [`Declined::NodeToolNeedsEditMode`].
///
/// Called from `PdfcerApp::dispatch_command`'s `view.tool_node` arm, beside the
/// trace it already wrote. The trace stays: it names the id and the reason for
/// a reader of a machine they cannot see, and this names the remedy for the
/// operator in front of it. Two audiences, two lines, one event.
pub(crate) fn record_node_tool_needs_edit_mode() {
    LAST.with_borrow_mut(|slot| *slot = Some(Declined::NodeToolNeedsEditMode));
}

/// Record that a **corner of a ce dimension** could not be added or taken
/// away — [`Declined::VertexEditRefused`].
///
/// Called from `canvas::dimdrag::count_edit` on the release frame of a
/// count-editing drag whose preflight refused, beside the trace it writes on
/// the same frame. Two audiences, two lines, one event — the split
/// [`record_node_tool_needs_edit_mode`] states.
///
/// ★ This is the **only** report of that refusal. The gesture preflights, so
/// no action is raised, no funnel is entered and no `EditRefused` is recorded;
/// if this call is removed the operator gets a drag that does nothing and says
/// nothing, which is the exact defect the variant exists for.
pub(crate) fn record_vertex_edit_refused(why: crate::text::measure::VertexEditRefusal) {
    LAST.with_borrow_mut(|slot| *slot = Some(Declined::VertexEditRefused(why)));
}

pub(crate) fn record_field_delete_refused() {
    LAST.with_borrow_mut(|slot| *slot = Some(Declined::FieldDeleteRefused));
}

/// Record that a **bookmark move** did not happen, and which of the two
/// sentences it owes.
///
/// ★★★ The whole point of the function, stated for whoever adds the next
/// refusal to this gesture: **a refusal must be a sentence, never a silence.**
/// A drag that is released and does nothing is this project's founding defect
/// shape, and a bookmark drag is the worst instance of it — the row leaves the
/// operator's pointer during the gesture, so a silence reads as *"it moved and
/// I cannot find it"*, which is a state this very feature can genuinely
/// produce.
///
/// ★★ Called from **inside** the `vector_edit` closure, one position rather
/// than [`record_rotate`]'s two, and the difference is worth stating: the
/// shell-side condition that gesture words from the canvas — *"this landing is
/// inside the thing you are dragging"* — is one the **engine** also refuses by
/// name, so there is nothing left for the panel to say and nothing gained by a
/// second door. The panel's forecast of it drives the caret and stops there.
/// See [`Declined::BookmarkMoveIntoOwnSubtree`].
///
/// `own_subtree` chooses between the two variants. A `bool` rather than the
/// `EditError` itself, so this module stays free of the engine's error type —
/// the same shape [`record_resize_not_rebuildable`] takes, and for the same
/// reason: what reaches the bar is a sentence, and which sentence is the only
/// fact that has to cross the boundary.
pub(crate) fn record_bookmark_move_refused(own_subtree: bool) {
    LAST.with_borrow_mut(|slot| {
        *slot = Some(if own_subtree {
            Declined::BookmarkMoveIntoOwnSubtree
        } else {
            Declined::BookmarkMoveRefused
        });
    });
}

/// Record that a resize was refused because the artwork cannot be rebuilt.
///
/// ★★ Called from **inside** the `vector_edit` closure, which is unusual and is
/// the honest place for it: whether an appearance is pdfcer's own is not
/// knowable before the call, so — unlike [`record_flatten_certified`], whose
/// refusal is a query — this one can only be recognised from the error the verb
/// returns. `record_save_failure` is called from the apply phase for the same
/// reason.
pub(crate) fn record_resize_not_rebuildable(uniform: bool) {
    LAST.with_borrow_mut(|slot| *slot = Some(Declined::ResizeNotRebuildable { uniform }));
}

/// Record that a **rotation** did not happen, and why.
///
/// ★★★ The whole point of the function, stated for whoever adds the next
/// refusal to `RotateRefusal`: **a refusal must be a sentence, never a
/// silence.** A rotate handle that is dragged, released, and does nothing with
/// no explanation is this project's founding defect shape, and it is exactly
/// what the eight resize grips did for the whole life of this shell.
///
/// ★★ Called from **two positions**, deliberately, and the split is the same
/// one this module already draws twice:
///
/// | caller | when | precedent |
/// |---|---|---|
/// | `canvas::rotating` | before any verb, for `NoDimensionRecord` — a condition the shell can **query** | [`record_flatten_certified`] |
/// | `app::actions::annots` | inside the `vector_edit` closure, for what the **engine** returns | [`record_resize_not_rebuildable`] |
///
/// A shell-side condition raised from the apply phase would put the same rule
/// in two places; an engine-side one raised from the dispatcher would be a
/// guess about a call that has not happened yet.
pub(crate) fn record_rotate(why: crate::text::rotating::RotateRefusal) {
    LAST.with_borrow_mut(|slot| *slot = Some(Declined::Rotate(why)));
}

/// Record that *"give this page its own copy"* did not happen.
///
/// ★★ Called from **three positions**, extending [`record_rotate`]'s split:
/// `app::dispatch::format` records `NothingInAForm` from the **selection**
/// ([`record_inside_form`]'s placement); `app::actions::xobject::fanout`
/// records `NotShared` from the **document**, after one page walk on the press
/// and before `vector_edit`; and `xobject::unshare` records what the **engine**
/// returns from inside the closure ([`record_resize_not_rebuildable`]'s).
///
/// ★★★ **There is deliberately no matching "it worked" call**, unlike
/// [`record`], which writes `None` on a grant. This verb's success is narrated
/// instead — `crate::text::unshare::unshared`, carried out through
/// `vector_edit`'s **disclosure** list rather than through this store, because
/// a disclosure and a decline are different speech acts and this module's
/// header forbids sharing a slot between them.
pub(crate) fn record_unshare(why: crate::text::unshare::UnshareRefusal) {
    LAST.with_borrow_mut(|slot| *slot = Some(Declined::Unshare(why)));
}

pub(crate) fn record_save_failure() {
    LAST.with_borrow_mut(|slot| *slot = Some(Declined::SaveFailed));
}

/// Record that the Settings window's Save reached no disk.
///
/// Called from `crate::app::settings_window::save_settings`, and **only** on
/// the failure path — there is deliberately no matching "the settings saved"
/// call. A successful settings save is not narrated for the same reason a
/// successful save-a-copy is not: the operator pressed a button in a window
/// they were looking at, the window closed, and a sentence telling them so
/// would narrate what they just did.
///
/// # ★ What must be true at the call site before this is reached
///
/// The configuration has **already been adopted**. That ordering is the whole
/// meaning of [`Declined::SettingsNotSaved`]'s sentence, and calling this
/// before the adoption — or instead of it — would make the sentence a lie in
/// the more damaging direction: the operator would be told their choice is
/// in force for this session when it is not.
pub(crate) fn record_settings_not_saved() {
    LAST.with_borrow_mut(|slot| *slot = Some(Declined::SettingsNotSaved));
}

/// Record that `edit.undo` or `edit.redo` arrived with an empty stack.
///
/// Called from `crate::app::actions::apply`'s history arm — the **apply**
/// phase, exactly as [`record_save_failure`] is, and for the same reason: the
/// arm that can tell is the one holding the session, and [`retire`] runs at the
/// top of `dispatch_command`, so a sentence recorded during the apply of the
/// same frame survives it.
///
/// # ★ Why the dispatcher does not decide this
///
/// It could: `PdfcerApp` has the session, and `view.zoom_selection` sets the
/// precedent of a dispatch arm recording an outcome. It must not, because the
/// dispatcher's arms **route** (`HANDOFF.md` §6), and "is there anything to
/// undo?" is a question about the document that the apply phase has to ask
/// anyway before it touches the session. Asking it in both places is how the
/// greyed control and the sentence come to disagree.
///
/// **`Declined` is the parameter rather than a `bool`**, so the call site reads
/// as the state it is reporting and a third stack would not silently become a
/// fourth meaning of `true`. Only the two history variants are constructible
/// here in practice; passing anything else records a decline the arm did not
/// mean, which is why the two callers are one arm apart in one function.
pub(crate) fn record_history_empty(declined: Declined) {
    LAST.with_borrow_mut(|slot| *slot = Some(declined));
}

/// Record that `adopt_widget` refused, and which of its two correctable
/// refusals it was.
///
/// Called from `crate::app::actions::forms`, the apply phase, exactly as
/// [`record_history_empty`] is and for the same reason: the arm holding the
/// session is the one that can tell.
///
/// # ★ Why only two of the engine's five refusals reach here
///
/// `adopt_widget` refuses five ways. Three of them cannot happen from this
/// surface and wording them would be wording states the operator cannot be in:
///
/// | refusal | why it is unreachable here |
/// |---|---|
/// | `NotAWidget` | the ids come from `page_annotations(..).is_widget()`, in this document |
/// | `WidgetAlreadyOwned` | the ids are exactly the ones no field claimed, from the same walk |
/// | `FieldNameEmpty` | the box is trimmed and an empty one sends `None`, not `Some("")` |
///
/// They still reach the trace through [`super::super::actions::apply::vector_edit`]'s
/// error branch, which is where an impossible refusal's *reason* belongs:
/// visible to whoever is debugging, and never on the status bar in the engine's
/// own words.
///
/// ★ **Corrected 2026-09-04 (O116):** this used to end *"absent from the status
/// bar an operator reads"*, and that is no longer true. Those three now reach
/// the bar as [`Declined::EditRefused`] — *"That change was refused, and the
/// document is unchanged."* — because the funnel's floor words every refusal no
/// verb claimed. That is the right outcome rather than a leak: an unreachable
/// refusal that somehow happened is still a gesture that did nothing, and the
/// operator is owed a sentence about it. What stays off the bar is the engine's
/// *prose*, which was always the property this paragraph was defending.
pub(crate) fn record_adopt_refusal(declined: Declined) {
    LAST.with_borrow_mut(|slot| *slot = Some(declined));
}
