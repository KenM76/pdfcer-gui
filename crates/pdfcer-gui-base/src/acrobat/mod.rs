//! # `acrobat` — finding the operator's Acrobat, and handing the file over to
//! it
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/acrobat/mod.md`.

pub mod discover;
pub mod windows;

#[cfg(test)]
mod tests;

use std::path::{Path, PathBuf};

/// Which Acrobat this is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Edition {
    /// Acrobat Pro / Acrobat DC — `Acrobat.exe`.
    Pro,
    /// Acrobat Reader — `AcroRd32.exe`.
    Reader,
}

impl Edition {
    /// The executable name Windows registers this edition under.
    #[must_use]
    pub const fn executable(self) -> &'static str {
        match self {
            // ui-text-exempt: a registry key name and a file name on disk,
            // never displayed.
            Self::Pro => "Acrobat.exe",
            // ui-text-exempt: a registry key name and a file name on disk,
            // never displayed.
            Self::Reader => "AcroRd32.exe",
        }
    }

    /// Preference order — **lower wins**.
    ///
    #[must_use]
    pub const fn rank(self) -> u8 {
        match self {
            Self::Pro => 0,
            Self::Reader => 1,
        }
    }
}

/// Where a candidate came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Source {
    /// The path the operator typed into Settings.
    Configured,
    /// An `App Paths` registration — the key Windows itself resolves.
    AppPaths,
    /// The registered `.pdf` handler's command line.
    PdfHandler,
}

impl Source {
    /// Preference order — **lower wins**, and it is a *tie-break*, not the
    /// first sort key.
    #[must_use]
    pub const fn rank(self) -> u8 {
        match self {
            Self::Configured => 0,
            Self::AppPaths => 1,
            Self::PdfHandler => 2,
        }
    }
}

/// An Acrobat that is installed, verified on disk, and ready to be handed a
/// file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Viewer {
    /// The executable to run.
    pub path: PathBuf,
    /// Pro or Reader.
    pub edition: Edition,
    /// Which of the three sources produced it.
    pub source: Source,
}

/// The two registry questions this module asks, and the one filesystem
/// question — the seam that keeps [`resolve`] pure.
pub trait Registrations {
    /// The `App Paths` default value for `executable`, from any of the three
    /// roots, or `None` if no root registers it.
    fn app_path(&self, executable: &str) -> Option<String>;

    /// The registered `.pdf` handler's `shell\open\command`, raw.
    ///
    /// Typically `"C:\…\Acrobat.exe" "%1"`. May name any program at all — see
    /// this module's §4 — so the caller filters it.
    fn pdf_handler_command(&self) -> Option<String>;

    /// Whether `path` is a file that exists right now.
    fn exists(&self, path: &Path) -> bool;
}

/// Starting the viewer — the other seam.
pub trait Launcher {
    /// Start `viewer` with `file` on its command line.
    fn launch(&self, viewer: &Viewer, file: &Path) -> std::io::Result<()>;
}

/// **Which Acrobat, if any.**
#[must_use]
pub fn resolve(registrations: &dyn Registrations, configured: Option<&str>) -> Option<Viewer> {
    if let Some(typed) = configured.map(str::trim).filter(|s| !s.is_empty()) {
        let path = PathBuf::from(typed);
        if !registrations.exists(&path) {
            return None;
        }
        let edition = discover::edition_of(&path).unwrap_or(Edition::Pro);
        return Some(Viewer {
            path,
            edition,
            source: Source::Configured,
        });
    }

    let mut candidates: Vec<Viewer> = Vec::new();

    for edition in [Edition::Pro, Edition::Reader] {
        if let Some(raw) = registrations.app_path(edition.executable())
            && let Some(path) = discover::executable_from_registration(&raw)
            && registrations.exists(&path)
        {
            candidates.push(Viewer {
                path,
                edition,
                source: Source::AppPaths,
            });
        }
    }

    if let Some(raw) = registrations.pdf_handler_command()
        && let Some(path) = discover::executable_from_command(&raw)
        // The filter this module's §4 exists for: the registered handler is
        // whatever opens PDFs here, which on the operator's own machine is
        // another vendor's product.
        && let Some(edition) = discover::edition_of(&path)
        && registrations.exists(&path)
    {
        candidates.push(Viewer {
            path,
            edition,
            source: Source::PdfHandler,
        });
    }

    // Edition first, source second. See `Source::rank` on why that order is
    // the operator's preference rather than an arbitrary one.
    candidates.sort_by_key(|v| (v.edition.rank(), v.source.rank()));
    candidates.into_iter().next()
}

/// What the operator must be told before the document is handed over.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Prompt {
    /// **The document has never been written anywhere**, so there is no file
    /// for Acrobat to open.
    ///
    /// A refusal, not a question — there is nothing to confirm and no button
    /// that would make it work. It is its own variant rather than being folded
    /// into "unsaved edits" because the two need different sentences: *"save
    /// first"* implies a destination that this document does not have, and an
    /// operator told to save something that has never been saved will look for
    /// a Save button that would have to ask them where.
    NoFileOnDisk,
    /// **Unsaved edits.** Save over the open file and then hand it over, or
    /// cancel. There is no third answer — see this module's §2.
    SaveFirst,
    /// **Clean.** Say that the document will be closed, and take OK or Cancel.
    ConfirmClose,
}

/// **Which of the three the operator gets**, from the two facts about the open
/// document.
#[must_use]
pub const fn prompt_for(has_file: bool, has_unsaved_edits: bool) -> Prompt {
    if !has_file {
        Prompt::NoFileOnDisk
    } else if has_unsaved_edits {
        Prompt::SaveFirst
    } else {
        Prompt::ConfirmClose
    }
}

/// Hand `file` to `viewer`.
pub fn launch(launcher: &dyn Launcher, viewer: &Viewer, file: &Path) -> std::io::Result<()> {
    launcher.launch(viewer, file)
}
