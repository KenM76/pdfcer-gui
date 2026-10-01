//! `read_mode_hides_the_chrome` — the regression test for **`view.read_mode`,
//! a command that had a control, a glyph, a group, a chord and a line in the
//! shortcuts reference, and no dispatch arm at all.**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/read_mode_chrome.md`.

use crate::checks::driving::{
    self, INVOKE_EVENT, ITEM_PREFIX, SHELL_DIAG_ENV, SHELL_TRACE_PREFIX, TAB_EVENT,
    UNIMPLEMENTED_EVENT, declared, declared_names, list, shell_trace,
};
use crate::checks::{Check, CheckContext, CheckReport};
use crate::error::{Error, Result};
use crate::geom::LRect;
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};

/// The command this check is about.
const SUBJECT_ID: &str = "view.read_mode";

/// The region the ribbon publishes for its control.
const SUBJECT: &str = "ribbon.item.view.read_mode";

/// Its twin in the same ribbon group, driven by phase 0.
const FULLSCREEN_ID: &str = "view.fullscreen";

/// The region the ribbon publishes for the twin's control.
const FULLSCREEN: &str = "ribbon.item.view.fullscreen";

/// `fullscreen asked=…` — `app::dispatch`'s own line for that arm.
const FULLSCREEN_EVENT: &str = "fullscreen";

/// `fullscreen-toggle reported=… pending=… asked=…` — the application's own
/// account of **why** it asked for what it asked for.
const TOGGLE_EVENT: &str = "fullscreen-toggle";

/// How many times a full-screen press is retried before the harness gives up
/// on it.
const PRESS_TRIES: usize = 3;

/// The tab it lives on, and the region that activates it.
const TAB_ID: &str = "view";
const TAB: &str = "ribbon.tab.view";

/// `read-mode on=…` — `app::dispatch`'s own line for the arm.
///
/// The event that separates "the command was invoked" from "the command ran",
/// which is the distinction phase B exists for.
const READ_MODE_EVENT: &str = "read-mode";

/// The region the canvas declares for itself, every frame.
const CANVAS: &str = "central-panel";

/// How far the canvas's top edge must rise, in logical points, for the ribbon
/// to be counted as gone.
const MIN_RISE_PTS: f32 = 40.0;

/// How different the pixels where the ribbon was must be, as a maximum
/// absolute per-channel difference in 0–255.
const MIN_REPAINT_DELTA: u16 = driving::MIN_PRESSED_DELTA;

/// Where and how large the window is placed, as `PDFCER_DIAG_VIEWPORT` takes it
/// (`x,y,w,h` in logical points).
const VIEWPORT: &str = "0,0,2560,1000";

pub struct ReadModeHidesTheChrome;

