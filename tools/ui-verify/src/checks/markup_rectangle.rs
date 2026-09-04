//! `markup_rectangle_arms_from_the_ribbon` — the regression test for a
//! **four-link chain in which every link has a passing unit test and nobody
//! had seen the button work**.
//!
//! # The defect class this exists for
//!
//! `HANDOFF.md` §2 lists nine defects found only by running the program. The
//! second one is this file's ancestor and its whole justification:
//!
//! > The icon painter existed, was tested, and was never passed to the ribbon
//! > — the whole ribbon was text buttons.
//!
//! Nothing was *wrong*. `icons` proved the painter draws, with 52 tests.
//! `egui-shell` proved its seam accepts a painter. Both were true, both were
//! green, and the one fact neither crate could state is that the application
//! hands the former to the latter — because that is a property of a **call
//! site**, and a call site's effect is observable only in a running window.
//!
//! Clicking `Markup ▸ Shapes ▸ Rectangle` is the same shape with four links
//! instead of one:
//!
//! | # | Link | Where | Its own test |
//! |---|---|---|---|
//! | 1 | the ribbon click reports the command | `egui-shell`'s `band::render_command` | yes |
//! | 2 | dispatch routes the id to a markup kind | `app/dispatch.rs`, via `shell::commands::markup_for_command` | yes |
//! | 3 | the kind arms the canvas tool | `canvas::tool::arm_markup` | yes |
//! | 4 | the armed tool renders the control **pressed** | `app/conditions.rs` publishing `selected:markup.rectangle`, read by `band::render_command` | yes |
//!
//! Four passing tests, four joins, and **no test anywhere observes two
//! adjacent links being connected**. Deleting the guard arm in step 2 breaks
//! the feature completely and breaks no test in the workspace — which is
//! exactly the state the icon painter shipped in.
//!
//! # Why this check could not be written before now
//!
//! Because nothing outside the process could find the button. The trace
//! published a `ui-rect` for every group caption and every mode segment and
//! **nothing for any individual command control**, so there was no way to
//! aim a click at Rectangle, and therefore no way to prove that clicking it
//! did anything. `egui_shell::ribbon::report::band_item` and its call site in
//! `band::render_command` landed with this check, for this check.
//!
//! # What it does, through the operating system
//!
//! Mouse only, because nothing below needs a key — not because a key could
//! not be sent.
//!
//! ★ **CORRECTED 2026-08-18.** These headers used to say synthetic keyboard
//! input does not reach the target window on this machine. It DOES — see
//! [`crate::checks::add_text`], which types real characters into a caret
//! draft and asserts they landed. The belief came from `Ctrl+E` producing no
//! trace, which was the dead-keymap defect (fourteen of twenty-one declared
//! chords were dispatched by nothing) misread as a property of the machine —
//! and while it stood nobody drove a chord, so nothing could contradict it.
//! Every gesture below is a real `SetCursorPos` +
//! `mouse_event` click at a point derived from a rectangle **the application
//! itself declared on the frame it drew it**; see
//! [`crate::coords::WindowFrame::declared_center`].
//!
//! 1. Click the **Review** mode segment. The Markup tab is in Review's and
//!    Edit's tab lists and not in Read's, and Read is the default, so without
//!    this step there is no Markup tab to activate.
//! 2. Click the **Markup** tab.
//! 3. Capture the window — the *before* picture.
//! 4. Click **Rectangle** in the Shapes group.
//! 5. Capture the window again — the *after* picture.
//!
//! # ★ The assertions, split by oracle, and why both are needed
//!
//! ## Trace evidence — that the arm happened
//!
//! | Assertion | Line | What its absence means |
//! |---|---|---|
//! | the click reached the control | `ribbon-command-invoked id=markup.rectangle` | the click missed, or the control is disabled |
//! | the tool was armed | `markup-tool tool=Markup(Rectangle)` | **link 2 or 3** — see the chain above |
//!
//! The second is the one that matters, and it is genuinely necessary: the
//! armed tool is otherwise invisible from outside the process. A crosshair is
//! a cursor, and a screenshot of an armed canvas and an unarmed one are the
//! same picture — which is `HANDOFF.md`'s defect 8 exactly, the grid that was
//! a wash, found by printing the ladder the running program had chosen rather
//! than by looking at it.
//!
//! ## Pixel evidence — that the control renders pressed
//!
//! **And this is the half a trace line alone would not have caught.** A trace
//! line is written by the code under test, about itself. `arm_markup` traces
//! unconditionally the moment it is called, so `markup-tool` proves links 2
//! and 3 and says *nothing whatsoever* about link 4: a build whose ribbon
//! never renders a pressed state — because `conditions.rs` stopped publishing
//! `selected:markup.rectangle`, or because `render_command` stopped reading
//! it — emits an identical trace and looks identical to a reader of that
//! trace. That is defect 2's structure precisely, one layer up: the thing
//! works, and the surface the operator looks at does not say so.
//!
//! So the pressed state is asserted from the **captured window**, three ways,
//! and the third is the one that cannot be faked:
//!
//! | # | Comparison | What it rules out |
//! |---|---|---|
//! | P1 | Rectangle after ≠ Rectangle before | the control never changed |
//! | P2 | **Rectangle after ≠ Ellipse after**, in one capture | a *global* repaint — a theme change, a hover, a resize — masquerading as a pressed state |
//! | P3 | Ellipse after = Ellipse before | the whole band changing, i.e. P1 passing for a reason that has nothing to do with the click |
//!
//! P2 is the load-bearing one. It is a differential inside a single frame, so
//! nothing that happens to *both* controls can satisfy it; only something
//! that happened to the one that was clicked. Together P1–P3 say: this
//! control, and only this control, changed, and it changed across this click.
//!
//! The measured quantity is the **dominant colour bucket** of the control's
//! region ([`crate::pixels::contrast_at`]'s `background`), which is the
//! button's fill: in `egui` 0.35 a `Button::selected(true)` takes
//! `visuals.selection.bg_fill` for its frame instead of the widget state's
//! `weak_bg_fill` (`widget_style.rs`'s `button_style`, `SELECTED_CLASS`
//! branch). Contrast *ratio* is deliberately not the measure — under the
//! palette this was calibrated against the two fills were a light grey and a
//! light blue, about 1.3:1 apart, which a legibility threshold would call
//! identical. Since 2026-09-04 that channel carries an opaque accent and the
//! pair is far apart, but the measure stays a channel difference so the check
//! keeps working under every palette this program has had. See
//! [`MIN_PRESSED_DELTA`], which enumerates all three.
//!
//! # Why the check does not simply assert on `selected:markup.rectangle`
//!
//! Because the condition set is not published in the trace, and if it were,
//! asserting on it would be link 4's own unit test written a second time in a
//! slower harness. What is unverified is not whether the condition is
//! computed — that is tested — but whether computing it changes what the
//! operator sees. Only pixels answer that.
//!
//! # Every way this reports SKIP, and why none of them is a pass
//!
//! * no binary, no `--pdf`, `--no-input` — the harness never began;
//! * the diagnostic switches did not reach the process;
//! * the mode segment, the Markup tab, or the Shapes group's controls were
//!   never declared — each names the specific surface that is missing, and
//!   the `ribbon.item.*` case names `report::band_item` and its call site,
//!   because a build without Part 1 of this work is the one build where this
//!   check has nothing to aim at;
//! * a tool was already armed before the click — `arm_markup` **toggles** on
//!   the same kind, so a click on an already-armed Rectangle correctly
//!   *disarms* it, and a check that did not notice would report the feature
//!   broken;
//! * the two controls already looked different before the click, so a
//!   difference afterwards could not be attributed to it.

