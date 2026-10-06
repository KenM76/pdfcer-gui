//! # `ocr::fetch` — downloading an engine's pinned model files
//!
//! The list is the engine's, `pdfcer_core::ocr::models::pinned::FETCHABLE_MODELS`;
//! each file is fetched with `pdfcer_fetch::fetch_verified`, which hashes the
//! bytes against the pin before anything is written. The work runs on its own
//! thread so the window keeps drawing.
//!
//! Contract:
//! - [`target`] is `<exe dir>/models/<folder>`, the bundled folder
//!   `ocr::catalog` searches first, so a downloaded model is found by the next
//!   Recognise text with no setting changed.
//! - [`Download::poll`] answers once, with every file written or the first
//!   failure; files fetched before a failure stay (each is verified).
//! - Present only with the `model-download` feature; the command that opens
//!   the window is registered under the same feature.

use std::path::{Path, PathBuf};
use std::sync::mpsc;

use pdfcer_core::ocr::models::FetchableModels;
use pdfcer_fetch::{PinnedArtifact, fetch_verified};

/// Where `set`'s files go: `<exe dir>/models/<folder>`.
#[must_use]
pub fn target(set: &FetchableModels) -> Option<PathBuf> {
    super::exe_dir().map(|d| target_in(&d, set))
}

/// `set`'s folder under the program folder `exe_dir`.
#[must_use]
pub fn target_in(exe_dir: &Path, set: &FetchableModels) -> PathBuf {
    exe_dir.join(super::catalog::BUNDLED_DIR).join(set.folder)
}

/// How a download ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// Every file was verified and written.
    Done {
        /// How many files.
        files: usize,
    },
    /// A file could not be fetched, verified or written: the engine's words.
    Failed(String),
}

/// A download running on its own thread.
#[derive(Debug)]
pub struct Download {
    rx: mpsc::Receiver<Outcome>,
}

impl Download {
    /// Fetch every file of `set` into `dir`, creating it.
    #[must_use]
    pub fn start(set: &'static FetchableModels, dir: PathBuf) -> Self {
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(run(set, &dir));
        });
        Self { rx }
    }

    /// The outcome, once the thread has finished.
    #[must_use]
    pub fn poll(&self) -> Option<Outcome> {
        match self.rx.try_recv() {
            Ok(outcome) => Some(outcome),
            Err(mpsc::TryRecvError::Empty) => None,
            // A panicked worker sends nothing; never leave the window waiting.
            Err(mpsc::TryRecvError::Disconnected) => {
                Some(Outcome::Failed(crate::text::ocrfetch::stopped().to_owned()))
            }
        }
    }
}

fn run(set: &FetchableModels, dir: &Path) -> Outcome {
    if let Err(e) = std::fs::create_dir_all(dir) {
        return Outcome::Failed(format!("{}: {e}", dir.display())); // ui-text-exempt: the engine-words slot of text::ocrfetch::failed
    }
    for f in set.files {
        let art = PinnedArtifact::new(f.url, f.sha256, f.file_name);
        if let Err(e) = fetch_verified(&art, dir) {
            return Outcome::Failed(e.to_string());
        }
    }
    Outcome::Done {
        files: set.files.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_target_is_the_bundled_folder_the_catalog_searches() {
        let set = pdfcer_core::ocr::models::fetchable_models("ocrs");
        let dir = set.map(|s| target_in(Path::new("C:/p"), s));
        assert_eq!(dir, Some(PathBuf::from("C:/p/models/ocrs")));
    }
}
