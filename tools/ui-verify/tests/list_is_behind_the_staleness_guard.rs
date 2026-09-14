//! **The count command must not answer from a stale harness.**
//!
//! # What this file asserts, and why it drives a process to do it
//!
//! `src/main.rs` calls `refuse_if_self_is_stale` before every path that
//! matters, and as of 2026-09-14 `--list` is one of them. That ordering is one
//! line of dispatch, it has no type attached to it, and moving it back above
//! the guard would compile, pass clippy, and break nothing any other test in
//! this workspace looks at. So it is asserted here, by **running the binary**.
//!
//! ## Why not a source scan
//!
//! The obvious instrument is to read `src/main.rs` and check that the byte
//! offset of the `--list` block is greater than the offset of the guard call.
//! That instrument is rejected for two reasons, both learned in this tree:
//!
//! * **A gate keyed on a name is discharged by prose.** `main.rs` now carries
//!   forty lines of doc comment that name `--list` and the guard in the same
//!   breath. A scanner would have to strip comments before it could mean
//!   anything, and a scanner that strips comments is one refactor away from
//!   measuring nothing.
//! * **A source scan asserts the shape of the code, not the behaviour of the
//!   program.** What is owed here is a statement about what an operator's
//!   shell gets back, and the only thing that can make that statement is a
//!   shell getting something back.
//!
//! ## How a stale harness is produced without touching the tree
//!
//! [`ui_verify::launch::staleness_complaint`] compares the running
//! executable's mtime against the newest `.rs` or `.toml` under
//! `CARGO_MANIFEST_DIR`, which is baked in at compile time and therefore still
//! points at the real crate directory even for a copy of the binary somewhere
//! else. So: copy `ui-verify.exe` into a scratch directory, **set the copy's
//! mtime to the epoch**, and it is stale against every source file in the
//! crate by construction — with nothing in the repository modified, no shared
//! state, and no dependence on what any other test did first.
//!
//! ⇒ That last property is the reason it is done this way rather than by
//! `touch`ing a source file: a suite that mutates the tree measures the order
//! it ran in.
//!
//! # Three assertions, and the third one is the control
//!
//! 1. `--list` on a stale harness prints **nothing** to stdout and exits 2, so
//!    the count command's `grep -c` answers **0** rather than a plausible
//!    roster size.
//! 2. `--help` on the same stale harness still answers, because it describes
//!    the argument surface and is what a reader reaches for when the tool has
//!    just refused them.
//! 3. `--allow-stale --list` on the same stale harness lists the roster.
//!
//! Without the third, a `--list` that had simply been broken outright would
//! satisfy the first two, and the suite would report that as the guard
//! working.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::SystemTime;

/// A copy of the freshly built harness whose mtime is the epoch.
///
/// `tag` separates the three tests' copies: `cargo test` runs them on threads
/// of one process, so a single fixed filename would be two tests writing one
/// path — and the loser's symptom would be an unrelated assertion going red.
///
/// The directory names `std::process::id()` for the same reason one level up:
/// two concurrent `cargo test` **processes** ask for the same filenames as
/// each other, and a thread-unique tag does nothing about that.
fn stale_copy_of_the_harness(tag: &str) -> PathBuf {
    let source = PathBuf::from(env!("CARGO_BIN_EXE_ui-verify"));
    assert!(
        source.is_file(),
        "cargo did not build the ui-verify binary for this test: {}",
        source.display()
    );

    let dir = std::env::temp_dir().join(format!("ui-verify-stale-guard-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("cannot create the scratch directory");

    let extension = source
        .extension()
        .map_or(String::new(), |e| format!(".{}", e.to_string_lossy()));
    let dest = dir.join(format!("ui-verify-{tag}{extension}"));
    std::fs::copy(&source, &dest).expect("cannot copy the harness binary");

    // ★ The whole fixture is this one call. A fresh copy carries TODAY's mtime,
    // which is newer than every source file, so an unmodified copy is not stale
    // and the guard would stay silent — the test would then pass for the wrong
    // reason on the day the ordering was reverted.
    let handle = std::fs::OpenOptions::new()
        .write(true)
        .open(&dest)
        .expect("cannot reopen the copy to age it");
    handle
        .set_modified(SystemTime::UNIX_EPOCH)
        .expect("cannot set the copy's modification time");

    dest
}

/// `grep -cE` over the check-name pattern, which is the count command
/// `RESUME.md` names, reimplemented so this test measures what an operator's
/// shell measures.
///
/// ★ Deliberately the same imprecise pattern as the documented command, down
/// to picking up the `--exe` target table's rows. Tightening it here would
/// make this test agree with a command nobody runs.
fn what_the_count_command_would_answer(stdout: &str) -> usize {
    stdout
        .lines()
        .filter(|line| {
            let Some(rest) = line.strip_prefix("  ") else {
                return false;
            };
            !rest.is_empty()
                && rest
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        })
        .count()
}

/// Run a binary with arguments and capture everything it said.
fn run(exe: &Path, args: &[&str]) -> Output {
    Command::new(exe)
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("cannot run {} {args:?}: {e}", exe.display()))
}

#[test]
fn list_on_a_stale_harness_prints_nothing_and_the_count_command_answers_zero() {
    let exe = stale_copy_of_the_harness("list");
    let out = run(&exe, &["--list"]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);

    assert_eq!(
        stdout.trim(),
        "",
        "a stale harness printed a roster on stdout. That number reaches \
         FEATURES.md and the release notes, and it describes the tree the \
         binary was COMPILED from, not the tree it is being asked about. \
         stdout was:\n{stdout}"
    );
    assert_eq!(
        what_the_count_command_would_answer(&stdout),
        0,
        "the documented count command would have answered a non-zero roster \
         size from a stale binary"
    );
    assert_eq!(
        out.status.code(),
        Some(2),
        "a refused command line exits 2. stderr was:\n{stderr}"
    );
    assert!(
        stderr.contains("HARNESS"),
        "the refusal must say which binary is stale — a message about 'the \
         build' sends the reader off to rebuild the application, which is not \
         what is wrong. stderr was:\n{stderr}"
    );

    let _ = std::fs::remove_file(&exe);
}

#[test]
fn help_still_answers_on_a_stale_harness() {
    let exe = stale_copy_of_the_harness("help");
    let out = run(&exe, &["--help"]);
    let stdout = String::from_utf8_lossy(&out.stdout);

    assert_eq!(
        out.status.code(),
        Some(0),
        "--help is in front of the guard on purpose: it is what a reader \
         reaches for when the tool has just refused them"
    );
    assert!(
        stdout.contains("--list"),
        "--help printed something, but not the usage text. stdout was:\n{stdout}"
    );

    let _ = std::fs::remove_file(&exe);
}

#[test]
fn allow_stale_list_still_answers_the_roster() {
    let exe = stale_copy_of_the_harness("allow");
    let out = run(&exe, &["--allow-stale", "--list"]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let counted = what_the_count_command_would_answer(&stdout);

    assert_eq!(
        out.status.code(),
        Some(0),
        "--allow-stale is the single flag for 'yes, I mean to use this binary'"
    );
    assert!(
        counted > 100,
        "the same stale binary that refused a bare --list must still list \
         under --allow-stale. Without this, a --list broken outright would \
         satisfy every other assertion in this file. It counted {counted}"
    );

    let _ = std::fs::remove_file(&exe);
}
