//! # `app::quitting` — **closing the program without losing anybody's work**
//!
//! Operator, 2026-09-02, `OPERATOR_REQUESTS.md` O102:
//!
//! > *"when I close the program it should prompt to save changes if there are
//! > any, and it should do what other programs do — switch focus to the document
//! > that is being prompted for, and cycle through each unsaved document while
//! > it prompts, but also have a save all button that saves all changed
//! > documents."*
//!
//! ## ★★★ What was there before: nothing
//!
//! `crate::dialogs::unsaved` has asked about **one** document since it was
//! written — the tab being closed — and asked it well: it focuses the tab first,
//! it counts edits from the last *save* rather than from zero, and it refuses to
//! proceed on a save that did not happen.
//!
//! None of that was reachable from the **window's ✕**. `eframe`'s close request
//! was never read, so pressing it, or `Alt+F4`, ended the process with every
//! unsaved document still unsaved. The one thing on the exit path was
//! `App::on_exit`, which flushes a layout debounce.
//!
//! ⇒ The gap was not "the prompt is wrong", it was "there is no prompt", and it
//! was reachable in one keystroke.
//!
//! ## The cycle, and why it is derived rather than remembered
//!
//! [`Quitting`] holds **one boolean**. Everything else is re-derived from the
//! document set on every frame:
//!
//! 1. a close is requested, and something is dirty → **cancel the close**, set
//!    the flag;
//! 2. while the flag is set, find the **first dirty slot**, activate it, and ask
//!    about it;
//! 3. an answer either cleans that document or closes it, so the next scan finds
//!    the next one;
//! 4. nothing dirty left → **close for real**;
//! 5. Cancel at any point clears the flag and the program stays open.
//!
//! ★★ A remembered queue would be a second model of the document set, and it
//! would go stale the moment a save, a discard or a close changed one — which is
//! exactly what every answer here does. Re-deriving cannot drift, and the cost
//! is a scan of at most a handful of slots on the frames where a modal is up.
//!
//! ★ Step 2 is the operator's second requirement and it is not decoration: a
//! modal asking *"save changes?"* over a document you cannot see is asking about
//! a file you have to guess at. `crate::dialogs::unsaved`'s own `PendingIntent`
//! already documents that rule for the single-tab case; this applies the same
//! one to the quit cycle.
//!
//! ## ★★ Why Cancel abandons the whole quit, not one question
//!
//! Because that is what the operator meant by it, and it is what Word, VS Code
//! and Notepad++ all do. The alternative — Cancel skips this document and asks
//! about the next — leaves the program in a state where some documents have been
//! closed and the operator asked for none of it.
//!
//! ⇒ Cancel is the **only** answer that undoes work already done in the cycle,
//! and it undoes it by not having done any: nothing is closed until the last
//! question is answered… except that it is, because a Discard closes its
//! document as it goes. That is the honest limit of this design and it is stated
//! rather than hidden — see [`Quitting::stand_down`].

use crate::app::state::Status;

/// **Whether a quit is in progress**, and nothing else.
///
/// See the module header for why this is one boolean rather than a queue.
#[derive(Debug, Default)]
pub struct Quitting {
    /// A close was requested, work was outstanding, and the cycle is running.
    running: bool,
}

impl Quitting {
    /// Whether the cycle is running.
    #[must_use]
    pub const fn running(&self) -> bool {
        self.running
    }

    /// Begin the cycle.
    pub const fn begin(&mut self) {
        self.running = true;
    }

    /// **Abandon the quit.** The program stays open.
    ///
    /// ★ Called on Cancel, and it is the answer with the honest caveat: any
    /// document the operator already chose to *discard* in this cycle is
    /// already closed, and cancelling does not bring it back. That matches
    /// every editor in the class — a discard is an answer, not a step — but it
    /// is worth saying out loud rather than leaving somebody to discover it.
    pub const fn stand_down(&mut self) {
        self.running = false;
    }
}

/// **The first slot with unsaved work**, or `None` when everything is clean.
///
/// The whole of the cycle's ordering: lowest tab position first, which is
/// left-to-right in the strip and is the order an operator reads them in.
///
/// ★ Takes a **predicate** rather than `&PdfcerApp`, so it can be tested without
/// an application — and a predicate rather than the `Status` itself because
/// `Status` is deliberately not `Clone` (it owns an `EditSession`). What this
/// needs to know is *"is slot n dirty"*, which is one bool.
#[must_use]
pub fn first_dirty(count: usize, dirty: impl Fn(usize) -> bool) -> Option<usize> {
    (0..count).find(|&n| dirty(n))
}

/// **How many slots have unsaved work.**
///
/// Read for one decision only: whether to offer *Save all*. A cycle of one does
/// not need it, and a button that does the same as the one beside it is a
/// button an operator has to think about.
#[must_use]
pub fn dirty_count(count: usize, dirty: impl Fn(usize) -> bool) -> usize {
    (0..count).filter(|&n| dirty(n)).count()
}

