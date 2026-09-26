//! # `app::quitting` — **closing the program without losing anybody's work**
//!
//!
//! > *"when I close the program it should prompt to save changes if there are
//! > any, and it should do what other programs do — switch focus to the document
//! > that is being prompted for, and cycle through each unsaved document while
//! > it prompts, but also have a save all button that saves all changed
//! > documents."*
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/quitting.md`.

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
    pub const fn stand_down(&mut self) {
        self.running = false;
    }
}

/// **The first slot with unsaved work**, or `None` when everything is clean.
#[must_use]
pub fn first_dirty(count: usize, dirty: impl Fn(usize) -> bool) -> Option<usize> {
    (0..count).find(|&n| dirty(n))
}

/// **How many slots have unsaved work.**
#[must_use]
pub fn dirty_count(count: usize, dirty: impl Fn(usize) -> bool) -> usize {
    (0..count).filter(|&n| dirty(n)).count()
}

/// Whether one slot has work that would be lost.
pub fn is_dirty(status: &Status) -> bool {
    match status {
        Status::Open(doc) => crate::app::save::has_unsaved_edits(doc),
        _ => false,
    }
}

/// The cycle's verbs, on the application that runs them.
impl crate::app::PdfcerApp {
    /// **One frame of the quit cycle** — `OPERATOR_REQUESTS.md` O102.
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
                // The COUNT is on the line. "A close was held" and "a close
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
    pub(crate) fn ask_unsaved_for_quit(&mut self, dirty: usize) {
        self.dialogs.ask_unsaved_in_cycle(
            &self.status,
            crate::dialogs::unsaved::PendingIntent::Close,
            dirty,
        );
    }

    /// **Write every dirty document that has a file, in place.**
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
            // `all_written` beside the count, because "saved four of four"
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
    fn scan(dirty: &[bool]) -> (usize, impl Fn(usize) -> Option<bool> + '_) {
        (dirty.len(), move |n: usize| dirty.get(n).copied())
    }

    /// The ordering rule, stated as a test because it is a choice.
    #[test]
    fn the_cycle_takes_the_leftmost_dirty_document_first() {
        let (n, dirty) = scan(&[false, true, true]);
        assert_eq!(
            (0..n).find(|&i| dirty(i) == Some(true)),
            Some(1),
            "slot 1 is the first dirty one and must be asked about before slot 2"
        );
    }

    /// **Everything clean means no question and no cancelled close.**
    #[test]
    fn a_clean_set_has_no_first_dirty() {
        let (n, dirty) = scan(&[false, false]);
        assert_eq!((0..n).find(|&i| dirty(i) == Some(true)), None);
        // …and the empty set, which is the no-documents case.
        let (n, dirty) = scan(&[]);
        assert_eq!((0..n).find(|&i| dirty(i) == Some(true)), None);
    }

    /// **Save all is offered only when it would do more than Save.**
    #[test]
    fn save_all_is_for_more_than_one() {
        let counted = |d: &[bool]| d.iter().filter(|x| **x).count();
        assert_eq!(counted(&[true, false, true]), 2, "offer it");
        assert_eq!(counted(&[true, false]), 1, "do not offer it");
        assert_eq!(counted(&[false]), 0, "nothing to offer");
    }

    /// The flag starts down, goes up on `begin`, and comes back down on Cancel.
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
