//! **A profile directory per check** — the fix for a suite that measured the
//! order it ran in.
//!
//! Design and rationale: `docs/modules/ui-verify/sandbox.md`.

use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

/// The directory, beside the binary under test, that every per-check sandbox is
/// created inside.
///
/// Dot-prefixed so it sorts away from the build outputs it will usually sit
/// among, and named for the tool that makes it so that a directory found in the
/// wild names its owner.
pub const ROOT: &str = ".ui-verify-profiles";

/// Sibling directories carried into a sandbox, because the application resolves
/// them relative to its own executable.
const SIBLING_DIRS: [&str; 1] = ["models"];

/// How the binary got into the sandbox.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum How {
    /// A hard link — same inode, same mtime, no bytes copied.
    Linked,
    /// A byte copy, because the link could not be made.
    Copied,
}

impl How {
    /// One word for a report line.
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::Linked => "hard-linked",
            Self::Copied => "copied",
        }
    }
}

/// A private profile directory for one check, deleted when it is dropped.
#[derive(Debug)]
pub struct Sandbox {
    dir: PathBuf,
    exe: PathBuf,
    how: How,
}

impl Sandbox {
    /// Build one for `check`, holding `source` (the binary the caller resolved).
    ///
    /// # Errors
    ///
    /// The sandbox root or the check's directory could not be created, or the
    /// binary could neither be linked nor copied into it. Every one of those is
    /// a reason the caller must **not** silently fall back to the shared
    /// profile: an isolation that quietly did not happen is the defect this
    /// module exists to close, wearing a passing run as a disguise.
    pub fn for_check(source: &Path, check: &str) -> Result<Self> {
        let parent = source.parent().ok_or_else(|| {
            Error::new(format!(
                "cannot isolate {}: it has no parent directory to put a sandbox beside.",
                source.display()
            ))
        })?;
        let name = source.file_name().ok_or_else(|| {
            Error::new(format!(
                "cannot isolate {}: it has no file name.",
                source.display()
            ))
        })?;
        let dir = parent.join(ROOT).join(check);
        // A directory left by a killed run. Removing it is the recovery path;
        // see the module header on why that is safe under the one-run-at-a-time
        // rule.
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).map_err(|e| {
            Error::new(format!(
                "cannot create the sandbox {}: {e}. Point --exe at a writable directory, or pass \
                 --shared-profile to run without isolation (and read its warning first).",
                dir.display()
            ))
        })?;

        let exe = dir.join(name);
        let how = place(source, &exe)?;
        for sibling in SIBLING_DIRS {
            let from = parent.join(sibling);
            if from.is_dir() {
                // Best effort: a sandbox without `models/` makes ONE check skip
                // with a precise reason of its own, and refusing to run the
                // other hundred and fifty over it would be the worse trade.
                let _ = place_tree(&from, &dir.join(sibling));
            }
        }
        for dll in siblings_matching(parent, "dll") {
            if let Some(file) = dll.file_name() {
                let _ = place(&dll, &dir.join(file));
            }
        }
        seed_prefs(&dir);
        Ok(Self { dir, exe, how })
    }

    /// The isolated binary to drive — what `--exe` becomes for this check.
    #[must_use]
    pub fn exe(&self) -> &Path {
        &self.exe
    }

    /// The sandbox directory, which is where the driven process will resolve
    /// its `userdata/`.
    #[must_use]
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// Whether the binary was linked or copied.
    #[must_use]
    pub const fn how(&self) -> How {
        self.how
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        if let Err(e) = std::fs::remove_dir_all(&self.dir) {
            // A warning, never a verdict. See the module header.
            eprintln!(
                "ui-verify: WARNING — could not remove the sandbox {} ({e}). It holds a link to \
                 the binary and one check's userdata; deleting it by hand is safe.",
                self.dir.display()
            );
        }
    }
}

/// The name `pdfcer-gui` reads its preferences from.
const PREFS_FILE: &str = "preferences.txt";

/// **The one thing this sandbox deliberately DOES seed**, and the reason is
/// worth reading before deleting it.
fn seed_prefs(dir: &Path) {
    let _ = write_prefs(&dir.join("userdata"), "");
}