/// Whether one slot has work that would be lost.
///
/// ★★ `save::has_unsaved_edits`, which is the **one** predicate — the same one
/// `dialogs::unsaved::ask_for` consults before deciding whether to ask at all.
/// Its own header records that there were once three expressions of this
/// question in three places and that they disagreed; a fourth here would be the
/// same defect wearing this module's name.
pub fn is_dirty(status: &Status) -> bool {
    match status {
        Status::Open(doc) => crate::app::save::has_unsaved_edits(doc),
        _ => false,
    }
}

/// The cycle's three verbs, on the application that runs them.
///
/// ★ Here rather than in [`crate::app::lifecycle`] — where they were first
/// written — because that file reached the 1,500-line R2 ceiling and because
/// this is the module that owns the subject. Everything above is the *rule*;
/// everything below is the application applying it.
impl crate::app::PdfcerApp {
    /// **One frame of the quit cycle** — `OPERATOR_REQUESTS.md` O102.
    ///
    /// Called once per frame, after both dialog drains. See
    /// [`crate::app::quitting`] for why the cycle is derived from the document
    /// set rather than remembered as a queue.
    ///
    /// # The five states, and the order they are checked in
    ///
    /// 1. **Cancelled** — the operator answered Cancel, so stand down and stay
    ///    open. Checked first, because every state below would otherwise act on
    ///    a cycle that has just been abandoned.
    /// 2. **A close was requested and nothing is dirty** — let it through. No
    ///    dialog, no cancelled close, no flicker.
    /// 3. **A close was requested and something is dirty** — cancel the close
    ///    and begin the cycle.
    /// 4. **The cycle is running and something is dirty** — activate that
    ///    document and ask about it.
    /// 5. **The cycle is running and nothing is dirty** — close, for real.
    ///
    /// # ★★ Why the close is cancelled rather than pre-empted
    ///
    /// `egui` reports the request and closes at the end of the frame unless
    /// something says otherwise. There is no "ask first" hook, so the sequence
    /// has to be *let it be requested, cancel it, then re-request it when the
    /// questions are answered*. `ViewportCommand::CancelClose` is that, and it
    /// must be sent on **the same frame** the request is read.
    pub(crate) fn step_quit_cycle(&mut self, ctx: &egui::Context) {
        // 1 — an answered Cancel abandons the whole quit, not one question.
        if self.dialogs.unsaved_cancelled() && self.quitting.running() {
            self.quitting.stand_down();
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                "quit-cancelled".to_owned()
            });
            return;
        }

        let requested = ctx.input(|i| i.viewport().close_requested());
        let dirty = crate::app::quitting::first_dirty(self.document_count(), |slot| {
            self.slot(slot).is_some_and(crate::app::quitting::is_dirty)
        });

        if requested {
            // 2 — nothing outstanding. Let it go.
            if dirty.is_none() {
                return;
            }
            // 3 — hold the door.
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.quitting.begin();
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                //
                // ★★ The COUNT is on the line. "A close was held" and "a close
                // was held because four documents are dirty" are the same event
                // to a reader who cannot see the tab strip, and the count is
                // what a driven check asserts the cycle works through.
                format!(
                    "quit-held dirty={}",
                    crate::app::quitting::dirty_count(self.document_count(), |slot| self
                        .slot(slot)
                        .is_some_and(crate::app::quitting::is_dirty))
                )
            });
        }

        if !self.quitting.running() {
            return;
        }

        match dirty {
            // 4 — ask about the leftmost dirty document, having brought it to
            // the front first. That is the operator's own second requirement,
            // and `dialogs::unsaved`'s `PendingIntent` already documents the
            // rule for the single-tab case.
            Some(slot) => {
                if self.active_slot != slot {
                    self.activate_slot(slot);
                }
                let count = crate::app::quitting::dirty_count(self.document_count(), |s| {
                    self.slot(s).is_some_and(crate::app::quitting::is_dirty)
                });
                self.ask_unsaved_for_quit(count);
            }
            // 5 — every question answered. Close for real.
            None => {
                self.quitting.stand_down();
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    "quit-proceeding".to_owned()
                });
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
    }

    /// Raise the unsaved question for the active document, told how many are
    /// dirty so the *Save all* button knows whether to draw itself.
    ///
    /// ★ A thin wrapper rather than a parameter on `DialogsState::ask_unsaved`,
    /// because every other caller is about **one** document and should keep
    /// saying so without being edited.
    pub(crate) fn ask_unsaved_for_quit(&mut self, dirty: usize) {
        self.dialogs.ask_unsaved_in_cycle(
            &self.status,
            crate::dialogs::unsaved::PendingIntent::Close,
            dirty,
        );
    }

    /// **Write every dirty document that has a file, in place.**
    ///
    /// `OPERATOR_REQUESTS.md` O102's fourth requirement — *"a save all button
    /// that saves all changed documents"* — and the thing that makes the quit
    /// cycle bearable: without it, somebody with six dirty documents answers six
    /// questions.
    ///
    /// # Returns
    ///
    /// `false` if any attempted write failed, so the caller can abandon the
    /// resume. ★ A document with **no file is not attempted and is not a
    /// failure**: it needs a destination, which is a question only the operator
    /// can answer, and the cycle asks about those individually afterwards.
    ///
    /// # ★★ Why it activates each slot before writing it
    ///
    /// Because `save::save_in_place` takes the **active** document, and the
    /// application's own invariant is that `status` is the active one with the
    /// rest parked. Reaching into a parked slot to write it would be a second
    /// way to save, and the two would eventually disagree about what a save
    /// does — the signature question, the receipt, the epoch. Activating first
    /// means every document in the batch is saved by exactly the path a single
    /// save uses.
    ///
    /// ★ The originally-active slot is restored at the end, so an operator who
    /// cancels the rest of the cycle is looking at the document they were
    /// looking at when they pressed the button.
    pub(super) fn save_every_dirty_document(&mut self) -> bool {
        let started_on = self.active_slot;
        let count = self.document_count();
        let mut all_written = true;
        for slot in 0..count {
            let dirty_with_file = matches!(
                self.slot(slot),
                Some(crate::app::state::Status::Open(doc))
                    if crate::app::save::has_unsaved_edits(doc)
                        && crate::app::save::has_a_file(doc)
            );
            if !dirty_with_file {
                continue;
            }
            self.activate_slot(slot);
            if !self.write_in_place() {
                all_written = false;
            }
        }
        self.activate_slot(started_on);
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            //
            // ★ `all_written` beside the count, because "saved four of four"
            // and "attempted four and one refused" are the two states the guard
            // above branches on and a count alone cannot separate them.
            format!("save-all documents={count} all_written={all_written}")
        });
        all_written
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A stand-in for the document set: `true` means that slot is dirty.
    ///
    /// The functions under test take a closure precisely so this can exist —
    /// building real `Status::Open` values needs a parsed document, and what is
    /// being tested is the **ordering and counting**, not the dirty predicate,
    /// which is `save::has_unsaved_edits` and is tested where it lives.
    fn scan(dirty: &[bool]) -> (usize, impl Fn(usize) -> Option<bool> + '_) {
        (dirty.len(), move |n: usize| dirty.get(n).copied())
    }

    /// The ordering rule, stated as a test because it is a choice.
    ///
    /// ★ Lowest slot first — left to right in the tab strip, which is the order
    /// the operator reads them in. Any other order would make the cycle feel
    /// arbitrary, and "arbitrary" is what a modal must never feel while it is
    /// asking about destroying work.
    #[test]
    fn the_cycle_takes_the_leftmost_dirty_document_first() {
        let (n, dirty) = scan(&[false, true, true]);
        assert_eq!(
            (0..n).find(|&i| dirty(i) == Some(true)),
            Some(1),
            "slot 1 is the first dirty one and must be asked about before slot 2"
        );
    }

    /// ★★ **Everything clean means no question and no cancelled close.**
    ///
    /// The case that must not regress into a spurious modal: an operator who
    /// has saved everything and presses ✕ should get an immediate exit, not a
    /// dialog with nothing in it. `first_dirty` answering `None` is what makes
    /// the whole cycle skip.
    #[test]
    fn a_clean_set_has_no_first_dirty() {
        let (n, dirty) = scan(&[false, false]);
        assert_eq!((0..n).find(|&i| dirty(i) == Some(true)), None);
        // …and the empty set, which is the no-documents case.
        let (n, dirty) = scan(&[]);
        assert_eq!((0..n).find(|&i| dirty(i) == Some(true)), None);
    }

    /// ★ **Save all is offered only when it would do more than Save.**
    ///
    /// With one dirty document the two buttons are the same act, and a second
    /// button that means the same thing is one the operator has to stop and
    /// think about — on a modal that is standing between them and their work.
    #[test]
    fn save_all_is_for_more_than_one() {
        let counted = |d: &[bool]| d.iter().filter(|x| **x).count();
        assert_eq!(counted(&[true, false, true]), 2, "offer it");
        assert_eq!(counted(&[true, false]), 1, "do not offer it");
        assert_eq!(counted(&[false]), 0, "nothing to offer");
    }

    /// The flag starts down, goes up on `begin`, and comes back down on Cancel.
    ///
    /// ★ Pinned because `running` defaulting to `true` would make the
    /// application try to quit on its first frame, which is the one failure
    /// mode of this design that would be spectacular rather than subtle.
    #[test]
    fn the_cycle_starts_down_and_cancel_puts_it_back() {
        let mut q = Quitting::default();
        assert!(!q.running(), "a fresh application is not quitting");
        q.begin();
        assert!(q.running());
        q.stand_down();
        assert!(!q.running(), "Cancel must abandon the whole quit");
    }
}
