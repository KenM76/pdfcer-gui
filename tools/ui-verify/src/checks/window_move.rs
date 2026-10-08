//! `a_document_moves_between_windows` — View ▸ Window's two moves, driven
//! across four pdfcer-gui windows: one window sends its only document to
//! another and closes; the receiver tears one of its two documents off into a
//! window of its own; then, with two other windows open, it is asked which one
//! takes its last document, picks one, and closes. Every window is placed off
//! the desktop and driven by the scripted pointer.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/window_move.md`.

use crate::checks::driving::{self, SHELL_DIAG_ENV};
use crate::checks::{Check, CheckContext, CheckReport};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::trace::TraceLine;

const OFFSCREEN: &str = "-4200,-4200,1400,900";
const TAB: &str = "view"; // ui-text-exempt: a ribbon tab id
const TO_OTHER: &str = "ribbon.item.view.move_to_window"; // ui-text-exempt: a trace region name
const TO_NEW: &str = "ribbon.item.view.move_to_new_window"; // ui-text-exempt: a trace region name
const PICK_PREFIX: &str = "window-pick."; // ui-text-exempt: a trace region name
const PEERS: &str = "window-peers"; // ui-text-exempt: a trace event name
const RECEIVED: &str = "window-move-received"; // ui-text-exempt: a trace event name
const SENT: &str = "window-move-sent"; // ui-text-exempt: a trace event name
const EMPTIED: &str = "window-emptied"; // ui-text-exempt: a trace event name
const TORN_OFF: &str = "window-torn-off"; // ui-text-exempt: a trace event name
const TABS: &str = "doc-tabs"; // ui-text-exempt: a trace event name

/// See the module documentation.
pub struct ADocumentMovesBetweenWindows;

impl Check for ADocumentMovesBetweenWindows {
    fn name(&self) -> &'static str {
        "a_document_moves_between_windows"
    }

    fn defect(&self) -> &'static str {
        "A document cannot be moved to another pdfcer-gui window or torn off into its own: the \
         command is missing or greyed, the other window does not open it, or the window it left \
         keeps it"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let mut torn = Reaper(None);
        match assess(ctx, &mut report, &mut torn) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// Kills the torn-off window, which no [`Session`] owns, on every path.
struct Reaper(Option<u32>);

impl Drop for Reaper {
    fn drop(&mut self) {
        if let Some(pid) = self.0 {
            let _ = std::process::Command::new("taskkill")
                .args(["/PID", &pid.to_string(), "/F"])
                .output();
        }
    }
}

/// A launched window and its pointer.
struct Window {
    session: Session,
    pointer: ScriptedPointer,
    /// The document it was launched on.
    doc: std::path::PathBuf,
}

fn assess(
    ctx: &CheckContext,
    report: &mut CheckReport,
    torn: &mut Reaper,
) -> Result<Option<String>> {
    let ui_rect = ctx.profile.vocab.ui_rect_event.unwrap_or("ui-rect");
    let a = launch(ctx, report, "four-pages.pdf", "a")?;
    let b = launch(ctx, report, "pure-k-square.pdf", "b")?;

    // A's only document goes to B, and A closes.
    if let Some(failure) = send_only_document(&a, &b, ui_rect)? {
        return Ok(Some(failure));
    }
    // B tears A's document off into a window of its own.
    let Some(c) = tear_off(&b, ui_rect, report)? else {
        return Ok(Some(format!(
            "Move to new window traced no `{TORN_OFF}` line. Trace: {}.",
            b.session.trace_path().display()
        )));
    };
    torn.0 = Some(c);
    // With C and a new D open, B is asked which takes its last document.
    let d = launch(ctx, report, "four-pages.pdf", "d")?;
    if let Some(failure) = peers_reach(&b, 2)? {
        return Ok(Some(failure));
    }
    if let Some(failure) = torn_off_opened(ctx, c, &a.doc)? {
        return Ok(Some(failure));
    }
    let judged = pick_destination(&b, &d, ui_rect, report);
    d.pointer.gone(&d.session)?;
    judged
}

