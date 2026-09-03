//! # `ocr::job` — **a recognition running on a thread, and the two ways to end
//! it early**
//!
//! Split out of `ocr::mod` on 2026-09-01 under R2, and the seam is a real one
//! rather than a line count: this file is about **running** a recognition —
//! starting it, watching it, and telling it to stop — while its parent is about
//! **recognising**, which is a different subject with different tests and a
//! different reason to change.
//!
//! ## What lives here
//!
//! [`Job`], the handle the dialog holds; [`Tally`], the running totals it draws
//! from; and [`Reporter`], the worker's end of the same relationship. The
//! `Wish`/`Control`/`Update`/`Outcome` vocabulary those three speak is in
//! [`super::progress`], whose header carries the argument that matters most:
//! **Cancel and Stop are different acts and must never collapse into one.**
//!
//! ## ★★ The two properties this file exists to keep true
//!
//! 1. **The UI thread never blocks on the worker.** [`Job::poll`] is
//!    `try_recv` in a loop and returns immediately whether there is anything
//!    there or not.
//! 2. **Totals are accumulated by the reader, not sent by the writer.** The
//!    worker reports events — *"page 7 finished, 40 words"* — and [`Job`] folds
//!    them. A message carrying a running total could not be drained in batches
//!    safely, and this one is drained in batches on purpose.

use std::sync::mpsc::{Receiver, TryRecvError, channel};

use super::{Refusal, Request, progress, recognise};

/// A recognition running on its own thread.
///
/// Held by [`crate::dialogs::ocr`] for exactly as long as one job takes. See
/// the module header for why this is a thread and why it carries neither a
/// cancellation token nor a staleness key.
pub struct Job {
    rx: Receiver<progress::Update>,
    done: bool,
    /// What the dialog has asked the worker to do. Cloned into the thread.
    control: progress::Control,
    /// Everything the worker has reported so far, folded as it arrives.
    ///
    /// ★ Accumulated HERE rather than sent as running totals, so a batch of
    /// messages drained in one frame cannot double-count and a dropped message
    /// cannot silently lower the total. The worker reports events; the UI keeps
    /// the sum.
    seen: Tally,
}

/// The running totals a progress line is drawn from.
#[derive(Debug, Clone, Copy, Default)]
pub struct Tally {
    /// Pages attempted.
    pub attempted: usize,
    /// Pages requested.
    pub of: usize,
    /// The page most recently finished, 0-based.
    pub last_page: Option<usize>,
    /// Words so far.
    pub words: usize,
    /// Characters so far.
    pub chars: usize,
}

impl std::fmt::Debug for Job {
    /// Hand-written because [`Receiver`] is not [`Debug`] in a useful way.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Job").field("done", &self.done).finish()
    }
}

/// **The worker's end of the progress channel, plus the control it reads.**
///
/// One type rather than two arguments, because they are one relationship: the
/// worker reports upward and is told downward, and a function that took only
/// the sender could not honour a Stop.
///
/// ★ Sending is deliberately allowed to fail and is ignored. A dropped receiver
/// means the dialog is gone; the run then finishes or is abandoned on its own
/// terms and nobody is listening either way. Treating it as an error would turn
/// "the operator closed the window" into a reported fault.
pub(super) struct Reporter {
    tx: std::sync::mpsc::Sender<progress::Update>,
    control: progress::Control,
}

impl Reporter {
    /// Say a page finished.
    pub(super) fn page(&self, done: progress::PageDone) {
        drop(self.tx.send(progress::Update::Page(done)));
    }

    /// What the operator has asked for.
    pub(super) fn wish(&self) -> progress::Wish {
        self.control.wish()
    }
}

impl Job {
    /// Start recognising, and return immediately.
    ///
    /// The thread is detached rather than joined: nothing the UI does depends
    /// on it finishing, and if the dialog is closed first the channel's
    /// receiver drops, the send fails harmlessly, and the thread exits when
    /// the work it was already doing completes. The alternative — joining on
    /// close — would freeze the window for exactly as long as the operation
    /// this thread exists to keep off the window.
    #[must_use]
    pub fn spawn(request: Request) -> Self {
        let (tx, rx) = channel();
        let control = progress::Control::new();
        let of = request.pages.len();
        let worker_control = control.clone();
        std::thread::spawn(move || {
            let reporter = Reporter {
                tx: tx.clone(),
                control: worker_control,
            };
            let outcome = recognise(&request, &reporter);
            // ★★ The three endings, kept apart. `Cancelled` arrives as a
            // refusal from the loop and is turned into its own outcome here,
            // rather than reaching the dialog as "nothing was recognised" —
            // which is what every other empty result means and is not what
            // happened.
            let finished = match outcome {
                Err(Refusal::Cancelled { attempted }) => progress::Outcome::Cancelled { attempted },
                Ok(done) if done.stopped_after.is_some() => progress::Outcome::Stopped {
                    attempted: done.stopped_after.unwrap_or_default(),
                    of,
                    result: Ok(Box::new(done)),
                },
                other => progress::Outcome::Complete(other.map(Box::new)),
            };
            // A failed send means the dialog is gone. That is a normal end,
            // not an error: the operator closed the window.
            drop(tx.send(progress::Update::Finished(Box::new(finished))));
        });
        Self {
            rx,
            done: false,
            control,
            seen: Tally {
                of,
                ..Tally::default()
            },
        }
    }

    /// **Finish the page in hand, then keep everything.**
    pub fn stop(&self) {
        self.control.stop();
    }

    /// **Abandon everything, including the page in hand.**
    pub fn cancel(&self) {
        self.control.cancel();
    }

    /// What has been reported so far.
    #[must_use]
    pub const fn tally(&self) -> Tally {
        self.seen
    }

    /// The result, once it exists. `None` while the job is still running.
    ///
    /// Non-blocking, and idempotent after the answer has been taken: `done`
    /// stops a second call reading a disconnected channel and reporting the
    /// disconnection as a refusal.
    pub fn poll(&mut self) -> Option<Box<progress::Outcome>> {
        if self.done {
            return None;
        }
        // ★★ DRAIN, rather than take one. A page can finish between two frames
        // and several can finish during one slow frame; reading a single
        // message per frame would make the progress line lag the work by
        // exactly as long as the work takes, which is the appearance of a
        // freeze this feature exists to remove.
        loop {
            match self.rx.try_recv() {
                Ok(progress::Update::Page(done)) => {
                    self.seen.attempted = done.attempted;
                    self.seen.of = done.of;
                    self.seen.last_page = Some(done.index);
                    self.seen.words += done.words;
                    self.seen.chars += done.chars;
                }
                Ok(progress::Update::Finished(outcome)) => {
                    self.done = true;
                    return Some(outcome);
                }
                Err(TryRecvError::Empty) => return None,
                Err(TryRecvError::Disconnected) => {
                    self.done = true;
                    return Some(Box::new(progress::Outcome::Complete(Err(Refusal::Engine(
                        // ui-text-exempt: reached only through `text::ocr::failed`,
                        // which is the catalog entry an operator actually reads.
                        "the recogniser stopped without reporting a result".to_owned(),
                    )))));
                }
            }
        }
    }

    /// Whether the job is still running.
    #[must_use]
    pub const fn is_running(&self) -> bool {
        !self.done
    }
}
