//! **A profile directory per check** — the fix for a suite that measured the
//! order it ran in.
//!
//! # ★★★ The defect this module exists for
//!
//! `pdfcer-gui` is **portable**: `pdfcer_core::settings::resolve_store()` looks
//! for a writable `userdata/` *beside the executable* first, and only falls back
//! to a platform config directory. `app::persistence`, `app::prefs`,
//! `app::recent`, `app::pickstore` and the shell's `userdata/shell.ron` all live
//! under that roof. One binary on disk is therefore **one profile**, and a
//! sweep that points every check at the same `--exe` points every check at the
//! same remembered state.
//!
//! That was harmless for exactly as long as the application threw the stored
//! mode away on every launch — which it did, by accident, from the day the
//! ribbon shipped until **2026-09-06** (`f4aeee6`, *"Open in the mode you were
//! last in has been dead since the day it shipped"*). Fixing it turned a
//! dormant harness defect live in one commit:
//!
//! > a check that clicks the Edit segment now leaves `mode: Some("edit")` in
//! > `userdata/layout.ron`, and **every later check in the sweep starts in
//! > Edit**.
//!
//! ## What it cost, the same afternoon
//!
//! `a_link_goes_to_the_page_it_names` reported that clicking a link *"produced
//! nothing"* — no `link-click`, no `page-links`. That is symptom-identical to a
//! regression in the press ladder, and it is where the investigation went. It
//! was not a regression: **in Edit a click *selects* a link rather than
//! following it**, deliberately, so the hit test that would emit those lines is
//! never reached. On a fresh profile the same binary, unchanged, reaches the
//! link.
//!
//! ⇒ **A suite that shares persistent state measures the order it ran in, and
//! the contamination is invisible in the failing check's own report.** The
//! failing check's evidence — its trace, its screenshot, its reason — is
//! complete, articulate, and about the wrong subject; nothing in it mentions the
//! check that ran forty minutes earlier and changed the mode. That is the same
//! family as the wrong-`--doc-point` failures `RESUME.md` records: *an
//! articulate failure message about nothing*.
//!
//! ## Why the fix is not "clear `userdata/` between checks"
//!
//! Two reasons, and the second is the one that decides it.
//!
//! 1. **A cleared profile is not a fresh profile.** `ui_scale::write_preference`
//!    already argues this for its own file: an *absent* `preferences.txt`
//!    exercises the absent-file path, which is a different state from a file
//!    holding defaults. Deleting a directory between checks makes every check
//!    after the first run against "first launch ever", which no operator's
//!    machine is after the first day.
//! 2. **It cannot be made total.** `userdata/` is where the state we know about
//!    lives. A future build that remembers one more thing somewhere else — a
//!    sibling file, a lock, a cache — silently re-opens the hole, and the tell
//!    is again a confident failure about the wrong subject. Isolating the
//!    *directory the binary resolves from* closes the class rather than the
//!    instance: whatever the binary decides to write beside itself, it writes
//!    inside one check's sandbox.
//!
//! # How it works
//!
//! One directory per check, holding a **hard link** to the binary under test:
//!
//! ```text
//! <exe dir>/.ui-verify-profiles/<check name>/pdfcer-gui.exe   ← the link
//! <exe dir>/.ui-verify-profiles/<check name>/models/…         ← linked too
//! <exe dir>/.ui-verify-profiles/<check name>/userdata/…       ← what the check writes
//! ```
//!
//! The launched process resolves its own directory, finds no `userdata/`, and
//! creates one — private to this check and deleted with it.
//!
//! ## ★ Why a hard link rather than a copy
//!
//! Three properties, each of which a copy would spend:
//!
//! | | hard link | copy |
//! |---|---|---|
//! | cost | one directory entry | 28 MB × ~150 checks ≈ 4 GB of writes per sweep |
//! | **mtime** | the *same* file, so the same timestamp | `CopyFileEx` preserves it on Windows, but nothing in the standard library promises that |
//! | staleness gate | [`crate::launch::staleness_complaint`] compares the exe's mtime against the sources and is **on by default** | a copy whose mtime moved forward would make a genuinely stale binary look fresh |
//!
//! The third is the one worth stating plainly: `--allow-stale` is off by default
//! because *a missing trace from an unbuilt change looks exactly like a broken
//! feature*. A sandbox that quietly refreshed the timestamp would disarm that
//! gate for the whole suite, and the failure it lets through is precisely the
//! one the gate was written to catch.
//!
//! ## ★★ Why the sandbox root sits beside the exe and not under `--out`
//!
//! **Hard links cannot cross volumes.** `--out` is routinely pointed at a
//! scratch directory on another drive — the 2026-09-06 sweep ran with
//! `--out C:\…\scratchpad\sweep\out` against an exe on `D:` — so a sandbox root
//! under `--out` would fall back to a full copy on the very configuration the
//! sweep uses. Beside the exe is the one location guaranteed to be the same
//! volume as the exe.
//!
//! It also inherits the operator's own discipline. The standing rule is **never
//! drive the published build**: copy `target/release/pdfcer-gui.exe` to a
//! scratch directory and point `--exe` at the copy, so the suite's side effects
//! do not land in the operator's saved state. The sandbox root is created
//! beside whatever `--exe` names, so it lands in that scratch directory too.
//!
//! A copy is still the fallback, for the case where the link cannot be made — a
//! filesystem without hard links, a permission refusal, a `--exe` on a volume
//! that will not take another entry. [`Sandbox::how`] says which happened, and
//! the run header prints it, because "this sweep copied 4 GB" is worth knowing
//! and "this sweep silently did something other than what the module says" is
//! not.
//!
//! ## ★ What is brought across, and what deliberately is not
//!
//! **Brought:** the executable, its sibling `models/` directory, and any sibling
//! `.dll`. `models/ocrs` is resolved *beside the executable* by
//! `crate::ocr::resolve_models`, and a sandbox without it would make
//! `ocr_finds_words_in_a_scan` report a missing model directory — which is a
//! SKIP, and a SKIP is not red, so the check could be dead for the rest of the
//! project's life while the suite looked healthy. That is the `repo_fixture`
//! shape [`crate::checks::CheckContext::out`] records paying for once already.
//!
//! **Not brought:** `userdata/`. That is the entire point. A sandbox seeded with
//! the operator's remembered mode would isolate the checks from each other and
//! leave every one of them contaminated by whatever the last real session did.
//!
//! ## Lifetime
//!
//! [`Sandbox`] deletes its directory on drop. A failure to delete is a **warning
//! on stderr, never a verdict**: a leaked 200-byte directory entry is a
//! tidiness problem, and downgrading a real pass because the harness could not
//! tidy up would be reporting the harness's housekeeping as a defect in the
//! program — `ui_scale`'s `RestoreScale` guard makes the same call for the same
//! reason.
//!
//! A directory left behind by a killed run is removed when the next run creates
//! the same sandbox. That recovery assumes the project's standing **one driven
//! run at a time** rule, which is not this module's to enforce and is already
//! mandatory for a much harder reason: the harness moves the real pointer, and
//! three concurrent runners on 2026-09-06 produced an entire sweep of worthless
//! verdicts.
//!
//! # Proving it works
//!
//! An isolation fix that does not isolate is worse than none, because the next
//! contaminated failure is investigated with the contamination ruled out. The
//! falsification is two checks in one invocation, the first of which switches
//! mode and the second of which is sensitive to it:
//!
//! ```text
//! ui-verify --exe <scratch>/pdfcer-gui.exe --pdf fixtures/a1-titleblock.pdf \
//!           --doc-point 0,300,500 \
//!           --check clipboard_mode_switches_the_ribbon \
//!           --check a_link_goes_to_the_page_it_names
//! ```
//!
//! and then the same pair with `--shared-profile`, which restores the old
//! behaviour. The evidence is in `evidence/ui-verify-20260906-isolation.txt`.

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
///
/// See the module header. This is an allowlist rather than "every sibling
/// directory": a `--exe` pointed at `target/release/` has `deps/`, `build/` and
/// `incremental/` beside it, which are gigabytes and which the application never
/// looks at.
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

