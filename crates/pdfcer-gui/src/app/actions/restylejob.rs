//! # `app::actions::restylejob` — a restyle spread over frames
//!
//! A restyle is one `format_text` call per show operator, and each call plans
//! against the page's whole content stream: about 100 ms a call on a dense
//! drawing, so a size change on a clicked object of a few hundred runs is
//! minutes of engine work. Done in one go that is minutes of a window that
//! neither paints nor takes input.
//!
//! A job runs the calls in slices of about [`SLICE`], one slice per frame, and
//! commits the gesture once.
//!
//! - **The picture stays.** A slice takes the session without bumping
//!   `edit_epoch`, so nothing re-renders and every cache keeps describing the
//!   picture on screen. The last call goes through the funnel, which bumps the
//!   epoch once and re-renders once.
//! - **One undo entry.** The stack is folded after every call past the first,
//!   so `Ctrl+Z` takes the gesture back whole and the history's depth limit
//!   never splits it or evicts the operator's earlier edits.
//! - **The pins are read once.** The requests arrive in the order
//!   `pin::operators_of_runs` gives them — per stream, span start descending —
//!   and a format edit moves only bytes at or after its own operator, so every
//!   pin still to come stays valid.
//! - **No edit sees the session half-restyled.** Any applied action and any
//!   other edit through the funnel first [`finish`]es the job, so a job only
//!   ever exists on the active document and a save never writes half of one.
//!   A render that lands between slices may draw it; the closing epoch bump
//!   replaces that picture.
//!
//! The job lives in a thread-local slot keyed by `OpenDoc::serial`: the UI
//! thread is its only reader.
//!
//! The engine has no batch format verb; one is requested as
//! `ENGINE_BACKLOG.md` G227. With it, the whole gesture is one plan and this
//! module becomes one call.

use std::cell::RefCell;
use std::time::{Duration, Instant};

use pdfcer_core::edit::{CommandKind, EditSession};
use pdfcer_core::settings::StylePolicy;
use pdfcer_core::text_edit::{FormatError, FormatRequest};

use super::textstyle::{emit_carried, format_in, refusal_of, resweep, stop};
use crate::app::state::OpenDoc;
use crate::app::status::decline;
use crate::text::status as t;

/// The engine time one frame may spend on a job before it yields. At least one
/// call runs per slice whatever it costs.
const SLICE: Duration = Duration::from_millis(50);

/// A restyle's calls, still to run and already run.
pub(super) struct RestyleJob {
    page: usize,
    policy: StylePolicy,
    change: &'static str,
    /// Still to run, the next at the END.
    queue: Vec<FormatRequest>,
    /// How many calls the gesture holds.
    ops: usize,
    /// How many runs the gesture covers.
    runs: usize,
    /// A run with glyphs would not pin, so the gesture stops after these.
    stopped: bool,
    applied: usize,
    carried: Vec<String>,
    failed: Option<String>,
    slices: usize,
    engine: Duration,
    started: Instant,
    /// Why the job was run to its end in one frame, if it was.
    hurried: Option<&'static str>,
}

impl RestyleJob {
    /// A job over `requests`, given in the order they must run.
    pub(super) fn new(
        page: usize,
        policy: StylePolicy,
        change: &'static str,
        mut requests: Vec<FormatRequest>,
        runs: usize,
        stopped: bool,
    ) -> Self {
        requests.reverse();
        Self {
            page,
            policy,
            change,
            ops: requests.len(),
            queue: requests,
            runs,
            stopped,
            applied: 0,
            carried: Vec::new(),
            failed: None,
            slices: 0,
            engine: Duration::ZERO,
            started: Instant::now(),
            hurried: None,
        }
    }

    /// One call on a session already taken. `false` when the engine refused.
    fn call(&mut self, session: &mut EditSession, req: &FormatRequest) -> bool {
        let began = Instant::now();
        let outcome = format_in(session, self.page, self.policy, req);
        self.engine += began.elapsed();
        match outcome {
            Ok(notes) => {
                if self.applied > 0 && !session.coalesce_last(2, CommandKind::FormatText) {
                    crate::diag::trace(|| {
                        // ui-text-exempt: diagnostic trace, never displayed in the UI
                        "text-style-fold-failed".to_owned()
                    });
                }
                self.applied += 1;
                self.carried.extend(notes);
                true
            }
            Err(error) => {
                decline::record_text_style(refusal_of(&error));
                self.failed = Some(error.to_string());
                false
            }
        }
    }

