//! The worker's second slot: one low-priority render (a page preview) that runs
//! beside the canvas's own and never waits on the UI thread.
//!
//! It lives on [`RenderWorker`] rather than on a worker of its own so that
//! [`RenderWorker::cancel_and_wait`], which every edit calls before
//! `Arc::get_mut` on the session, stops it too: a background render holds an
//! `Arc<EditSession>` exactly as the canvas's does.
//!
//! The slot sits behind a `RefCell` because the page rail reads the document
//! through `&OpenDoc`; every method here takes `&self`.

use std::cell::RefCell;
use std::sync::mpsc::{TryRecvError, sync_channel};
use std::time::{Duration, Instant};

use pdfcer_render::cancel::RenderCancel;

use super::{InFlight, Outcome, RenderKey, RenderOutcome, RenderRefusal, RenderRequest};
use super::{RefusalReason, RenderWorker, render_on_worker};

/// What a finished background render reports.
pub struct BackgroundResult {
    /// What was rendered: the request's page, scale and display options.
    pub key: RenderKey,
    /// The pixels, or the refusal. `None` when the render was cancelled.
    pub outcome: Option<RenderOutcome>,
    /// From spawn to collection.
    pub elapsed: Duration,
}

/// The slot. `Default` is empty.
#[derive(Default)]
pub(super) struct BackgroundSlot(RefCell<Option<InFlight>>);

impl BackgroundSlot {
    /// Cancel, drain and join whatever is running. Idempotent.
    pub(super) fn cancel(&self) {
        let Some(mut flight) = self.0.borrow_mut().take() else {
            return;
        };
        flight.cancel.cancel();
        if let Some(handle) = flight.handle.take() {
            let _ = handle.join();
        }
    }
}

impl RenderWorker {
    /// Start `request` in the background slot, replacing whatever was there,
    /// and return at once. Returns `false` when exactly this render is already
    /// running.
    ///
    /// Never touches the canvas slot, and the canvas slot never touches this
    /// one; only [`Self::cancel_and_wait`] stops both.
    pub fn spawn_background(&self, request: RenderRequest) -> bool {
        let key = RenderKey::of(&request);
        if self
            .background
            .0
            .borrow()
            .as_ref()
            .is_some_and(|f| f.key == key)
        {
            return false;
        }
        self.background.cancel();
        let generation = self.next_background.get().wrapping_add(1);
        self.next_background.set(generation);
        let cancel = RenderCancel::new();
        let worker_cancel = cancel.clone();
        let (tx, rx) = sync_channel(1);
        let page = request.page_index;
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        crate::diag::trace(|| format!("render-background-spawn gen={generation} page={page}"));
        let handle = std::thread::spawn(move || {
            let _ = tx.send(render_on_worker(&request, &worker_cancel));
        });
        *self.background.0.borrow_mut() = Some(InFlight {
            rx,
            cancel,
            handle: Some(handle),
            key,
            generation,
            started: Instant::now(),
        });
        true
    }

    /// Collect the background render if it has finished. Never blocks.
    pub fn poll_background(&self) -> Option<BackgroundResult> {
        let mut slot = self.background.0.borrow_mut();
        let outcome = match slot.as_ref()?.rx.try_recv() {
            Ok(outcome) => Some(outcome),
            Err(TryRecvError::Empty) => return None,
            Err(TryRecvError::Disconnected) => None,
        };
        let mut flight = slot.take()?;
        drop(slot);
        if let Some(handle) = flight.handle.take() {
            let _ = handle.join();
        }
        let outcome = match outcome {
            Some(Outcome::Done(pixels)) => Some(Ok(*pixels)),
            Some(Outcome::Failed(refusal)) => Some(Err(refusal)),
            Some(Outcome::Cancelled) => None,
            None => Some(Err(RenderRefusal::other(RefusalReason::WorkerStopped))),
        };
        Some(BackgroundResult {
            key: flight.key,
            outcome,
            elapsed: flight.started.elapsed(),
        })
    }

    /// What the background slot is rendering and for how long, if anything.
    #[must_use]
    pub fn background_in_flight(&self) -> Option<(RenderKey, Duration)> {
        self.background
            .0
            .borrow()
            .as_ref()
            .map(|f| (f.key, f.started.elapsed()))
    }

    /// Stop the background render only, leaving the canvas's running.
    pub fn cancel_background(&self) {
        self.background.cancel();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_slot_reports_nothing() {
        let worker = RenderWorker::default();
        assert!(worker.poll_background().is_none());
        assert!(worker.background_in_flight().is_none());
        worker.cancel_background();
    }
}