/// The header every sandbox-written preferences file carries, including the
/// one key that must survive any check's own seeding.
const PREFS_HEADER: &str = "\
# Written by ui-verify. See `sandbox::write_prefs`.
#
# `ask_default_app = false` suppresses the O173 startup offer, which would
# otherwise open a real OS window in front of the check. It is written by the
# sandbox and RE-written by every check that seeds its own preferences, so that
# seeding one key cannot silently restore the offer. A check that DRIVES the
# offer must delete this file; see `checks/default_app_offer.rs`.
ask_default_app = false
";

/// **Write a preferences file for a sandboxed run, with the startup offer
/// suppressed whatever else the caller asked for.**
///
/// `body` is the caller's own keys, one `key = value` per line, appended after
/// [`PREFS_HEADER`]. Pass `""` for the bare seed.
///
/// # Why this function exists, and what it cost not to have it
///
/// `seed_prefs` has written `ask_default_app = false` into every sandbox since
/// the offer shipped, and the reasoning above it is correct. It had one hole:
/// **three checks write this same file themselves**, each overwriting the seed
/// with their own keys —
///
/// | check | what it did | effect |
/// |---|---|---|
/// | `ui_scale_resizes_the_chrome` | wrote a file containing only `ui_scale` | seed gone |
/// | `the_print_window_opens_on_the_settings_you_last_used` | wrote its thirteen seeded answers, **and deleted the file before its control run** | seed gone twice over |
/// | `a_page_display_choice_survives_a_close…` | **deleted** the file to normalise | seed gone |
///
///
/// ## What this function did NOT fix, for a day, and it is in the table above
///
/// **A guard centralised into "the only write path" does not cover the delete
/// path, because deleting is not writing.** A default state has two routes into
/// it — *set the defaults*, or *remove the thing that overrode them* — and a
/// header prepended to every write covers exactly one.
///
/// `the_print_window_opens_on_the_settings_you_last_used` took both routes. Its
/// seeding write was repaired to come through this function, and its control
/// launch sixty lines earlier went on calling `std::fs::remove_file` to reach the
/// shipped print defaults. So the 2026-09-12 sweep found the offer in that
/// check's trace **after** the repair, and the check skipped saying *"the click on
/// `ribbon.tab.file` produced no `ribbon-tab-activated tab=file` line, so no click
/// reached the ribbon"* — a sentence that names a cause it never measured. Five
/// places in three documents then recorded "the File-tab route" as a suite-wide
/// ribbon blocker. The ribbon was never involved. Fixed 2026-09-13 by
/// [`reset_prefs`], and the check passes.
///
/// ⇒ **When centralising a guard into a write path, grep the same modules for
/// `remove_file`, `remove_dir`, truncation and "reset to defaults" before
/// believing the guard is universal.** And ⚠ a doc comment listing the cases a
/// fix repairs is a **claim to verify per case**, not a receipt: this table was
/// correct about what each check had done and wrong about one of them being fixed,
/// and re-driving the three named checks would have caught it in minutes.
///
/// In `ui_scale`'s case that produced a **FAIL against the application**:
/// `find_window_for_pid` returns the front-most window of the process, the
/// offer was front-most, so the check measured the *dialog's* client area and
/// reported that the UI-scale preference "did not reach
/// `Context::set_zoom_factor` at all". It had reached it. The application was
/// correct and the accusation was specific, confident and wrong — which is the
/// failure mode this project has recorded more times than any other.
///
/// ⇒ **A suppression a caller has to remember is a suppression that will be
/// forgotten.** Making it a header of the only write path removes the memory
/// from the loop: a check can write whatever keys it likes and cannot restore
/// the offer by doing so.
///
/// # Errors
///
/// The `userdata` directory could not be created, or the file could not be
/// written. Callers in the sandbox path swallow this (see `seed_prefs`);
/// callers in a check should report it as a SKIP, because a preference that
/// could not be written means the check never began.
pub fn write_prefs(userdata: &Path, body: &str) -> std::io::Result<()> {
    std::fs::create_dir_all(userdata)?;
    let mut text = String::from(PREFS_HEADER);
    if !body.is_empty() {
        text.push_str(body);
        if !text.ends_with('\n') {
            text.push('\n');
        }
    }
    std::fs::write(userdata.join(PREFS_FILE), text)
}

