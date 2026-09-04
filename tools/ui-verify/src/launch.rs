//! Start the built binary, capture its diagnostic trace, find its window, and
//! never leave it running.
//!
//! ## The binary, not a test harness
//!
//! This module launches `pdfcer-gui.exe` — the actual artefact, built in
//! release, opening an actual file. That is the entire premise of the crate:
//! D1 and D2 both live in the space between our code and the framework, and
//! neither is reachable from inside a test binary that constructs an
//! `egui::Context` by hand.
//!
//! ## Two guarantees this module owes the operator
//!
//! **1. It never kills a process it did not start.** The operator may well
//! have the application open for their own work — this harness drives the real
//! desktop, which is precisely the situation where that is most likely — and a
//! harness that tidied up by killing "all pdfcer-gui processes" would close
//! their document. So [`Session`] holds a child handle and kills exactly that.
//!
//! **2. It never leaks the process it did start.** pdfcer's predecessor script
//! killed its child on its last line, so any error before that line left a
//! window running: parked off-screen, invisible, and still consuming pointer
//! input on the operator's desktop. The operator reported it as *"do you have
//! some gui processes leftover that are interfering with my mouse?"* — twice
//! in one session, which is what made it a defect in the tool rather than an
//! operating mistake. Here the kill is in [`Drop`], so it happens on every
//! path including a panic.
//!
//! ## The staleness gate
//!
//! [`Session::launch`] refuses a binary older than the newest source file
//! under `crates/`, unless explicitly told not to. The failure this prevents
//! is the worst kind: the traces a developer expects are simply **absent**,
//! which reads as "the feature does not work" rather than "the feature was
//! never compiled". pdfcer recorded an agent nearly concluding a panel did not
//! render, when the binary predated every change it had made.
//!
//! An absence is only evidence when the thing that would have produced it was
//! actually built.

use std::cell::{Cell, RefCell};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use crate::coords::WindowFrame;
use crate::error::{Error, Result};
use crate::sys::{self, WindowHandle};
use crate::trace::Trace;

/// How to start the application.
#[derive(Clone, Debug)]
pub struct LaunchSpec {
    /// The binary to run.
    pub exe: PathBuf,
    /// The document to open — passed as `argv[1]`, the way an operator would
    /// open it from the shell.
    pub pdf: Option<PathBuf>,
    /// Environment to add. The diagnostic switch goes here.
    pub env: Vec<(String, String)>,
    /// Where the captured stderr is written.
    pub stderr_path: PathBuf,
    /// How long to wait for a window to appear.
    pub window_timeout: Duration,
    /// Skip the staleness gate. An escape hatch, and it is opt-in for the
    /// reason in the module docs.
    pub allow_stale: bool,
    /// Source tree to check the binary's age against.
    pub source_root: Option<PathBuf>,
}

impl LaunchSpec {
    /// A spec with the harness's defaults.
    #[must_use]
    pub fn new(exe: impl Into<PathBuf>, stderr_path: impl Into<PathBuf>) -> Self {
        Self {
            exe: exe.into(),
            pdf: None,
            env: Vec::new(),
            stderr_path: stderr_path.into(),
            // Generous: a cold start that also has to parse and raster a large
            // CAD drawing is slow, and a timeout that fires early produces a
            // SKIP that looks like a hang.
            window_timeout: Duration::from_secs(30),
            allow_stale: false,
            source_root: None,
        }
    }
}

/// The smallest client area the harness will accept as "the window is up".
///
/// Not a guess at the application's size — a floor below which the window
/// cannot be a laid-out application window. See the polling loop in
/// [`Session::launch`] for what happens without it.
/// **Where every launched window is put**, in desktop pixels.
///
/// To the **right** of the top-left corner, which is where always-on-top
/// furniture docks — see [`Session::place`]. Measured on this machine
/// 2026-08-20: the Windows on-screen keyboard occupies `(-5, 0)-(746, 266)`,
/// and it cannot be closed from a process of ordinary integrity. `780` clears
/// it, and a 1100 px client still ends at 1880 on a 1920-wide desktop.
///
/// ★ This is a **mitigation, not the guard.** `Driver::confirm_uncovered`
/// refuses a click on a point another window owns, wherever the window is;
/// this only makes that refusal rare. A machine whose furniture docks
/// somewhere else will still be caught, and will be told so.
const SAFE_ORIGIN_X: i32 = 780;
/// See [`SAFE_ORIGIN_X`].
const SAFE_ORIGIN_Y: i32 = 40;