    /// Run one slice. `true` when the job is over and closed.
    fn slice(&mut self, doc: &mut OpenDoc) -> bool {
        self.slices += 1;
        let deadline = Instant::now() + SLICE;
        if self.queue.len() > 1 {
            let Some(session) = super::funnel::take_quietly(doc) else {
                self.failed = Some("the session is held elsewhere".to_owned()); // ui-text-exempt: trace detail, never displayed
                self.close(doc);
                return true;
            };
            while self.queue.len() > 1 {
                let Some(req) = self.queue.pop() else { break };
                if !self.call(session, &req) {
                    self.close(doc);
                    return true;
                }
                if Instant::now() >= deadline {
                    self.trace_slice();
                    return false;
                }
            }
        }
        self.close(doc);
        true
    }

    fn trace_slice(&self) {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!(
                "text-style-slice page={} done={} of={} engine_ms={}",
                self.page,
                self.applied,
                self.ops,
                self.engine.as_millis()
            )
        });
    }

    /// The last call through the funnel, which commits the whole gesture,
    /// then the gesture's own summary.
    fn close(&mut self, doc: &mut OpenDoc) {
        let last = if self.failed.is_none() {
            self.queue.pop()
        } else {
            None
        };
        if last.is_some() || self.applied > 0 {
            let page = self.page;
            super::apply::vector_edit(doc, "format-text", page, self.ops, |session| {
                if let Some(req) = &last
                    && !self.call(session, req)
                    && self.applied == 0
                {
                    return Err(FormatError::NoOp);
                }
                Ok(self.carried.clone())
            });
        }
        self.summarise(doc);
        resweep(doc, self.page);
    }

    /// The traces and disclosures the gesture owes, once it is over.
    fn summarise(&self, doc: &mut OpenDoc) {
        let (page, applied, runs) = (self.page, self.applied, self.runs);
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!(
                "text-style-batch page={page} ops={} applied={applied} slices={} engine_ms={} wall_ms={} hurried={}",
                self.ops,
                self.slices,
                self.engine.as_millis(),
                self.started.elapsed().as_millis(),
                self.hurried.unwrap_or("-"),
            )
        });
        if let Some(detail) = &self.failed {
            if applied > 0 {
                decline::record_text_style(t::TextStyleRefusal::PartOnly);
                emit_carried(doc, page, applied, runs, &self.carried);
            }
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!(
                    "text-style-declined page={page} applied={applied} runs={runs} detail={detail}"
                )
            });
            return;
        }
        emit_carried(doc, page, applied, runs, &self.carried);
        if self.stopped {
            stop(doc, applied, t::TextStyleRefusal::Unpinnable);
            return;
        }
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!(
                // `applied` counts show operators and `runs` counts runs.
                "text-style-applied page={page} change={} applied={applied} runs={runs}",
                self.change
            )
        });
    }
}

thread_local! {
    /// The running job and the serial of the document it belongs to.
    static JOB: RefCell<Option<(u64, RestyleJob)>> = const { RefCell::new(None) };
}

/// Start `job` on `doc`: its first slice now, the rest on later frames.
pub(super) fn start(doc: &mut OpenDoc, mut job: RestyleJob) {
    finish(doc, "restyle");
    if !job.slice(doc) {
        JOB.set(Some((doc.serial, job)));
    }
}

/// Run one slice of `doc`'s job, if it has one, and ask for the next frame.
pub(crate) fn pump(ctx: &egui::Context, doc: &mut OpenDoc) {
    let Some(mut job) = take(doc) else {
        return;
    };
    if !job.slice(doc) {
        JOB.set(Some((doc.serial, job)));
        ctx.request_repaint();
    }
}

/// Run `doc`'s job to its end now, because `why` cannot wait for it.
pub(crate) fn finish(doc: &mut OpenDoc, why: &'static str) {
    let Some(mut job) = take(doc) else {
        return;
    };
    job.hurried = Some(why);
    while !job.slice(doc) {}
}

/// `(done, of)` for `doc`'s job, for the status bar.
pub(crate) fn progress(doc: &OpenDoc) -> Option<(usize, usize)> {
    JOB.with_borrow(|slot| {
        slot.as_ref()
            .filter(|(serial, _)| *serial == doc.serial)
            .map(|(_, job)| (job.applied, job.ops))
    })
}

fn take(doc: &OpenDoc) -> Option<RestyleJob> {
    JOB.with_borrow_mut(|slot| {
        if slot
            .as_ref()
            .is_some_and(|(serial, _)| *serial == doc.serial)
        {
            slot.take().map(|(_, job)| job)
        } else {
            None
        }
    })
}
