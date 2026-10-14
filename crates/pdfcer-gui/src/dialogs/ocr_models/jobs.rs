//! The OCR models window's work off the UI thread: removals, installs and
//! downloads, each answering once.

use std::path::PathBuf;
use std::sync::mpsc;

use crate::ocr::catalog::Choice;
use crate::ocr::manage::{self, InstallError, Installed, RemoveError, Source};
use crate::text::ocrmanage as t;

use super::listing::Listing;

/// How a job ended.
#[derive(Debug)]
pub(super) enum Finished {
    /// Removals ran; `names` are the ones that went.
    Removed {
        names: Vec<String>,
        labels: Vec<String>,
        freed: u64,
        /// The first failure: the model's label and why.
        failure: Option<(String, RemoveError)>,
        writable: bool,
    },
    /// Boxed: an install's answer is several times a removal's.
    Installed(Box<Result<Installed, InstallError>>),
    #[cfg(feature = "model-download")]
    Downloaded {
        engine: &'static str,
        outcome: crate::ocr::fetch::Outcome,
        attribution: String,
    },
}

/// A job running on its own thread.
#[derive(Debug)]
pub(super) struct Job {
    rx: mpsc::Receiver<Finished>,
}

impl Job {
    fn spawn(work: impl FnOnce() -> Finished + Send + 'static) -> Self {
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(work());
        });
        Self { rx }
    }

    /// Remove `models` in order, stopping at the first refusal.
    pub(super) fn remove(models: Vec<Choice>, writable: bool) -> Self {
        Self::spawn(move || {
            let mut out = Finished::Removed {
                names: Vec::new(),
                labels: Vec::new(),
                freed: 0,
                failure: None,
                writable,
            };
            let Finished::Removed {
                names,
                labels,
                freed,
                failure,
                ..
            } = &mut out
            else {
                return out;
            };
            for c in &models {
                let label = crate::text::ocrmodels::label(c);
                match manage::remove(&c.model) {
                    Ok(bytes) => {
                        names.push(c.name.clone());
                        labels.push(label);
                        *freed += bytes;
                    }
                    Err(e) => {
                        *failure = Some((label, e));
                        break;
                    }
                }
            }
            out
        })
    }

    /// Install `source` into `models`.
    pub(super) fn install(source: Source, models: PathBuf) -> Self {
        Self::spawn(move || Finished::Installed(Box::new(manage::install(&source, &models))))
    }

    /// Fetch `set`'s pinned files into `dir`.
    #[cfg(feature = "model-download")]
    pub(super) fn download(
        set: &'static pdfcer_core::ocr::models::FetchableModels,
        dir: PathBuf,
    ) -> Self {
        Self::spawn(move || {
            let download = crate::ocr::fetch::Download::start(set, dir);
            let outcome = loop {
                if let Some(o) = download.poll() {
                    break o;
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            };
            Finished::Downloaded {
                engine: set.engine,
                outcome,
                attribution: set.attribution(),
            }
        })
    }

    /// The outcome, once the thread has finished.
    pub(super) fn poll(&self) -> Option<Finished> {
        match self.rx.try_recv() {
            Ok(f) => Some(f),
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => {
                Some(Finished::Installed(Box::new(Err(InstallError::Io(
                    PathBuf::new(),
                    std::io::Error::other(crate::text::ocrfetch::stopped()),
                )))))
            }
        }
    }
}

/// The sentence a finished job leaves, traced; `default_before` is the
/// model Recognise text started with before it, `after` the new discovery.
pub(super) fn sentence(
    finished: &Finished,
    default_before: Option<&str>,
    after: &Listing,
) -> String {
    match finished {
        Finished::Installed(result) => installed(result, after),
        Finished::Removed {
            names,
            labels,
            freed,
            failure,
            writable,
        } => removed(
            names,
            labels,
            *freed,
            failure.as_ref(),
            *writable,
            default_before,
            after,
        ),
        #[cfg(feature = "model-download")]
        Finished::Downloaded { .. } => downloaded(finished),
    }
}

fn installed(result: &Result<Installed, InstallError>, after: &Listing) -> String {
    match result {
        Ok(done) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!(
                    "ocr-models-installed name={} checked={} bytes={} folder={}",
                    done.model.name,
                    done.checked,
                    done.bytes,
                    done.model.folder.display()
                )
            });
            let label = after
                .catalog
                .choices
                .iter()
                .find(|c| c.name == done.model.name)
                .map_or_else(|| done.model.name.clone(), crate::text::ocrmodels::label);
            t::installed(&label, done.bytes, done.checked)
        }
        Err(e) => {
            // ui-text-exempt: diagnostic trace, never displayed.
            crate::diag::trace(|| format!("ocr-models-install-failed why={}", install_token(e)));
            t::install_failed(e)
        }
    }
}

#[cfg(feature = "model-download")]
fn downloaded(finished: &Finished) -> String {
    let Finished::Downloaded {
        engine,
        outcome,
        attribution,
    } = finished
    else {
        unreachable!("only a download reaches here") // ui-text-exempt: a panic message, never displayed
    };
    match outcome {
        crate::ocr::fetch::Outcome::Done { files } => {
            // ui-text-exempt: diagnostic trace, never displayed
            crate::diag::trace(|| format!("fetch-models-done engine={engine} files={files}"));
            crate::text::ocrfetch::done(*files, attribution)
        }
        crate::ocr::fetch::Outcome::Failed(why) => {
            // ui-text-exempt: diagnostic trace, never displayed
            crate::diag::trace(|| format!("fetch-models-failed engine={engine} why={why}"));
            crate::text::ocrfetch::failed(why)
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn removed(
    names: &[String],
    labels: &[String],
    freed: u64,
    failure: Option<&(String, RemoveError)>,
    writable: bool,
    default_before: Option<&str>,
    after: &Listing,
) -> String {
    let default_gone = default_before.is_some_and(|d| names.iter().any(|n| n == d));
    let then = default_gone.then(|| {
        after.default.map_or_else(
            || t::no_default_left().to_owned(),
            |i| t::new_default(&crate::text::ocrmodels::label(&after.catalog.choices[i])),
        )
    });
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "ocr-models-removed names={} freed={freed} default={} failed={}",
            if names.is_empty() {
                "none".to_owned()
            } else {
                names.join(",")
            },
            if default_gone {
                after.default_name().unwrap_or("none")
            } else {
                "unchanged"
            },
            failure.map_or("none", |(_, e)| remove_token(e)),
        )
    });
    let done = match labels {
        [] => None,
        [one] => Some(t::removed(one, freed, then.as_deref())),
        many => Some(t::removed_all(many.len(), freed, then.as_deref())),
    };
    let failed = failure.map(|(label, e)| t::remove_failed(label, e, writable));
    t::both(done, failed)
}

// ui-text-exempt: trace tokens, never displayed.
const fn remove_token(e: &RemoveError) -> &'static str {
    match e {
        RemoveError::IsRoot(_) => "is-root",
        RemoveError::Gone(_) => "gone",
        RemoveError::Changed(_) => "changed",
        RemoveError::Refused(..) => "refused",
        RemoveError::Partial(..) => "partial",
    }
}

// ui-text-exempt: trace tokens, never displayed.
const fn install_token(e: &InstallError) -> &'static str {
    match e {
        InstallError::Zip(_) => "zip",
        InstallError::Io(..) => "io",
        InstallError::NoModel(_) => "no-model",
        InstallError::Several(_) => "several",
        InstallError::Verify(_) => "verify",
        InstallError::Exists(..) => "exists",
    }
}