const MIN_CLIENT_PX: u32 = 200;

/// A running application, its captured trace, and its window.
pub struct Session {
    /// ★ `RefCell` so that liveness can be asked with `&self`.
    ///
    /// `Child::try_wait` needs `&mut`, and [`Session::trace`] — which every
    /// check calls, and which is where the liveness guard has to live to be
    /// unforgettable — takes `&self` at roughly 150 call sites. Threading `&mut`
    /// through all of them would be a 150-file mechanical diff in service of a
    /// borrow, and every one of those diffs is a chance to change a check's
    /// meaning by accident. The cell is the smaller and more honest change:
    /// nothing here is shared across threads, and the borrow is held for the
    /// length of one `try_wait`.
    child: RefCell<Child>,
    pid: u32,
    stderr_path: PathBuf,
    window: Option<WindowHandle>,
    trace_prefix: String,
    /// Whether this check has said, in as many words, that the process is
    /// allowed to have exited.
    ///
    /// See [`Session::trace`] for what this guards and why it defaults to
    /// `false`. `Cell` for the same reason the child is a `RefCell`.
    exit_expected: Cell<bool>,
}

impl Session {
    /// Launch, and wait for a window.
    ///
    /// # Errors
    ///
    /// Every error here is a **precondition** failure — the harness could not
    /// begin — so callers report SKIPPED, not FAIL. Each message names the
    /// specific thing that was missing.
    pub fn launch(spec: &LaunchSpec, trace_prefix: &str) -> Result<Self> {
        if !spec.exe.is_file() {
            return Err(Error::new(format!(
                "no binary at {}. Build it first (cargo build --release), or point the \
                 harness at one with --exe.",
                spec.exe.display()
            )));
        }
        if let Some(pdf) = &spec.pdf
            && !pdf.is_file()
        {
            return Err(Error::new(format!("no document at {}", pdf.display())));
        }
        if !spec.allow_stale
            && let Some(root) = &spec.source_root
            && let Some(msg) = staleness_complaint(&spec.exe, root)
        {
            return Err(Error::new(msg));
        }

        if let Some(dir) = spec.stderr_path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let stderr = std::fs::File::create(&spec.stderr_path).map_err(|e| {
            Error::new(format!(
                "cannot create the trace file {}: {e}",
                spec.stderr_path.display()
            ))
        })?;

        let mut cmd = Command::new(&spec.exe);
        if let Some(pdf) = &spec.pdf {
            cmd.arg(pdf);
        }
        for (k, v) in &spec.env {
            cmd.env(k, v);
        }
        cmd.stderr(Stdio::from(stderr));
        cmd.stdout(Stdio::null());
        cmd.stdin(Stdio::null());

        let child = cmd
            .spawn()
            .map_err(|e| Error::new(format!("cannot start {}: {e}", spec.exe.display())))?;
        let pid = child.id();

        let mut session = Self {
            child: RefCell::new(child),
            pid,
            stderr_path: spec.stderr_path.clone(),
            window: None,
            trace_prefix: trace_prefix.to_owned(),
            // The safe default: a check that says nothing about exiting is
            // asserting the program survived it. See `trace`.
            exit_expected: Cell::new(false),
        };

        // Poll for the window rather than sleeping a fixed time. A fixed sleep
        // is either too short on a slow machine (a SKIP that looks like a
        // crash) or wasted on a fast one, and the harness runs this per check.
        let deadline = Instant::now() + spec.window_timeout;
        while Instant::now() < deadline {
            if let Some(status) = session.child.borrow_mut().try_wait()? {
                return Err(Error::new(format!(
                    "the application exited with {status} before showing a window. Its \
                     stderr is at {}.",
                    session.stderr_path.display()
                )));
            }
            // A window is only accepted once it has a REAL client area.
            //
            // Found the expensive way, by this harness, against the old GUI: a
            // winit window is created, becomes `IsWindowVisible`, and is
            // enumerable **while its client rect is still 0x0**. Accepting it
            // there produced a window frame whose centre was the window's own
            // top-left corner, so the layout-probe click landed on the desktop
            // behind the application, the application received no input at all,
            // and the check reported "this build does not trace its canvas
            // layout" — a confident, wrong diagnosis of the program under test
            // caused entirely by the harness measuring too early.
            //
            // That is the failure class this whole crate is about, committed by
            // the crate itself, so the fix is a precondition rather than a
            // longer sleep: keep polling until the client area is big enough to
            // be a real application window.
            if let Some(w) = sys::find_window_for_pid(pid)
                && let Ok(frame) = sys::window_frame(w)
                && frame.client_size.0 >= MIN_CLIENT_PX
                && frame.client_size.1 >= MIN_CLIENT_PX
            {
                session.window = Some(w);
                session.place();
                break;
            }
            std::thread::sleep(Duration::from_millis(100));
        }

        if session.window.is_none() {
            return Err(Error::new(format!(
                "no window appeared for pid {pid} within {:?}. On a platform that cannot \
                 enumerate windows this is always the outcome, and the check is correctly \
                 reported as SKIPPED rather than failed.",
                spec.window_timeout
            )));
        }
        Ok(session)
    }

