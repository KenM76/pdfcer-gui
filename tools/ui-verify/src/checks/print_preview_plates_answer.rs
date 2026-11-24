//! `print_preview_plates_keep_the_window_answering` — the print preview's
//! plates render off the window's thread, are kept across sheets, keep the
//! next-ink arrow still, and a typed sheet number goes to that sheet (O460).
//!
//! Opens Print (off the desktop, scripted pointer) on the licensed output suite's
//! X4 test pages, chooses Separations and the plates switch, then:
//!
//! 1. Plates on, then straight to sheet 2. Oracle: sheet 2 and its plates
//!    land, and no frame meanwhile is held a second, or half as long as the
//!    slowest sheet or plates render (`frame-long ms=` against `print-preview-sheet ms=`
//!    and `print-preview-plates ms=`).
//! 2. Waits for sheet 2's plates, steps back to sheet 1 and on to sheet 2.
//!    Oracle: sheet 2's plates show again with no new `print-preview-plates
//!    page=1` line.
//! 3. Presses the next-ink arrow twice. Oracle: its declared rect is the same
//!    before and after.
//! 4. Types `3` into the sheet-number box and presses Enter. Oracle: a
//!    `print-preview-render page=2` line follows.
//!
//! Skips when that file is not on this machine.

use crate::checks::driving::{SHELL_DIAG_ENV, declared_in};
use crate::checks::region_press::press_region;
use crate::checks::{Check, CheckContext, CheckReport};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};

const OFFSCREEN: &str = "-4200,-4200,1400,900";
/// Not in the repository: the operator's copy of the licensed output suite,
/// found by its file-name suffix so the suite's name stays out of this tree.
const SUITES: &str = "D:/Dev/pdfTests";
const SUITE_FILE: &str = "_PDF-Output-Test-V50_ALL_X4.pdf";
const RENDER: &str = "print-preview-render";
const PLATES: &str = "print-preview-plates";
const HELD: &str = "print-preview-plates-held";
const SHEET: &str = "print-preview-sheet";
const LONG: &str = "frame-long"; // ui-text-exempt: a trace event name
const UI_RECT: &str = "ui-rect";
const TAB: &str = "print.tab.colour";
const SEPARATIONS: &str = "print.colour.separations";
const SWITCH: &str = "print.preview.plates";
const NEXT_INK: &str = "print.preview.plate-next";
const NEXT_SHEET: &str = "print.preview.next";
const PREVIOUS_SHEET: &str = "print.preview.previous";
const SHEET_BOX: &str = "print.preview.sheet-number";
/// A frame held this long, milliseconds, is a frozen window whatever the
/// render took.
const HELD_MS: u64 = 1000;
/// How long a plates render may take before the check gives up, seconds.
const PLATES_WAIT_S: u64 = 180;

pub struct PrintPreviewPlatesAnswer;

impl Check for PrintPreviewPlatesAnswer {
    fn name(&self) -> &'static str {
        "print_preview_plates_keep_the_window_answering"
    }

    fn defect(&self) -> &'static str {
        "the print preview's plates freeze the window while they render, are rendered again \
         on a sheet already shown, move the next-ink arrow, or a typed sheet number does not \
         go to that sheet"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive(ctx, &mut report) {
            Ok(failures) if failures.is_empty() => report.pass(),
            Ok(failures) => report.fail(failures.join("; ")),
            Err(why) => report.from_error(&why),
        }
    }
}