/// Restore a sandbox's preferences file to the bare seed.
///
/// The correct way for a check to *normalise* persisted preferences between
/// launches. **Deleting the file is not** — deletion takes the suppression
/// with it, which is precisely how
/// `a_page_display_choice_survives_a_close_and_reaches_a_new_document` grew
/// the startup offer in front of its own second launch. See [`write_prefs`].
///
/// **And how `the_print_window_opens_on_the_settings_you_last_used` grew it in
/// front of its own FIRST launch, a day after [`write_prefs`] was written to stop
/// exactly that.** Its control run needs the shipped print defaults, and the
/// argument it made for deleting rather than resetting was good: the neutral state
/// is *no print preferences at all*, and writing the defaults back would make a
/// later reader think the operator had chosen them. Every clause true, conclusion
/// wrong, because **the file holds one key that is not a print preference.** This
/// function is that argument's actual answer — every print key absent and taking
/// its compiled-in default, nothing pinned, and the offer still shut.
///
/// ⇒ So the rule has no exception worth writing down: **a check that wants a
/// pristine preferences state calls this.** The only caller that may delete is one
/// that exists to DRIVE the offer, and there is exactly one of those
/// (`checks/default_app_offer.rs`).
///
/// # Errors
///
/// As [`write_prefs`].
pub fn reset_prefs(userdata: &Path) -> std::io::Result<()> {
    write_prefs(userdata, "")
}

/// Hard-link `from` to `to`, falling back to a byte copy.
fn place(from: &Path, to: &Path) -> Result<How> {
    match std::fs::hard_link(from, to) {
        Ok(()) => Ok(How::Linked),
        Err(link_err) => match std::fs::copy(from, to) {
            Ok(_) => Ok(How::Copied),
            Err(copy_err) => Err(Error::new(format!(
                "cannot place {} at {}: hard link failed ({link_err}) and so did the copy \
                 ({copy_err}).",
                from.display(),
                to.display()
            ))),
        },
    }
}

/// [`place`] applied to every file under a directory, recursively.
fn place_tree(from: &Path, to: &Path) -> Result<()> {
    std::fs::create_dir_all(to)
        .map_err(|e| Error::new(format!("cannot create {}: {e}", to.display())))?;
    let entries = std::fs::read_dir(from)
        .map_err(|e| Error::new(format!("cannot read {}: {e}", from.display())))?;
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name() else {
            continue;
        };
        if path.is_dir() {
            place_tree(&path, &to.join(name))?;
        } else {
            place(&path, &to.join(name))?;
        }
    }
    Ok(())
}

