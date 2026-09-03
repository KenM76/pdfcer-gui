//! `read_mode_hides_the_chrome` — the regression test for **`view.read_mode`,
//! a command that had a control, a glyph, a group, a chord and a line in the
//! shortcuts reference, and no dispatch arm at all.**
//!
//! # The defect class this exists for
//!
//! `RIBBON_IA.md` §3, on the shell being replaced:
//!
//! > Read mode and full screen have **no ribbon control at all** — they are
//! > keyboard-only (Ctrl+H, F11) on a tab literally named View. This is the
//! > single most confusing thing in the current ribbon.
//!
//! This shell gave both a control, and for a day that was all it gave them:
//! `shell::commands::reach` found `view.read_mode` among the eleven registered
//! commands whose honest status was *"the control should not be drawn yet"*,
//! with the note that a chord that does nothing **cannot even be greyed**. The
//! arm landed on 2026-08-15 (`app::window`), and this is the check that says so
//! from outside the process.
//!
//! # ★ Why a unit test cannot cover it, which is the bar for being here
//!
//! [`crate::checks`]' rule: *"it must fail against a build where the wiring is
//! absent, and the wiring must be something no unit test in the workspace can
//! observe."*
//!
//! `app::window` is fully unit-tested — the memory slot flips, `draws_chrome`
//! is its negation, both directions, three times over. **Every one of those
//! tests passes against a build where `PdfcerApp::ui` never calls
//! `draws_chrome`.** The whole behaviour is one `if` in the frame composition,
//! and a composition step's effect is observable only in a window: it is the
//! identical shape to the defect `measure_linear` exists for, where four
//! passing unit tests sat behind a `conditions` call site nobody had written
//! and the button never lit up.
//!
//! # The phases, and why phase B alone would not do
//!
//! | Phase | Move | Evidence | If it does not hold |
//! |---|---|---|---|
//! | 0 | click `view.fullscreen`, twice | the **client area** grows to the display and comes back | FAIL — see the phase-0 section below |
//! | A | click View, read `central-panel` | the canvas's rect **before** | SKIP — nothing was measured to compare against |
//! | B | click `view.read_mode` | `ribbon-command-invoked id=view.read_mode`, then `read-mode on=true` | SKIP if no invoke (no click landed); **FAIL** if invoked and no `read-mode` line — the arm is missing |
//! | C | read `central-panel` again, and capture | the rect's **top moved up**, and the pixels where the ribbon was **changed** | FAIL — the command ran and the frame did not change |
//!
//! Phase B alone is what a trace-only check would assert, and it is not enough:
//! `read-mode on=true` proves the *toggle* flipped, which is precisely what the
//! unit tests already prove. Phase C is the part that is new — that the frame
//! composition read the flag — and it is asserted **twice, in two channels**,
//! because each covers the other's blind spot:
//!
//! * **the rect** is exact and cheap and would be satisfied by a build that
//!   moved the canvas without repainting anything;
//! * **the pixels** cannot be faked by an arithmetic error, and would be
//!   satisfied by any global repaint — a hover, a theme change, a resize —
//!   which is why the rect is checked as well.
//!
//! # ★ It goes one way, and cannot come back
//!
//! **The exit from read mode is `Ctrl+H`, and this machine cannot inject
//! keystrokes reliably** — `find_bar`'s first run reported Find broken on a
//! build where Find worked, which is why every other driving check in this
//! suite uses the mouse only. Once the chrome is hidden there is no ribbon
//! control left to click, by construction: that is the feature.
//!
//! So this check ends in read mode and lets the session be killed, which costs
//! nothing (the process is torn down after every check and read mode is
//! per-session by design — see `app::window` §3, which is also why the *next*
//! check's launch is unaffected). What it means for coverage is stated plainly
//! rather than papered over: **the return trip is not driven here.** It is
//! covered by `app::window::tests::read_mode_starts_off_and_toggles_both_ways`
//! as a state machine, and by nothing at all as a frame. If a way to inject
//! `Ctrl+H` arrives, phase D is one more `settle` and one more rect read.
//!
//! # ★ Phase 0 drives `view.fullscreen`, and does it FIRST
//!
//! The two arms landed in the same change, in the same module, in the same
//! ribbon group, and they share the one expensive precondition this check has
//! (a window wide enough for View's seventh group — see the launch site). A
//! second check module would have to restate that placement argument, which is
//! the "two hand-written tables" smell one level up.
//!
//! It runs **before** read mode because it is the only half that can be
//! reversed: full screen keeps the ribbon, so the same control is still on
//! screen and a second click restores the window. Read mode removes the ribbon
//! and is therefore terminal. Doing them the other way round would leave the
//! display filled with a window that has no visible way back.
//!
//! Its oracle is the **client area**, read from the window manager rather than
//! from the application: `WindowFrame::client_size` before and after. That is
//! the strongest available statement, because it is the windowing system
//! agreeing — the application can only *ask* for full screen
//! (`ViewportCommand::Fullscreen`), which is exactly why the dispatch arm
//! traces `fullscreen asked=` rather than `on=`. A check that believed the
//! trace would be reporting the request, not the result.
//!
//! **It takes the display for about four seconds**, and then gives it back.
//! That is a real cost to whoever is at the machine and it is bounded
//! deliberately: the restoring click is made before anything else happens, and
//! a failure to restore is reported in words that say the display has been left
//! filled rather than being silent about it.
//!
//! Measured on this machine: 2560 × 1000 → **3440 × 1440** → 2560 × 1000.

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
///
/// `asked`, not `on`: a viewport command is queued and answered by the
/// windowing backend, so the application cannot honestly claim the window *is*
/// full screen on the frame it requested it. Phase 0 therefore reads this line
/// only as evidence that the **arm ran**, and asks the window manager for the
/// result.
const FULLSCREEN_EVENT: &str = "fullscreen";