fn launch(ctx: &CheckContext, report: &mut CheckReport) -> Result<(Session, ScriptedPointer)> {
    let exe = ctx
        .resolve_exe()
        .ok_or_else(|| Error::new("no binary to drive. Pass --exe, or build the default."))?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let suite = suite_file().ok_or_else(|| {
        Error::new(format!(
            "no `*{SUITE_FILE}` under {SUITES} on this machine."
        ))
    })?;
    let doc = ctx.out("suite-x4.pdf");
    std::fs::copy(&suite, &doc)
        .map_err(|e| Error::new(format!("copying the suite's X4 file: {e}")))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("print_preview_plates_answer.trace.txt"));
    spec.pdf = Some(doc);
    for (k, v) in [
        (ctx.profile.diag_env.0, ctx.profile.diag_env.1),
        SHELL_DIAG_ENV,
        (viewport_env, OFFSCREEN),
        ("PDFCER_DIAG_INVOKE", "file.print"),
        ("PDFCER_DIAG_PRINT_DRY_RUN", "1"),
    ] {
        spec.env.push((k.to_owned(), v.to_owned()));
    }
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(
        &mut spec,
        ctx.out("print_preview_plates_answer.pointer.txt"),
    )?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(60);
    Ok((session, pointer))
}

/// Wait until an `event` line for document page `page` that is not
/// cancelled lands after `mark`, and return it.
/// The suite's X4 file: `SUITE_FILE` one folder below `SUITES`.
fn suite_file() -> Option<std::path::PathBuf> {
    std::fs::read_dir(SUITES)
        .ok()?
        .flatten()
        .filter_map(|dir| std::fs::read_dir(dir.path()).ok())
        .flat_map(|files| files.flatten())
        .map(|file| file.path())
        .find(|path| {
            path.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.ends_with(SUITE_FILE))
        })
}

fn landed(session: &Session, event: &str, page: usize, mark: usize) -> Result<String> {
    let wanted = page.to_string();
    for _ in 0..PLATES_WAIT_S * 4 {
        let trace = session.trace()?;
        if let Some(line) = trace
            .events(event)
            .filter(|l| l.lineno > mark && l.get("page") == Some(wanted.as_str()))
            .find(|l| l.get("outcome") != Some("cancelled"))
        {
            return Ok(line.raw.clone());
        }
        std::thread::sleep(std::time::Duration::from_millis(250));
    }
    Err(Error::new(format!(
        "no `{event} page={page}` landed within {PLATES_WAIT_S} s."
    )))
}