use crate::checks::{Check, CheckContext};
use crate::coords::WindowFrame;
use crate::error::{Error, Result};
use crate::geom::LRect;
use crate::image::{Image, Rgb};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::trace::Trace;

/// The mode whose tab list contains Markup, and the segment to click.
///
/// Read is the default and its tabs are `["file", "view"]`; Markup is in
/// Review's list and in Edit's. Review rather than Edit because it is the
/// weaker claim — a markup tool that works in Review works in Edit, and Edit
/// carries text editing, which is `HANDOFF.md`'s Phase 5 and unbuilt.
const MODE_SEGMENT: &str = "ribbon.mode.review";

/// The tab that carries the Shapes group.
const TAB: &str = "ribbon.tab.markup";

/// **The control under test.**
const SUBJECT: &str = "ribbon.item.markup.rectangle";

/// The command id of [`SUBJECT`], as dispatch and the shell spell it.
const SUBJECT_ID: &str = "markup.rectangle";

/// **The control that must NOT change** — the sibling in the same group,
/// drawn by the same code, in the same capture, on the same frame.
///
/// This is what makes the pixel evidence a differential rather than a
/// before-and-after: see the module header's P2.
const SIBLING: &str = "ribbon.item.markup.ellipse";

/// Where the pointer is parked before each capture.
///
/// The Shapes group's caption, which is an `egui::Label` and therefore has no
/// hover styling of its own, sitting directly beneath the controls being
/// measured. Parking matters: after a click the pointer is *on* the control,
/// `egui` paints it in its hovered visuals, and a before/after comparison
/// would then be measuring a hover as well as a selection. Parking on a
/// declared, inert region rather than at some corner of the screen keeps the
/// rule that this crate aims only at rectangles the application published.
const PARK: &str = "ribbon.group.markup.shapes.caption";

/// The namespace [`crate::checks::markup_rectangle`] depends on existing.
const ITEM_PREFIX: &str = "ribbon.item.";

/// The shell's own diagnostic switch, and the prefix its lines carry.
///
/// Two channels, deliberately, and this check reads both. `egui-shell` traces
/// under `EGUI_SHELL_DIAG` with the prefix an application sets via
/// `verify::set_prefix` — which `pdfcer-gui` does not call, so the lines arrive
/// under the crate's default. The application traces separately under
/// `PDFCER_DIAG` with `pdfcer-diag`.
///
/// The split is not an accident of this build: `verify`'s own header explains
/// that one variable name lets a harness arm tracing on *any* `egui-shell`
/// application without first discovering its name. The consequence here is
/// that "the click reached the control" (the shell's fact) and "the tool was
/// armed" (the application's fact) come from two different streams in one
/// file, which is exactly what makes the failure attributable: a missing
/// `markup-tool` **with** a present `ribbon-command-invoked` names the
/// application's dispatch and nothing else.
const SHELL_DIAG_ENV: (&str, &str) = ("EGUI_SHELL_DIAG", "1");

/// The line prefix `egui-shell` uses when the application has not set one.
const SHELL_TRACE_PREFIX: &str = "egui-shell-diag";

/// `ribbon-mode-selected mode=…` — the shell reporting a mode segment click.
const MODE_EVENT: &str = "ribbon-mode-selected";

/// `ribbon-tab-activated tab=…` — the shell reporting a tab click.
const TAB_EVENT: &str = "ribbon-tab-activated";

