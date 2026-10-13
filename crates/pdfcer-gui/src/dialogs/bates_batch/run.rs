//! # `dialogs::bates_batch::run` — the worker half of Bates numbering across files
//!
//! Two jobs, each on its own thread so the window never blocks: [`probe`]
//! reads one file and reports its page count or why the engine would refuse
//! it; [`run`] stamps every file in order and writes the copies.
//!
//! A run is all-or-nothing up to the last step: each copy is written to a
//! `.pdfcer-part` file beside its target, and only when every file has been
//! numbered are the parts renamed into place. A file that fails mid-batch
//! therefore leaves no copies, which matters because the copies after it
//! would carry numbers that assumed it was there.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/bates_batch.md`.

use std::path::PathBuf;
use std::sync::mpsc::Sender;

use pdfcer_core::bates::BatesStamp;
use pdfcer_core::document::{DocError, Document};
use pdfcer_core::settings::Settings;

use crate::app::settings::SettingsExt;
use crate::text::bates_batch as t;

/// What the worker tells the window.
pub enum Msg {
    /// `path` has `Ok(pages)`, or the reason it cannot be numbered.
    Probed(PathBuf, Result<usize, String>),
    /// File `index` of the run is numbered and its part written.
    Wrote(usize),
    /// The run ended.
    Finished(Outcome),
}

/// How a run ended.
pub enum Outcome {
    Done {
        files: usize,
        pages: usize,
        first: String,
        last: String,
        next: u64,
    },
    /// Nothing written: these copies already exist.
    Exists(Vec<String>),
    /// Nothing written: this file failed.
    Failed { name: String, detail: String },
    /// The rename step failed after `written` copies were placed.
    Partly {
        written: usize,
        of: usize,
        detail: String,
    },
}

/// One file of a run.
pub struct Job {
    pub source: PathBuf,
    pub target: PathBuf,
    pub first: u64,
}

fn name_of(p: &std::path::Path) -> String {
    p.file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// Read `path` and decide whether it can be numbered, the way a run would.
///
/// The trial stamp is the engine's own preflight: it refuses an encrypted
/// document, a certification forbidding page changes, and a page whose new
/// object would expose hidden ones before writing anything, so the window
/// reports those per file instead of the run failing half way.
pub fn probe(settings: &Settings, path: &std::path::Path) -> Result<usize, String> {
    let doc = match Document::load(path) {
        Ok(doc) => doc,
        Err(DocError::PasswordRequired | DocError::PasswordRequiresNormalisation) => {
            return Err(t::needs_password().to_owned());
        }
        Err(e) => return Err(t::unreadable(&e.to_string())),
    };
    let mut session = settings.open_session(doc);
    let signatures = session.signature_census().signatures;
    if signatures > 0 {
        return Err(t::signed(signatures));
    }
    let pages = session
        .pages()
        .map_err(|e| t::unreadable(&e.to_string()))?
        .len();
    session
        .stamp_bates(&BatesStamp::default(), 1)
        .map_err(|e| t::engine_refused(&e.to_string()))?;
    Ok(pages)
}

/// Start probing `paths` on a worker; one [`Msg::Probed`] each.
pub fn spawn_probe(ctx: &egui::Context, settings: Settings, paths: Vec<PathBuf>, tx: Sender<Msg>) {
    let repaint = ctx.clone();
    std::thread::spawn(move || {
        for path in paths {
            let verdict = probe(&settings, &path);
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "bates-batch-probed path={path:?} pages={} refused={}",
                    verdict.as_ref().map_or(0, |n| *n),
                    u8::from(verdict.is_err()),
                )
            });
            if tx.send(Msg::Probed(path, verdict)).is_err() {
                return;
            }
            repaint.request_repaint();
        }
    });
}

