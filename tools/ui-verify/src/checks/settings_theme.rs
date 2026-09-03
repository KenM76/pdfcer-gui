//! `settings_theme_takes_effect` — the second half of `DEFECTS.md` **D10**,
//! proved in pixels.
//!
//! # The defect
//!
//! D10 was *"the theme system is built, tested, gated, and never installed"*.
//! Three presets, a palette, a role per colour, a rendered-pair contrast gate
//! over all five widget states, and its own self-test — compiled into every
//! shipped binary and never handed to the `Context`. Every colour an operator
//! had ever seen in this shell was `egui`'s stock light style.
//!
//! It had two halves. The first — *`Theme::apply` is never called* — was fixed
//! on 2026-08-14. D10's own text named the second and said what blocked it:
//!
//! > There is also **no way to choose a preset**: the settings dialog is one of
//! > the unsalvaged Class-B surfaces, so even once `apply` is wired, the preset
//! > is whatever the code picks until that dialog lands.
//!
//! The application's install site agreed from the other end, calling its
//! hard-coded preset *"a placeholder in the honest sense: the mechanism is real
//! and reachable, and only the chooser is missing."*
//!
//! The chooser landed on 2026-08-17. This is the check that says so.
//!
//! # ★ What this measures, and why nothing cheaper would do
//!
//! **A rendered pixel, before and after, from the running program.**
//!
//! Every cheaper oracle was already available and every one of them was green
//! throughout D10's shipped life, which is the whole lesson of that defect:
//!
//! | oracle | why it passed anyway |
//! |---|---|
//! | the theme gate | asserts every colour is a *named role in the theme module*. True of the source whether or not the theme is installed. |
//! | the contrast gate | renders pairs **from the theme** and measures those. Never asks the application what it drew. |
//! | a unit test on the picker | would assert the token changes. A token is not a colour. |
//! | `FEATURES.md` | ticked the row. Three themes an operator could not reach were ticked for their whole shipped life. |
//!
//! D10's own summary of that is the sentence this check exists to make false
//! next time: *"No test saw it."*
//!
//! So the assertion is a **difference between two captures of one window**. The
//! window's own body region is sampled with the light preset in force, the Dark
//! radio is clicked, and it is sampled again. Two claims follow:
//!
//! 1. **The colour moved.** A picker that writes a token nothing reads leaves
//!    this identical.
//! 2. **It moved the right way** — the second sample is substantially *darker*.
//!    A picker wired to the wrong preset, or a repaint that happened to change
//!    an unrelated pixel, moves it some other way.
//!
//! Claim 2 is what stops claim 1 being satisfied by noise. Neither is stated as
//! an exact colour: the composited value depends on the window's opacity, the
//! platform's rounding and whatever the operator's own settings file says, and a
//! check that pinned `#2B2B2B` would fail on a machine where the theme worked.
//!
//! # Why the sample is the window body and not a swatch
//!
//! Because the failure D10 records is that the framework's chrome and `egui`'s
//! widgets painted from **two different palettes** — `apply` both writes the
//! styles and stashes the `Theme` where `Theme::of` retrieves it, and only the
//! second was missing. A swatch drawn by this dialog would prove the dialog
//! knows the preset. The window's background is drawn by `egui` from the style
//! `apply` wrote, so it proves the *installation* happened.
//!
//! # What this check does NOT do
//!
//! **It does not press Save.** The theme is deliberately the one setting that
//! takes effect on the draft, before Save, because a theme cannot be judged
//! from a radio label — and that is exactly the behaviour under test. Pressing
//! Save would additionally write the operator's real `settings.txt`, which a
//! harness has no business doing: this check runs on a developer's machine
//! against their own configuration, and a verification tool that silently
//! changes what it verifies is not a verification tool.
//!
//! It closes the window with the ✕, which the application treats as a Cancel,
//! so the session ends on whatever the operator had.

