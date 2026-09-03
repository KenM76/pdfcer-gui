//! # `ocr::progress` — **what the recogniser is doing, and the two ways to end
//! it early**
//!
//! Operator request, 2026-09-01:
//!
//! > *"can you make it so the recognizing ocr gives feedback on what it is
//! > doing when it is running (pages done, words/characters detected, etc) so
//! > that the user can see that it is doing something and hasn't frozen on
//! > large documents? Maybe a cancel and stop button too. The cancel throws
//! > away what was done, and the stop finished the page it is on and keeps the
//! > work it has done."*
//!
//! Three things in one sentence, and the third is the one with a sharp edge.
//!
//! ## ★★★ Cancel and Stop are DIFFERENT, and the difference is the whole design
//!
//! | | the current page | what survives |
//! |---|---|---|
//! | **Cancel** | abandoned | **nothing** — the document is untouched |
//! | **Stop** | **finished** | every page recognised so far, offered for review as usual |
//!
//! They are not two names for one act and must never collapse into one. An
//! operator who presses Stop on page 40 of 200 has *asked for* those forty
//! pages; one who presses Cancel has asked for none of them. Getting that
//! backwards either throws away twenty minutes of work or writes a partial
//! layer somebody did not want.
//!
//! ★★ **Stop finishes the page it is on**, which is his wording and is also the
//! only coherent reading: a page is recognised as a unit — rendered, run
//! through the model, converted to page space — and half of one is not a thing
//! that can be kept. The wait is bounded by one page, which on a scanned sheet
//! is a second or two.
//!
//! ## ★ Why a flag and not a channel message
//!
//! The worker is a plain loop over pages; it does not select on anything. A
//! shared flag it reads at the top of each iteration costs one atomic load per
//! page and needs no runtime, no timeout and no second thread to deliver it.
//!
//! The flag is checked **between** pages and never inside one, which is what
//! makes "Stop keeps the finished pages" true by construction rather than by
//! care: there is no point in the loop where a half-recognised page exists.
//!
//! ## ★★ Why progress is a channel message and not a shared counter
//!
//! A counter would need the UI to poll a lock, and — more importantly — a
//! counter cannot carry *what was found*. The operator asked for words and
//! characters, which are per-page facts the worker computes and then folds
//! into a total; publishing them as they happen is free, and reconstructing
//! them from a shared number afterwards is impossible.
//!
//! It also means the UI thread never blocks on the worker: `try_recv` in a
//! loop, drain what is there, draw what arrived.

use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};

/// What the operator has asked the running job to do.
///
/// One atomic rather than two booleans, because the states are **ordered** and
/// mutually exclusive: a job cannot be both cancelled and stopped, and a Cancel
/// arriving after a Stop must win. An enum in a `u8` makes that a single
/// compare-and-set instead of two loads whose order a reader has to reason
/// about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Wish {
    /// Keep going — the ordinary state.
    Continue = 0,
    /// Finish the page in hand, then return what has been done.
    StopAfterThisPage = 1,
    /// Abandon everything, including the page in hand.
    Cancel = 2,
}

impl Wish {
    /// Decode a stored discriminant.
    ///
    /// An unrecognised value answers `Continue`, which is the safe direction:
    /// the failure mode of a corrupt read is a job that keeps going and can be
    /// asked again, not one that silently discards an operator's work.
    const fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::StopAfterThisPage,
            2 => Self::Cancel,
            _ => Self::Continue,
        }
    }
}

/// The handle both sides hold: the dialog writes it, the worker reads it.
#[derive(Debug, Clone, Default)]
pub struct Control(Arc<AtomicU8>);

impl Control {
    /// A fresh control, wishing `Continue`.
    #[must_use]
    pub fn new() -> Self {
        Self(Arc::new(AtomicU8::new(Wish::Continue as u8)))
    }

    /// What the worker should do now.
    ///
    /// `Relaxed` is correct and deliberate. There is no other memory being
    /// published alongside this flag — the results travel by channel, which
    /// carries its own ordering — so the only requirement is that the value
    /// eventually arrives, and a page of OCR is several orders of magnitude
    /// longer than any plausible propagation delay.
    #[must_use]
    pub fn wish(&self) -> Wish {
        Wish::from_u8(self.0.load(Ordering::Relaxed))
    }

    /// **Finish the page in hand and keep everything.**
    ///
    /// ★ Refuses to downgrade a Cancel. An operator who cancelled and then hit
    /// Stop — two clicks in the same second on adjacent buttons — must not have
    /// the abandonment quietly turned into a partial write.
    pub fn stop(&self) {
        let _ = self.0.compare_exchange(
            Wish::Continue as u8,
            Wish::StopAfterThisPage as u8,
            Ordering::Relaxed,
            Ordering::Relaxed,
        );
    }