/// `ribbon-command-invoked id=… handler=…` — the shell reporting that a band
/// control was clicked and its token handed to the application.
const INVOKE_EVENT: &str = "ribbon-command-invoked";

/// `markup-tool tool=…` — the application reporting which tool the canvas is
/// now armed with. Emitted by `canvas::tool::arm_markup`, unconditionally,
/// every time it is called.
const ARM_EVENT: &str = "markup-tool";

/// The `Debug` spelling of `CanvasTool::Markup(MarkupKind::Rectangle)`.
const ARM_VALUE: &str = "Markup(Rectangle)";

/// `command-unimplemented id=…` — `app/dispatch.rs`'s fall-through arm.
///
/// Read only to *improve a failure message*. Its presence alongside a missing
/// `markup-tool` is the signature of a dispatch that received the command and
/// had no arm for it, which is a different fix from a dispatch that never
/// received it at all.
const UNIMPLEMENTED_EVENT: &str = "command-unimplemented";

/// How far apart two dominant fills must be to count as "one of these is
/// pressed", as a maximum absolute per-channel difference in 0–255.
///
/// # ★ Three candidate palettes, and the threshold is below the smallest
///
/// The number is deliberately derived from every pair the running build could
/// plausibly produce, rather than from the one somebody assumed. Three have
/// been true of this program at different times, and the check has to survive
/// all of them, because a threshold tuned to one palette is a check that goes
/// red on a restyle and reports it as a broken feature.
///
/// **(a) The `quiet` preset as it shipped until 2026-09-04.** A band button's
/// unpressed frame fill is `widgets.inactive.weak_bg_fill` = `panel` =
/// `#E8E8EA`; a pressed one takes `visuals.selection.bg_fill`, which that
/// theme pointed at `selection_fill` = `rgba(90, 140, 220, 70)`, composited
/// over it —
///
/// ```text
/// α = 70/255 = 0.2745
/// r = 90·α + 232·(1−α) = 193
/// g = 140·α + 232·(1−α) = 207
/// b = 220·α + 234·(1−α) = 230
/// ```
///
/// — a maximum channel difference of **39**.
///
/// **(b) `egui`'s own stock light values.** Measured from a real capture on
/// 2026-08-14: unpressed `#E5E5E5`, pressed `#90D1FF`, a difference of **85**
/// (`widgets.inactive.weak_bg_fill` = grey 230, `selection.bg_fill` =
/// 144, 209, 255).
///
/// ⚠ **The sentence that used to explain (b) was false, and is corrected
/// here** — `REVIEW_TRIAGE.md` **T3**. It said the stock palette was what the
/// built binary paints with *"because nothing in `crates/pdfcer-gui` calls
/// `Theme::apply`"*. That was true when it was written and stopped being true
/// on 2026-08-14, when `DEFECTS.md` D10 was fixed: `app::frame` calls
/// `theme.apply(&ctx)` every frame, from the operator's own settings. The
/// capture behind the 85 was taken from a build on the wrong side of that
/// commit. **The constant is unaffected** — it was chosen to cover both
/// palettes and says so — so this is a false sentence rather than a wrong
/// number, which is exactly the kind that survives: nothing recomputes when a
/// premise expires.
///
/// **(c) The `quiet` preset since 2026-09-04** — `REVIEW_TRIAGE.md` **T2**.
/// `visuals.selection` is `egui`'s SELECTED-WIDGET channel and now carries the
/// pair it is named for: a pressed control's fill is `accent` = `#175CC4`,
/// opaque, against the same `#E8E8EA` unpressed fill. Maximum channel
/// difference **209** — by far the largest of the three, so the check gets
/// easier rather than harder. (Before that change the theme had handed the
/// channel to the canvas, which is why (a)'s pressed "fill" was a 27 % wash
/// that composited *paler than the button beside it*.)
///
/// # Why the threshold is 12, and why it survives all three
///
/// Above **0**, which is exactly what a lossless BGRA capture of two
/// identically filled controls produces — the measured before-click gap
/// between Rectangle and Ellipse is literally zero, not a small number. Below
/// **39**, the smallest of the three real differences, by a factor of three.
/// So the verdict is the same under any of them, and neither a palette tweak
/// nor a preset change nor a display colour profile flips it.
///
/// ★ That margin is the whole reason this constant did not have to move when
/// the theme was installed (b→a) and did not have to move again when the
/// selection channel was re-pointed (a→c). A threshold derived from ONE
/// measured pair would have been wrong twice.
///
/// # Why a channel difference and not a contrast ratio
///
/// Because the two older pairs are near-equal in luminance. The pre-2026-09-04
/// `quiet` pair is about **1.3:1** and the stock pair about **1.5:1** — both
/// far under [`crate::pixels::AA_LARGE`]'s 3.0, so a legibility oracle would
/// call a pressed control and an unpressed one the same colour. Contrast
/// answers "can this be read"; the question here is "is this a different
/// colour", and those are not the same measurement. (Pair (c) would pass a
/// contrast oracle comfortably, which is a fact about the fix and not a reason
/// to change the measure — the check must stay able to fail.)
const MIN_PRESSED_DELTA: u16 = 12;

/// See the module documentation.
pub struct MarkupRectangleArmsFromTheRibbon;

impl Check for MarkupRectangleArmsFromTheRibbon {
    fn name(&self) -> &'static str {
        "markup_rectangle_arms_from_the_ribbon"
    }

    fn defect(&self) -> &'static str {
        "clicking Markup ▸ Shapes ▸ Rectangle does not arm the canvas tool, or arms it \
         without the control rendering pressed — a four-link chain in which every link \
         has a passing unit test and no test observes two of them connected"
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