/// Start the run on a worker; it ends with one [`Msg::Finished`].
pub fn spawn_run(
    ctx: &egui::Context,
    settings: Settings,
    jobs: Vec<Job>,
    stamp: BatesStamp,
    replace: bool,
    tx: Sender<Msg>,
) {
    let repaint = ctx.clone();
    std::thread::spawn(move || {
        let outcome = run(&settings, &jobs, &stamp, replace, &|i| {
            let _ = tx.send(Msg::Wrote(i));
            repaint.request_repaint();
        });
        trace_outcome(&outcome);
        let _ = tx.send(Msg::Finished(outcome));
        repaint.request_repaint();
    });
}

fn trace_outcome(outcome: &Outcome) {
    crate::diag::trace(|| match outcome {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        Outcome::Done {
            files,
            pages,
            first,
            last,
            next,
        } => {
            format!(
                "bates-batch-done files={files} pages={pages} first_label={first} last_label={last} next={next}"
            )
        }
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        Outcome::Exists(names) => format!("bates-batch-refused reason=exists n={}", names.len()),
        Outcome::Failed { name, detail } => {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!("bates-batch-refused reason=failed name={name:?} detail={detail:?}")
        }
        Outcome::Partly { written, of, .. } => {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!("bates-batch-refused reason=partly written={written} of={of}")
        }
    });
}

/// The part file `target` is first written as.
fn part_of(target: &std::path::Path) -> PathBuf {
    let mut name = target.as_os_str().to_owned();
    name.push(".pdfcer-part"); // ui-text-exempt: a temporary file-name suffix
    PathBuf::from(name)
}

/// Number every job in order, then put the copies in place.
pub fn run(
    settings: &Settings,
    jobs: &[Job],
    stamp: &BatesStamp,
    replace: bool,
    wrote: &dyn Fn(usize),
) -> Outcome {
    if !replace {
        let existing: Vec<String> = jobs
            .iter()
            .filter(|j| j.target.exists())
            .map(|j| name_of(&j.target))
            .collect();
        if !existing.is_empty() {
            return Outcome::Exists(existing);
        }
    }
    let mut parts: Vec<PathBuf> = Vec::with_capacity(jobs.len());
    let mut labels: Option<(String, String)> = None;
    let (mut pages, mut next) = (0, 0);
    let fail = |parts: &[PathBuf], job: &Job, detail: String| {
        for p in parts {
            let _ = std::fs::remove_file(p);
        }
        Outcome::Failed {
            name: name_of(&job.source),
            detail,
        }
    };
    for (index, job) in jobs.iter().enumerate() {
        let doc = match Document::load(&job.source) {
            Ok(doc) => doc,
            Err(e) => return fail(&parts, job, e.to_string()),
        };
        let mut session = settings.open_session(doc);
        let outcome = match session.stamp_bates(stamp, job.first) {
            Ok(o) => o,
            Err(e) => return fail(&parts, job, e.to_string()),
        };
        let bytes = match session.to_incremental_bytes(&settings.save_options()) {
            Ok((bytes, _)) => bytes,
            Err(e) => return fail(&parts, job, e.to_string()),
        };
        let part = part_of(&job.target);
        if let Err(e) = std::fs::write(&part, &bytes) {
            let _ = std::fs::remove_file(&part);
            return fail(&parts, job, e.to_string());
        }
        parts.push(part);
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "bates-batch-wrote index={index} source={:?} target={:?} pages={} first_label={} last_label={} next={}",
                name_of(&job.source),
                name_of(&job.target),
                outcome.pages.len(),
                outcome.first_label,
                outcome.last_label,
                outcome.next,
            )
        });
        pages += outcome.pages.len();
        next = outcome.next;
        let first = labels.take().map_or(outcome.first_label, |(f, _)| f);
        labels = Some((first, outcome.last_label));
        wrote(index);
    }
    for (placed, (part, job)) in parts.iter().zip(jobs).enumerate() {
        if let Err(e) = std::fs::rename(part, &job.target) {
            for rest in &parts[placed..] {
                let _ = std::fs::remove_file(rest);
            }
            return Outcome::Partly {
                written: placed,
                of: jobs.len(),
                detail: e.to_string(),
            };
        }
    }
    let (first, last) = labels.unwrap_or_default();
    Outcome::Done {
        files: jobs.len(),
        pages,
        first,
        last,
        next,
    }
}