    /// **Abandon everything**, including the page in hand.
    ///
    /// Unconditional: Cancel outranks Stop, because it is the one that cannot
    /// be undone by waiting and because it is what an operator reaches for when
    /// they have realised the whole run was a mistake.
    pub fn cancel(&self) {
        self.0.store(Wish::Cancel as u8, Ordering::Relaxed);
    }
}

/// One message from the worker.
///
/// ★★ `Page` is sent **after** the page is recognised, carrying that page's own
/// counts. The dialog accumulates; the worker does not send running totals,
/// because a message that is a total rather than an event cannot be dropped
/// safely and this channel is allowed to be drained in batches.
#[derive(Debug)]
pub enum Update {
    /// A page finished.
    Page(PageDone),
    /// The run ended. Always exactly one, and always last.
    Finished(Box<Outcome>),
}

/// What one finished page contributed.
#[derive(Debug, Clone, Copy)]
pub struct PageDone {
    /// The 0-based page index, so the dialog can say *"page 7"* rather than
    /// *"the 3rd page you selected"*.
    pub index: usize,
    /// How many pages of the request have now been attempted.
    pub attempted: usize,
    /// How many were requested in total.
    pub of: usize,
    /// Words recognised on this page. Zero for a page that was skipped.
    pub words: usize,
    /// Characters recognised on this page.
    ///
    /// ★ Asked for by name. It is the more responsive of the two on a dense
    /// drawing — a page can produce hundreds of characters in a handful of
    /// "words" — so it is the number that best shows the thing is alive.
    pub chars: usize,
}

/// How a run ended.
///
/// ★★★ `Stopped` is a distinct outcome and NOT a successful run with fewer
/// pages. The disclosure has to say the run ended early, or an operator who
/// stopped at page 40 of 200 is left believing the whole document was
/// recognised — which they will discover months later, searching for a word
/// that is on page 150 and is not in the layer.
#[derive(Debug)]
pub enum Outcome {
    /// Every requested page was attempted.
    Complete(Result<Box<super::Recognised>, super::Refusal>),
    /// The operator pressed Stop. Carries what was finished before it.
    Stopped {
        /// The work to keep. `Err` when Stop arrived before anything was
        /// recognised, which is not a failure but has nothing to offer.
        result: Result<Box<super::Recognised>, super::Refusal>,
        /// How many pages were attempted before stopping.
        attempted: usize,
        /// How many had been requested.
        of: usize,
    },
    /// The operator pressed Cancel. Nothing is kept and nothing is offered.
    Cancelled {
        /// How many pages had been attempted. Reported so the status line can
        /// say what was discarded rather than only that something was.
        attempted: usize,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fresh_control_wishes_to_continue() {
        assert_eq!(Control::new().wish(), Wish::Continue);
    }

    #[test]
    fn stop_and_cancel_each_take_effect() {
        let c = Control::new();
        c.stop();
        assert_eq!(c.wish(), Wish::StopAfterThisPage);

        let c = Control::new();
        c.cancel();
        assert_eq!(c.wish(), Wish::Cancel);
    }

    /// ★★★ **Cancel outranks Stop, and Stop cannot downgrade a Cancel.**
    ///
    /// Two adjacent buttons and a run the operator has decided against: the
    /// order the clicks land in must not decide whether a partial layer is
    /// written. Abandonment wins in both orders.
    #[test]
    fn cancel_wins_whichever_order_the_two_arrive_in() {
        let c = Control::new();
        c.stop();
        c.cancel();
        assert_eq!(c.wish(), Wish::Cancel, "cancel after stop must abandon");

        let c = Control::new();
        c.cancel();
        c.stop();
        assert_eq!(
            c.wish(),
            Wish::Cancel,
            "stop after cancel must NOT turn an abandonment into a partial write"
        );
    }

    /// The control is shared by clone, or the worker would read its own copy
    /// and never see a press.
    #[test]
    fn a_clone_sees_what_the_original_was_told() {
        let dialog = Control::new();
        let worker = dialog.clone();
        dialog.cancel();
        assert_eq!(worker.wish(), Wish::Cancel);
    }

    /// An unknown discriminant reads as `Continue` — the direction that keeps
    /// work rather than discarding it.
    #[test]
    fn an_unrecognised_value_continues_rather_than_cancelling() {
        assert_eq!(Wish::from_u8(0), Wish::Continue);
        assert_eq!(Wish::from_u8(7), Wish::Continue);
        assert_eq!(Wish::from_u8(255), Wish::Continue);
    }
}
