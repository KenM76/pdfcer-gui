//! # `panels::preflight::job` — one preflight run, off the UI thread
//!
//! The worker owns a fork of the session (`EditSession::fork_for_print`: the
//! same base, a copy of the overlay, no history), so the run sees unsaved
//! edits and never holds the open document's `Arc<EditSession>` — an edit
//! made while it runs proceeds at once and only makes the answer stale.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, TryRecvError, channel};

use pdfcer_core::edit::EditSession;
use pdfcer_core::preflight::{PreflightReport, Profile, RunOptions, run_with_meter};
use pdfcer_render::TacMeter;

/// A run in flight. Dropping it cancels the run.
pub(super) struct Job {
    cancel: Arc<AtomicBool>,
    done: Arc<AtomicUsize>,
    total: Arc<AtomicUsize>,
    answer: Receiver<PreflightReport>,
}

/// What [`Job::poll`] found.
pub(super) enum Poll {
    /// Still running.
    Running,
    /// Finished, cancelled or not.
    Done(PreflightReport),
    /// The worker went away without an answer.
    Lost,
}

impl Job {
    /// Start `profile` over `session` on a worker thread.
    pub(super) fn spawn(session: EditSession, profile: Profile, ctx: egui::Context) -> Self {
        let cancel = Arc::new(AtomicBool::new(false));
        let done = Arc::new(AtomicUsize::new(0));
        let total = Arc::new(AtomicUsize::new(0));
        let (tx, answer) = channel();
        let (c, d, t) = (cancel.clone(), done.clone(), total.clone());
        let spawned = std::thread::Builder::new()
            // ui-text-exempt: a thread name, never displayed
            .name("pdfcer-preflight".to_owned())
            .spawn(move || {
                let progress = |pages: usize, of: usize| {
                    d.store(pages, Ordering::Relaxed);
                    t.store(of, Ordering::Relaxed);
                    ctx.request_repaint();
                };
                let options = RunOptions {
                    max_findings: RunOptions::DEFAULT_MAX_FINDINGS,
                    cancel: Some(&c),
                    progress: Some(&progress),
                };
                let meter = TacMeter::new();
                let report = run_with_meter(&session.view(), &profile, &options, Some(&meter));
                let _ = tx.send(report);
                ctx.request_repaint();
            });
        if let Err(e) = spawned {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!("preflight-spawn-failed error={e}")
            });
        }
        Self {
            cancel,
            done,
            total,
            answer,
        }
    }

    /// Ask the run to stop before its next page.
    pub(super) fn cancel(&self) {
        self.cancel.store(true, Ordering::Relaxed);
    }

    /// `(pages done, page count)` so far; the count is 0 before the first
    /// page finishes.
    pub(super) fn progress(&self) -> (usize, usize) {
        (
            self.done.load(Ordering::Relaxed),
            self.total.load(Ordering::Relaxed),
        )
    }

    /// Where the run is. Takes the answer when there is one.
    pub(super) fn poll(&self) -> Poll {
        match self.answer.try_recv() {
            Ok(report) => Poll::Done(report),
            Err(TryRecvError::Empty) => Poll::Running,
            Err(TryRecvError::Disconnected) => Poll::Lost,
        }
    }
}

impl Drop for Job {
    fn drop(&mut self) {
        self.cancel();
    }
}