/// A sends its one document to B: B receives it and shows two tabs, and A
/// closes.
fn send_only_document(a: &Window, b: &Window, ui_rect: &str) -> Result<Option<String>> {
    if let Some(failure) = peers_reach(a, 1)? {
        return Ok(Some(failure));
    }
    crate::checks::ocr::click_tab(&a.session, &a.pointer, ui_rect, TAB)?;
    a.session.expect_exit();
    click(a, ui_rect, TO_OTHER)?;
    let Some(received) = await_line(&b.session, RECEIVED, |_| true)? else {
        return Ok(Some(format!(
            "the first window was told to move its document and the second traced no \
             `{RECEIVED}` line. First trace: {}.",
            a.session.trace_path().display()
        )));
    };
    if !names(&received, &a.doc) {
        return Ok(Some(format!(
            "the second window received `{}`.",
            received.raw
        )));
    }
    if await_line(&b.session, TABS, |l| l.get("open") == Some("2"))?.is_none() {
        return Ok(Some(
            "the second window never showed two document tabs.".to_owned(),
        ));
    }
    let sent = await_line(&a.session, SENT, |_| true)?;
    let emptied = await_line(&a.session, EMPTIED, |_| true)?;
    if sent.is_none() || emptied.is_none() || !exits(&a.session) {
        return Ok(Some(format!(
            "the first window gave its only document away and did not close: `{SENT}` {:?}, \
             `{EMPTIED}` {:?}.",
            sent.map(|l| l.raw),
            emptied.map(|l| l.raw)
        )));
    }
    Ok(None)
}

/// B tears its active document off. The new window's process id, when B
/// traced one.
fn tear_off(b: &Window, ui_rect: &str, report: &mut CheckReport) -> Result<Option<u32>> {
    crate::checks::ocr::click_tab(&b.session, &b.pointer, ui_rect, TAB)?;
    click(b, ui_rect, TO_NEW)?;
    let Some(line) = await_line(&b.session, TORN_OFF, |_| true)? else {
        return Ok(None);
    };
    report.note(format!("tear-off: `{}`", line.raw));
    Ok(line.get("pid").and_then(|p| p.parse().ok()))
}

/// The torn-off window published the document it was started on.
fn torn_off_opened(ctx: &CheckContext, pid: u32, doc: &std::path::Path) -> Result<Option<String>> {
    let exe = ctx
        .resolve_exe()
        .ok_or_else(|| Error::new("no binary to drive. Pass --exe."))?;
    let file = exe.parent().map(|d| {
        d.join("userdata")
            .join("windows")
            .join(format!("{pid}.txt"))
    });
    let text = file.as_ref().and_then(|f| std::fs::read_to_string(f).ok());
    let name = doc.file_name().map(|n| n.to_string_lossy().into_owned());
    match (&text, &name) {
        (Some(text), Some(name)) if text.contains(name.as_str()) => Ok(None),
        _ => Ok(Some(format!(
            "the torn-off window ({pid}) published {text:?} at {file:?}, not the document it was \
             started on ({name:?})."
        ))),
    }
}

/// B, holding one document with two windows to send it to, asks which; D is
/// picked, receives it, and B closes.
fn pick_destination(
    b: &Window,
    d: &Window,
    ui_rect: &str,
    report: &mut CheckReport,
) -> Result<Option<String>> {
    b.session.expect_exit();
    click(b, ui_rect, TO_OTHER)?;
    b.session.settle(20);
    let pick = format!("{PICK_PREFIX}{}", d.session.pid());
    let trace = b.session.trace()?;
    let Some(rect) = driving::declared(&trace, ui_rect, &pick) else {
        return Ok(Some(format!(
            "with two other windows open, Move to other window drew no `{pick}` choice. Choices \
             drawn: {}.",
            driving::list(&driving::declared_names(&trace, ui_rect, PICK_PREFIX))
        )));
    };
    crate::input::Click::click_rect(&b.pointer, &b.session, rect)?;
    let Some(received) = await_line(&d.session, RECEIVED, |_| true)? else {
        return Ok(Some(format!(
            "the picked window traced no `{RECEIVED}` line. Trace: {}.",
            d.session.trace_path().display()
        )));
    };
    report.note(format!("picked window: `{}`", received.raw));
    if !names(&received, &b.doc) {
        return Ok(Some(format!(
            "the picked window received `{}`.",
            received.raw
        )));
    }
    if await_line(&b.session, EMPTIED, |_| true)?.is_none() || !exits(&b.session) {
        return Ok(Some(
            "the window whose last document moved away did not close.".to_owned(),
        ));
    }
    Ok(None)
}

