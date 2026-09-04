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
//!
//! # ★★★ The third thing this file gained on 2026-09-04
//!
//! An outside review of the harness (`REVIEW_TRIAGE.md` rows **PartC** and
//! **A20x**) named three holes, all of them in this file's subject. Two were
//! about the presets themselves and are answered next door in
//! [`super::theme_page`], whose header carries them; the third is here,
//! because it is about this check's own window. All three are the same
//! species: **a thing the suite believed and had never driven.**
//!
//! ## 3. Cancel really reverting a live theme change — a SILENT one-line coupling
//!
//! The theme is the one setting in that window that takes effect **before
//! Save**, because a theme cannot be judged from a radio label. The mechanism is
//! one line in `app/frame.rs`:
//!
//! ```ignore
//! let theme_token = self.settings_draft.as_ref()
//!     .map_or(self.settings.theme.as_str(), |draft| draft.working.theme.as_str());
//! ```
//!
//! `Draft`'s own header states the consequence as the design: *"Cancel drops the
//! draft, so the look reverts with it — no separate undo path, and nothing that
//! can get out of step."*
//!
//! Break that line the other way — read the draft and never fall back, or adopt
//! the token on close — and **nothing looks wrong**. The window shuts, the
//! application is dark, and the operator who was only *trying it on* has an
//! appearance they did not choose and no undo. Nothing on screen says a setting
//! was applied; nothing on disk changed either, so a restart silently undoes it
//! and the operator learns the program is unpredictable. The failure is
//! invisible in exactly the way `DEFECTS.md` D10 was.
//!
//! [`SettingsThemeTakesEffect`] already opens the window, already measures it,
//! and now measures **three** pictures instead of two: light, dark, and light
//! again after the window is dismissed.
//!
//! ### ⚠ It dismisses with Escape, not with the Cancel BUTTON, and that is a gap
//!
//! **`dialogs::settings` publishes no `ui-rect` for its Cancel button.** There
//! is nothing to aim at, and this check will not compute a coordinate for one:
//! the button beside it is **Save**, which writes the operator's real
//! `settings.txt`, and a harness that guesses at a button position and lands one
//! control to the left is a harness that silently rewrites the configuration of
//! the machine it is verifying. That trade is not close.
//!
//! What is driven instead is the route `app::settings_window` documents as
//! **contractually identical**:
//!
//! > If closing by ✕ did anything different from closing by Cancel — kept the
//! > draft, half-applied it, saved it — the window would have two exits with two
//! > meanings and no way to tell which one an operator took. **Both paths drop
//! > the draft.**
//!
//! Escape is read from the child window's own input (`dialogs::host`) and sets
//! the same `frame.closed`, which the host turns into `settings_draft = None` —
//! the identical assignment `Outcome::Cancel` makes. So the coupling under test
//! is the one that ships.
//!
//! ★ What this does **not** cover, stated plainly so nobody reads more into a
//! green run than is there: a future edit that made `Outcome::Cancel` alone
//! behave differently — saving, or adopting the theme — would pass this check.
//! Closing that requires one line in the application:
//! `crate::diag::ui_rect("settings.cancel", <the Cancel button's rect>)` beside
//! the existing `REGION_BODY` declaration. This crate may not write it.

