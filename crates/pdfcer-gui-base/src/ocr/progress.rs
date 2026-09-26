//! # `ocr::progress` — **what the recogniser is doing, and the two ways to end
//! it early**
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/ocr/progress.md`.

use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};

/// What the operator has asked the running job to do.
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
    #[must_use]
    pub fn wish(&self) -> Wish {
        Wish::from_u8(self.0.load(Ordering::Relaxed))
    }

    /// **Finish the page in hand and keep everything.**
    pub fn stop(&self) {
        let _ = self.0.compare_exchange(
            Wish::Continue as u8,
            Wish::StopAfterThisPage as u8,
            Ordering::Relaxed,
            Ordering::Relaxed,
        );
    }

    /// **Abandon everything**, including the page in hand.
    pub fn cancel(&self) {
        self.0.store(Wish::Cancel as u8, Ordering::Relaxed);
    }
}

/// One message from the worker.
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
    /// Asked for by name. It is the more responsive of the two on a dense
    /// drawing — a page can produce hundreds of characters in a handful of
    /// "words" — so it is the number that best shows the thing is alive.
    pub chars: usize,
}

/// How a run ended.
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

    /// **Cancel outranks Stop, and Stop cannot downgrade a Cancel.**
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