use crate::checks::driving::{
    ITEM_PREFIX, SHELL_DIAG_ENV, TAB_EVENT, declared, declared_names, fill_of, frame_of, list,
    shell_trace,
};
use crate::checks::{Check, CheckContext, CheckReport};
use crate::error::{Error, Result};
use crate::image::Rgb;
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};

/// The command that opens the window.
const SUBJECT: &str = "ribbon.item.file.settings";
const SUBJECT_ID: &str = "file.settings";

/// The tab it lives on. File, which is in every mode's tab list.
const TAB_ID: &str = "file";
const TAB: &str = "ribbon.tab.file";

/// The window's own body. `dialogs::settings::REGION_BODY`, spelled as a
/// literal because that is the contract between the two crates.
const DIALOG: &str = "dialog:settings";

/// The Dark preset's radio. `dialogs::settings::REGION_THEME_PREFIX` plus
/// `egui_shell::theme::Preset::Dark`'s key.
const DARK_RADIO: &str = "settings.theme.dark";

/// The Appearance group's collapsible heading.
///
/// ★ **It has to be clicked, because that group is CLOSED when the window
/// opens** — and that is a deliberate design decision rather than an oversight,
/// so the check accommodates it rather than the application being changed to
/// suit the check.
///
/// The window collapses thirteen settings into seven subject groups because an
/// operator arrives with a *symptom* and the headings are how a symptom finds
/// its setting. Exactly one starts expanded, and it is **Colour** — it holds
/// the setting most likely to have brought someone here and the only default
/// that knowingly differs from other PDF viewers.
///
/// This check found the consequence on its second run: the window opened, the
/// body published its rect, and no theme radio existed, because a collapsed
/// `CollapsingHeader` does not run its body closure at all. The failure message
/// said *"the window has no theme picker in it"*, which was a true statement
/// about the frame and a false one about the build.
const APPEARANCE_HEADING: &str = "settings.heading.appearance";

/// How much darker the window must get before the change counts as a theme
/// rather than as a hover highlight or a repaint artefact.
///
/// The `quiet` preset's panel is a light grey around `#E8E8EA`; the `dark`
/// preset's is in the forties. That is a gap of roughly 190 per channel, so a
/// floor of **60** is comfortably inside it while being far outside anything a
/// selection highlight, a focus ring or an antialiasing difference produces.
///
/// Stated as a *minimum drop* rather than as a target colour deliberately — see
/// the module header on why pinning an exact value would fail on a machine
/// where the feature works.
const MIN_DARKENING: i32 = 60;

pub struct SettingsThemeTakesEffect;

impl Check for SettingsThemeTakesEffect {
    fn name(&self) -> &'static str {
        "settings_theme_takes_effect"
    }

    fn defect(&self) -> &'static str {
        "three themes ship, and nothing an operator can press chooses one (DEFECTS.md D10)"
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