    /// The window handle.
    #[must_use]
    pub fn window(&self) -> Option<WindowHandle> {
        self.window
    }

    /// Measure the window's client area and DPI scale, now.
    ///
    /// Re-measured on demand rather than cached: the window can be moved or
    /// resized between one assertion and the next, and a cached frame would
    /// convert against a geometry that no longer exists — which produces
    /// clicks that land near the target, the hardest failure to diagnose.
    pub fn frame(&self) -> Result<WindowFrame> {
        let w = self
            .window
            .ok_or_else(|| Error::new("the session has no window"))?;
        sys::window_frame(w)
    }

    /// Bring the window to the front.
    pub fn raise(&self) {
        if let Some(w) = self.window {
            sys::raise_window(w);
        }
    }

    /// Maximise the window, so a ribbon control past the fold is on screen
    /// rather than in the overflow menu.
    ///
    /// # ★ Call this before looking for a control the tab lists LAST
    ///
    /// A ribbon overflows when it is wider than its window, and a control in
    /// the overflow **stops publishing a rect** — which a check cannot tell
    /// apart from a control that does not exist. `settings_theme` found this
    /// the hard way: it asked the File tab for `ribbon.item.file.settings`, was
    /// handed ten controls ending at `file.print`, and would have reported a
    /// shipped feature as missing.
    ///
    /// It is opt-in per check rather than done on every launch, because a
    /// maximised window is a **different layout**, and several checks measure
    /// things — the canvas rect, the find bar's placement, the page strip — for
    /// which the size is part of the subject. Making it universal would change
    /// what those are testing without changing a line of them.
    ///
    /// A no-op on platforms with no window control, exactly as [`Self::raise`]
    /// is: a check that cannot maximise still runs, against whatever size the
    /// window opened at.
    pub fn maximize(&self) {
        if let Some(w) = self.window {
            sys::maximize_window(w);
        }
    }

    /// **Put the window at a known place, clear of the top of the screen.**
    ///
    /// Two reasons, and the second is the one that cost an afternoon.
    ///
    /// **Determinism.** Letting Windows choose gets a *cascade*: every launch
    /// steps down and right from the last, so a long session marches its
    /// windows toward the edge of the desktop. A failure that depends on where
    /// the window happened to open is a failure nobody can reproduce.
    ///
    /// **★★ Always-on-top windows live at the top of the screen.** The
    /// Windows on-screen keyboard docks there, it is summoned by synthetic
    /// keystrokes — so this harness brings it on itself — and it cannot be
    /// closed from a process of ordinary integrity. It lay across the ribbon's
    /// tab row for an afternoon on 2026-08-20 and made two checks report a
    /// working ribbon as unresponsive, intermittently. `Driver::confirm_uncovered`
    /// is the guard that now catches it; this is the measure that stops it
    /// happening. Below [`SAFE_ORIGIN_Y`] there is nothing docked.
    fn place(&self) {
        if let Some(w) = self.window {
            sys::move_window(w, SAFE_ORIGIN_X, SAFE_ORIGIN_Y);
        }
    }