/// The last rect the application declared under `name`, if any.
///
/// **Last wins.** A region is re-declared whenever it moves, and an early
/// frame can carry a rect from before the layout settled — the find bar's
/// one-frame misplacement was exactly that, and taking the first occurrence
/// would aim this check's clicks at it.
fn declared(trace: &Trace, ui_rect: &str, name: &str) -> Option<LRect> {
    trace
        .events(ui_rect)
        .filter(|l| l.get("name") == Some(name))
        .filter_map(|l| l.get_rect("rect"))
        .last()
}

/// Every distinct region name the application declared beginning with
/// `prefix`, in first-seen order.
///
/// Used only for SKIP reasons. A reason that says "I did not find X" and does
/// not say what it *did* find sends its reader to guess; this crate has a
/// standing rule about that ([`crate::checks`] rule 5).
fn declared_names(trace: &Trace, ui_rect: &str, prefix: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for line in trace.events(ui_rect) {
        let Some(name) = line.get("name") else {
            continue;
        };
        if name.starts_with(prefix) && !out.iter().any(|n| n == name) {
            out.push(name.to_owned());
        }
    }
    out
}

/// The dominant colour of a declared region in a capture — the control's
/// fill.
///
/// `None` when the region resolved to no pixels, which means the application
/// declared it outside its own client area. That is a finding rather than a
/// measurement and the caller reports it as one.
fn fill_of(image: &Image, frame: &WindowFrame, rect: LRect) -> Option<Rgb> {
    let px = frame.logical_to_capture_pixels(rect);
    if px.area() == 0 {
        return None;
    }
    let report = crate::pixels::contrast_at(image, px);
    (report.sampled > 0).then_some(report.background)
}

/// Maximum absolute per-channel difference between two colours.
fn delta(a: Rgb, b: Rgb) -> u16 {
    let d = |x: u8, y: u8| u16::from(x.abs_diff(y));
    d(a.r, b.r).max(d(a.g, b.g)).max(d(a.b, b.b))
}

