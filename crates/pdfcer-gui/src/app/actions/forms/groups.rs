//! # `app::actions::forms::groups` — deleting a grouping node, and the preview
//! that has to happen first
//!
//! The apply half of [`super::FieldAction::ArmGroupDeletion`] and
//! [`super::FieldAction::DeleteGroup`]. The vocabulary stays in [`super`] with
//! the rest of the field verbs; only the logic and the store live here.
//!
//! ## Why this is a file rather than two more functions in [`super`]
//!
//! **R2, and the seam is real rather than size-driven.** `super` is *"everything
//! done to a form FIELD"* — verbs that address a control by its fully-qualified
//! name and change what one control is or holds. This module addresses a name
//! that **is not a control**: a grouping node has no type, no value, no widget
//! and no rectangle, and the whole difficulty of the verb is that its
//! consequences are invisible and must therefore be *stated in advance*. That
//! is a different subject with a different failure mode, and it is the reason
//! this is the only form verb in the shell that takes **two** operator presses.
//!
//! ## ★★★ The two-press protocol, and why it is not a confirmation dialog
//!
//! ```text
//!   press 1  ──▶  ArmGroupDeletion(Some(name))
//!                    │  field_group_deletion_preview(&mut session)
//!                    │  writes NOTHING, bumps NO epoch
//!                    ▼
//!                 ARMED  ──▶  the panel draws what would go, in numbers and names
//!                    │
//!   press 2  ──▶  DeleteGroup { group }
//!                    │  vector_edit ▸ delete_field_group
//!                    ▼
//!                 the engine's REPORT ──▶ the status bar's disclosure row
//! ```
//!
//! ### Why the preview cannot happen in the panel
//!
//! Because `EditSession::field_group_deletion_preview` takes **`&mut self`**,
//! and a panel body is handed `&OpenDoc` — a *shared* reference, which is the
//! compile-time expression of "no code path runs from a widget to a document"
//! (see `app/actions/OVERVIEW.md`). The session lives behind an `Arc`, and the
//! only place `Arc::get_mut` succeeds is inside the funnel, after the frame.
//!
//! ⇒ So the preview **is** an action, even though it changes nothing. It is the
//! same shape as `FieldAction::Select`: raised by a widget, applied by the
//! funnel, bumping no epoch and invalidating no page. What is unusual is only
//! that its *result* has to travel back to the panel, which is what the store
//! below is for.
//!
//! ### Why not a modal dialog, when `crate::dialogs` is full of them
//!
//! Three reasons, in order of weight:
//!
//! 1. **The section already answers the question the dialog would re-ask.** The
//!    Forms panel has the parsed `/AcroForm` in hand and is already listing the
//!    grouping nodes. A window would have re-parsed it, at a second moment,
//!    producing a second answer to one question — which is exactly the shape
//!    `panels::forms::tab_order::register`'s header argues against for the
//!    Register rows, and it fails the same silent way: the list and the button
//!    beside it come to disagree about the set.
//! 2. **It is where the operator already is.** They opened Field groups because
//!    they wanted to know what was in one. The answer belongs under the row
//!    they were reading.
//! 3. **A dialog needs six files this work does not own** — a variant, a host
//!    arm, a window, a region, a close path and a focus policy — and the
//!    disclosure is the substance here, not the chrome.
//!
//! ## ★★ Where the armed preview is kept, and why it is a thread-local
//!
//! Exactly [`crate::app::actions::disclosure`]'s answer, restated because the
//! reasoning has to be re-checked rather than inherited:
//!
//! - It **should** be a field on `OpenDoc`, beside `selected_field`. It is
//!   per-document state with a per-document lifetime.
//! - `OpenDoc` is declared in `crate::app::state`, which sits at 1,494 of R2's
//!   1,500-line budget. Extending it is not a design judgement here, it is a
//!   file that has no room — stated so whoever splits that file knows where
//!   this belongs.
//! - It is nonetheless sound: this is **not document state**. It cannot change
//!   a pixel, it cannot reach a save, and nothing reads it except the section
//!   that drew the row. `eframe`'s update loop is one thread, so the writer and
//!   the reader are the same thread, and a test on another thread gets its own
//!   empty slot rather than another test's leftovers.
//!
//! ### ★★★ Staleness is handled by the EPOCH, not by remembering to clear
//!
//! [`Armed::epoch`] is the `OpenDoc::edit_epoch` current **when the preview was
//! taken**, and [`armed`] answers `None` for anything else. That one comparison
//! retires the preview on every path that could invalidate it, without any of
//! them knowing this store exists:
//!
//! | what happens | epoch | the armed preview |
//! |---|---|---|
//! | the deletion is confirmed | bumps | gone — the group it named no longer exists |
//! | some other form edit lands | bumps | gone — its counts were taken against an older form |
//! | **undo / redo** | bumps | gone — and this is the one a `clear()` call would have missed |
//! | the operator cancels | — | cleared explicitly, by `ArmGroupDeletion(None)` |
//!
//! The undo row is why the epoch rule is not merely tidy. Undo and redo do not
//! clear selections, and a preview surviving one would sit under a row
//! describing a subtree that had come back with different contents.
//!
//! ## Trace names
//!
//! `form-group-preview` for the arm, `form-group-preview-refused` for the
//! refusal, and — inside the funnel closure —
//! **`delete-field-group-applied`**, with the `-applied` suffix.
//!
//! ★★★ That suffix is not decoration. `vector_edit` writes its own line for the
//! same edit under the bare label (`delete-field-group page=0 n=1 epoch=…`),
//! and trace matching is on the **exact event name**, so a driven check taking
//! `.last()` would read the funnel's line — which carries no `terminals=` key —
//! and report zero fields removed about a deletion that removed four. This
//! project has made that mistake twice: `text-style` and `import-form-data`,
//! both fixed the same way. **A module's own summary line takes a verb suffix;
//! the funnel's label keeps the bare name.**