    /// Read and parse everything the application has written so far.
    ///
    /// Safe to call while it is still running: the trace goes to stderr
    /// unbuffered, one line per event, and reading a file another process has
    /// open for writing is permitted on Windows with the share mode Rust's
    /// `File::open` requests.
    pub fn expect_exit(&self) -> &Self {
        self.exit_expected.set(true);
        self
    }

    /// Read the trace, **and refuse to hand back a trace from a process that
    /// died unless the check said it might**.
    ///
    /// # ★★★ WHY THIS GUARD EXISTS — 2026-09-03 (evening)
    ///
    /// An outside reviewer opened `pdfcer ▸ Keyboard shortcuts` on a fresh
    /// launch and the **process aborted**, taking the operator's unsaved markup
    /// with it. `dialogs_open_in_their_own_window` drives that exact dialog and
    /// had been reporting
    ///
    /// > ★ Keyboard shortcuts is a real OS window: [[186.0 209.0] - [606.0 689.0]]
    ///
    /// **PASS, on the crashing build.** Not by luck: the `viewport-inner` line
    /// the check reads is written *before* the panic, so by the time the
    /// process died the evidence the check wanted already existed. The check
    /// was not wrong about what it asserted. It simply had no opinion about
    /// whether the program was still alive, and neither did any of the others.
    ///
    /// ⇒ **Every trace-reading check in this harness could pass on a build that
    /// crashes**, provided the crash comes after the line it greps for. That is
    /// a whole-harness defect, so the fix is in the one function they all call
    /// rather than in a rule each of them has to remember — a hand-written list
    /// inside a completeness sweep being exactly the shape this project has now
    /// been caught by three times.
    ///
    /// A check that legitimately expects an exit — `ctrl_s_after_an_edit_saves_and_the_program_is_still_running`
    /// asks the question directly, and a "does it quit cleanly" check would —
    /// calls [`Session::expect_exit`] first. That is greppable, and it is a
    /// statement rather than an omission.
    ///
    /// # Errors
    ///
    /// The exit is reported as a hard error rather than a SKIP: a process that
    /// died is a failure of the thing under test, not a missing precondition.
    /// Callers turn `Err` into SKIP by convention, so the message says plainly
    /// that this one is different, and the panic line is lifted out of the
    /// trace into the message because that is the sentence somebody needs.
    pub fn trace(&self) -> Result<Trace> {
        if !self.exit_expected.get()
            && let Some(status) = self.child.borrow_mut().try_wait()?
        {
            let tail = std::fs::read_to_string(&self.stderr_path).unwrap_or_default();
            let panic = tail
                .lines()
                .find(|l| l.contains("panicked at"))
                .unwrap_or("(no panic line in the capture)");
            return Err(Error::new(format!(
                "THE PROCESS DIED before its trace was read — {status}.\n  {panic}\n\nThis is a FAILURE of the build, not a missing precondition. The trace was still \
                 readable, and every line the check wanted may well be in it: a crash after \
                 the evidence is written is invisible to a grep. Full capture: {}\n\nIf this check genuinely expects the program to exit, say so with \
                 `session.expect_exit()` before reading the trace.",
                self.stderr_path.display()
            ))
            // ★ FATAL, so it is reported RED. Without this it would take the
            // harness's ordinary `Err` -> SKIPPED route, and a crashed program
            // reported as "did not run" is barely better than one reported as a
            // pass. See `Error::fatal`.
            .fatal());
        }
        self.trace_unchecked()
    }