/// Run the sequence.
///
/// The three-way return is [`crate::report`]'s rule made structural: `Err` is
/// a precondition that was absent (SKIP), `Ok(Some(_))` is an assertion that
/// did not hold (FAIL), `Ok(None)` is a pass. Reaching for `?` therefore
/// yields a SKIP, which is the safe default; the unsafe default would be a
/// pass.
#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    // --- preconditions -----------------------------------------------------
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    // Not optional. Every `markup.*` command is registered
    // `.enabled_when("doc.pages")`, so with nothing open the control is
    // correctly greyed — and a check that drove that would be asserting the
    // enable predicate works while claiming to assert that Rectangle does.
    let pdf = ctx.pdf.clone().ok_or_else(|| {
        Error::new(
            "no --pdf. Every markup command is gated on `doc.pages`, so with no document open \
             the Rectangle control is correctly disabled and this check would be measuring the \
             gate rather than the feature.",
        )
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check is three real clicks and two window \
             captures, and every one of them needs the pointer and the foreground. Reported as \
             SKIPPED rather than passed: a check that did not run has learned nothing.",
        ));
    }

    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the application cannot state \
             where its controls are and this check has nothing to aim at.",
            ctx.profile.name
        ))
    })?;

    // --- launch, with BOTH diagnostic channels armed -----------------------
    let mut spec = LaunchSpec::new(&exe, ctx.out("markup_rectangle.trace.txt"));
    spec.pdf = Some(pdf.clone());
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
        "launched {} as pid {} with {}={} and {}={}",
        exe.display(),
        session.pid(),
        ctx.profile.diag_env.0,
        ctx.profile.diag_env.1,
        SHELL_DIAG_ENV.0,
        SHELL_DIAG_ENV.1
    ));
    report.artifact(session.trace_path().to_path_buf());
    // Generous: the ribbon is chrome and is laid out on the first frame, but
    // the fixture still has to parse and raster, and a window captured
    // mid-raster is a window whose controls are drawn over a placeholder.
    session.settle(40);

    let trace = session.trace()?;
    if !trace.started(ctx.profile.vocab.start_event) {
        return Err(Error::new(format!(
            "the trace has no `{}` line, so the diagnostic switch {}={} did not reach the \
             process and this check has no way to learn where any control is. Captured stderr \
             is at {}.",
            ctx.profile.vocab.start_event,
            ctx.profile.diag_env.0,
            ctx.profile.diag_env.1,
            session.trace_path().display()
        )));
    }
    for reject in trace.rejected_steps() {
        report.note(format!(
            "the application REJECTED a script step: {}",
            reject.raw
        ));
    }

    let frame = session.frame()?;
    report.note(format!(
        "window client area {}x{} px at desktop ({}, {}), DPI scale {:.2}",
        frame.client_size.0,
        frame.client_size.1,
        frame.client_origin.0,
        frame.client_origin.1,
        frame.scale
    ));
    let driver = Driver::new(session.window());

    // --- step 1: switch to Review -----------------------------------------
    let review = declared(&trace, ui_rect, MODE_SEGMENT).ok_or_else(|| {
        Error::new(format!(
            "the application declared no `{MODE_SEGMENT}` region, so there is no mode segment \
             to click and the Markup tab — which is in Review's and Edit's tab lists and not in \
             Read's — cannot be reached. Regions it did declare under `ribbon.mode.`: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.mode."))
        ))
    })?;
    report.note(format!(
        "{MODE_SEGMENT} declared at {review:?}; clicking its centre"
    ));
    driver.click_at(frame.declared_center(review))?;
    session.settle(12);

    // The shell's own report that the segment took the click. This is the
    // input-channel proof, and it is a SKIP rather than a FAIL for the reason
    // `find_bar`'s Ctrl+2 control exists: a check that could not deliver a
    // click has learned nothing about the application, and naming a feature
    // as the culprit when nothing was ever clicked at it is worse than no
    // check at all.
    let shell = shell_trace(&session)?;
    if !shell
        .events(MODE_EVENT)
        .any(|l| l.get("mode") == Some("review"))
    {
        return Err(Error::new(format!(
            "the click on `{MODE_SEGMENT}` produced no `{MODE_EVENT} mode=review` line, so no \
             click reached the ribbon and nothing below would mean anything. Two readings, and \
             this check declines to choose between them: the pointer injection is not reaching \
             this window, or the shell diagnostic switch {}={} did not reach the process — the \
             shell trace carries {} line(s) under `{SHELL_TRACE_PREFIX}`. Trace: {}.",
            SHELL_DIAG_ENV.0,
            SHELL_DIAG_ENV.1,
            shell.lines.len(),
            session.trace_path().display()
        )));
    }
    report.note("the Review segment reported the click, so pointer input reaches the ribbon");

    // --- step 2: activate the Markup tab -----------------------------------
    let trace = session.trace()?;
    let tab = declared(&trace, ui_rect, TAB).ok_or_else(|| {
        Error::new(format!(
            "the application declared no `{TAB}` region after switching to Review. Either this \
             build has no Markup tab, or the tab strip is too narrow and the tab has moved into \
             the strip's overflow menu — which this check cannot open, because the menu's \
             contents are not published as regions. Tabs declared: {}. Strip affordance \
             declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab.")),
            declared(&trace, ui_rect, "ribbon.tabs.overflow")
                .map_or_else(|| "no".to_owned(), |r| format!("yes, at {r:?}")),
        ))
    })?;
    report.note(format!("{TAB} declared at {tab:?}; clicking its centre"));
    driver.click_at(frame.declared_center(tab))?;
    session.settle(12);

    let shell = shell_trace(&session)?;
    if !shell
        .events(TAB_EVENT)
        .any(|l| l.get("tab") == Some("markup"))
    {
        return Err(Error::new(format!(
            "the click on `{TAB}` produced no `{TAB_EVENT} tab=markup` line. The Review click \
             DID land, so pointer input works and this is not the input channel; the likely \
             cause is that the tab moved between the frame that declared its rect and the frame \
             that received the click. Re-run; if it persists, the tab strip is reflowing every \
             frame, which is itself the finding."
        )));
    }
    report.note("the Markup tab reported the click");

    // --- step 3: locate the two controls ----------------------------------
    let trace = session.trace()?;
    let items = declared_names(&trace, ui_rect, ITEM_PREFIX);
    if items.is_empty() {
        // ★ The build-without-Part-1 SKIP. It names the exact call site,
        // because a reader seeing this has a ribbon full of working controls
        // and no idea why the harness cannot find one.
        return Err(Error::new(format!(
            "the application declared no `{ITEM_PREFIX}*` regions at all, so no individual \
             ribbon command can be located from outside the process and there is nothing to \
             click. This is not a defect in Markup: it is a build whose ribbon publishes rects \
             for its group captions, its tabs and its mode segments but not for its command \
             controls. The publisher is `egui_shell::ribbon::report::band_item`, called from \
             `band::render_command`. Regions declared under `ribbon.`: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon."))
        )));
    }
    report.note(format!(
        "{} command control(s) declared their rects on the Markup tab",
        items.len()
    ));

    let subject = declared(&trace, ui_rect, SUBJECT).ok_or_else(|| {
        Error::new(format!(
            "the Markup tab is active and its controls publish their rects, but none of them is \
             `{SUBJECT}`. Controls declared: {}.",
            list(&items)
        ))
    })?;
    let sibling = declared(&trace, ui_rect, SIBLING).ok_or_else(|| {
        Error::new(format!(
            "`{SUBJECT}` is declared but `{SIBLING}` is not, and this check needs both: the \
             pressed state is asserted as a difference between the control that was clicked and \
             a sibling in the same group that was not, measured in one capture. Without the \
             sibling the only available evidence is before-and-after, which any repaint of the \
             whole band would satisfy. Controls declared: {}.",
            list(&items)
        ))
    })?;
    let park = declared(&trace, ui_rect, PARK).ok_or_else(|| {
        Error::new(format!(
            "the Shapes group declared no `{PARK}` region, and this check parks the pointer \
             there before each capture so that a hover is not mistaken for a selection. \
             Captions declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.group.markup."))
        ))
    })?;
    if !subject.is_substantial() || !sibling.is_substantial() {
        return Err(Error::new(format!(
            "a control was declared at no usable size — `{SUBJECT}` at {subject:?}, `{SIBLING}` \
             at {sibling:?}. A click aimed at a degenerate rectangle proves nothing, so this is \
             reported rather than driven."
        )));
    }
    report.note(format!(
        "{SUBJECT} at {subject:?}, {SIBLING} at {sibling:?}, parking on {PARK} at {park:?}"
    ));

    // A tool armed BEFORE the click would make the click a *disarm*:
    // `arm_markup` toggles on the same kind. Nothing persists it across a
    // launch today — the tool lives in `egui::Memory`'s temporary data — but
    // "today" is not an assertion, and a check that mis-read a disarm as a
    // failure would blame working code.
    if let Some(line) = trace.last(ARM_EVENT) {
        return Err(Error::new(format!(
            "a tool was already armed before this check clicked anything: `{}`. \
             `canvas::tool::arm_markup` TOGGLES on the same kind, so a click on an \
             already-armed Rectangle correctly retires it — and this check would then be \
             asserting that a working disarm is a broken arm.",
            line.raw
        )));
    }

    // --- step 4: the BEFORE capture ---------------------------------------
    driver.move_to(frame.declared_center(park))?;
    session.settle(8);
    let before_path = ctx.out("markup_rectangle.before.png");
    let before = crate::capture::window_to_png(&session, &before_path)?;
    report.artifact(before_path);

    let subject_before = fill_of(&before, &frame, subject).ok_or_else(|| {
        Error::new(format!(
            "`{SUBJECT}` was declared at {subject:?}, which resolves to no pixels of the \
             captured client area — the application declared a control outside its own window."
        ))
    })?;
    let sibling_before = fill_of(&before, &frame, sibling).ok_or_else(|| {
        Error::new(format!(
            "`{SIBLING}` was declared at {sibling:?}, which resolves to no pixels of the capture."
        ))
    })?;
    let before_gap = delta(subject_before, sibling_before);
    report.note(format!(
        "before the click: Rectangle fills {subject_before}, Ellipse fills {sibling_before} \
         (max channel gap {before_gap})"
    ));
    if before_gap >= MIN_PRESSED_DELTA {
        return Err(Error::new(format!(
            "Rectangle and Ellipse already differ by {before_gap} (>= {MIN_PRESSED_DELTA}) \
             BEFORE anything was clicked, so a difference afterwards could not be attributed to \
             the click. Two controls in one group, neither selected, should be drawn in the \
             same fill. Look at the before capture."
        )));
    }

    // --- step 5: click Rectangle ------------------------------------------
    driver.click_at(frame.declared_center(subject))?;
    session.settle(16);

    // --- step 6: the AFTER capture ----------------------------------------
    //
    // Park FIRST. The pointer is sitting on Rectangle after the click, so a
    // capture taken now would show it hovered as well as selected, and the
    // difference this check measures would be partly a hover — which the
    // sibling, never hovered, would not share. That would make P2 pass for
    // the wrong reason, which is the worst outcome available: a green check
    // measuring the wrong thing.
    driver.move_to(frame.declared_center(park))?;
    session.settle(8);
    let after_path = ctx.out("markup_rectangle.after.png");
    let after = crate::capture::window_to_png(&session, &after_path)?;
    report.artifact(after_path);

    // --- the TRACE assertions ---------------------------------------------
    let shell = shell_trace(&session)?;
    let invoked = shell
        .events(INVOKE_EVENT)
        .any(|l| l.get("id") == Some(SUBJECT_ID));
    if !invoked {
        let seen: Vec<&str> = shell
            .events(INVOKE_EVENT)
            .filter_map(|l| l.get("id"))
            .collect();
        return Ok(Some(format!(
            "TRACE: the click did not reach the control. The harness clicked the centre of \
             {subject:?} — the rectangle the application itself published for `{SUBJECT}` on \
             the frame it drew it — and the shell traced no `{INVOKE_EVENT} id={SUBJECT_ID}`. \
             Commands the shell reported as invoked this run: {}. Two readings: the control was \
             disabled, which for a `doc.pages`-gated command means the fixture opened no pages; \
             or the band reported a rectangle it did not draw the control in. Both are \
             findings.",
            list_str(&seen)
        )));
    }
    report.note(format!(
        "the shell traced `{INVOKE_EVENT} id={SUBJECT_ID}`, so the click reached the control"
    ));

    let trace = session.trace()?;
    let armed = trace
        .events(ARM_EVENT)
        .any(|l| l.get("tool") == Some(ARM_VALUE));
    if !armed {
        let unimplemented = trace
            .events(UNIMPLEMENTED_EVENT)
            .any(|l| l.get("id") == Some(SUBJECT_ID));
        let tools: Vec<&str> = trace
            .events(ARM_EVENT)
            .filter_map(|l| l.get("tool"))
            .collect();
        return Ok(Some(format!(
            "TRACE: the click reached the control and armed nothing. The shell traced \
             `{INVOKE_EVENT} id={SUBJECT_ID}`, so the command was invoked and its token was \
             handed to the application — and the application traced no `{ARM_EVENT} \
             tool={ARM_VALUE}`. `canvas::tool::arm_markup` traces unconditionally the moment it \
             is called, so its silence means it was never called. Tools reported this run: {}. \
             {} Look at `app/dispatch.rs`'s guard arm — the one matching on \
             `shell::commands::markup_for_command(id).is_some()` — and at \
             `markup_for_command` itself, which is the single binding between a command id and \
             a `MarkupKind`. Every link in this chain has a passing unit test; what has no test \
             is the join.",
            list_str(&tools),
            if unimplemented {
                format!(
                    "The application DID trace `{UNIMPLEMENTED_EVENT} id={SUBJECT_ID}`, which \
                     is `dispatch_command`'s fall-through arm: the command arrived at dispatch \
                     and dispatch had no arm for it."
                )
            } else {
                format!(
                    "No `{UNIMPLEMENTED_EVENT} id={SUBJECT_ID}` either, so the command did not \
                     reach `dispatch_command`'s fall-through — check `dispatch_token`'s \
                     token-to-id lookup."
                )
            }
        )));
    }
    report.note(format!(
        "the application traced `{ARM_EVENT} tool={ARM_VALUE}`, so the canvas tool is armed"
    ));

    // --- the PIXEL assertions ---------------------------------------------
    //
    // Reached only once the trace has established that the tool really is
    // armed, so anything that fails from here on is link 4 — the ribbon not
    // showing a state the application is genuinely in — and the failure text
    // can say so without hedging.
    let subject_after = fill_of(&after, &frame, subject).ok_or_else(|| {
        Error::new(format!(
            "`{SUBJECT}` resolved to no pixels in the after capture, though it did in the \
             before capture — the control moved off the client area across the click."
        ))
    })?;
    let sibling_after = fill_of(&after, &frame, sibling).ok_or_else(|| {
        Error::new(format!(
            "`{SIBLING}` resolved to no pixels in the after capture, though it did in the \
             before capture."
        ))
    })?;

    let p1 = delta(subject_after, subject_before);
    let p2 = delta(subject_after, sibling_after);
    let p3 = delta(sibling_after, sibling_before);
    report.note(format!(
        "after the click: Rectangle fills {subject_after}, Ellipse fills {sibling_after}"
    ));
    report.note(format!(
        "P1 Rectangle across the click: {p1}; P2 Rectangle vs Ellipse in one capture: {p2}; \
         P3 Ellipse across the click: {p3}; threshold {MIN_PRESSED_DELTA}"
    ));

    if p2 < MIN_PRESSED_DELTA {
        return Ok(Some(format!(
            "PIXELS: the tool is armed and the control does not look it. In the SAME capture, \
             Rectangle's fill is {subject_after} and Ellipse's is {sibling_after} — a maximum \
             channel difference of {p2}, under the {MIN_PRESSED_DELTA} floor — so the control \
             that was clicked is drawn exactly like the one that was not. The trace already \
             proved `{ARM_EVENT} tool={ARM_VALUE}`, which is why this is a rendering finding \
             and not a dispatch one: look at `app/conditions.rs` publishing \
             `selected:{SUBJECT_ID}` and at `egui_shell::ribbon::band::render_command` reading \
             it. THIS IS THE ASSERTION A TRACE LINE CANNOT MAKE — a build whose ribbon never \
             shows a pressed state emits an identical trace, and that is the shape of the icon \
             painter that shipped drawing nothing."
        )));
    }
    if p1 < MIN_PRESSED_DELTA {
        return Ok(Some(format!(
            "PIXELS: Rectangle differs from Ellipse ({p2}) but did not change across the click \
             ({p1} < {MIN_PRESSED_DELTA}), so it looked that way already and the click is not \
             what made it. Either the before capture was taken after the control was already \
             pressed, or the two controls are drawn differently for some reason that has \
             nothing to do with selection."
        )));
    }
    if p3 >= MIN_PRESSED_DELTA {
        return Ok(Some(format!(
            "PIXELS: Ellipse ALSO changed across the click ({p3} >= {MIN_PRESSED_DELTA}), so \
             what was measured is the whole band repainting rather than one control being \
             pressed. A pressed state that spreads to its neighbours is not a pressed state; \
             compare the two captures."
        )));
    }

    report.note(
        "pressed rendering confirmed from the pixels: the clicked control changed, its \
         un-clicked sibling did not, and the two differ in one capture",
    );
    Ok(None)
}

