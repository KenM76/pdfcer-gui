//! `checks::harness` — **what a check IS**: the [`Check`] trait every check
//! implements and the [`CheckContext`] every check is handed.
//!
//! # Why this is its own file
//!
//! [`super`] is an **index** — one `pub mod` declaration per check, each carrying
//! the argument for why that check exists — and an index grows by a few lines
//! every time the project ships a feature. This file is a **contract**, and it
//! barely moves.
//!
//! That is the same seam `checks::roster`'s header argues one level up, applied
//! once more. Three subjects, three files, and the tell is still who edits
//! them:
//!
//! | | `harness.rs` | `mod.rs` | `roster.rs` |
//! |---|---|---|---|
//! | holds | the contract | which checks exist | the order they run in |
//! | changes when | the harness gains a capability every check can use | a check is added or removed | a check is added, removed or **re-ordered** |
//! | how often | rarely | every landing that ships a driven check | the same, plus re-orderings |
//!
//! Keeping an unboundedly growing list in the same file as a stable contract
//! makes every landing touch the contract's file, and makes that file's size
//! somebody's problem at random rather than the problem of whoever owns the
//! growth. Under **R2** the index is what will need a seam next, and the
//! remedy for it is family subdirectories rather than a raised limit.
//!
//! The names are re-exported from [`super`], so every check still spells them
//! `super::CheckContext` / `crate::checks::Check`. Moving them did not rename
//! anything.

use std::path::{Path, PathBuf};

use crate::coords::DocPoint;
use crate::profile::Profile;
use crate::report::CheckReport;

/// Everything a check needs to know about this run.
#[derive(Clone, Debug)]
pub struct CheckContext {
    /// The target binary's vocabulary and regions.
    pub profile: &'static Profile,
    /// The binary to drive. `None` means the checks that drive one SKIP.
    pub exe: Option<PathBuf>,
    /// The document to open.
    pub pdf: Option<PathBuf>,
    /// **A second, DIFFERENT document**, for the checks that need two open at
    /// once.
    ///
    /// It must not be the same file as [`Self::pdf`]. `crate::app::documents`
    /// §3 makes pdfcer activate the tab a path is already open in rather than
    /// open a duplicate — deliberately, because two `EditSession`s over one
    /// file would be two undo stacks and a save from either would discard the
    /// other's work. So passing the same path twice would make a multi-document
    /// check assert the opposite of what it is for, and the checks that need
    /// this SKIP rather than fall back to [`Self::pdf`].
    pub second_pdf: Option<PathBuf>,
    /// An already-captured image to assert against instead of driving the
    /// application — the offline mode for pixel checks. Its purpose is
    /// falsification against a dated artefact; see
    /// [`crate::profile::Calibration`].
    pub image: Option<PathBuf>,
    /// Where screenshots and trace copies are written.
    pub out_dir: PathBuf,
    /// WCAG contrast floor. Defaults to [`crate::pixels::AA_LARGE`].
    pub contrast_threshold: f64,
    /// Whether the harness may take the operator's pointer and keyboard.
    /// `false` makes every driving check SKIP — never pass.
    pub allow_input: bool,
    /// Drive a binary older than its sources.
    pub allow_stale: bool,
    /// Source tree the staleness gate compares against.
    pub source_root: Option<PathBuf>,
    /// Explicit page size, when the fixture's `/MediaBox` cannot be read.
    pub page_size: Option<(f64, f64)>,
    /// The document point a driving check aims at.
    ///
    /// **There is deliberately no default.** A default would be a guess about
    /// where the fixture keeps an object, and a click on empty page is
    /// symptom-identical to a broken hit test — the confusion that produced a
    /// filed-then-retracted defect in this codebase. Absent, the driving
    /// checks SKIP and say what to pass.
    pub target: Option<DocPoint>,
}

impl CheckContext {
    /// A path under the run's output directory — **and the directory is made
    /// to exist before the path is handed back.**
    ///
    /// # ★★★ Why the `create_dir_all` is here and not at every call site
    ///
    /// [`crate::launch`] creates the parent of the trace file it is about to
    /// open and [`crate::image`] creates the parent of a PNG it is about to
    /// save, so a check whose first use of this directory is a launch or a
    /// screenshot would work without this. But several checks write a
    /// **fixture** into it *before* they launch anything —
    /// `save_writes_over_the_file_you_opened` copies the document it is going
    /// to overwrite, `redaction_removes_and_proves_it` writes the PDF it will
    /// redact, and `insert_image_places_a_picture`,
    /// `the_insert_window_steps_aside_so_you_can_point` and
    /// `a_dropped_image_reaches_the_placement_window` each write a PNG to drop
    /// — and for those, nothing has created the directory yet when `--out`
    /// points at a fresh per-check path.
    ///
    /// ★★ **A path that cannot resolve produces a SKIP, and a SKIP is not a
    /// failure, so a check can be dead for ever while the suite looks
    /// healthy.** That is why the guarantee is a FUNNEL and not a
    /// `create_dir_all` at each of the writing call sites: a sixth such check
    /// added later inherits it, whereas a sixth call site has to remember.
    ///
    /// # Why the error is swallowed
    ///
    /// The return type is a path, not a result, and forty call sites read it in
    /// expression position. A directory that genuinely cannot be created — a
    /// read-only volume, a name that is not a directory — still produces an
    /// error at the moment of the write, from the code that knows what it was
    /// writing and can say so. Making this fallible would trade a precise
    /// message at the write for a vague one here.
    #[must_use]
    pub fn out(&self, name: &str) -> PathBuf {
        // Best effort, deliberately: see the doc comment.
        let _ = std::fs::create_dir_all(&self.out_dir);
        self.out_dir.join(name)
    }

    /// The exe to use: the explicit one, or the profile's default if it is
    /// actually there.
    ///
    /// A default that does not exist is `None` rather than a path, so the SKIP
    /// reason says "no binary" once rather than describing a path the caller
    /// never chose.
    #[must_use]
    pub fn resolve_exe(&self) -> Option<PathBuf> {
        if let Some(e) = &self.exe {
            return Some(e.clone());
        }
        let default = Path::new(self.profile.default_exe);
        default.is_file().then(|| default.to_path_buf())
    }
}

/// One check.
pub trait Check {
    /// The name `--check` accepts.
    fn name(&self) -> &'static str;
    /// Which defect it detects, in one line, for the report.
    fn defect(&self) -> &'static str;
    /// Run it. A check never panics and never returns an error: every outcome,
    /// including "I could not start", is a [`CheckReport`].
    fn run(&self, ctx: &CheckContext) -> CheckReport;
}
