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
use pdfcer_core::preflight::{
    PdfaLevel, PdfuaPart, PdfxLevel, PreflightReport, Profile, RunOptions, Severity,
    run_with_meter, validate_pdfa_with, validate_pdfua, validate_pdfx_with,
};
use pdfcer_render::TacMeter;

/// A run in flight. Dropping it cancels the run.
pub(super) struct Job {
    cancel: Arc<AtomicBool>,
    done: Arc<AtomicUsize>,
    total: Arc<AtomicUsize>,
    answer: Receiver<Answer>,
}

/// A finished run: the findings, and for a standard's profile its verdict.
pub(super) struct Answer {
    pub report: PreflightReport,
    pub standard: Option<Standard>,
}

/// The engine's answer to "does the file conform to this standard?".
pub(super) struct Standard {
    /// `PDF/X-4`, `PDF/A-2b`, `PDF/UA-1`.
    pub claim: String,
    /// The engine's verdict token (`conforms`, `fails`, `unconfirmed`), or
    /// `None` where the engine gives no overall verdict (PDF/UA).
    pub verdict: Option<&'static str>,
    /// Error-severity findings.
    pub failed: usize,
    /// Warning-severity findings.
    pub warned: usize,
    /// Findings a person must decide.
    pub needs_person: usize,
    /// Requirements the run did not test, where the engine counts them.
    pub not_checked: Option<usize>,
    /// The PDF/X level, so the panel can offer Convert to PDF/X.
    pub pdfx: Option<PdfxLevel>,
}

/// What [`Job::poll`] found.
pub(super) enum Poll {
    /// Still running.
    Running,
    /// Finished, cancelled or not.
    Done(Answer),
    /// The worker went away without an answer.
    Lost,
}

impl Job {
    /// Start `profile` over `session` on a worker thread.
    /// `key` is the `Profile::BUILTIN_NAMES` name: a standard's `Profile::name`
    /// is its claim (`PDF/X-4`), so the key is what picks the validator.
    pub(super) fn spawn(
        session: EditSession,
        profile: Profile,
        key: &'static str,
        ctx: egui::Context,
    ) -> Self {
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
                let _ = tx.send(run(&session, &profile, key, &options));
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
            Ok(answer) => Poll::Done(answer),
            Err(TryRecvError::Empty) => Poll::Running,
            Err(TryRecvError::Disconnected) => Poll::Lost,
        }
    }
}

/// Run `profile`; a standard's profile runs through its validator, which
/// gives the same findings with clauses and the verdict.
fn run(session: &EditSession, profile: &Profile, name: &str, options: &RunOptions<'_>) -> Answer {
    let view = session.view();
    if let Some(level) = PdfxLevel::ALL
        .into_iter()
        .find(|l| l.profile_name() == name)
    {
        let r = validate_pdfx_with(&view, level, options);
        let verdict = Some(r.verdict().as_str());
        return standard(
            r.preflight,
            level.claim().to_owned(),
            verdict,
            None,
            Some(level),
        );
    }
    if let Some(level) = PdfaLevel::ALL
        .into_iter()
        .find(|l| l.profile_name() == name)
    {
        let r = validate_pdfa_with(&view, level, options);
        let verdict = Some(r.verdict().as_str());
        return standard(r.preflight, level.claim().to_owned(), verdict, None, None);
    }
    let meter = TacMeter::new();
    let report = run_with_meter(&view, profile, options, Some(&meter));
    let part = name
        .strip_prefix("pdfua-") // ui-text-exempt: an engine profile-name prefix
        .and_then(|n| n.parse().ok())
        .and_then(PdfuaPart::from_number);
    let Some(part) = part else {
        return Answer {
            report,
            standard: None,
        };
    };
    let ua = validate_pdfua(&view, part);
    let not_checked = Some(ua.not_checked.len());
    let claim = format!("PDF/UA-{}", part.number()); // ui-text-exempt: a standard's name
    standard(report, claim, None, not_checked, None)
}

fn standard(
    report: PreflightReport,
    claim: String,
    verdict: Option<&'static str>,
    not_checked: Option<usize>,
    pdfx: Option<PdfxLevel>,
) -> Answer {
    let count = |s: Severity| report.findings.iter().filter(|f| f.severity == s).count();
    let standard = Standard {
        claim,
        verdict,
        failed: count(Severity::Error) + report.dropped,
        warned: count(Severity::Warning),
        needs_person: report.needs_person().count(),
        not_checked,
        pdfx,
    };
    Answer {
        report,
        standard: Some(standard),
    }
}

impl Drop for Job {
    fn drop(&mut self) {
        self.cancel();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A standard's profile is named by its claim (`PDF/X-4`), so the
    /// validator must be picked by the picker's key or no verdict is ever given.
    #[test]
    fn every_standard_key_gives_a_verdict_line() {
        let bytes = include_bytes!("../../../../../fixtures/prepress-mix.pdf");
        let session = EditSession::new(
            pdfcer_core::document::Document::from_bytes(bytes.to_vec()).expect("the fixture opens"),
        );
        let options = RunOptions {
            max_findings: RunOptions::DEFAULT_MAX_FINDINGS,
            cancel: None,
            progress: None,
        };
        for key in Profile::BUILTIN_NAMES {
            let profile = Profile::builtin(key).expect("a built-in name");
            let answer = run(&session, &profile, key, &options);
            assert_eq!(
                answer.standard.is_some(),
                key != "digital-printing",
                "{key} (named {:?})",
                profile.name
            );
        }
    }
}