/// Wait until `w` sees `n` other windows, moving its pointer so it runs
/// frames to look.
fn peers_reach(w: &Window, n: usize) -> Result<Option<String>> {
    let want = n.to_string();
    for _ in 0..40 {
        let trace = w.session.trace()?;
        if trace.last(PEERS).and_then(|l| l.get("n")) == Some(want.as_str()) {
            return Ok(None);
        }
        w.pointer.gone(&w.session)?;
        w.session.settle(10);
    }
    let trace = w.session.trace()?;
    let last = trace.last(PEERS).map(|l| l.raw.clone());
    Ok(Some(format!(
        "window {} never saw {n} other window(s); its last `{PEERS}` line was {last:?}.",
        w.session.pid()
    )))
}

/// Whether the line's `path` names `doc`.
fn names(line: &TraceLine, doc: &std::path::Path) -> bool {
    let name = doc.file_name().map(|n| n.to_string_lossy().into_owned());
    name.is_some_and(|n| line.raw.contains(&n))
}

/// Whether the process ends within a few seconds.
fn exits(session: &Session) -> bool {
    for _ in 0..40 {
        if session.has_exited().unwrap_or(false) {
            return true;
        }
        session.settle(4);
    }
    false
}

fn await_line(
    session: &Session,
    name: &str,
    ok: impl Fn(&TraceLine) -> bool,
) -> Result<Option<TraceLine>> {
    for _ in 0..30 {
        let trace = session.trace()?;
        if let Some(line) = trace.events(name).filter(|l| ok(l)).last() {
            return Ok(Some(line.clone()));
        }
        session.settle(10);
    }
    Ok(None)
}

fn click(w: &Window, ui_rect: &str, name: &str) -> Result<()> {
    let Some(item) = driving::declared_or_in_overflow(&w.session, &w.pointer, ui_rect, name)?
    else {
        return Err(Error::new(format!(
            "the View tab declares no `{name}`, on the band or in a collapsed group."
        )));
    };
    crate::input::Click::click_rect(&w.pointer, &w.session, item)?;
    w.session.settle(30);
    Ok(())
}

fn launch(
    ctx: &CheckContext,
    report: &mut CheckReport,
    fixture: &str,
    role: &str,
) -> Result<Window> {
    let exe = ctx
        .resolve_exe()
        .ok_or_else(|| Error::new("no binary to drive. Pass --exe."))?;
    let viewport_env = ctx
        .profile
        .viewport_env
        .ok_or_else(|| Error::new("the profile has no viewport variable."))?;
    let source = driving::repo_fixture(fixture, "It is checked in.")?;
    let doc = ctx.out(&format!("window-move-{role}.pdf"));
    std::fs::copy(&source, &doc).map_err(|e| Error::new(format!("copying {fixture}: {e}")))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out(&format!("window-move-{role}.trace.txt")));
    spec.pdf = Some(doc.clone());
    for (k, v) in [
        (ctx.profile.diag_env.0, ctx.profile.diag_env.1),
        SHELL_DIAG_ENV,
        (viewport_env, OFFSCREEN),
    ] {
        spec.env.push((k.to_owned(), v.to_owned()));
    }
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(
        &mut spec,
        ctx.out(&format!("window-move-{role}.pointer.txt")),
    )?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!("window {role}: pid {}", session.pid()));
    session.settle(40);
    Ok(Window {
        session,
        pointer,
        doc,
    })
}
