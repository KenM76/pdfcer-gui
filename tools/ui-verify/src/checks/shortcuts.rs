//! `shortcuts` — **the keyboard reference opens, and every chord it declares
//! names a command this build has.**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/shortcuts.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The command that opens the window.
const ITEM: &str = "ribbon.item.file.shortcuts";
/// The body region `dialogs::shortcuts` publishes.
const BODY: &str = "dialog:shortcuts";
/// The region the row list publishes.
const LIST: &str = "shortcuts.list";
/// The census line the dialog traces as it folds the keymap.
const CENSUS: &str = "shortcuts-listed";

/// See the module documentation.
pub struct ShortcutsReferenceIsLive;

impl Check for ShortcutsReferenceIsLive {
    fn name(&self) -> &'static str {
        "shortcuts_reference_is_live"
    }

    fn defect(&self) -> &'static str {
        "the keyboard reference does not open, opens empty, or lists fewer commands than the \
         keymap binds — so a chord that is declared and dispatches nothing is invisible to the \
         one surface whose job is to say which keys work (DEFECTS.md D5)"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks a ribbon control. Reported as \
             SKIPPED rather than passed.",
        ));
    }
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;

    // ★ No `--pdf`, and that is an assertion rather than a saving.
    //
    // `DialogsState::show` draws this window **above** the no-document guard,
    // beside About, with a comment saying why: a keyboard reference an operator
    // opened before loading anything must not vanish because nothing is loaded.
    // Driving it on an empty shell is what proves the ordering survived, and it
    // is a one-line edit away from being wrong at any time.
    let mut spec = LaunchSpec::new(&exe, ctx.out("shortcuts.trace.txt"));
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.note(format!(
        "launched {} as pid {} with NO document — the state this window is drawn above the \
         guard for",
        exe.display(),
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);
    // ★★ MAXIMISE — at the harness's default 1,100 pt window the File tab's
    // last two groups fold away entirely and this check reports a lost
    // command. See `about.rs` for the measurement; three checks shared it.
    session.maximize();
    session.settle(20);
    let driver = Driver::new(session.window());

    // --- A: open it --------------------------------------------------------
    // Through the overflow when the ribbon has folded it there — at the
    // harness's window width the File tab's rightmost groups correctly are.
    let item = crate::checks::driving::declared_or_in_overflow(&session, &driver, ui_rect, ITEM)?
        .ok_or_else(|| {
        Error::new(format!(
            "no `{ITEM}` region on the File tab or in its overflow. Items declared: {}.",
            list(&declared_names(
                &session.trace().unwrap_or_default(),
                ui_rect,
                "ribbon.item.file."
            ))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(item))?;
    session.settle(20);

    let trace = session.trace()?;
    if declared(&trace, ui_rect, BODY).is_none() {
        return Ok(Some(format!(
            "clicking Keyboard shortcuts declared no `{BODY}` region, so the window did not open \
             — or opened and drew nothing. Regions declared this run: {}.",
            list(&declared_names(&trace, ui_rect, "dialog:"))
        )));
    }
    report.note("the reference opened and declared its body with no document loaded");

    // --- B: the census, which is the whole point ---------------------------
    let Some(line) = trace.events(CENSUS).last() else {
        return Ok(Some(format!(
            "the window opened and traced no `{CENSUS}` line, so it drew its body and never \
             folded the keymap. The list is derived at draw time in \
             `dialogs::shortcuts::rows_from`; no line means the fold did not run."
        )));
    };
    let commands: usize = line
        .get("commands")
        .and_then(|v| v.parse().ok())
        .unwrap_or_default();
    let dropped: usize = line
        .get("dropped")
        .and_then(|v| v.parse().ok())
        .unwrap_or_default();

    if commands == 0 {
        return Ok(Some(format!(
            "the reference listed ZERO commands. Either the manifest's keymap failed to load — \
             in which case no chord works at all this session, and the window says so in its own \
             words — or it loaded and binds nothing. `dropped={dropped}`. Look at \
             `crates/pdfcer-gui/src/shell/ron/built_in.ron`."
        )));
    }

    // ★ THE assertion. See this module's header for why zero is required and
    // why it is not tautological.
    //
    // ⚠ If pdfcer is ever built with a capability stripped — the exe-to-DLL move
    // makes that the ordinary case — a non-zero count here is CORRECT, and the
    // window discloses it in words. The fix then is to teach this check which
    // build it is looking at (compare against the registry's own census, not
    // against zero), NOT to relax the comparison. A gate that accepts any value
    // is a gate that has stopped asking the question.
    if dropped > 0 {
        return Ok(Some(format!(
            "{dropped} chord(s) in this build's keymap name a command the registry does not \
             have, so those keys are declared and do NOTHING when pressed. On a full build this \
             must be zero — it means the manifest and the command registry have drifted. \
             {commands} command(s) did list. Compare `shell::ron::built_in.ron`'s keymap against \
             `shell::commands::catalog`."
        )));
    }
    report.note(format!(
        "{commands} commands have a chord, and every chord names a command this build registered"
    ));

    // --- C: the rows reached the screen ------------------------------------
    //
    // Separate from B on purpose. The census proves the FOLD produced rows; the
    // region proves they were LAID OUT. A window that computed 40 rows and drew
    // them into a zero-height area passes B and fails here, and that is exactly
    // the class of defect `D:\dev\rag\egui\` records as having shipped with
    // every gate green.
    let Some(rect) = declared(&trace, ui_rect, LIST) else {
        return Ok(Some(format!(
            "the reference folded {commands} commands and declared no `{LIST}` region, so the \
             rows were computed and never laid out. Regions declared: {}.",
            list(&declared_names(&trace, ui_rect, "shortcuts."))
        )));
    };
    if rect.height() <= 1.0 {
        return Ok(Some(format!(
            "the `{LIST}` region is {:.1} pt high with {commands} commands folded into it, so \
             the rows were laid out into no space — the window is a title over an empty band.",
            rect.height()
        )));
    }
    report.note(format!(
        "the list drew {:.0} x {:.0} pt",
        rect.width(),
        rect.height()
    ));

    // --- D: the picture ----------------------------------------------------
    let shot = ctx.out("shortcuts.png");
    match crate::capture::window_to_png(&session, &shot) {
        Ok(_) => {
            report.artifact(shot);
        }
        Err(e) => {
            report.note(format!(
                "the window could not be captured ({e}); the trace assertions above still hold"
            ));
        }
    }
    Ok(None)
}
