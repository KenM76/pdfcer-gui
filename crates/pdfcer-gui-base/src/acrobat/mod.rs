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
///
/// **Pro beats Reader**, and [`Edition::rank`] is where that is written down.
/// Pro is the superset: somebody who has both installed reached for Pro when
/// they bought it, and a button that sent them to Reader would be answering a
/// question they did not ask. Reader is the fallback, not the preference.
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
///
/// Carried on the [`Viewer`] rather than discarded, for two reasons that are
/// both about the operator rather than about the code: Settings shows it, so a
/// person who cannot tell whether their typed path is being used can look; and
/// a trace naming the source turns *"the wrong program opened"* into a
/// one-line diagnosis.
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
    ///
    /// The ordering between [`Self::AppPaths`] and [`Self::PdfHandler`] only
    /// ever decides between two candidates of the **same** edition, because
    /// [`resolve`] sorts on [`Edition::rank`] first. A Reader found in
    /// `App Paths` therefore does **not** beat a Pro found through the `.pdf`
    /// handler, which is the operator's stated preference and is asserted by
    /// `pro_beats_reader_even_when_reader_is_the_registered_handler`.
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
///
/// Constructing one is a **claim that the executable existed** at the moment
/// discovery ran — see [`Registrations::exists`] and this module's §4. It is
/// not a claim that it still exists when the operator finally presses the
/// button, which is why [`launch`] reports failure rather than assuming
/// success.
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
///
/// # Why `exists` is on this trait and not `Path::exists`
///
/// It is the same kind of fact as the other two: something only the real
/// machine can answer, which a test must be able to state. A [`resolve`] that
/// called `Path::exists` directly would be untestable in precisely the case
/// that matters most — *the registry names an Acrobat that has been
/// uninstalled* — because a test could only produce that state by creating and
/// deleting real files at real paths.
pub trait Registrations {
    /// The `App Paths` default value for `executable`, from any of the three
    /// roots, or `None` if no root registers it.
    ///
    /// Implementations return the value **as the registry holds it**: quoting,
    /// surrounding whitespace and all. Cleaning it up is [`discover`]'s job,
    /// so that the cleaning is tested.
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
///
/// Separate from [`Registrations`] because the two are used at different
/// times by different code: discovery runs when the shell starts and when a
/// setting changes, launching runs when the operator presses a button. A
/// single "platform" trait would force every test that cares about one to
/// stub the other.
pub trait Launcher {
    /// Start `viewer` with `file` on its command line.
    ///
    /// # Errors
    ///
    /// Whatever the platform reports: the executable has been removed since
    /// discovery, the operator lacks permission, the process table is full.
    /// The caller words it; this trait does not.
    fn launch(&self, viewer: &Viewer, file: &Path) -> std::io::Result<()>;
}

/// **Which Acrobat, if any.**
///
/// The whole decision, as a pure function over [`Registrations`]. See this
/// module's §4 for the sources and §3 for why the impurity is behind a trait.
///
/// `configured` is the operator's Settings value. An empty or whitespace-only
/// string means *"not configured"* rather than *"configured to nothing"*:
/// clearing a text field is how a person un-sets it, and reading a cleared
/// field as a path would turn the escape hatch into a trap that permanently
/// suppresses the button.
///
/// # Why a configured path that does not exist yields `None` rather than a
/// `Viewer`
///
/// It is tempting to honour whatever the operator typed on the grounds that
/// they know their own machine. But the failure that produces — a button that
/// is present and does nothing — is worse than the failure it avoids, and the
/// operator has no way to tell the two apart from the ribbon. The escape hatch
/// still works: **Settings shows what discovery resolved**, so a typo is
/// visible where it was made, next to the field that caused it. See
/// `pdfcer_gui::dialogs::settings`.
///
/// # A configured path does NOT fall back to discovery
///
/// If the operator typed a path and it does not exist, [`resolve`] answers
/// `None` — it does not quietly go and find a different Acrobat. Falling back
/// would mean a person who deliberately pointed pdfcer at their second
/// installation gets silently sent to their first one, with nothing on screen
/// saying so, which is the whole reason the setting exists being undone by the
/// code that implements it.
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
///
/// Three variants because there are three genuinely different situations, and
/// collapsing any two of them produces a sentence that is false in one of
/// them. See this module's §2.
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
///
/// A pure function over two `bool`s so the branch is asserted rather than
/// inferred from a screenshot. Both facts come from
/// `pdfcer_gui::app::save` — `has_a_file` and `has_unsaved_edits` — and that is
/// deliberate: *"does this document have unsaved edits?"* already has exactly
/// one answer in the application, and a second one here would be a second
/// thing to keep in step with the tab strip's unsaved marker.
///
/// `has_file` is asked **first**, and the order is the whole content of the
/// function. A never-saved document is also a dirty one, so testing dirtiness
/// first would offer *"Save and open"* over a document with nowhere to save
/// to — a button that either does nothing or silently opens a file picker the
/// operator did not ask for.
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
///
/// A thin wrapper over the [`Launcher`] seam, present so that call sites read
/// as intent and so the trace line has one home. The ordering around it is the
/// caller's to keep, not this function's: the document is closed **after** a
/// successful spawn, because a launch that failed after the close would leave
/// the operator with no document on screen and no Acrobat either. See
/// `pdfcer_gui::app::actions::acrobat`.
///
/// # Errors
///
/// Propagates the [`Launcher`]'s error unchanged.
pub fn launch(launcher: &dyn Launcher, viewer: &Viewer, file: &Path) -> std::io::Result<()> {
    launcher.launch(viewer, file)
}