fn ms(line: &str) -> u64 {
    line.split(" ms=")
        .nth(1)
        .and_then(|r| r.split(' ').next())
        .and_then(|n| n.parse().ok())
        .unwrap_or(0)
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Vec<String>> {
    let (session, pointer) = launch(ctx, report)?;
    let mut failures = Vec::new();
    press_region(&session, &pointer, TAB, "print.tab.")?;
    press_region(&session, &pointer, SEPARATIONS, "print.colour.")?;
    session.settle(30);
    render_off_thread(&session, &pointer, report, &mut failures)?;
    plates_held(&session, &pointer, report, &mut failures)?;
    arrow_still(&session, &pointer, report, &mut failures)?;
    sheet_typed(&session, &pointer, report, &mut failures)?;
    pointer.gone(&session)?;
    Ok(failures)
}

fn render_off_thread(
    session: &Session,
    pointer: &ScriptedPointer,
    report: &mut CheckReport,
    failures: &mut Vec<String>,
) -> Result<()> {
    // 1. Plates on and straight to sheet 2: neither render holds a frame.
    let start = session.trace()?.mark();
    press_region(session, pointer, SWITCH, "print.preview.")?;
    press_region(session, pointer, NEXT_SHEET, "print.preview.")?;
    let drawn = landed(session, RENDER, 1, start)?;
    let second = landed(session, PLATES, 1, start)?;
    report.note(format!("sheet 2 drew `{drawn}`; its plates: `{second}`"));
    let trace = session.trace()?;
    let slowest = trace
        .events(SHEET)
        .chain(trace.events(PLATES))
        .filter(|l| l.lineno > start && l.get("outcome") != Some("cancelled"))
        .map(|l| ms(&l.raw))
        .max()
        .unwrap_or(0);
    let longest = trace
        .events(LONG)
        .filter(|l| l.lineno > start)
        .map(|l| ms(&l.raw))
        .max()
        .unwrap_or(0);
    report.note(format!(
        "slowest render {slowest} ms; longest frame meanwhile {longest} ms"
    ));
    if slowest < 600 {
        return Err(Error::new(format!(
            "the slowest render took {slowest} ms; a held frame cannot be told from one that short."
        )));
    }
    if longest >= HELD_MS || longest * 2 >= slowest {
        failures.push(format!(
            "a frame was held {longest} ms against a {slowest} ms render: a render ran on the window's thread"
        ));
    }
    Ok(())
}

fn plates_held(
    session: &Session,
    pointer: &ScriptedPointer,
    report: &mut CheckReport,
    failures: &mut Vec<String>,
) -> Result<()> {
    // 2. Back to sheet 1 and on to sheet 2: sheet 2's plates are held.
    press_region(session, pointer, PREVIOUS_SHEET, "print.preview.")?;
    let back = session.trace()?.mark();
    press_region(session, pointer, NEXT_SHEET, "print.preview.")?;
    session.settle(30);
    landed(session, RENDER, 1, back)?;
    let trace = session.trace()?;
    let again: Vec<&str> = trace
        .events(PLATES)
        .filter(|l| l.lineno > back && l.get("page") == Some("1"))
        .map(|l| l.raw.as_str())
        .collect();
    let held = trace.last_after(HELD, back).is_some();
    let shown = trace
        .events(RENDER)
        .filter(|l| l.lineno > back && l.get("page") == Some("1"))
        .last()
        .map(|l| l.raw.clone());
    report.note(format!("back on sheet 2: held={held}, drew {shown:?}"));
    if !again.is_empty() {
        failures.push(format!("sheet 2's plates rendered again: {again:?}"));
    }
    if !shown.as_deref().is_some_and(|l| !l.contains("plate=none")) {
        failures.push(format!(
            "back on sheet 2 the preview drew {shown:?}, not a plate"
        ));
    }
    Ok(())
}

fn arrow_still(
    session: &Session,
    pointer: &ScriptedPointer,
    report: &mut CheckReport,
    failures: &mut Vec<String>,
) -> Result<()> {
    // 3. The next-ink arrow stays put.
    let rect_now = |s: &Session| -> Result<String> {
        let trace = s.trace()?;
        declared_in(&trace, UI_RECT, NEXT_INK)
            .map(|(r, _)| format!("{r:?}"))
            .ok_or_else(|| Error::new(format!("no `{NEXT_INK}` region")))
    };
    let before = rect_now(session)?;
    press_region(session, pointer, NEXT_INK, "print.preview.")?;
    let middle = rect_now(session)?;
    press_region(session, pointer, NEXT_INK, "print.preview.")?;
    let after = rect_now(session)?;
    report.note(format!("next-ink arrow: {before} → {middle} → {after}"));
    if before != middle || middle != after {
        failures.push(format!(
            "the next-ink arrow moved as the ink changed: {before} → {middle} → {after}"
        ));
    }
    Ok(())
}

fn sheet_typed(
    session: &Session,
    pointer: &ScriptedPointer,
    report: &mut CheckReport,
    failures: &mut Vec<String>,
) -> Result<()> {
    // 4. A typed sheet number.
    let (rect, viewport) = declared_in(&session.trace()?, UI_RECT, SHEET_BOX)
        .ok_or_else(|| Error::new(format!("no `{SHEET_BOX}` region")))?;
    pointer.click_in(session, viewport.as_deref(), WindowPoint::centre_of(rect))?;
    session.settle(10);
    pointer.key(session, viewport.as_deref(), "A", Some("ctrl"))?;
    let typed = session.trace()?.mark();
    pointer.type_text(session, viewport.as_deref(), "3")?;
    pointer.key(session, viewport.as_deref(), "Enter", None)?;
    session.settle(30);
    let went = landed(session, RENDER, 2, typed).ok();
    let trace = session.trace()?;
    report.note(format!(
        "typed 3: {:?} / {went:?}",
        trace
            .last_after("print-preview-sheet-typed", typed)
            .map(|l| &l.raw)
    ));
    if went.is_none() {
        failures.push("typing 3 and Enter in the sheet box did not draw sheet 3".to_owned());
    }
    Ok(())
}