    /// The trace, with no opinion about whether the process is alive.
    ///
    /// Used by [`Self::trace`] once it has decided, and by the few places that
    /// read a capture for reporting rather than for asserting.
    pub fn trace_unchecked(&self) -> Result<Trace> {
        Trace::read(&self.stderr_path, &self.trace_prefix)
    }

    /// Where the trace file is, for a failure report to point at.
    #[must_use]
    pub fn trace_path(&self) -> &Path {
        &self.stderr_path
    }

    /// The process id, for messages.
    #[must_use]
    pub fn pid(&self) -> u32 {
        self.pid
    }

    /// Wait until the application has actually **drawn** `frames` frames.
    ///
    /// Named in frames because that is the unit the thing being waited for is
    /// measured in: a raster rebuild, a layout pass, a provider swap.
    ///
    /// # ★★★ It used to be a wall clock wearing the word "frames"
    ///
    /// The whole body was `sleep(frames * 25ms)`. On an idle machine 25 ms is
    /// about a frame and the name is nearly true. **Under load it is not** — the
    /// application renders fewer frames in the same wall time, so every check
    /// that settled and then clicked was acting before the interface had caught
    /// up.
    ///
    /// Measured 2026-09-02, running the suite in batches: three checks failed
    /// with substantive, believable messages — a bookmark that went to the page
    /// and did not zoom, a canvas that stopped seeing the pointer, a list of
    /// rows that never drew — and **all three passed when re-run alone against
    /// the same binary**. The convenient reading was "contention", which
    /// explains nothing and excuses everything. The mechanism was this function.
    ///
    /// # How it waits now
    ///
    /// The application emits `frame n=<count>` on the diagnostic channel every
    /// tenth frame. This reads the newest such line, then polls until the count
    /// has advanced by `frames`. Fast when idle, patient when loaded — which is
    /// what the name always claimed.
    ///
    /// ★ **The old sleep is the floor, not the ceiling.** A short wall-clock
    /// wait still happens first, because some of what a check waits for is not a
    /// frame at all — a file written, a child viewport created, an OS window
    /// map. Removing it would trade one class of flake for another.
    ///
    /// ★★ **And there is a cap**, after which it returns rather than blocking.
    /// An application that has stopped drawing is a finding for the check's own
    /// assertions to report, in their own words, against the state they can see.
    /// A settle that waited forever would turn every such defect into a hung
    /// suite with no message at all — which is strictly less informative than
    /// the false failure this change removes.
    pub fn settle(&self, frames: u32) {
        let floor = Duration::from_millis(u64::from(frames) * 25);
        std::thread::sleep(floor);
        let Some(start) = self.frame_count() else {
            // No counter on the channel — an older binary, or diagnostics off.
            // The floor above is then the whole of the wait, which is exactly
            // the previous behaviour.
            return;
        };
        let want = start + u64::from(frames);
        // Four times the floor, which on an idle machine is never reached and on
        // a loaded one is the difference between a false failure and a true one.
        let deadline = std::time::Instant::now() + floor * 4;
        while std::time::Instant::now() < deadline {
            if self.frame_count().is_some_and(|n| n >= want) {
                return;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    /// The newest `frame n=` count on the diagnostic channel, if there is one.
    ///
    /// Reads the tail of the trace rather than the whole file: the counter is
    /// emitted every tenth frame, so the answer is always within the last few
    /// hundred bytes, and a sweep against a long-running session would otherwise
    /// re-read megabytes on every settle.
    fn frame_count(&self) -> Option<u64> {
        use std::io::{Read, Seek, SeekFrom};
        let mut f = std::fs::File::open(&self.stderr_path).ok()?;
        let len = f.metadata().ok()?.len();
        let back = len.min(8192);
        f.seek(SeekFrom::Start(len - back)).ok()?;
        let mut buf = String::new();
        f.take(back).read_to_string(&mut buf).ok();
        buf.rsplit("frame n=")
            .nth(0)
            .filter(|_| buf.contains("frame n="))
            .and_then(|rest| {
                rest.split(|c: char| !c.is_ascii_digit())
                    .next()
                    .filter(|d| !d.is_empty())
                    .and_then(|d| d.parse().ok())
            })
    }
}

impl Session {
    /// **Has the application gone?**
    ///
    /// ★★★ Added 2026-09-01 for `ctrl_s_after_an_edit_saves_and_the_program_is_still_running`,
    /// and it is the first check in this harness whose subject is the process
    /// rather than the pixels. The operator reported that pressing `Ctrl+S`
    /// after an edit **closed the program**, and nothing here could express
    /// that: every other oracle is a trace line, and a program that has exited
    /// writes none — which is indistinguishable from a missed click.
    ///
    /// `try_wait` rather than `wait`: it must never block. A check calling this
    /// is asking a question, not waiting for an answer.
    ///
    /// ★ `&mut self` is why `Session` is held mutably by the one check that
    /// uses it. Reaping here is harmless — `Drop` kills and waits again, and
    /// both tolerate an already-exited child.
    pub fn has_exited(&self) -> Result<bool> {
        Ok(self.child.borrow_mut().try_wait()?.is_some())
    }
}

impl Drop for Session {
    /// Kill the child on every path — normal return, early error, or panic.
    ///
    /// See the module docs: the alternative left an invisible window
    /// consuming the operator's pointer input, twice in one session.
    fn drop(&mut self) {
        let mut child = self.child.borrow_mut();
        let _ = child.kill();
        let _ = child.wait();
    }
}

/// Is the binary older than the sources? Returns the complaint, or `None`.
///
/// Deliberately a *complaint string* rather than a bool: the message has to
/// carry both timestamps and the rebuild command, because whoever sees it is
/// about to spend an hour diagnosing a feature that was never compiled.
pub fn staleness_complaint(exe: &Path, source_root: &Path) -> Option<String> {
    let exe_time = std::fs::metadata(exe).ok()?.modified().ok()?;
    let mut newest: Option<(PathBuf, std::time::SystemTime)> = None;
    let mut stack = vec![source_root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // `target/` is build output; its timestamps are always newer
                // than the binary and would make this gate fire on every run.
                if path.file_name().is_some_and(|n| n == "target") {
                    continue;
                }
                stack.push(path);
                continue;
            }
            let is_source = path.extension().is_some_and(|e| e == "rs" || e == "toml");
            if !is_source {
                continue;
            }
            if let Ok(t) = entry.metadata().and_then(|m| m.modified())
                && newest.as_ref().is_none_or(|(_, best)| t > *best)
            {
                newest = Some((path, t));
            }
        }
    }

    let (path, t) = newest?;
    if t <= exe_time {
        return None;
    }
    // ★ The GAP, in words, rather than two `SystemTime` debug prints.
    //
    // This message is read by somebody about to spend an hour on a feature that
    // was never compiled, and it printed
    // `SystemTime { intervals: 134324724498206576 }` twice — which carries the
    // information and does not deliver it. **How far behind the binary is
    // decides what the reader does next**: two minutes is a rebuild they
    // forgot; three hours is a session's work that never ran, which is exactly
    // what happened to a 108-check sweep on 2026-08-29.
    let behind = t.duration_since(exe_time).map_or_else(
        |_| "an unmeasurable amount".to_owned(),
        |d| {
            let secs = d.as_secs();
            if secs <= 90 {
                format!("{secs} second(s)")
            } else if secs <= 5400 {
                format!("{} minute(s)", secs / 60)
            } else {
                format!("{} hour(s) {} minute(s)", secs / 3600, (secs % 3600) / 60)
            }
        },
    );
    Some(format!(
        "STALE BINARY — refusing to run.\n  binary : {}\n  newest : {}\n\nThe source is \
         {behind} newer than the binary.\n\nThe traces you are about to collect would describe \
         code that is NOT the code you just wrote, and a missing trace looks exactly like a \
         broken feature.\n\n  cargo build --release\n\nPass --allow-stale only if you intend \
         to drive the older build.",
        exe.display(),
        path.display(),
    ))
}
