//! # `app::actions::reviewstate` — recording a comment's review status
//!
//! One verb: [`RecordStatus::record`], which is
//! [`pdfcer_core::edit::EditSession::add_review_state`] behind the edit funnel.
//! Raised by [`crate::app::actions::Action::RecordReviewState`], which is
//! raised by [`crate::panels::comments::reviewstate`]'s *Record status*
//! chooser, and by nothing else. [`RecordStatus`] is that action's payload, and
//! its doc carries why the verb is not an `AnnotAction`.
//!
//! ## ★★★ Why it is not in [`super::annots`]
//!
//! That module's own header draws the line it lives on: *"this file is what
//! happens to a thing that already exists"* — the verbs whose subject is an
//! annotation the operator can see, and which **change** it. Recording a status
//! changes nothing about the annotation it names. The engine says so in the
//! strongest terms available to it (`edit.rs:26804`):
//!
//! > *"★★ THE STATUS IS NOT WRITTEN ONTO THE ANNOTATION IT DESCRIBES —
//! > §12.5.6.3 puts it on a **separate** `/Text` annotation that points at the
//! > reviewed one through `/IRT` … That is why this verb returns a new `ObjId`
//! > rather than mutating the target, and why nothing about the target
//! > changes."*
//!
//! So this is an **addition**, and it belongs with the placement verbs by
//! subject even though it is reached from a list of existing ones. Its own file
//! rather than a third home, under R2 and for [`super::funnel`]'s reason: one
//! subject, one rate of change.
//!
//! ## ★★ What this module is careful NOT to do
//!
//! **It does not decide what the current status is.** The engine ships no
//! resolver, deliberately — *"it says nothing whatever about ordering or
//! currency … a `/M`-sorted resolver would be guessing"* — and neither does
//! this. `add_review_state` walks the `/IRT` chain itself and reports where it
//! attached; this module reports what it was told and adds nothing.
//!
//! **It does not draw anything on the canvas.** R8b: a review status is a
//! disclosure about a comment and lives off-canvas, in the panel and on the
//! status row. Nothing here touches an appearance.
//!
//! ## ★ Undo is one press, and the engine made sure of it
//!
//! A status is two dictionary keys on a new annotation, and the obvious
//! implementation writes the annotation and then the keys — two commands, two
//! undos, and a half-undone status that is a `/Text` with an empty
//! `/Contents` and no `/State`, which renders as an empty note nobody wrote.
//! `add_review_state` does not do that: it goes through `add_reply_with`'s
//! `extra` seam *"so the two keys land in the SAME command and one undo removes
//! the whole status"*, and the stack entry is
//! [`pdfcer_core::edit::CommandKind::AddReviewState`]. So this module needs no
//! grouping of its own, and the safety net under the *Record status* chooser is
//! the ordinary one press of Undo — which is why the control asks no
//! confirmation, exactly as `super::annots`' Delete does not.

use pdfcer_core::edit::ReviewState;
use pdfcer_core::object::ObjId;

use crate::app::state::OpenDoc;
use crate::text::reviewstate as t;

/// ★★★ **What [`crate::app::actions::Action::RecordReviewState`] carries** —
/// the comment to review, and the status to append.
///
/// # ★★ Why it is a payload here rather than fields on the `Action`
///
/// Two reasons, and the second is the load-bearing one.
///
/// 1. `action.rs` is at **R2's 1,500-line ceiling**, and this is the pattern
///    that file already uses eight times over — [`super::annot::AnnotAction`],
///    [`super::forms::FieldAction`], `VectorAction`, `WriteAction` — a family's
///    payload and its argument live with the module that applies it, and the
///    router carries one line.
/// 2. ★ The argument below is **about the engine's verb**, not about the action
///    bus. Written next to the call it constrains, it can be checked against
///    `add_review_state` by a reader who has that function open; written in the
///    router it would be prose about a function three files away.
///
/// # ★★★ Why this is NOT a [`super::annot::AnnotAction`]
///
/// That enum's stated family property is *"the verbs whose subject is a whole
/// annotation — move it, resize it, remove it"*, and every one of them
/// **changes the annotation it names**. This one changes nothing about it:
///
/// > *"§12.5.6.3 puts it on a **separate** `/Text` annotation that points at
/// > the reviewed one through `/IRT`, and says so with a `shall`. That is why
/// > this verb returns a new `ObjId` rather than mutating the target, and why
/// > **nothing about the target changes**."*
///
/// ⇒ [`Self::id`] is not the operand being altered; it is what the new
/// annotation will *point at*. Filing this under `AnnotAction` would put a verb
/// that ADDS an object into a family documented as verbs that MODIFY one, and
/// the next reader would have to discover the difference from the engine rather
/// than from the type.
///
/// # ★ It carries no page, and that is not an omission
///
/// `add_review_state` resolves the page itself — *"`/IRT` requires both
/// annotations on the same page, so the state is placed on the target's page"*
/// — and a page index carried from the panel would be a second answer to a
/// question the engine already answers, resolved at the press rather than at
/// apply time. The rule `AnnotAction::Move` follows by carrying a delta.
///
/// # ★ And no author, for the reason `AnnotAction::SetNote` gives
///
/// The name is a fact about **the operator**, which only [`super::apply`]'s
/// scope can see (`Prefs::author_name`); the panel knows only what the document
/// says. It is read at apply time, exactly as the note verbs read it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecordStatus {
    /// The comment being reviewed — the `/IRT` target, and the **root** of the
    /// chain rather than whatever the engine ends up attaching to. See
    /// [`pdfcer_core::edit::ReviewStateAdded::attached_to`]: a second status by
    /// the same author chains onto their first, and the engine does that walk.
    pub id: ObjId,
    /// The status to append — [`ReviewState`], the **closed** set pdfcer
    /// authors, as against the open set it reads.
    pub state: ReviewState,
}

