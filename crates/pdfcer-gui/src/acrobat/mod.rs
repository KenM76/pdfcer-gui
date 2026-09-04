//! # `acrobat` — finding the operator's Acrobat, and handing the file over to
//! it
//!
//! `OPERATOR_REQUESTS.md` **O122**, the operator, 2026-09-04:
//!
//! > *"also beside our read-review-edit buttons at the top there should be an
//! > open in acrobat button which will open the active pdf in acrobat reader or
//! > pro depending on what is installed - we'll have to add a feature to
//! > automatically locate and open the installed acrobat on the system, and
//! > have a setting where people can change it. When clicked it will check if
//! > the file has been changed (forms filled out for example, etc) and ask to
//! > save changes first, but if it hasn't changed it will note the file will be
//! > closed when opened in acrobat with and ok button to continue - there will
//! > be a cancel button as well."*
//!
//! ## 1. ★★★ What this module is FOR, in one sentence
//!
//! pdfcer **gives the file up**. It does not open a second window over a
//! document it is still holding: it closes the document, and only then does
//! Acrobat get the path.
//!
//! That is the operator's own instruction — point 6 of the request — and the
//! reason is worth stating rather than merely obeying. **Acrobat takes its own
//! lock on the file it opens.** Two editors on one PDF is how an afternoon's
//! work disappears: pdfcer writes its revision, Acrobat writes its own over the
//! top from a copy it read before pdfcer saved, and neither program ever
//! reports an error because neither one did anything wrong. The only defence
//! that actually holds is for exactly one program to have the file at a time,
//! and the program giving it up is the one the operator just told to hand it
//! over.
//!
//! So the confirmation is not ceremony. Closing a document is a thing that
//! happens to the operator's work, and it is announced before it happens.
//!
//! ## 2. The four questions, and the four different answers
//!
//! | State of the open document | What happens | Why |
//! |---|---|---|
//! | no Acrobat found, none configured | **the button is not there** | R9: an unavailable capability renders nothing |
//! | never saved — no file on disk | a refusal in its own words | Acrobat opens *files*; there is no file. This is not "Acrobat is missing" and must not sound like it |
//! | unsaved edits | **Save and open**, or Cancel | the document is about to be closed, so a third "open anyway" button would be data loss dressed as a choice |
//! | clean | **OK**, or Cancel | it says the file will be closed, which is the operator's point 6 |
//!
//! The third row is the one worth arguing. Every close prompt an operator has
//! ever seen offers *Save · Don't save · Cancel*, and
//! [`crate::dialogs::unsaved`] is this crate's implementation of exactly that
//! shape. **This dialog deliberately is not that one.** Its middle button would
//! read *"open in Acrobat without saving"*, and pressing it would close the
//! document, discard the edits, and hand Acrobat the **old bytes** — the
//! operator would be looking at a file that does not contain the form they just
//! filled in, in a program that is perfectly capable of saving it, and the two
//! facts together are how a morning of data entry is silently overwritten. The
//! answer that loses nothing and the answer that loses everything are not two
//! points on a scale here; only one of them is a coherent request.
//!
//! ## 3. ★★ The seam: where the impurity is, and what stays testable
//!
//! Two things in this module can only be true of a real machine — reading the
//! Windows registry, and starting a process — and both are behind a trait so
//! that everything *interesting* is a pure function over values.
//!
//! | Trait | The impure act | The test double |
//! |---|---|---|
//! | [`Registrations`] | `reg query` over `App Paths` and the `.pdf` handler | a table of `&str` readings, plus a set of paths that "exist" |
//! | [`Launcher`] | `std::process::Command::spawn` | a recorder that keeps the argv it was given |
//!
//! Everything above those two lines — which candidate wins, whether Pro beats
//! Reader, whether a configured override beats discovery, whether the button
//! is drawn at all, which of the three dialogs is raised — is decided by
//! [`resolve`] and [`prompt_for`], which take values and return values. The
//! test suite never touches a registry and never starts a process, and that is
//! not a convenience: a test that shelled out to `reg.exe` would pass or fail
//! according to what Adobe installer last ran on the machine running it, which
//! makes it a report about the machine rather than about the code.
//!
//! ## 4. ★★★ Discovery reads Windows' own registration. It never guesses a
//! path.
//!
//! `C:\Program Files\Adobe\…` is wrong the first time somebody installs
//! anywhere else, and **this operator's own working volume is `D:`**. So the
//! three sources, in the order [`resolve`] consults them:
//!
//! 1. **The configured override**, from Settings. Beats everything, because a
//!    person who typed a path has answered the question this module is
//!    otherwise guessing at.
//! 2. **`App Paths`** — `HKLM`, its `WOW6432Node` mirror, and `HKCU` — for
//!    `Acrobat.exe` (Pro) and `AcroRd32.exe` (Reader). This is the key Windows
//!    itself reads when something says "run Acrobat.exe" with no path, so it is
//!    the registration Adobe's installer is obliged to keep correct.
//! 3. **The registered `.pdf` handler's command**, as a fallback, parsed for
//!    its executable.
//!
//! ### ⚠ Why the `.pdf` handler is filtered rather than trusted
//!
//! Source 3 answers *"what opens PDFs here"*, which is **not** the question.
//! Verified on this machine, 2026-09-04: `HKLM\SOFTWARE\Classes\.pdf` reads
//! `OpenPDFStudio.pdf` — a different vendor's product entirely. A fallback that
//! took whatever the handler named would have put a button labelled *Open in
//! Acrobat* over a launcher for PDF Studio, which is a lie the operator finds
//! out about after their document is already closed.
//!
//! So a handler command is accepted only when the executable it names is
//! called `Acrobat.exe` or `AcroRd32.exe`. See [`discover::edition_of`].
//!
//! ### Every candidate is checked against the disk before it is offered
//!
//! A registry key outlives the program it points at: an uninstall that leaves a
//! stale `App Paths` value is ordinary, and so is a path typed into Settings
//! with a letter missing. [`Registrations::exists`] is asked about every
//! candidate before it becomes a [`Viewer`], because the alternative is a
//! button that is present, enabled, and does nothing when pressed — the exact
//! shape R9 exists to prevent, arrived at from the other direction.
//!
//! ## 5. Why `std::process::Command` and not `combridge`
//!
//! `combridge` is this machine's canonical COM-automation bridge and it is the
//! right tool for *driving* an application that is already running. Nothing
//! here drives anything: pdfcer starts a program with a file name on its
//! command line and stops caring. That is a plain process launch, and routing
//! it through a COM bridge would add a dependency, a running-instance
//! requirement and an attach step to an operation whose whole content is one
//! `spawn`.
//!
//! ## 6. Why the registry is read by `reg query` and not by a crate
//!
//! Three constraints meet here and they only have one intersection:
//!
//! - `crates/pdfcer-gui/src/lib.rs` carries `#![forbid(unsafe_code)]`, which
//!   cannot be relaxed by an inner `allow`. Calling `advapi32`'s
//!   `RegGetValueW` from this crate is therefore not available at all.
//! - The workspace has no registry crate — no `winreg`, no `windows-registry`
//!   (checked against `Cargo.lock`, 2026-09-04) — and this work is not
//!   permitted to edit `Cargo.toml`.
//! - `native-window` is the crate that quarantines `unsafe`, but it exists to
//!   hold *four `user32` calls for window ownership* and says so in its own
//!   manifest. Growing it a registry reader would make it "the unsafe crate"
//!   rather than "the window-ownership crate", which is the drift its
//!   documentation was written to prevent.
//!
//! `reg.exe` ships with Windows, needs no dependency, and its output is a
//! two-line format that has been stable since NT. It is read at most a handful
//! of times — once when the shell starts, and again whenever the setting
//! changes — so the cost of a process spawn is not on any path an operator can
//! feel. And it is behind [`Registrations`], so the day a registry crate is
//! permissible the swap is one file.
//!
//! ★ The one non-obvious part is [`windows::CREATE_NO_WINDOW`]: a GUI process
//! that spawns a console program on Windows gets a **console window flashed on
//! screen** unless it says otherwise. Without that flag, discovery would blink
//! a black box over the operator's document every time the shell started.