impl Check for ReadModeHidesTheChrome {
    fn name(&self) -> &'static str {
        "read_mode_hides_the_chrome"
    }

    fn defect(&self) -> &'static str {
        "View ▸ Read mode is drawn, bound to Ctrl+H, and does not hide the ribbon or the panels"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match assess(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn assess(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    // A document is a precondition rather than a convenience: with nothing
    // open the dock mounts nothing and the canvas draws one sentence, so
    // "the chrome went away" would be measuring a window that had very little
    // chrome to begin with.
    let pdf = ctx.pdf.clone().ok_or_else(|| {
        Error::new(
            "no --pdf. Read mode hides the ribbon AND the panels, and the panels are mounted \
             only for an open document — so without one this check would be asserting half of \
             what it claims to.",
        )
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input), and this check is two clicks on ribbon controls. \
             Reported as SKIPPED rather than passed — a check that did not run has learned \
             nothing.",
        ));
    }

    let mut spec = LaunchSpec::new(&exe, ctx.out("read_mode_chrome.trace.txt"));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    // **The only check in the suite that places its own window, and the
    // reason is a finding rather than a convenience.**
    //
    // At the shipped default of 1100 × 800 the View tab's band overflows after
    // its third group: `page_display`, `render` and `navigate` are drawn and
    // `zoom`, `display`, `panels` and **`window`** are folded into the band's
    // `»` affordance. Read mode is in the last of those, so at the default size
    // its control is reachable only through a menu — and a menu's contents are
    // not published as regions, so this harness cannot aim at one.
    //
    // Widening the window is therefore what makes the control clickable at all.
    // What it must not be mistaken for is a claim that the control is on screen
    // at any ordinary size: it is not, and that is a real (pre-existing) ribbon
    // finding about View's seven groups rather than anything to do with the
    // arm this check is about. The chord `Ctrl+H` and the overflow menu are the
    // operator's routes on a small window.
    //
    // `PDFCER_DIAG_VIEWPORT` also switches `with_active` off, which is harmless
    // here: `Driver::click_at` raises the target window before every click, and
    // that raise is what every driving check already depends on.
    if let Some(name) = ctx.profile.viewport_env {
        spec.env.push((name.to_owned(), VIEWPORT.to_owned()));
    }
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.note(format!(
        "launched {} as pid {}",
        exe.display(),
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());
    // Long enough for the first page of a dense CAD sheet to raster: a capture
    // taken mid-render would differ from the one after for reasons that have
    // nothing to do with the ribbon.
    session.settle(60);

    let trace = session.trace()?;
    if !trace.started(ctx.profile.vocab.start_event) {
        return Err(Error::new(format!(
            "the trace has no `{}` line, so the diagnostic switch {}={} did not reach the \
             process and nothing below could be observed. Captured stderr is at {}.",
            ctx.profile.vocab.start_event,
            ctx.profile.diag_env.0,
            ctx.profile.diag_env.1,
            session.trace_path().display()
        )));
    }

    let driver = Driver::new(session.window());
    let ui_rect = ctx.profile.vocab.ui_rect_event.unwrap_or("ui-rect");

    // --- A. the View tab, and the canvas as it stands with chrome ----------
    let tab = click_tab(&session, &driver, ui_rect)?;

    // --- 0. full screen, there and back, before anything is hidden ---------
    if let Some(failure) = fullscreen_round_trip(&session, &driver, report, ui_rect)? {
        return Ok(Some(failure));
    }

    let trace = session.trace()?;
    let before_canvas = declared(&trace, ui_rect, CANVAS).ok_or_else(|| {
        Error::new(format!(
            "the application declared no `{CANVAS}` region, so there is no canvas rect to \
             compare against and phase C could not reach a verdict either way."
        ))
    })?;
    report.note(format!(
        "with the chrome drawn, `{CANVAS}` starts {:.1} pt down the client area",
        before_canvas.min.y
    ));

    let Some(control) = declared(&trace, ui_rect, SUBJECT) else {
        return Ok(Some(format!(
            "the View tab is active and its controls publish their rects, but none of them is \
             `{SUBJECT}`. `RIBBON_IA.md` §3 names Read mode as one of the two commands the old \
             shell offered by keyboard alone, and giving it a control was the fix — a registered \
             command with no reachable control is that defect reinstated. Controls declared: {}.",
            list(&declared_names(&trace, ui_rect, ITEM_PREFIX))
        )));
    };
    if !control.is_substantial() {
        return Ok(Some(format!(
            "`{SUBJECT}` was declared at {control:?}, which has no usable area — the control is \
             laid out and not on screen. Three panels in the old shell shipped with a body, a \
             rail entry and no control anyone could click, and passed every verification for \
             their whole shipped life."
        )));
    }

    // The BEFORE capture, taken while the ribbon is still there and with the
    // pointer parked off the control, so a hover is not mistaken for the
    // change this check is about.
    driver.move_to(session.frame()?.declared_center(before_canvas))?;
    session.settle(6);
    let before_png = ctx.out("read_mode_chrome.before.png");
    let before_image = crate::capture::window_to_png(&session, &before_png)?;
    report.artifact(before_png);
    let frame = session.frame()?;
    // The active tab's accent rule, a 2 pt strip along its top edge inset past
    // the rounded corners (`egui-shell`'s `tabshape`): the one place on the
    // band whose colour is the accent, so its distance from any backdrop is a
    // property of the palette. A build that hid the band but left the tab
    // strip fails here and passes at the control.
    let rule = crate::geom::LRect::new(
        crate::geom::Pt {
            x: tab.min.x + 8.0,
            y: tab.min.y,
        },
        crate::geom::Pt {
            x: (tab.max.x - 8.0).max(tab.min.x + 9.0),
            y: tab.min.y + 2.0,
        },
    );
    let Some(before_fill) = driving::fill_of(&before_image, &frame, rule) else {
        return Ok(Some(format!(
            "`{TAB}` was declared at {tab:?}, which resolves to no pixels of the capture — the \
             application declared a tab outside its own window."
        )));
    };

    // --- B. press it ------------------------------------------------------
    let invokes_before = shell_trace(&session)?
        .events(INVOKE_EVENT)
        .filter(|l| l.get("id") == Some(SUBJECT_ID))
        .count();
    driver.click_at(frame.declared_center(control))?;
    // The composition changes on the next frame, and an active `FitMode`
    // recomputes its zoom from the new viewport — which on a CAD sheet means a
    // re-raster. Long enough for both.
    session.settle(40);

    let invokes_after = shell_trace(&session)?
        .events(INVOKE_EVENT)
        .filter(|l| l.get("id") == Some(SUBJECT_ID))
        .count();
    if invokes_after <= invokes_before {
        let shell = shell_trace(&session)?;
        return Err(Error::new(format!(
            "the click on `{SUBJECT}` produced no new `{INVOKE_EVENT} id={SUBJECT_ID}` line, so \
             no click reached the ribbon and nothing after it would mean anything. Two readings, \
             and this check declines to choose between them: the pointer injection is not \
             reaching this window, or the shell diagnostic switch {}={} did not reach the \
             process — the shell trace carries {} line(s) under `{SHELL_TRACE_PREFIX}`. \
             Trace: {}.",
            SHELL_DIAG_ENV.0,
            SHELL_DIAG_ENV.1,
            shell.lines.len(),
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "the shell traced `{INVOKE_EVENT} id={SUBJECT_ID}`, so the click reached the control"
    ));

    let trace = session.trace()?;
    if !trace
        .events(READ_MODE_EVENT)
        .any(|l| l.get("on") == Some("true"))
    {
        let unimplemented = trace
            .events(UNIMPLEMENTED_EVENT)
            .any(|l| l.get("id") == Some(SUBJECT_ID));
        return Ok(Some(format!(
            "`{SUBJECT_ID}` was invoked and traced no `{READ_MODE_EVENT} on=true`. {}",
            if unimplemented {
                format!(
                    "The application traced `{UNIMPLEMENTED_EVENT} id={SUBJECT_ID}`, so the \
                     token arrived at `app::dispatch` and there is no arm for it — which is \
                     exactly the state this command shipped in until 2026-08-15, and the fix \
                     is one match arm calling `app::window::toggle_read_mode`."
                )
            } else {
                "The application traced no `command-unimplemented` for it either, so the token \
                 reached an arm that did not do what the arm is for — look at \
                 `app::window::toggle_read_mode`, which is the one place the memory slot is \
                 written."
                    .to_owned()
            }
        )));
    }
    report.note(format!(
        "the application traced `{READ_MODE_EVENT} on=true`, so the toggle flipped"
    ));

    // --- C. did the FRAME change? -----------------------------------------
    //
    // The part no unit test can see, in two channels. See the module header
    // for why neither alone is admissible.
    let trace = session.trace()?;
    let after_canvas = declared(&trace, ui_rect, CANVAS).ok_or_else(|| {
        Error::new(format!(
            "the application stopped declaring `{CANVAS}` after the press. That is not the \
             expected failure and it is not a verdict on read mode: the canvas is drawn in \
             every state this build has, so a missing declaration means the frame stopped \
             being composed at all."
        ))
    })?;
    let rise = before_canvas.min.y - after_canvas.min.y;
    if rise < MIN_RISE_PTS {
        return Ok(Some(format!(
            "`{READ_MODE_EVENT} on=true` was traced and the canvas did not move: `{CANVAS}` \
             started {:.1} pt down before and {:.1} pt down after, a rise of {rise:.1} pt \
             against the {MIN_RISE_PTS:.0} pt a ribbon band is worth. The toggle flipped and \
             the frame did not read it — look for the `window::draws_chrome` guard around \
             `self.ribbon_band(...)` in `PdfcerApp::ui`, which is the whole of the behaviour and \
             is invisible to every unit test in the workspace.",
            before_canvas.min.y, after_canvas.min.y
        )));
    }
    report.note(format!(
        "`{CANVAS}` rose {rise:.1} pt, so the ribbon band is no longer taking space"
    ));

    let after_png = ctx.out("read_mode_chrome.after.png");
    let after_image = crate::capture::window_to_png(&session, &after_png)?;
    report.artifact(after_png);
    let frame = session.frame()?;
    let Some(after_fill) = driving::fill_of(&after_image, &frame, rule) else {
        return Ok(Some(format!(
            "the region `{TAB}` occupied resolves to no pixels after the press. The window is \
             expected to keep its size — read mode hides the chrome, it does not resize the \
             window — so this says the client area changed, and the pixel half of phase C \
             cannot reach a verdict."
        )));
    };
    let delta = driving::delta(before_fill, after_fill);
    if delta < MIN_REPAINT_DELTA {
        return Ok(Some(format!(
            "the canvas moved but the pixels where the active tab was did not: {before_fill:?} \
             → {after_fill:?}, a difference of {delta} against the {MIN_REPAINT_DELTA} two \
             genuinely different fills are worth. The active tab's rule is `accent`, so this is \
             saying the tab strip is still on screen — a layout that reserves no space while \
             still painting the old band over the canvas is worse than one that does neither, \
             because the operator sees a ribbon they can no longer click."
        )));
    }
    report.note(format!(
        "the pixels where `{TAB}` was changed by {delta} ({before_fill:?} → {after_fill:?})"
    ));
    //
    //
    // Driving the return buys two things:
    //
    // 1. **Coverage.** The header said the return was covered "as a state
    //    machine, and by nothing at all as a frame". It is now a frame.
    // 2. **It stops this check poisoning what runs after it.** Leaving the
    //    application in read mode is the persisted-state hazard that made
    //    `delete_key` report the mode gate as a selection defect this morning;
    //    this check was also seen failing in-suite and passing alone, which is
    //    that signature exactly.
    //
    // Soft: if the chord does not land the check still PASSES on everything
    // asserted above. The exit is hygiene, not the property under test, and
    // downgrading a real result because the tidy-up failed would be the
    // harness reporting its own housekeeping as a defect in the program.
    driver.press_chord(&[crate::sys::vk::CONTROL], crate::sys::vk::H)?;
    session.settle(12);
    if session
        .trace()?
        .events(READ_MODE_EVENT)
        .any(|l| l.get("on") == Some("false"))
    {
        report.note(
            "Ctrl+H brought the chrome back, so the return trip is driven rather than \
             covered by unit test alone — and this check no longer leaves the application \
             in read mode for whatever runs next",
        );
    } else {
        report.note(
            "Ctrl+H produced no `read-mode on=false` line, so the check ends in read mode. \
             Not a failure of anything asserted above — the exit is hygiene. If this \
             persists, the chord gap in `sys::win32::key_stroke_with` has regressed",
        );
    }

    Ok(None)
}

/// **Phase 0** — press Full screen, prove the *window manager* agrees, and put
/// the window back.
fn fullscreen_round_trip(
    session: &Session,
    driver: &Driver,
    report: &mut CheckReport,
    ui_rect: &str,
) -> Result<Option<String>> {
    let trace = session.trace()?;
    let control = declared(&trace, ui_rect, FULLSCREEN).ok_or_else(|| {
        Error::new(format!(
            "the View tab is active and no `{FULLSCREEN}` region was declared, so phase 0 has \
             nothing to click. Controls declared: {}.",
            list(&declared_names(&trace, ui_rect, ITEM_PREFIX))
        ))
    })?;
    let before = session.frame()?.client_pixels();

    if !press_until_invoked(session, driver, ui_rect, control)? {
        return Err(Error::new(format!(
            "{PRESS_TRIES} clicks on `{FULLSCREEN}` produced no new `{INVOKE_EVENT} \
             id={FULLSCREEN_ID}` line, so no click reached the ribbon. Nothing has been done to \
             the display and nothing was learned; see `{SHELL_TRACE_PREFIX}` in {}.",
            session.trace_path().display()
        )));
    }
    let trace = session.trace()?;
    if !trace
        .events(FULLSCREEN_EVENT)
        .any(|l| l.get("asked") == Some("true"))
    {
        let unimplemented = trace
            .events(UNIMPLEMENTED_EVENT)
            .any(|l| l.get("id") == Some(FULLSCREEN_ID));
        return Ok(Some(format!(
            "`{FULLSCREEN_ID}` was invoked and traced no `{FULLSCREEN_EVENT} asked=true`. {}",
            if unimplemented {
                format!(
                    "The application traced `{UNIMPLEMENTED_EVENT} id={FULLSCREEN_ID}`, so the \
                     token arrived at `app::dispatch` and there is no arm for it — which is the \
                     state this command shipped in until 2026-08-15."
                )
            } else {
                "The application traced no `command-unimplemented` for it either, so the token \
                 reached an arm that did not send the viewport command."
                    .to_owned()
            }
        )));
    }

    let filled = session.frame()?.client_pixels();
    // Put it back FIRST, so that every return below leaves the operator's
    // display as it found it.
    //
    let restored_press = press_until_invoked(session, driver, ui_rect, control)?;
    // **Three seconds, and the asymmetry with the 1 s above is measured
    // rather than cautious.** Entering full screen was complete inside 1 s on
    // this machine; *leaving* it was not, and the first run of this phase failed
    // with `asked=false` traced, the click confirmed in the shell trace, and the
    // client area still 3440 × 1440. Nothing was wrong with the application: the
    // window manager had simply not finished restoring the window when the
    // measurement was taken.
    //
    // That failure is worth recording rather than just fixing, because it is the
    // shape `crate::coords` warns about — a harness measuring too early and
    // producing a confident wrong diagnosis of the program under test. The
    // sentence it printed named `app::window::next_fullscreen` as the culprit,
    // and `next_fullscreen` was correct.
    session.settle(120);
    let restored = session.frame()?.client_pixels();

    //
    // The predicate was `filled.w <= before.w || filled.h <= before.h`: BOTH
    // dimensions had to grow. That holds on a wide desktop where the window is
    // a fraction of the screen, which is where it was written — the run that
    // wrote it measured 2560x1000 becoming 3440x1440.
    //
    // It is wrong whenever the window is already as wide as the monitor. On a
    // 1920x1080 display the client area went **1920x1000 -> 1920x1080**: full
    // screen worked, and all it could add was the strip the title bar and
    // taskbar had been taking. Width was unchanged, so the check reported *"the
    // windowing system did not act on it"* about a windowing system that had.
    //
    // Area is the honest question — *did the window get bigger?* — and the
    // `no axis shrank` clause keeps it from being satisfied by a window that
    // grew tall while getting narrower, which is not a full-screen transition
    // and would be worth failing on.
    //
    // The general form is the one this suite keeps meeting: **a predicate
    // written from one machine's geometry encodes that machine.** It is the
    // same class as the `ui_scale` check asserting a point size that only holds
    // at one zoom factor, and as `delete_key` assuming a mode.
    let grew = u64::from(filled.w) * u64::from(filled.h)
        > u64::from(before.w) * u64::from(before.h)
        && filled.w >= before.w
        && filled.h >= before.h;
    if !grew {
        return Ok(Some(format!(
            "`{FULLSCREEN_EVENT} asked=true` was traced and the window did not grow: the client \
             area was {} x {} px and became {} x {} px. The arm sent the viewport command and \
             the windowing system did not act on it — which is the one failure the trace alone \
             could never report, because the application can only ask. (The test is AREA plus \
             no axis shrinking, not both axes growing: a window already as wide as its monitor \
             legitimately gains only height.)",
            before.w, before.h, filled.w, filled.h
        )));
    }
    report.note(format!(
        "full screen grew the client area from {} x {} px to {} x {} px",
        before.w, before.h, filled.w, filled.h
    ));

    // THE PRESS BEFORE THE VERDICT. Nothing below may be read as a
    // statement about the application until the application has been shown to
    // have heard the press it is being judged on — `checks/mod.rs` rule 3, and
    // the reason this whole phase re-reads the shell trace rather than trusting
    // `click_at`'s `Ok`.
    //
    if !restored_press {
        return Err(Error::new(format!(
            "the window is full screen and {PRESS_TRIES} clicks on `{FULLSCREEN}` produced no \
             new `{INVOKE_EVENT} id={FULLSCREEN_ID}` line, so the restoring press never reached \
             the ribbon and NOTHING was learned about whether full screen toggles back.\n  \
             ⚠ **THE DISPLAY HAS BEEN LEFT FILLED.** The operator's routes back are `F11` — \
             `view.fullscreen`'s chord, which is bound and unaffected by this — and the same \
             ribbon control, which full screen keeps on screen. Closing the window also works.\n  \
             This is reported as SKIPPED rather than failed because a press that was not \
             delivered says nothing about `app::window::next_fullscreen`: on 2026-08-29 this \
             exact state was reported as a defect in that function, which was correct. See \
             `{SHELL_TRACE_PREFIX}` in {}.",
            session.trace_path().display()
        )));
    }

    // The mirror of the growth test above and it needs the same correction for
    // the same reason: on a monitor the window already spans, restoring gives
    // back only the height, so `restored.w` stays equal to `filled.w` and an
    // `&&` of two `>=` would call a correct restore a failure. Area again.
    if u64::from(restored.w) * u64::from(restored.h) >= u64::from(filled.w) * u64::from(filled.h) {
        // The application's own account of the press, quoted rather than
        // paraphrased. `fullscreen-toggle` carries BOTH the viewport's report
        // and this shell's outstanding request — the two whose disagreement was
        // the original defect — so a reader of a red run can tell "it asked for
        // the wrong thing" (`asked=true` twice) from "it asked correctly and the
        // window manager declined" (`asked=false` and the window still filled),
        // which are different defects in different code.
        let toggles = session
            .trace()?
            .events(TOGGLE_EVENT)
            .map(|l| l.raw.clone())
            .collect::<Vec<_>>();
        return Ok(Some(format!(
            "the second press of `{FULLSCREEN_ID}` reached the ribbon and did not restore the \
             window: it is still {} x {} px. Full screen is a toggle — \
             `app::window::next_fullscreen` reads the viewport's own state, and remembers its \
             own outstanding request for a few frames so that a second press cannot read a \
             report that has not caught up — so a press that ARRIVES and does not restore means \
             one of those two is wrong. What the application said: {}. **The display has been \
             left filled**; press F11, or close the window, to recover it.",
            restored.w,
            restored.h,
            list(&toggles)
        )));
    }
    report.note(format!(
        "a second press restored it to {} x {} px",
        restored.w, restored.h
    ));
    Ok(None)
}

/// **Press the Full screen control until the application says it heard**, and
/// answer whether it ever did.
fn press_until_invoked(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    fallback: LRect,
) -> Result<bool> {
    let count = |session: &Session| -> Result<usize> {
        Ok(shell_trace(session)?
            .events(INVOKE_EVENT)
            .filter(|l| l.get("id") == Some(FULLSCREEN_ID))
            .count())
    };
    let before = count(session)?;
    for _ in 0..PRESS_TRIES {
        let here = declared(&session.trace()?, ui_rect, FULLSCREEN).unwrap_or(fallback);
        driver.click_at(session.frame()?.declared_center(here))?;
        // A window-manager transition plus a re-fit and a re-raster at the new
        // size. Longer than a ribbon click needs, because what is measured
        // afterwards is the *window*, and reading it mid-transition would be
        // reading neither state.
        session.settle(40);
        if count(session)? > before {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Click the View tab and confirm the shell reported it, returning **its
/// rect** — which is also the pixel probe phase C uses, for the reason recorded
/// at the before-capture.
fn click_tab(session: &Session, driver: &Driver, ui_rect: &str) -> Result<LRect> {
    let trace = session.trace()?;
    let rect = declared(&trace, ui_rect, TAB).ok_or_else(|| {
        Error::new(format!(
            "the application declared no `{TAB}` region. Either this build does not show that \
             tab in the mode it opened in, or the tab strip is too narrow and it has moved into \
             the overflow menu — which this check cannot open, because a menu's contents are \
             not published as regions. Tabs declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(rect))?;
    session.settle(12);
    if !shell_trace(session)?
        .events(TAB_EVENT)
        .any(|l| l.get("tab") == Some(TAB_ID))
    {
        return Err(Error::new(format!(
            "the click on `{TAB}` produced no `{TAB_EVENT} tab={TAB_ID}` line, so no click \
             reached the ribbon. Every phase below aims at a control on that tab, so this is a \
             SKIP rather than a verdict on read mode."
        )));
    }
    Ok(rect)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The region names are derived from the ids they describe.
    #[test]
    fn the_region_names_match_the_ids_they_describe() {
        assert_eq!(SUBJECT, format!("{ITEM_PREFIX}{SUBJECT_ID}"));
        assert_eq!(TAB, format!("ribbon.tab.{TAB_ID}"));
    }

    /// The rise threshold is a floor under one ribbon row and over any
    /// rounding.
    #[test]
    fn the_rise_threshold_is_a_floor_rather_than_a_band_height() {
        let threshold = std::hint::black_box(MIN_RISE_PTS);
        assert!(threshold > 1.0, "above any rounding or splitter");
        assert!(threshold < 60.0, "under one row of controls plus caption");
    }
}
