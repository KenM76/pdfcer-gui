//! # pdfcer-gui — the binary
//!
//! **This file reads `argv` and starts the application. That is all it may
//! ever do.** Everything else lives in the library (`src/lib.rs`), which is
//! importable by `tools/ui-verify`, by integration tests, and by `cargo
//! doc`. The crate this one replaces put **25,005 lines in `main.rs`**,
//! half the entire GUI; `PROJECT_PLAN.md` §3 is the answer to that, and
//! keeping this file at a dozen lines is the visible part of the answer.
//!
//! Argument handling belongs *here* rather than in [`pdfcer_gui::run`]
//! because anything that can be answered without a window must be answered
//! before one exists. A terminal invocation — `--help`, `--version`, a bad
//! path — must not open a window it then has to be told to close.
//!
//! Usage: `pdfcer-gui [<file> [--page <n>]]`. `--page` is one-based and shows
//! that page of the file; a window tearing a tab off passes it so the new
//! window shows what the tab showed.

// On Windows, prevent a console window from popping up behind the GUI in
// release builds (the process is a GUI app, not a console app). Debug
// builds keep the console so `eprintln!`/panics remain visible while
// developing.
//
// This attribute is a property of the BINARY and cannot move to the
// library, which is the one reason this file carries any attribute at all.
//
// Note for anyone chasing a `PDFCER_DIAG` trace out of a RELEASE build: the
// GUI subsystem detaches from the parent console, so `2>&1` into a pipe
// shows nothing, but `2> trace.txt` still works — the file handle is
// inherited before the subsystem matters.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![forbid(unsafe_code)]

use std::path::PathBuf;

fn main() -> eframe::Result {
    // `args_os` rather than `args` because a path is not required to be
    // valid UTF-8, and a non-UTF-8 path is the operator's business rather
    // than ours to reject.
    let args: Vec<std::ffi::OsString> = std::env::args_os().skip(1).collect();
    let initial = args.first().map(PathBuf::from);
    let page = args
        .windows(2)
        .find(|w| w[0] == pdfcer_gui::PAGE_ARG)
        .and_then(|w| w[1].to_str()?.parse::<usize>().ok())
        .and_then(|n| n.checked_sub(1));
    pdfcer_gui::run(initial, page)
}