/// `fullscreen-toggle reported=… pending=… asked=…` — the application's own
/// account of **why** it asked for what it asked for.
///
/// Quoted verbatim into the restore failure, because it is the one line that
/// separates the two defects that produce an unrestored window: a shell that
/// asked for full screen twice (`asked=true` on both) and a window manager that
/// declined the restore (`asked=false` and the window still filled).
const TOGGLE_EVENT: &str = "fullscreen-toggle";

/// How many times a full-screen press is retried before the harness gives up
/// on it.
///
/// # ★★★ Why a retry and not a longer settle
///
/// Because the thing that goes wrong is **delivery**, not timing. This suite's
/// record for this control is three runs and three different outcomes: one
/// where the first click never reached the ribbon (2026-08-27, SKIPPED with
/// exactly this diagnosis), one where the foreground was held by a browser
/// (2026-08-28), and one where the *second* click never reached it
/// (2026-08-29) — and that last one was reported as a defect in the
/// application, because only the first press was ever verified.
///
/// A click that is not delivered is not delivered no matter how long the
/// harness then waits, so the answer is to press again and read the
/// application's own invocation log, not to sleep longer. Three, because the
/// cost of a wasted press here is a fraction of a second and the cost of
/// giving up too early is the operator's whole display staying filled.
///
/// ★ It is safe to retry precisely BECAUSE the count is read between attempts:
/// each landed press toggles once, so pressing again after a press that landed
/// would undo it. [`press_until_invoked`] returns the moment the count moves.
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
///
/// `app::mod`'s `REGION_CENTRAL_PANEL`. It is the outermost region the
/// application owns, and its **top edge** is the measurement this check turns
/// on: with the ribbon drawn, the central panel starts below it; with the
/// ribbon gone, it starts at the top of the client area.
const CANVAS: &str = "central-panel";

/// How far the canvas's top edge must rise, in logical points, for the ribbon
/// to be counted as gone.
///
/// A floor rather than an equality. The exact height of a two-row ribbon band
/// is a layout fact this harness must not re-derive — `check-file-size.sh`'s
/// sibling argument, and the reason `profile::PDFCER_GUI` ships no region
/// fractions at all: a number written here is correct until the first time the
/// band's padding changes, and then it is a check that fails for a reason that
/// is not a defect.
///
/// 40 pt is comfortably under one row of controls (the theme's `control_height`
/// is 24 pt before the band's own padding and its caption row) and comfortably
/// over any rounding, a scrollbar, or a one-pixel splitter. What it is really
/// asserting is *"a whole bar's worth"*, not a specific bar.
const MIN_RISE_PTS: f32 = 40.0;

/// How different the pixels where the ribbon was must be, as a maximum
/// absolute per-channel difference in 0–255.
///
/// [`driving::MIN_PRESSED_DELTA`]'s derivation applies unchanged and is not
/// restated: two identically filled regions in a lossless BGRA capture differ
/// by **0**, not by a small number, so anything above the noise floor is a real
/// change. Ribbon band against canvas backdrop is a far larger difference than
/// the pressed/unpressed pair that constant was measured on, so borrowing it is
/// conservative in the direction that matters.
const MIN_REPAINT_DELTA: u16 = driving::MIN_PRESSED_DELTA;