#[allow(
    clippy::too_many_lines,
    reason = "one linear scripted sequence; splitting it would hide the order the steps must happen in"
)] // ui-text-exempt: lint justification, never displayed
fn assess(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input), and this check is three clicks. Reported as SKIPPED \
             rather than passed — a check that did not run has learned nothing.",
        ));
    }

    let mut spec = LaunchSpec::new(&exe, ctx.out("settings_theme.trace.txt"));
    // ★ NO `--pdf`, and that is the property under test as much as a
    // convenience. `file.settings` is application-scoped: these are choices
    // about pdfcer, meaningful with nothing loaded, and an operator who has just
    // launched the program and wants a dark window must not have to open a
    // document first. If the command ever gains a `doc.open` gate, this check
    // fails at the click rather than passing quietly with a fixture propping it
    // up.
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
        "launched {} as pid {} with NO document",
        exe.display(),
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());

    // ★ Maximise BEFORE looking for anything, and this is not cosmetic.
    //
    // `file.settings` is in the File tab's LAST group, and at the window size
    // the application opens at, that group is in the ribbon's **overflow menu**
    // — where a control publishes no rect. This check's first run reported
    // *"none of them is `ribbon.item.file.settings`"* and listed ten controls
    // ending at `file.print`, which was a true statement about the regions and
    // a false one about the feature.
    //
    // A maximised window is also the state an operator running a drawing tool
    // is overwhelmingly in, so this is the layout most worth verifying. What it
    // does NOT verify is that these controls are reachable in a small window;
    // that is a separate question and needs a check of its own.
    session.maximize();
    session.settle(20);

    let trace = session.trace()?;
    if !trace.started(ctx.profile.vocab.start_event) {
        return Err(Error::new(format!(
            "the trace has no `{}` line, so the diagnostic switch did not reach the process. \
             Captured stderr is at {}.",
            ctx.profile.vocab.start_event,
            session.trace_path().display()
        )));
    }

    let driver = Driver::new(session.window());
    let ui_rect = ctx.profile.vocab.ui_rect_event.unwrap_or("ui-rect");

    // ★ Nothing may have opened this window unasked. `view.app_initiative`'s
    // specified default is Never, and a settings window that floats over the
    // canvas on its own breaks it in the most annoying way available.
    if declared(&trace, ui_rect, DIALOG).is_some() {
        return Ok(Some(format!(
            "`{DIALOG}` was declared before anything was clicked — the Settings window opened \
             on its own, which pdfcer may not do."
        )));
    }

    // --- A. the File tab ---------------------------------------------------
    let tab = declared(&trace, ui_rect, TAB).ok_or_else(|| {
        Error::new(format!(
            "the application declared no `{TAB}` region. Tabs declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(tab))?;
    session.settle(12);
    if !shell_trace(&session)?
        .events(TAB_EVENT)
        .any(|l| l.get("tab") == Some(TAB_ID))
    {
        return Err(Error::new(format!(
            "the click on `{TAB}` produced no `{TAB_EVENT} tab={TAB_ID}` line, so no click \
             reached the ribbon."
        )));
    }

    // --- B. the Settings control -------------------------------------------
    let trace = session.trace()?;
    let Some(control) = declared(&trace, ui_rect, SUBJECT) else {
        return Ok(Some(format!(
            "the File tab is active and its controls publish their rects, but none of them is \
             `{SUBJECT}`. Controls declared: {}.",
            list(&declared_names(&trace, ui_rect, ITEM_PREFIX))
        )));
    };
    if !control.is_substantial() {
        return Ok(Some(format!(
            "`{SUBJECT}` was declared at {control:?}, which has no usable area."
        )));
    }
    driver.click_at(session.frame()?.declared_center(control))?;
    session.settle(20);

    let trace = session.trace()?;
    let Some(body) = declared(&trace, ui_rect, DIALOG) else {
        return Ok(Some(format!(
            "the click on `{SUBJECT}` opened no window: no `{DIALOG}` region was declared. \
             `{SUBJECT_ID}` is registered and its control is on screen, so the dispatch arm did \
             not run or did not open the window. Trace: {}.",
            session.trace_path().display()
        )));
    };
    if !body.is_substantial() {
        return Ok(Some(format!(
            "`{DIALOG}` was declared at {body:?}, which has no usable area — the window is laid \
             out and not on screen. That is the class of defect that shipped the redaction \
             panel's apply control below the bottom of its own pane."
        )));
    }

    // --- C. expand Appearance -----------------------------------------------
    //
    // Before the BEFORE capture, deliberately. Expanding a group changes the
    // window's layout — the body grows and the scroll area re-flows — and a
    // "before" sampled with the group shut would be compared against an "after"
    // sampled with it open, so part of any measured difference would be the
    // expansion rather than the theme.
    let trace = session.trace()?;
    let Some(heading) = declared(&trace, ui_rect, APPEARANCE_HEADING) else {
        return Ok(Some(format!(
            "the Settings window is open but declared no `{APPEARANCE_HEADING}`. Headings declared: {}.",
            list(&declared_names(&trace, ui_rect, "settings.heading."))
        )));
    };
    driver.click_at(
        frame_of(&session, &trace, ui_rect, APPEARANCE_HEADING)?.declared_center(heading),
    )?;
    session.settle(12);

    // --- D. the picture, before ---------------------------------------------
    //
    // ★★ THE DIALOG'S OWN WINDOW, not the application's. Settings became a real
    // OS window on 2026-08-21, and a capture of the application shows the page
    // where the dialog used to be — while the sampler goes on sampling and
    // reports a confident colour about a piece of the drawing. A measurement of
    // the wrong surface is indistinguishable from a measurement of a broken one.
    let before_frame = frame_of(&session, &trace, ui_rect, DIALOG)?;
    let before_path = ctx.out("settings_theme.before.png");
    let before_image = crate::capture::frame_to_png(&session, &before_frame, &before_path)?;
    report.artifact(before_path);
    let Some(before) = fill_of(&before_image, &before_frame, body) else {
        return Err(Error::new(
            "the window's body region did not map onto the capture, so no colour could be \
             sampled. This is a coordinate failure in the harness, not a verdict on the \
             application.",
        ));
    };

    // --- E. choose Dark -----------------------------------------------------
    let trace = session.trace()?;
    let Some(radio) = declared(&trace, ui_rect, DARK_RADIO) else {
        return Ok(Some(format!(
            "the Settings window is open and publishes its regions, but there is no \
             `{DARK_RADIO}` — so the window has no theme picker in it. Regions declared under \
             the theme namespace: {}.",
            list(&declared_names(&trace, ui_rect, "settings.theme."))
        )));
    };
    driver.click_at(before_frame.declared_center(radio))?;
    // ★ Generous, and for a stated reason: the theme is installed at the TOP of
    // the next frame, and `Theme::apply` rewrites both of egui's styles and
    // re-stashes the theme. A settle tuned to a single repaint would sample a
    // frame in which the click had registered and the style had not.
    session.settle(20);

    // --- F. the picture, after ----------------------------------------------
    let trace = session.trace()?;
    let after_frame = frame_of(&session, &trace, ui_rect, DIALOG)?;
    let after_path = ctx.out("settings_theme.after.png");
    let after_image = crate::capture::frame_to_png(&session, &after_frame, &after_path)?;
    report.artifact(after_path);
    // ★ Re-read the rect rather than reusing `body`. A theme change alters
    // metrics as well as colours — the `airy` preset is explicitly roomier —
    // so the window may have been laid out differently, and sampling the old
    // rectangle could land outside it. `D:\dev\rag\egui` records the general
    // form of this: harness coordinates go stale the moment a layout changes.
    let trace = session.trace()?;
    let after_body = declared(&trace, ui_rect, DIALOG).unwrap_or(body);
    let Some(after) = fill_of(&after_image, &after_frame, after_body) else {
        return Err(Error::new(
            "the window's body region did not map onto the second capture.",
        ));
    };

    // --- G. the verdict -----------------------------------------------------
    let luma = |c: Rgb| i32::from(c.r) + i32::from(c.g) + i32::from(c.b);
    let drop = (luma(before) - luma(after)) / 3;

    report.note(format!(
        "window body {before:?} -> {after:?} after choosing Dark (mean channel drop {drop})"
    ));

    if before == after {
        return Ok(Some(format!(
            "choosing Dark changed nothing. The window body measured {before:?} before and \
             after, so the picker writes a token that nothing installs — which is `DEFECTS.md` \
             D10 exactly: three presets compiled in, and a chooser that chooses nothing."
        )));
    }
    if drop < MIN_DARKENING {
        return Ok(Some(format!(
            "choosing Dark moved the window body from {before:?} to {after:?} — a mean channel \
             change of {drop}, against a floor of {MIN_DARKENING}. Something repainted, but it \
             is not a dark theme: either the picker is wired to the wrong preset, or what moved \
             is a hover highlight on the radio rather than the window's own fill."
        )));
    }

    Ok(None)
}