use crate::checks::driving::{
    ITEM_PREFIX, SHELL_DIAG_ENV, TAB_EVENT, declared, declared_names, delta, fill_of, frame_of,
    list, shell_trace,
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
pub(super) const DIALOG: &str = "dialog:settings";

/// The Dark preset's radio. `dialogs::settings::REGION_THEME_PREFIX` plus
/// `egui_shell::theme::Preset::Dark`'s key.
const DARK_RADIO: &str = "settings.theme.dark";

/// The namespace every theme radio publishes under, for a SKIP reason that
/// lists what *was* there when the one being looked for was not.
pub(super) const THEME_PREFIX: &str = "settings.theme.";

/// The application's own central area — `crate::app::REGION_CENTRAL_PANEL`.
///
/// Sampled by the Cancel half of [`SettingsThemeTakesEffect`], which runs with
/// **no document**: with nothing open the central panel is the application's
/// own surface and takes the preset's colour directly, so it is the cheapest
/// honest witness that a live theme change reached the application window
/// rather than only the dialog that chose it.
const CENTRAL_PANEL: &str = "central-panel";

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

/// Above this mean channel value, the window that opened is a **light** one.
///
/// ★ A guard against a vacuous run, and against the worse thing a vacuous run
/// does here: a **false defect report**.
///
/// This check launches the operator's own binary against the operator's own
/// `settings.txt`, and if that file already says `theme = dark` then the window
/// opens dark, clicking Dark changes nothing, and every assertion below reads
/// as *"the picker writes a token nothing installs"* — `DEFECTS.md` D10, filed
/// against a build in which the feature works perfectly. A fixture cannot
/// defeat this, because the condition is not a default but a **starting
/// state**.
///
/// 140 sits above the darkest light preset (`quiet`'s surface measures ≈ 241)
/// by a hundred levels and above the lightest dark one (`dark`'s surface
/// measures ≈ 38) by the same again, so no plausible palette lands on it.
const LIGHT_START_FLOOR: i32 = 140;

/// How far the application may drift from its opening colour after the window
/// is dismissed, before the revert counts as not having happened.
///
/// **Zero is the expected reading.** The theme token after Cancel is the same
/// `String` it was before the window opened, so the same style is written, the
/// same fill is painted, and a lossless BGRA capture of it differs by nothing
/// at all. This is not a tolerance for a real difference; it is a tolerance for
/// the sampler, which reports the *mean of a quantised bucket* and can therefore
/// move by a level or two if a scrollbar, a focus ring or a hover highlight
/// falls inside the region on one capture and not the other.
///
/// **6** is one quantisation bucket ([`crate::pixels`] quantises to 5 bits, so a
/// bucket is 8 levels wide) and is two orders below the ≈ 200 that separates the
/// light presets from Dark — so a Cancel that failed to revert cannot hide under
/// it, and a repaint artefact cannot trip it.
const MAX_REVERT_DRIFT: u16 = 6;

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
            Err(why) => report.from_error(&why),
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

    // --- A, B, C. the window, open, with Appearance expanded ----------------
    if let Some(failure) = open_the_theme_picker(&session, &driver, ui_rect, &trace)? {
        return Ok(Some(failure));
    }

    let trace = session.trace()?;
    let Some(body) = declared(&trace, ui_rect, DIALOG) else {
        return Err(Error::new(format!(
            "`{DIALOG}` stopped being declared between opening the window and measuring it, so \
             there is nothing to sample. Trace: {}.",
            session.trace_path().display()
        )));
    };

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

    // ★★ THE STARTING STATE, CHECKED BEFORE ANYTHING IS CONCLUDED FROM IT.
    //
    // See [`LIGHT_START_FLOOR`]. Everything below reads a *drop* in luminance as
    // proof that a theme installed, and a window that opened dark has no room
    // to drop — so on a machine whose saved theme is already Dark this check
    // would report `DEFECTS.md` D10 against a build in which the feature works.
    // A SKIP naming the cause is the only honest outcome; a fixture cannot fix
    // it, because the condition is the operator's own configuration.
    if mean_channel(before) < LIGHT_START_FLOOR {
        return Err(Error::new(format!(
            "this machine's saved theme is ALREADY a dark one — the Settings window opened at \
             {before:?}, a mean channel of {}, below the {LIGHT_START_FLOOR} that says `light`. \
             Choosing Dark cannot be shown to darken anything from here, and every assertion \
             below would read as D10 against a build where the picker works. SKIPPED rather \
             than failed. Set the theme to Quiet or Airy and run again.",
            mean_channel(before)
        )));
    }

    // --- D2. the APPLICATION's own window, before ---------------------------
    //
    // ★★★ The second surface, and it is the one the Cancel half needs.
    //
    // Step D measured the DIALOG, which proves the window that chose the preset
    // took it. That is not the same claim as *the application* took it — and it
    // is the application the operator is looking at. It is also the only surface
    // that still exists after the window is dismissed, which is the whole
    // subject of step H.
    //
    // `session.frame()` is the application's own window: `Session::window` is
    // the handle captured at launch and does not follow a dialog.
    let app_frame = session.frame()?;
    let trace = session.trace()?;
    let Some(central) = declared(&trace, ui_rect, CENTRAL_PANEL) else {
        return Err(Error::new(format!(
            "the application declared no `{CENTRAL_PANEL}` region, so there is no surface of its \
             own to measure the theme against. Regions it did declare: {}. This is a harness \
             aim, not a verdict on the build — the constant is `app::REGION_CENTRAL_PANEL` and \
             a rename there un-aims it. SKIPPED.",
            list(&declared_names(&trace, ui_rect, "central"))
        )));
    };
    let app_light_path = ctx.out("settings_theme.app-light.png");
    let app_light_image = crate::capture::frame_to_png(&session, &app_frame, &app_light_path)?;
    report.artifact(app_light_path);
    let Some(app_light) = fill_of(&app_light_image, &app_frame, central) else {
        return Err(Error::new(
            "the application's central panel did not map onto its own capture. A harness \
             coordinate failure, not a verdict on the build.",
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
    let drop = mean_channel(before) - mean_channel(after);

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

    // --- H. the Cancel contract — `REVIEW_TRIAGE.md` A20x -------------------
    //
    // Everything above proves the theme went ON. This proves it comes OFF, and
    // the module header carries the argument for why that half is the one whose
    // failure is silent. Three pictures, not two: light, dark, light again.
    //
    // ★ The application's own window, measured a second time first. A live
    // preview that reached the dialog and not the application would be a real
    // and separate defect — and it would make the revert measurement below
    // meaningless, because a surface that never darkened cannot be seen to
    // lighten. So this is the precondition, asserted before its consequence.
    let trace = session.trace()?;
    let central_dark = declared(&trace, ui_rect, CENTRAL_PANEL).unwrap_or(central);
    let app_dark_path = ctx.out("settings_theme.app-dark.png");
    let app_dark_image = crate::capture::frame_to_png(&session, &app_frame, &app_dark_path)?;
    report.artifact(app_dark_path);
    let Some(app_dark) = fill_of(&app_dark_image, &app_frame, central_dark) else {
        return Err(Error::new(
            "the application's central panel did not map onto the dark capture.",
        ));
    };
    let app_drop = mean_channel(app_light) - mean_channel(app_dark);
    report.note(format!(
        "application central panel {app_light:?} -> {app_dark:?} while the window is open \
         (mean channel drop {app_drop})"
    ));
    if app_drop < MIN_DARKENING {
        return Ok(Some(format!(
            "the Settings window went dark and THE APPLICATION DID NOT. Its central panel \
             measured {app_light:?} before and {app_dark:?} after — a mean channel change of \
             {app_drop} against a floor of {MIN_DARKENING} — while the window itself moved by \
             {drop}. The preview is reaching the dialog's own `Context` and not the \
             application's, which is the half of `DEFECTS.md` D10 that a check measuring only \
             the dialog cannot see."
        )));
    }

    // ★ ONE HARMLESS CLICK, and it is not a spare step.
    //
    // The capture above raised the APPLICATION's window, so the application now
    // holds the foreground — and `Driver::press` deliberately declines to raise
    // when the application already has it (a harness that raises on every
    // keystroke steals focus from a dialog that legitimately owns it). Escape
    // would therefore be delivered to the application window, where it closes
    // nothing, and this check would report the Cancel contract as broken
    // because of its own capture.
    //
    // Re-clicking the radio that is ALREADY selected brings the dialog forward
    // and gives it the keyboard, and changes nothing: `appearance::theme` writes
    // `draft.working.theme = preset.key()` on a click, which is the value it
    // already holds. Nothing else in the window is touched, and in particular
    // nothing near Save is.
    let trace = session.trace()?;
    let dialog_frame = frame_of(&session, &trace, ui_rect, DIALOG)?;
    let radio_now = declared(&trace, ui_rect, DARK_RADIO).unwrap_or(radio);
    driver.click_at(dialog_frame.declared_center(radio_now))?;
    session.settle(8);

    // ★★ Escape, read from the CHILD window's own input. See the module header
    // for why this route rather than the Cancel button, and for what that costs.
    driver.press(crate::sys::vk::ESCAPE)?;
    session.settle(24);

    // ★★★ DID IT ACTUALLY CLOSE? A SKIP, NOT A VERDICT.
    //
    // If the window is still standing, the draft still exists, the application
    // is still dark, and a check that went straight to the colour would report
    // *"Cancel does not revert the theme"* — a confident defect report about a
    // coupling that was never exercised, caused by a keystroke that went
    // somewhere else. The absence of the region is the mechanism's own
    // evidence: `declared` reads `ui-rect-gone` as well as `ui-rect`, so a
    // window that stopped drawing reports as gone rather than as a fossil.
    let trace = session.trace()?;
    if declared(&trace, ui_rect, DIALOG).is_some() {
        return Err(Error::new(format!(
            "Escape did not close the Settings window — `{DIALOG}` is still being declared — so \
             the Cancel contract was never exercised and nothing is claimed about it. SKIPPED, \
             not failed: the keystroke is the harness's, and a key that did not arrive says \
             nothing about the application. Look first at whether the application window had \
             the foreground when the key was sent. Trace: {}.",
            session.trace_path().display()
        )));
    }

    let central_back = declared(&trace, ui_rect, CENTRAL_PANEL).unwrap_or(central);
    let app_back_path = ctx.out("settings_theme.app-reverted.png");
    let app_back_image = crate::capture::frame_to_png(&session, &app_frame, &app_back_path)?;
    report.artifact(app_back_path);
    let Some(app_back) = fill_of(&app_back_image, &app_frame, central_back) else {
        return Err(Error::new(
            "the application's central panel did not map onto the capture taken after the \
             window closed.",
        ));
    };

    let drift = delta(app_back, app_light);
    report.note(format!(
        "application central panel {app_dark:?} -> {app_back:?} after the window was dismissed \
         (it opened at {app_light:?}; drift from that {drift})"
    ));
    if drift > MAX_REVERT_DRIFT {
        return Ok(Some(format!(
            "★ DISMISSING THE SETTINGS WINDOW DID NOT PUT THE THEME BACK. The application \
             opened at {app_light:?}, went to {app_dark:?} while Dark was chosen, and settled at \
             {app_back:?} once the window was gone — {drift} away from where it started, against \
             a tolerance of {MAX_REVERT_DRIFT}. \
             \
             The theme is the ONE setting in that window that takes effect before Save, and the \
             only thing that takes it back is `app/frame.rs` falling back to \
             `self.settings.theme` when `settings_draft` is `None`. If that fallback is gone, or \
             the draft is being adopted on close, then an operator who opened Settings to LOOK \
             at Dark now has it — with nothing on screen saying a setting was applied, and \
             nothing on disk, so a restart undoes it again. `dialogs::settings::Draft` states \
             the contract this breaks: *\"Cancel drops the draft, so the look reverts with it — \
             no separate undo path, and nothing that can get out of step.\"*"
        )));
    }

    Ok(None)
}

/// The mean of a colour's three channels, `0..=255`.
///
/// Every luminance claim in this file is stated as a mean channel rather than as
/// a perceptual luminance, and deliberately: the question here is *"did this
/// surface change colour"*, not *"can this be read"*. [`crate::pixels`] answers
/// the second, weights the channels for the eye, and would report a green and a
/// blue of equal brightness as the same — which is exactly wrong for detecting
/// a theme that tinted something.
fn mean_channel(c: Rgb) -> i32 {
    (i32::from(c.r) + i32::from(c.g) + i32::from(c.b)) / 3
}

/// **File tab → Settings → expand Appearance**, the three clicks both checks in
/// this file begin with.
///
/// `Ok(None)` means the theme radios are on screen and clickable. `Ok(Some(_))`
/// is a FAILURE message — a control that should be there and is not. `Err` is a
/// SKIP: the harness could not deliver a click, or the application never said
/// anything, and neither is a verdict on the feature.
///
/// # Why this is shared and `markup_rectangle`'s equivalent is not
///
/// `checks::driving`'s header records the rule: a move is shared when the two
/// callers would otherwise **drift apart while both passing**. These two are the
/// same three clicks at the same three regions, and every failure message names
/// a specific application constant — `file.settings`, `dialog:settings`,
/// `settings.heading.appearance`. Two copies would mean two places to update
/// when one of those is renamed, and the copy nobody updated would go on
/// reporting *"the window has no theme picker in it"* about a window that has
/// one. That is the failure this suite has already recorded twice.
///
/// It stays inside this module rather than moving to `driving` because nothing
/// outside this file opens the Settings window, and a helper hoisted before it
/// has a second caller is a helper whose shape is guessed.
pub(super) fn open_the_theme_picker(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    trace: &crate::trace::Trace,
) -> Result<Option<String>> {
    // ★ Nothing may have opened this window unasked. `view.app_initiative`'s
    // specified default is Never, and a settings window that floats over the
    // canvas on its own breaks it in the most annoying way available.
    if declared(trace, ui_rect, DIALOG).is_some() {
        return Ok(Some(format!(
            "`{DIALOG}` was declared before anything was clicked — the Settings window opened \
             on its own, which pdfcer may not do."
        )));
    }

    // --- A. the File tab ---------------------------------------------------
    let tab = declared(trace, ui_rect, TAB).ok_or_else(|| {
        Error::new(format!(
            "the application declared no `{TAB}` region. Tabs declared: {}.",
            list(&declared_names(trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(tab))?;
    session.settle(12);
    if !shell_trace(session)?
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
    // Before any capture, deliberately. Expanding a group changes the window's
    // layout — the body grows and the scroll area re-flows — and a "before"
    // sampled with the group shut would be compared against an "after" sampled
    // with it open, so part of any measured difference would be the expansion
    // rather than the theme.
    let Some(heading) = declared(&trace, ui_rect, APPEARANCE_HEADING) else {
        return Ok(Some(format!(
            "the Settings window is open but declared no `{APPEARANCE_HEADING}`. Headings \
             declared: {}.",
            list(&declared_names(&trace, ui_rect, "settings.heading."))
        )));
    };
    driver.click_at(
        frame_of(session, &trace, ui_rect, APPEARANCE_HEADING)?.declared_center(heading),
    )?;
    session.settle(12);
    Ok(None)
}