use std::cell::RefCell;

use pdfcer_core::edit::FieldGroupDeletion;

use crate::app::state::OpenDoc;
use crate::text::forms as t;

/// A grouping-node deletion the operator has asked about and not yet confirmed.
///
/// Carries the engine's own [`FieldGroupDeletion`] verbatim rather than a
/// flattened set of counts, because the panel needs the **names** as well as
/// the numbers and because re-shaping the report here would be a second
/// vocabulary for facts the engine already has words for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Armed {
    /// The `OpenDoc::edit_epoch` this preview was taken against.
    ///
    /// See the module header's staleness table. A preview whose epoch is not
    /// the document's current one describes a form that has since changed and
    /// must not be drawn from.
    pub epoch: u64,
    /// What the engine says deleting it would remove.
    ///
    /// `nodes_removed` here is a **prediction** — core says so explicitly, and
    /// overwrites it with the truth in the returned report of the deletion
    /// itself. The pre-press sentence is allowed to be a prediction; the
    /// post-press one is not, and [`delete`] below builds it from the report.
    pub preview: FieldGroupDeletion,
}

thread_local! {
    /// The one armed preview, if any.
    ///
    /// ★ One rather than a map keyed by group name, and that is a decision.
    /// Arming a second group while a first is armed **replaces** it, because
    /// two disclosure blocks open at once in a narrow dock pane is two
    /// destructive confirmations competing for one glance — and the operator
    /// can only be about to press one of them.
    static ARMED: RefCell<Option<Armed>> = const { RefCell::new(None) };
}

/// What the operator has armed, if it still describes the document on screen.
///
/// **The panel's read** — see [`crate::panels::forms::groups`]. Returns `None`
/// when nothing is armed, or when the armed preview was taken against a
/// revision the document has since moved off.
#[must_use]
pub fn armed(epoch: u64) -> Option<Armed> {
    ARMED.with_borrow(|slot| slot.as_ref().filter(|a| a.epoch == epoch).cloned())
}