impl RecordStatus {
    /// **Append this review status**, as one undoable command.
    ///
    /// `author` is `Prefs::author_name`, already trimmed by the caller — see
    /// [`RecordStatus`] on why the name travels from that scope rather than
    /// being read here.
    ///
    /// # ★★★ The two things the operator cannot see, and both are said
    ///
    /// A status is written onto a **new, invisible, empty-`/Contents`
    /// annotation**. Nothing about the page changes; nothing about the comment
    /// changes. So without a sentence on the status row the operator's evidence
    /// that anything happened at all is a blank row appearing in a list they may
    /// not be looking at.
    ///
    /// 1. [`crate::text::reviewstate::status_recorded`] names the state, and — from
    ///    the engine's own [`pdfcer_core::edit::ReviewStateAdded::chain_depth`] —
    ///    says how many statuses this author now holds on this comment. On the
    ///    **second** press it reads *"This is your 2 status on this comment — the
    ///    earlier ones are kept"*, which is the panel telling the truth about a log
    ///    at the exact moment the operator would otherwise assume they had
    ///    overwritten something.
    /// 2. [`crate::text::reviewstate::status_recorded_unsigned`] fires when the
    ///    operator has no name set. `add_review_state` uses the author as the
    ///    **chain key** (`deepest_state_for_author` matches `/T` equality), so an
    ///    empty one writes an empty `/T` that a later, named status **cannot
    ///    continue** — it will start a second history beside it. That consequence
    ///    is invisible until it has already happened twice, so it is stated when it
    ///    is caused, and it names the place to fix it.
    ///
    /// # ★★ `attached_to` and `chain_depth` on the diagnostic channel
    ///
    /// The engine exposes both precisely because the failure they guard is
    /// invisible on screen:
    ///
    /// > *"the wrong shape is invisible. A star of state annotations all pointing
    /// > at the target renders the same as a correct chain in every viewer, so
    /// > nothing would ever report it, and the history a reviewer's chain encodes
    /// > would simply not be there."*
    ///
    /// ⇒ Without `attached=` in the trace, a driven check could not distinguish a
    /// correct per-author chain from a star, on either a screenshot or the saved
    /// file's rendering. It is the same argument `markup_move`'s `keys=` makes for
    /// the half of a move a screenshot cannot see.
    ///
    /// # ★ The trace name takes a suffix; the funnel keeps the bare one
    ///
    /// `record-review-state-applied`, per `tools/gates/check-trace-names.py`: the
    /// funnel writes `record-review-state page=… n=… epoch=…` for the same edit,
    /// and a module line sharing that first token would be shadowed by it —
    /// `TraceLog::last(name)` returns the funnel's, and a check asking for `state=`
    /// would find nothing and report *"the verb did nothing"* about a verb that
    /// worked.
    pub(super) fn record(self, doc: &mut OpenDoc, author: &str) {
        // ★ Taken before the closure, because `pdf_date_utc` can fail (a clock
        // before the epoch) and `add_review_state` takes an `Option<&str>` for
        // exactly that: `/M` is optional on a markup annotation, and writing a
        // fabricated date would be worse than writing none.
        let modified = crate::app::clock::pdf_date_utc();
        let signed = !author.is_empty();
        super::apply::vector_edit(doc, "record-review-state", 0, 1, |session| {
            session
                .add_review_state(self.id, self.state, author, modified.as_deref())
                .map(|added| {
                    crate::diag::trace(|| {
                        // ui-text-exempt: diagnostic trace, never displayed.
                        format!(
                            "record-review-state-applied target={} state={} model={} \
                                 status={} attached={} depth={} signed={signed}",
                            added.target_id.num,
                            added.state.as_str(),
                            added.state.model(),
                            added.state_id.num,
                            added.attached_to.num,
                            added.chain_depth,
                        )
                    });
                    // ★ The unsigned sentence goes SECOND, and the order is the
                    // decision: the first line is the one an operator reads if they
                    // read only one, and between "what was recorded" and "it was
                    // recorded without a name", only the first answers the question
                    // they just asked. The second is a consequence they will meet
                    // later, and it is worded so it still makes sense read alone.
                    let mut said = vec![t::status_recorded(
                        t::state_name(added.state),
                        added.chain_depth,
                    )];
                    if !signed {
                        said.push(t::status_recorded_unsigned().to_owned());
                    }
                    said
                })
        });
    }
}