/// Where and how large the window is placed, as `PDFCER_DIAG_VIEWPORT` takes it
/// (`x,y,w,h` in logical points).
///
/// Wide enough for **all seven** of the View tab's groups, because the control
/// this check clicks is in the last one — see the note at the launch site for
/// why that is a finding and not a workaround. `0,0` rather than a negative
/// off-desktop origin: the harness has to photograph this window, and a window
/// placed off the visible desktop is captured as whatever the compositor last
/// had for it.
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
            Err(skip) => report.skip(skip.to_string()),
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
    // ★ **The only check in the suite that places its own window, and the
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
    // ★ **The probe is the ACTIVE TAB, not the control**, and the first draft
    // of this check got that wrong in a way worth recording: aimed at
    // `ribbon.item.view.read_mode`, the measured difference was `#E8E8EA` →
    // `#F2F2F3`, a delta of **10** against a threshold of 12 — on a run where
    // the canvas had demonstrably risen 109 pt and the ribbon was demonstrably
    // gone. An unpressed control's fill and the canvas backdrop behind it are
    // both near-white greys in this theme, so the honest reading is that the
    // probe was measuring two shades of the same thing, not that the feature
    // was broken.
    //
    // The active tab is the one region on the band whose colour is chosen to be
    // unmistakable: it paints `accent` on `on_accent` (`FEATURES.md`'s theme
    // row, after the stock-style defect D10 was fixed), so its distance from
    // any backdrop is a property of the palette rather than an accident of the
    // widget. It is also the *strictest* place to look — a build that hid the
    // band but left the tab strip would fail here and pass at the control.
    let Some(before_fill) = driving::fill_of(&before_image, &frame, tab) else {
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
    let Some(after_fill) = driving::fill_of(&after_image, &frame, tab) else {
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
             genuinely different fills are worth. An active tab paints `accent`, so this is \
             saying the tab strip is still on screen — a layout that reserves no space while \
             still painting the old band over the canvas is worse than one that does neither, \
             because the operator sees a ribbon they can no longer click."
        )));
    }
    report.note(format!(
        "the pixels where `{TAB}` was changed by {delta} ({before_fill:?} → {after_fill:?})"
    ));
    // ★ PHASE D — come back out, added 2026-08-17.
    //
    // This check used to end in read mode and say so, on the grounds that
    // `Ctrl+H` was the only exit and "this machine cannot inject keystrokes".
    // **That was wrong for the whole life of the project.** Chords failed
    // because `sys::win32::key_stroke_with` posted the modifier and the key in
    // the same instant, giving the application no frame in which the modifier
    // was held and the key was not — so `Ctrl+H` arrived as a bare `h`. Three
    // 12 ms pauses fixed it, and `find_opens_and_finds` passes for the first
    // time.
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
///
/// Returns `Ok(None)` for a clean round trip, `Ok(Some(_))` for a failure
/// sentence, and `Err` only for the states that are the harness's business —
/// no control declared, no click delivered, no window frame readable.
///
/// # Why the client area and not the trace
///
/// `app::window::toggle_fullscreen` sends `ViewportCommand::Fullscreen`, which
/// is a **request**. The application cannot know whether it was granted, which
/// is why its trace line says `asked=` — so a check that read the trace alone
/// would report the request and call it the result. `WindowFrame::client_size`
/// comes from `GetClientRect` on the real window: it is the windowing system's
/// answer, and it is the only one worth having here.
///
/// # Why it restores before returning, even on the failing paths that can
///
/// A check that filled the operator's display and then reported a failure would
/// leave them to find the window and fix it. Every path below that has already
/// pressed the control presses it again; the one that cannot — a second click
/// that does not land — says so in its own sentence rather than being silent
/// about a display it has taken.
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
    // ★★★ AND PROVE THE PRESS LANDED, WHICH IT DID NOT ON 2026-08-29. See
    // [`press_until_invoked`]: this line used to be a bare `click_at`, and the
    // failure sentence below was reached with **one** `{INVOKE_EVENT}
    // id={FULLSCREEN_ID}` line in the whole trace. The restoring press had never
    // arrived, and the check reported `app::window::next_fullscreen` — a
    // function that had just been fixed by driving, and was correct — as
    // reading a stale state. A confident, specific, wrong defect report about
    // working code, produced by measuring the result of an input nobody had
    // shown was delivered.
    let restored_press = press_until_invoked(session, driver, ui_rect, control)?;
    // ★ **Three seconds, and the asymmetry with the 1 s above is measured
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

    // ★ AREA, not both axes — corrected 2026-08-17 after this fired on a
    // window that had gone full screen perfectly well.
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

    // ★★★ THE PRESS BEFORE THE VERDICT. Nothing below may be read as a
    // statement about the application until the application has been shown to
    // have heard the press it is being judged on — `checks/mod.rs` rule 3, and
    // the reason this whole phase re-reads the shell trace rather than trusting
    // `click_at`'s `Ok`.
    //
    // `Err` rather than `Ok(Some(_))`, deliberately: a press that was never
    // delivered is the HARNESS's failure, and a suite that recorded it as a
    // program defect would be doing exactly what the run of 2026-08-29 did.
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
        // ★ The application's own account of the press, quoted rather than
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
///
/// Returns `true` as soon as a new `ribbon-command-invoked id=view.fullscreen`
/// line appears in the shell trace, and `false` after [`PRESS_TRIES`] attempts
/// with no new line. It never presses again after a press that landed, so the
/// toggle is moved exactly once whatever happens.
///
/// # ★★★ Why phase 0 cannot use a bare `click_at`, and the day that cost
///
/// `Driver::click_at` answers `Ok(())` when the **pointer input was sent**. It
/// raises the owning window, refuses if the foreground could not be taken, and
/// confirms the point is not covered — all real guards, and none of them is the
/// statement *"the application processed a click on that control"*. Between the
/// two lies a window-manager transition, an egui frame boundary, and a ribbon
/// that may have re-laid itself out.
///
/// On 2026-08-29 the entering press landed, the restoring press did not, and
/// the check — which verified only the first — measured a client area still
/// 3440 × 1440 and reported:
///
/// > *"a second press that does not restore means the state it reads is not the
/// > state the OS is in"*
///
/// naming `app::window::next_fullscreen`. The trace carried **one**
/// `ribbon-command-invoked id=view.fullscreen` line and **one**
/// `fullscreen-toggle reported=Some(false) pending=None asked=true` for the
/// whole run: there was no second press to read a stale state with.
/// `next_fullscreen` had itself been written to fix a real instance of exactly
/// that defect a fortnight earlier, so the report was a plausible accusation
/// against the code that already closed it.
///
/// ⇒ The rule this encodes, which is `checks/mod.rs` rule 3 applied to input
/// rather than to selection:
///
/// > **Nothing measured after a press is evidence about the program until the
/// > press is shown to have arrived.** The application's own invocation log
/// > says so; `Ok(())` from the input layer does not.
///
/// ★ Re-reading the control's rectangle each attempt is not defensive padding.
/// The window changes size between the two presses, and a ribbon is laid out
/// from the window's width — `ui-rect` is a change log, so an unmoved control
/// simply publishes nothing and [`declared`] answers with the rect that still
/// stands. Reading it fresh costs one trace parse and follows the control if it
/// ever does move.
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
///
/// `markup_shapes::click_tab`'s shape, with its own tab and its own reason.
/// Not folded into [`driving`] in this change: that module's header records
/// that it is *"a widening, not a refactor"*, and a third copy is the point at
/// which folding becomes worth doing on its own rather than in the change that
/// happens to need it.
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
    ///
    /// Spelled as literals in this file and as `format!`s in the shell, so this
    /// is the seam where a rename in `egui_shell::ribbon::report` would
    /// otherwise turn every assertion above into a silent SKIP — *"the
    /// application declared no region"* — which reads as a missing control
    /// rather than as a renamed one.
    #[test]
    fn the_region_names_match_the_ids_they_describe() {
        assert_eq!(SUBJECT, format!("{ITEM_PREFIX}{SUBJECT_ID}"));
        assert_eq!(TAB, format!("ribbon.tab.{TAB_ID}"));
    }

    /// The rise threshold is a floor under one ribbon row and over any
    /// rounding.
    ///
    /// Pinned so that a future edit which "tightens" it to an exact band height
    /// has to argue with this comment first: an equality here would fail the
    /// day the band's padding changes, for a reason that is not a defect.
    ///
    /// Written against a runtime copy rather than the constant itself, because
    /// `clippy::assertions_on_constants` refuses an assertion the compiler can
    /// fold — correctly, in general, and this is the case where the assertion's
    /// value is the **sentence attached to it** rather than the arithmetic.
    #[test]
    fn the_rise_threshold_is_a_floor_rather_than_a_band_height() {
        let threshold = std::hint::black_box(MIN_RISE_PTS);
        assert!(threshold > 1.0, "above any rounding or splitter");
        assert!(threshold < 60.0, "under one row of controls plus caption");
    }
}