/// **Arm a grouping-node deletion, or disarm.**
///
/// `Some(name)` asks the engine what deleting that node would remove and stores
/// the answer; `None` clears — the operator pressed Cancel, or clicked away.
///
/// ★ `None` is a real event and not a no-op, for `FieldAction::Select`'s
/// reason: a disclosure block that will not let go is worse than none, because
/// its contents look current.
///
/// # ★★ It changes NO document and must never bump the epoch
///
/// `field_group_deletion_preview` writes nothing — it resolves the node, runs
/// the gates and describes the removal set. So this does not go through
/// [`crate::app::actions::apply::vector_edit`]: there is no render worker to
/// stop, no epoch to move, no page to invalidate, and moving the epoch would
/// *immediately retire the preview it had just taken*, which is the one bug
/// this function could plausibly ship with.
///
/// It nonetheless needs `Arc::get_mut`, because the engine's signature is
/// `&mut self`. A borrowed session is reported and dropped, exactly as
/// `vector_edit` reports its own — a preview that silently did not happen would
/// leave a Delete-group button that does nothing when pressed, which is the
/// inert-control failure this project exists to remove.
pub(in crate::app::actions) fn arm(doc: &mut OpenDoc, group: Option<String>) {
    let Some(name) = group else {
        ARMED.with_borrow_mut(|slot| *slot = None);
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            // ★★ `-cleared`, not the bare name, and the suffix is load-bearing
            // for the same reason `check-trace-names` exists — one rung further
            // out than that gate can see.
            //
            // The armed line four screens down is also `form-group-preview …`,
            // and a harness reads a trace by its FIRST TOKEN. Today nothing
            // presses Cancel, so `.last()` happens to find the armed line; add
            // the cancel step this two-press protocol invites and it finds this
            // one instead, `terminals` parses to 0 through `unwrap_or_default`,
            // and the check fires *"the preview resolved and reported ZERO
            // fields"* against a correct build.
            //
            // ⇒ `check-trace-names.py` compares module lines against **funnel
            // labels** and never against each other, so this class is outside
            // it. Named here rather than left as a latent trap.
            "form-group-preview-cleared cleared=1".to_owned()
        });
        return;
    };

    let epoch = doc.edit_epoch;
    // ★★★ **The render worker holds the other `Arc`, so this line is the whole
    // difference between a preview and an inert button.**
    //
    // `RenderWorker` clones `doc.session` into its request (`app::state`), so
    // `Arc::get_mut` fails for as long as a raster is in flight — which is after
    // every scroll, zoom, page turn and mode change. Without this call, pressing
    // *Delete group…* a moment after moving the view wrote one trace line and
    // **nothing to the screen**: no block, no numbers, no sentence. Press again
    // a second later and it worked.
    //
    // ⇒ This function's own doc comment four lines up calls that outcome *"the
    // inert-control failure this project exists to remove"* — and then produced
    // it, because it reported to the trace and to nothing else. It was the only
    // production `Arc::get_mut` in the crate outside `vector_edit`, which takes
    // this step as its **first statement** for exactly this reason.
    //
    // ★ A preview is a read and cancelling a raster for a read looks wasteful.
    // It is not: the operator pressed a button, the raster restarts on the next
    // frame from a cache that the preview does not invalidate, and the
    // alternative is a control that works only when the page happens to be
    // still. `vector_edit`'s own note on this call — *"the choke point"* —
    // carries the argument.
    doc.render_worker.cancel_and_wait();
    let Some(session) = std::sync::Arc::get_mut(&mut doc.session) else {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "form-group-preview-refused reason=session-borrowed".to_owned()
        });
        return;
    };

    match session.field_group_deletion_preview(&name) {
        Ok(preview) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI.
                // The group's NAME is not carried: a field's name is the
                // operator's own words about their document, and `adopt-row`
                // and `bookmark-add` make the same ruling for the same reason.
                // The COUNTS are what a check needs, and they are what proves
                // the preview was actually asked.
                format!(
                    "form-group-preview epoch={epoch} terminals={} widgets={} nodes={}",
                    preview.terminals.len(),
                    preview.widgets_removed,
                    preview.nodes_removed,
                )
            });
            ARMED.with_borrow_mut(|slot| *slot = Some(Armed { epoch, preview }));
        }
        Err(error) => {
            // ★★ Close to unreachable, because the section asks
            // `deletion_refusal` before it draws the control that raises this —
            // and `field_group_deletion_preview` runs those same two gates.
            // Reaching it means the pure query and the preview have come apart,
            // or the name resolves to a terminal rather than a node.
            //
            // Worded anyway. R83 buys the operator a control that is not
            // offered when it cannot work; it does not buy them silence when
            // the offer turns out to have been wrong.
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!("form-group-preview-refused epoch={epoch} detail={error}")
            });
            ARMED.with_borrow_mut(|slot| *slot = None);
            crate::app::status::decline::record_field_group_preview_refused();
        }
    }
}