/// Read the same captured stderr a second time, under the **shell's** line
/// prefix.
///
/// One file, two vocabularies. `Session::trace` parses with the profile's
/// prefix (`pdfcer-diag`); everything `egui-shell` writes carries its own, and
/// lands in [`Trace::other`] on that parse. Re-parsing is cheap next to a
/// click and keeps both streams honest — a line is attributed to whichever
/// crate actually wrote it, which is the whole point of the prefix.
fn shell_trace(session: &Session) -> Result<Trace> {
    Trace::read(session.trace_path(), SHELL_TRACE_PREFIX)
}

/// Render a list of names for a reason string, or say plainly that there were
/// none. `"none"` rather than `""`, because an empty list printed as nothing
/// reads as a formatting bug and hides the fact that was being reported.
fn list(names: &[String]) -> String {
    if names.is_empty() {
        "none".to_owned()
    } else {
        names.join(", ")
    }
}

/// [`list`] for borrowed strings.
fn list_str(names: &[&str]) -> String {
    if names.is_empty() {
        "none".to_owned()
    } else {
        names.join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geom::Pt;

    /// The names this check greps for are the ones `egui-shell` builds.
    ///
    /// Pinned here as well as in `egui-shell`'s own
    /// `the_reported_names_are_a_stability_contract`, because the two crates
    /// are joined by a **string** and nothing else: this crate drives a
    /// process, so it cannot import the constant, and a rename would leave
    /// both sides compiling while every assertion here quietly stopped
    /// matching. A check that matches nothing passes vacuously, and that is
    /// the failure this pair of tests exists to make impossible.
    #[test]
    fn the_selectors_match_the_shells_own_spelling() {
        assert_eq!(SUBJECT, format!("ribbon.item.{SUBJECT_ID}"));
        assert!(SUBJECT.starts_with(ITEM_PREFIX));
        assert!(SIBLING.starts_with(ITEM_PREFIX));
        assert_eq!(MODE_SEGMENT, "ribbon.mode.review");
        assert_eq!(TAB, "ribbon.tab.markup");
        assert_eq!(PARK, "ribbon.group.markup.shapes.caption");
        // The parking spot must not be one of the controls being measured,
        // or the pointer would be hovering the thing under test.
        assert!(!PARK.starts_with(ITEM_PREFIX));
        assert_ne!(SUBJECT, SIBLING);
    }

    /// Last wins, per name, and a name that was never declared is `None`.
    #[test]
    fn a_regions_last_declaration_is_the_one_that_is_used() {
        let trace = Trace::parse(
            "pdfcer-diag start argv1=None\n\
             pdfcer-diag ui-rect name=ribbon.item.markup.rectangle rect=[[0.0 0.0] - [10.0 10.0]]\n\
             pdfcer-diag ui-rect name=ribbon.item.markup.ellipse rect=[[20.0 0.0] - [30.0 10.0]]\n\
             pdfcer-diag ui-rect name=ribbon.item.markup.rectangle rect=[[4.0 30.0] - [84.0 54.0]]",
            "pdfcer-diag",
        );
        assert_eq!(
            declared(&trace, "ui-rect", SUBJECT),
            Some(LRect::new(Pt::new(4.0, 30.0), Pt::new(84.0, 54.0))),
            "an early frame can carry a rect from before the layout settled"
        );
        assert_eq!(
            declared(&trace, "ui-rect", "ribbon.item.markup.arrow"),
            None
        );
        assert_eq!(
            declared_names(&trace, "ui-rect", ITEM_PREFIX),
            vec![SUBJECT.to_owned(), SIBLING.to_owned()],
            "each name once, in first-seen order"
        );
    }

    /// **The two channels are parsed out of one file without contaminating
    /// each other.**
    ///
    /// The shell's lines land in `other` under the application's prefix and
    /// vice versa. If a future prefix change made one a prefix of the other,
    /// this test is what says so — and the symptom otherwise would be a check
    /// that reads a `ribbon-command-invoked` that is not there, or misses one
    /// that is.
    #[test]
    fn the_application_and_shell_streams_do_not_contaminate_each_other() {
        let text = "pdfcer-diag start argv1=None\n\
                    egui-shell-diag ribbon-command-invoked id=markup.rectangle handler=500\n\
                    pdfcer-diag markup-tool tool=Markup(Rectangle)\n";
        let app = Trace::parse(text, "pdfcer-diag");
        let shell = Trace::parse(text, SHELL_TRACE_PREFIX);

        assert!(app.started("start"));
        assert!(
            app.events(INVOKE_EVENT).next().is_none(),
            "the shell's line must not be read as the application's"
        );
        assert_eq!(
            app.last(ARM_EVENT).and_then(|l| l.get("tool")),
            Some(ARM_VALUE),
            "the `Debug` spelling of CanvasTool::Markup(MarkupKind::Rectangle)"
        );
        assert!(
            shell
                .events(INVOKE_EVENT)
                .any(|l| l.get("id") == Some(SUBJECT_ID))
        );
        assert!(
            shell.events(ARM_EVENT).next().is_none(),
            "the application's line must not be read as the shell's"
        );
    }

    /// **The threshold separates pressed from unpressed under BOTH palettes
    /// this build might paint with — and a contrast ratio separates neither.**
    ///
    /// Two pairs, because the running binary does not use the palette the
    /// shell's theme defines: `#E5E5E5`/`#90D1FF` is what was measured from a
    /// real capture (`egui`'s stock light values, because nothing calls
    /// `Theme::apply`), and `#E8E8EA`/`#C1CFE6` is what `egui-shell`'s `quiet`
    /// preset would produce if it were installed. See
    /// [`MIN_PRESSED_DELTA`]'s documentation for the derivation of each.
    ///
    /// The second assertion in each pair is the one that matters: `AA_LARGE`
    /// is 3.0 and these fills are 1.3:1 and 1.5:1 apart, so a check written
    /// against the harness's usual legibility oracle would report "no
    /// difference" about a control that is visibly blue.
    #[test]
    fn the_threshold_separates_pressed_from_unpressed_under_both_palettes() {
        // (unpressed, pressed, expected gap, what it is)
        let pairs = [
            (
                Rgb::new(229, 229, 229),
                Rgb::new(144, 209, 255),
                85_u16,
                "egui's stock light palette — MEASURED from a real capture",
            ),
            (
                Rgb::new(232, 232, 234),
                Rgb::new(193, 207, 230),
                39_u16,
                "egui-shell's `quiet` preset, composited — computed",
            ),
        ];

        for (unpressed, pressed, expected, what) in pairs {
            assert_eq!(delta(unpressed, unpressed), 0, "identical fills ({what})");
            assert_eq!(delta(unpressed, pressed), expected, "{what}");
            assert!(
                expected > MIN_PRESSED_DELTA * 3,
                "the threshold must sit well below the difference produced by {what}"
            );
            let ratio = crate::pixels::contrast_ratio(unpressed, pressed);
            assert!(
                ratio < crate::pixels::AA_LARGE,
                "a contrast threshold would call these two fills the same colour \
                 ({ratio:.2}:1) under {what}, which is why this check measures a channel \
                 difference instead"
            );
        }
    }

    /// The difference is symmetric and takes the largest channel, so a shift
    /// confined to one channel still registers.
    #[test]
    fn the_difference_is_the_largest_channel_and_is_symmetric() {
        let a = Rgb::new(200, 100, 50);
        let b = Rgb::new(190, 100, 90);
        assert_eq!(delta(a, b), 40);
        assert_eq!(delta(b, a), 40);
    }
}