/// Every sibling file with the given extension, case-insensitively.
fn siblings_matching(dir: &Path, extension: &str) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.is_file()
                && p.extension()
                    .and_then(|x| x.to_str())
                    .is_some_and(|x| x.eq_ignore_ascii_case(extension))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A scratch directory that removes itself, so these tests leave nothing
    /// behind on a machine that is also the operator's.
    struct Scratch(PathBuf);

    impl Scratch {
        fn new(tag: &str) -> Self {
            let dir = std::env::temp_dir().join(format!(
                "ui-verify-sandbox-test-{tag}-{}",
                std::process::id()
            ));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).expect("scratch");
            Self(dir)
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// The sandbox holds the binary under its own name, in its own directory,
    /// and the two checks do not share one.
    #[test]
    fn two_checks_get_two_directories() {
        let scratch = Scratch::new("two");
        let exe = scratch.0.join("pdfcer-gui.exe");
        std::fs::write(&exe, b"not really a binary").expect("write");

        let a = Sandbox::for_check(&exe, "check_a").expect("sandbox a");
        let b = Sandbox::for_check(&exe, "check_b").expect("sandbox b");

        assert_ne!(a.dir(), b.dir());
        assert!(a.exe().is_file(), "the sandbox holds the binary");
        assert!(b.exe().is_file());
        assert_eq!(a.exe().file_name(), exe.file_name(), "same name as source");
    }

    /// What one sandbox writes is not visible to the next.
    #[test]
    fn what_one_sandbox_writes_the_next_does_not_see() {
        let scratch = Scratch::new("write");
        let exe = scratch.0.join("pdfcer-gui.exe");
        std::fs::write(&exe, b"not really a binary").expect("write");

        let first = Sandbox::for_check(&exe, "writes_a_mode").expect("sandbox");
        let userdata = first.dir().join("userdata");
        std::fs::create_dir_all(&userdata).expect("userdata");
        std::fs::write(userdata.join("layout.ron"), b"mode: Some(\"edit\")").expect("layout");

        let second = Sandbox::for_check(&exe, "reads_a_mode").expect("sandbox");
        assert!(
            !second.dir().join("userdata").join("layout.ron").exists(),
            "the second check must not inherit the first check's remembered mode"
        );
    }

    /// Dropping a sandbox takes its directory with it.
    #[test]
    fn a_dropped_sandbox_leaves_nothing_behind() {
        let scratch = Scratch::new("drop");
        let exe = scratch.0.join("pdfcer-gui.exe");
        std::fs::write(&exe, b"not really a binary").expect("write");

        let dir = {
            let sandbox = Sandbox::for_check(&exe, "transient").expect("sandbox");
            sandbox.dir().to_path_buf()
        };
        assert!(!dir.exists(), "the sandbox is removed on drop");
        assert!(exe.is_file(), "and the source binary is untouched");
    }

    /// A sandbox re-uses the name a killed run left behind rather than refusing.
    #[test]
    fn a_leftover_directory_is_reclaimed() {
        let scratch = Scratch::new("leftover");
        let exe = scratch.0.join("pdfcer-gui.exe");
        std::fs::write(&exe, b"not really a binary").expect("write");

        let leftover = scratch.0.join(ROOT).join("abandoned");
        std::fs::create_dir_all(leftover.join("userdata")).expect("leftover");
        std::fs::write(leftover.join("userdata").join("layout.ron"), b"stale").expect("stale");

        let fresh = Sandbox::for_check(&exe, "abandoned").expect("sandbox");
        assert!(
            !fresh.dir().join("userdata").join("layout.ron").exists(),
            "a killed run's leftovers must not become the next run's starting state"
        );
    }

    /// The sibling `models/` directory comes across, so the OCR check can find
    /// its engine.
    #[test]
    fn the_models_directory_comes_with_the_binary() {
        let scratch = Scratch::new("models");
        let exe = scratch.0.join("pdfcer-gui.exe");
        std::fs::write(&exe, b"not really a binary").expect("write");
        let models = scratch.0.join("models").join("ocrs");
        std::fs::create_dir_all(&models).expect("models");
        std::fs::write(models.join("text-detection.rten"), b"weights").expect("weights");

        let sandbox = Sandbox::for_check(&exe, "ocr").expect("sandbox");
        assert!(
            sandbox
                .dir()
                .join("models")
                .join("ocrs")
                .join("text-detection.rten")
                .is_file(),
            "models/ocrs must be resolvable beside the sandboxed binary"
        );
    }

    /// The binary in the sandbox carries the source's modification time.
    #[test]
    fn the_sandboxed_binary_keeps_the_sources_timestamp() {
        let scratch = Scratch::new("mtime");
        let exe = scratch.0.join("pdfcer-gui.exe");
        std::fs::write(&exe, b"not really a binary").expect("write");
        let before = std::fs::metadata(&exe)
            .expect("meta")
            .modified()
            .expect("t");

        let sandbox = Sandbox::for_check(&exe, "timestamped").expect("sandbox");
        let after = std::fs::metadata(sandbox.exe())
            .expect("meta")
            .modified()
            .expect("t");
        assert_eq!(
            before, after,
            "the sandbox must not look newer than the build"
        );
    }
}