/// Hard-link `from` to `to`, falling back to a byte copy.
///
/// # Errors
///
/// Neither the link nor the copy could be made. The message carries **both**
/// errors, because the link failure alone is usually the uninteresting one — a
/// cross-volume `--exe`, expected and handled — and the copy failure is the one
/// that says what is actually wrong.
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
///
/// # Errors
///
/// The destination could not be created, or the source could not be read.
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
///
/// A packaged portable build may ship a runtime DLL beside the executable; a
/// `cargo build` output does not. Asking the directory rather than assuming
/// either shape means the same sandbox code serves both.
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
    ///
    /// This is the property the whole module is for, asserted at the level a
    /// unit test can reach: **two different check names produce two different
    /// directories**. What a unit test cannot reach — that the *application*
    /// resolves its `userdata/` from that directory — is what the driven
    /// falsification in the module header is for.
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
    ///
    /// Stands in for the real defect: `userdata/layout.ron` written by a check
    /// that clicked the Edit segment must not be there when the next check
    /// launches.
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
    ///
    /// ★ Load-bearing: [`crate::launch::staleness_complaint`] compares that
    /// timestamp against the sources and is on by default, because *a missing
    /// trace from an unbuilt change looks exactly like a broken feature*. A
    /// sandbox that refreshed the mtime would disarm the gate for every check
    /// in the suite.
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