pub mod discover;
pub mod windows;

#[cfg(test)]
mod tests;

use std::path::{Path, PathBuf};

/// Which Acrobat this is.
///
/// ★ **Pro beats Reader**, and [`Edition::rank`] is where that is written down.
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
    /// A number rather than a `match` at the comparison site, so that the
    /// preference is stated once. Adding a third edition (Acrobat Standard has
    /// historically also registered as `Acrobat.exe`, so it would arrive as a
    /// variant rather than as a new key) means writing one row here, not
    /// hunting for every place two editions are compared.
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
    /// ★ The ordering between [`Self::AppPaths`] and [`Self::PdfHandler`] only
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
/// # ★ Why `exists` is on this trait and not `Path::exists`
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
    /// this module's §4 ⚠ — so the caller filters it.
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
/// # ★ Why a configured path that does not exist yields `None` rather than a
/// `Viewer`
///
/// It is tempting to honour whatever the operator typed on the grounds that
/// they know their own machine. But the failure that produces — a button that
/// is present and does nothing — is worse than the failure it avoids, and the
/// operator has no way to tell the two apart from the ribbon. The escape hatch
/// still works: **Settings shows what discovery resolved**, so a typo is
/// visible where it was made, next to the field that caused it. See
/// [`crate::dialogs::settings`].
///
/// # ★★ A configured path does NOT fall back to discovery
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
        // ⚠ The filter this module's §4 exists for: the registered handler is
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
    /// ★ **The document has never been written anywhere**, so there is no file
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
/// [`crate::app::save`] — `has_a_file` and `has_unsaved_edits` — and that is
/// deliberate: *"does this document have unsaved edits?"* already has exactly
/// one answer in this crate, and a second one written here would be a second
/// thing to keep in step with the tab strip's unsaved marker.
///
/// ★ `has_file` is asked **first**, and the order is the whole content of the
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
/// as intent and so the trace line has one home. The caller has already closed
/// the document — see this module's §1 — and that ordering is the caller's to
/// keep, not this function's: a launch that fails must not leave the document
/// closed *and* unopened anywhere, so the close happens after a successful
/// spawn.
///
/// # Errors
///
/// Propagates the [`Launcher`]'s error unchanged.
pub fn launch(launcher: &dyn Launcher, viewer: &Viewer, file: &Path) -> std::io::Result<()> {
    launcher.launch(viewer, file)
}