/// **Delete a grouping node and everything beneath it**, as one undoable
/// command.
///
/// # ★★★ The disclosure is built from the ENGINE'S REPORT, not from the preview
///
/// `delete_field_group` returns a [`FieldGroupDeletion`] whose `nodes_removed`
/// is *"what the cascade ACTUALLY emptied, not a prediction"* — core replaces
/// the preview's figure with the truth and keeps a `debug_assert` for the day
/// the two disagree. Building the sentence from the report rather than from the
/// armed preview means the operator reads what happened, on the day those two
/// stop agreeing, instead of reading what was expected to.
///
/// # Rule 4, and why the sentence is owed rather than optional
///
/// Deleting a grouping node changes **nothing an operator can see**. The page
/// is identical, the canvas is identical, the raster is identical; a form field
/// is not drawn as such and a grouping node is not drawn at all. The disclosure
/// is not a courtesy on this verb, it is the only evidence that the press did
/// anything — which is why it names all three counts and the group.
///
/// # ★ The armed preview is not cleared here, and does not need to be
///
/// A successful deletion bumps the epoch through `vector_edit`, and [`armed`]
/// filters on the epoch. A *failed* one does not bump it, so the preview
/// survives a refusal — which is right: the operator is looking at a block
/// describing a group that is still there, beside a sentence saying it was not
/// removed.
///
/// # ★★★ The refusal is worded HERE, because `vector_edit`'s refusal arm only
/// traces
///
/// That arm's own comment is explicit about it: a refusal *"is deliberately not
/// routed"* to the disclosure row, because a disclosure is after-the-fact and a
/// decline is not, and *"sharing one slot would mean an undone gesture and a
/// completed one wearing the same wording in the same place."*
///
/// The argument is right about the **slot** and it does not license a silence
/// on this verb. Every consequence of a grouping-node deletion is off-canvas,
/// so an operator who has just read a list of four field names and pressed a
/// button labelled *"Delete 4 fields"* has **no evidence at all** of what
/// happened — a success and a refusal are the identical screen. A trace line is
/// not a disclosure; the operator cannot read it.
///
/// So this follows `import_data`'s precedent — the one other form verb that
/// words its own decline through [`crate::app::actions::record_note`] — and
/// resolves the slot-sharing hazard in the **wording** rather than by adding a
/// second mechanism: [`crate::text::forms::field_group_delete_refused`] says
/// *"the form is unchanged. Nothing was removed."* in the sentence itself, so
/// it cannot be misread as a report of a completed act however it is placed.
///
/// The epoch is captured **before** the call and is the right stamp on either
/// outcome: a refusal does not move it, so the sentence is current; a success
/// moves it, and the success path's own disclosure is stamped with the new one
/// by `vector_edit`.
pub(in crate::app::actions) fn delete(doc: &mut OpenDoc, group: &str) {
    // The selection is a field, and a field this deletion may be about to
    // remove. Cleared for `delete_field`'s reason: a properties pane describing
    // a field that is gone is worse than an empty one.
    doc.selected_field = None;
    let name = group.to_owned();
    super::super::apply::vector_edit(doc, "delete-field-group", 0, 1, move |session| {
        let outcome = session.delete_field_group(&name);
        if outcome.is_err() {
            crate::app::status::decline::record_field_group_delete_refused();
        }
        outcome.map(|report| {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI.
                // `-applied`, NOT the bare label — see the module header for
                // the two occasions this project got that wrong.
                format!(
                    "delete-field-group-applied terminals={} widgets={} nodes={}",
                    report.terminals.len(),
                    report.widgets_removed,
                    report.nodes_removed,
                )
            });
            vec![t::field_group_deleted(
                &report.group_name,
                report.terminals.len(),
                report.widgets_removed,
                report.nodes_removed,
            )]
        })
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The engine's own two-level form**: `Personal.Name` and
    /// `Personal.Address.Zip`, so `Personal` and `Personal.Address` are both
    /// grouping nodes and the second is emptied by deleting the first.
    ///
    /// ★ The fixture is chosen for the **cascade**, which is the case the
    /// disclosure exists for. A one-level group would exercise the verb and
    /// prove nothing about the number an operator cannot predict — how many
    /// *other* nodes go with the one they named. `PROVENANCE.md` records that
    /// `Personal.Name` sits one level shallower on purpose.
    const FIXTURE: &str = "forms/nested-form.pdf";
    /// The node this asks about — the root of the subtree, so both of its
    /// terminals and both grouping nodes are in the removal set.
    const GROUP: &str = "Personal";

    /// ★★★ The whole two-press protocol, end to end, against a real engine.
    ///
    /// Written as one test rather than three because the facts it asserts are
    /// only meaningful in sequence: a preview that is readable is worth nothing
    /// if the deletion that follows removes a different set, and an epoch rule
    /// is worth nothing unless something actually moves the epoch.
    ///
    /// The four claims, in order:
    ///
    /// 1. **The preview reaches the store**, with the counts the panel draws.
    /// 2. **It is invisible at any other revision** — which is the entire
    ///    safety argument for keeping it outside `OpenDoc`, and covers undo and
    ///    redo without either of them knowing this store exists.
    /// 3. **The deletion removes the whole subtree in one command**, so the
    ///    epoch moves exactly once.
    /// 4. **The armed preview retires itself** on that move, with nothing
    ///    having called a clear.
    #[test]
    fn a_preview_is_taken_then_confirmed_and_retires_itself() {
        let mut doc = crate::app::state::open_fixture(FIXTURE);

        // 1 — arm.
        arm(&mut doc, Some(GROUP.to_owned()));
        let live = armed(doc.edit_epoch).expect(
            "the preview must reach the store: without it the panel draws a destructive \
             confirmation with no numbers in it",
        );
        assert_eq!(live.preview.group_name, GROUP);
        // ★ Three, measured rather than assumed: `Personal.Name`,
        // `Personal.Address.City` and `Personal.Address.Zip`. The first draft
        // of this test said two, from reading `PROVENANCE.md`'s summary of the
        // fixture rather than asking the engine — which is the same mistake in
        // miniature that this whole surface exists to prevent, and it is why
        // the panel draws the preview's numbers instead of counting rows.
        assert_eq!(
            live.preview.terminals.len(),
            3,
            "`nested-form.pdf` files three terminals under `{GROUP}`: {:?}",
            live.preview.terminals
        );
        assert!(
            live.preview.nodes_removed >= 2,
            "★ the cascade is the point: deleting `{GROUP}` also empties \
             `Personal.Address`, a node the operator never named, and the disclosure has to \
             say so. Got {} node(s): {:?}",
            live.preview.nodes_removed,
            live.preview.nodes,
        );

        // 2 — invisible at any other revision.
        let taken_at = doc.edit_epoch;
        assert!(
            armed(taken_at.wrapping_add(1)).is_none(),
            "an edit or a redo moved past it"
        );
        assert!(
            armed(taken_at.wrapping_sub(1)).is_none(),
            "an undo moved behind it"
        );

        // 3 — confirm.
        delete(&mut doc, GROUP);
        assert_eq!(
            doc.edit_epoch,
            taken_at.wrapping_add(1),
            "★ ONE undo entry for one operator gesture. A loop of `delete_field` would have \
             moved the epoch twice and made Ctrl+Z peel the subtree back one field at a time."
        );
        let view = doc.session.view();
        let form = pdfcer_core::forms::parse_acroform(&view).expect("the form survives");
        assert!(
            !form
                .groups
                .iter()
                .any(|g| g.fully_qualified_name.starts_with(GROUP)),
            "the named node and the one beneath it are both gone: {:?}",
            form.groups
                .iter()
                .map(|g| &g.fully_qualified_name)
                .collect::<Vec<_>>()
        );
        assert!(
            !form
                .fields
                .iter()
                .any(|f| f.fully_qualified_name.starts_with(GROUP)),
            "every terminal beneath it went with it"
        );

        // 4 — the preview retired itself, with nothing having cleared it.
        assert!(
            armed(doc.edit_epoch).is_none(),
            "★★★ the epoch rule, which is the reason nothing has to remember to clear this: \
             a confirmed deletion moves the epoch, and the block describing what WOULD go \
             must not survive into a document where it already has"
        );
    }

    /// **Cancel changes nothing and clears the block.**
    ///
    /// ★ Asserted separately because it is the one path that must move no
    /// epoch: an operator who backs out of a destructive confirmation has done
    /// nothing, and a shell that bumped the revision for it would silently
    /// retire whatever disclosure was on screen and mark a clean document
    /// edited.
    #[test]
    fn cancelling_clears_the_block_and_edits_nothing() {
        let mut doc = crate::app::state::open_fixture(FIXTURE);
        let before = doc.edit_epoch;

        arm(&mut doc, Some(GROUP.to_owned()));
        assert!(armed(doc.edit_epoch).is_some(), "armed");
        assert_eq!(doc.edit_epoch, before, "a preview writes nothing");

        arm(&mut doc, None);
        assert!(armed(doc.edit_epoch).is_none(), "Cancel clears it");
        assert_eq!(doc.edit_epoch, before, "and still writes nothing");
    }

    /// **A terminal field's name is refused, not silently redirected.**
    ///
    /// ★★ The engine rules that `NotAGroupingNode` is a *wrong verb on a sound
    /// document* and deliberately does not fall back to `delete_field` —
    /// *"the two remove different amounts, and guessing which the caller meant
    /// is exactly the sneakiness rule 4 forbids on a destructive verb."* This
    /// asserts the shell inherits that rather than arming something.
    ///
    /// It is unreachable from the panel, which only ever passes names out of
    /// `AcroForm::groups`. Asserted anyway: the day a caller passes a field
    /// name, the operator must get nothing armed rather than a confirmation
    /// block describing a deletion of the wrong size.
    #[test]
    fn a_terminal_name_arms_nothing() {
        let mut doc = crate::app::state::open_fixture(FIXTURE);
        arm(&mut doc, Some("Personal.Name".to_owned()));
        assert!(
            armed(doc.edit_epoch).is_none(),
            "a terminal is not a grouping node, and must not arm a block that would claim to \
             remove a subtree"
        );
    }
}
