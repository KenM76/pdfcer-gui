//! `the_line_weight_switch_reaches_the_resize` — **tick the switch, drag a
//! grip, and the border thickens with the shape.**
//!
//! # What this is for
//!
//! `OPERATOR_REQUESTS.md` **O51**:
//!
//! > *"if that was the resize question about scaling line weight, etc with
//! > resize it got the answer wrong. default should be what it said, but there
//! > should be an option that they do scale with resize. Inkscape has options
//! > for this and I want the same."*
//!
//! ## ★★★ What this check exists to catch, which is not "the switch is missing"
//!
//! The switch is a checkbox writing a `bool` into `egui::Memory`. Nothing about
//! that can plausibly fail. **The chain in front of the engine is what fails**,
//! and it has five links, three of which are pure wiring:
//!
//! | # | link | a unit test can see it? |
//! |---|---|---|
//! | 1 | the Select tool's option row is drawn at all | partly — `armed::options` returns early for every other tool |
//! | 2 | the checkbox writes the store | yes — `canvas::scaling`'s round trip |
//! | 3 | the value reaches `resizing::Frame` on the commit frame | **no** |
//! | 4 | it travels on the action rather than being re-read at apply time | **no** |
//! | 5 | it reaches `ResizeOptions` and the engine acts on it | yes — `to_options` |
//!
//! ★★ Link 3 is the one that was wrong for the life of the feature and in the
//! opposite direction: `annots::resize` **derived** `scale_stroke_width` from
//! whether the drag was proportional, so an operator's answer could not reach
//! the engine at all. A build that regressed to that would pass every unit test
//! in the chain, because each end of it is correct in isolation.
//!
//! ## ★★ The oracle is `stroke=` on the applied line, not a pixel
//!
//! `resize-annotation-applied … stroke=true|false` reports whether the engine
//! wrote a new `/BS /W`. A screenshot cannot separate *"the border thickened
//! because `/BS /W` changed"* from *"the border thickened because §12.5.5's
//! matrix scaled the drawn stroke"* — those are different outcomes with the
//! same picture, and only the first is the switch doing its job.
//!
//! ⇒ **The picture is the same in the case this check is about.** That is why
//! it reads the trace, and it is a fact about the format rather than a
//! limitation of the harness — the identical argument `markup_move` makes for
//! its `keys=` field.
//!
//! ## The sequence
//!
//! | # | step | oracle |
//! |---|---|---|
//! | A | Review mode, rectangle tool, drag a shape | `markup-commit` |
//! | B | click View ▸ Select to put the pen down, then click the shape | `ribbon-command-invoked id=view.tool_select`, `annot-select` |
//! | C | open the Tool panel, click the *Scale line weight* switch | `resize-modifiers stroke=true` |
//! | D | drag a corner grip **proportionally** | `resize-annotation-applied … stroke=true` |
//!
//! ## ★★★ IT WAS FLAKY, AND THE CURE WAS TO STOP TYPING — 2026-08-28
//!
//! This check passed its real assertion — `resize-annotation-applied …
//! stroke=true`, the switch reaching the engine — and also failed three runs
//! out of six **before getting that far**, at step B. The subject was never
//! implicated. What was unreliable was putting the markup pen down:
//!
//! | attempt | result |
//! |---|---|
//! | `V` (the `view.tool_select` chord) | never arrived — no invocation traced at all |
//! | one Escape | arrived sometimes |
//! | five Escapes, polling for the region | arrived on attempt 1, or not in five |
//!
//! ⇒ **A keystroke is not a reliable harness primitive while a dock panel this
//! check itself raised is open.** A chord is routed through whatever holds
//! keyboard focus, and this check opens the Tool panel by construction — it has
//! to, the switches live there. Note the shape of the failure: `V` produced
//! *no line anywhere*, so the check reported the Tool panel as drawing the
//! wrong block when the truth was that nothing had ever reached the
//! application.
//!
//! ★★ **The fix is a pointer, not a key**: step B clicks the View tab and then
//! `ribbon.item.view.tool_select`. Clicking a ribbon control is this harness's
//! most exercised primitive, it does not depend on focus, and it carries its
//! own oracle — the shell writes `ribbon-command-invoked id=view.tool_select`,
//! so *"the click did not land"* and *"the panel did not follow"* are now two
//! different messages instead of one ambiguous one.
//!
//! ★ And the arm it reaches is idempotent. `view.tool_select` calls
//! `canvas::tool::arm::select`, a plain write; the two neighbouring pointer
//! commands (`view.tool_hand`, `view.tool_text`) are **toggles** and would flip
//! on a second press. Choosing the one control on that row that cannot be wrong
//! about its own state is what makes this step deterministic rather than merely
//! more reliable.
//!
//! ★ Recorded here rather than left to be rediscovered: a check that fails
//! half the time is worse than one that fails always, because the failure gets
//! attributed to whatever changed most recently.
//!
//! ★★ **Not every keystroke in this suite is suspect, and the distinction
//! matters.** Typing into a field the check has just clicked is fine — focus is
//! where the keys are meant to go, which is what `dimension_groups` and
//! `bookmark_add` do. What is unsafe is a keystroke that has to be *routed to a
//! command* while a raised panel holds focus. And a chord that IS the subject —
//! `tool_row`'s bare `T`/`A`, `find_bar`'s `Ctrl+F`, `read_mode_chrome`'s
//! `Ctrl+H` — must stay a chord: converting it would delete the assertion.
//!
//! ★ Step D drags **diagonally by equal amounts** on purpose. A non-uniform
//! resize of a pdfcer-authored appearance is fine — it is rebuilt — but making
//! the drag uniform keeps this check about the switch rather than about the
//! distortion refusal, which is a different feature with a different sentence.

use crate::checks::driving::{
    SELECT_TOOL_ID, SHELL_DIAG_ENV, arm_select_from_ribbon, declared, declared_names, list,
};
use crate::checks::text_selection::aim;
use crate::checks::{Check, CheckContext};
use crate::coords::{DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Review mode, the rectangle tool, and the Tool panel — all three rung.
///
/// ★ `view.panel_tool` last, because the panel changes the dock's width and
/// therefore the canvas rect. Opening it **before** the shape is drawn would
/// mean every coordinate this check computes was taken against a layout that
/// then moved — the harness-coordinate hazard `D:/dev/rag/egui/` records.
///
/// ⇒ It is rung after the drawing steps for that reason and the mapping is
/// re-read afterwards.
const INVOKE: &str = "mode.review,markup.rectangle";
/// Opens the Tool panel, rung separately after the shape exists.
const PANEL_COMMAND: &str = "view.panel_tool";

/// The Tool panel's own region — the oracle for *"is it actually open?"*.
///
/// ★ Needed because [`PANEL_COMMAND`] is a **toggle**, not an opener, and the
/// dock layout persists across runs. See the normalisation at the top of the
/// body for the incident that made this necessary.
const PANEL_REGION: &str = "panel:tool";

/// The ribbon item that toggles the Tool panel, for putting it back when the
/// launch invoke turned it off.
const PANEL_ITEM_REGION: &str = "ribbon.item.view.panel_tool";
/// The line the canvas writes when a shape is authored.
const COMMIT_EVENT: &str = "markup-commit";
/// The line the canvas writes when a click selects an annotation.
const SELECT_EVENT: &str = "annot-select";
/// The switch's own published rect.
const SWITCH_REGION: &str = "tool.scale.stroke";
/// The line the panel writes when a switch changes.
const MODIFIERS_EVENT: &str = "resize-modifiers";
/// The line the apply arm writes when the engine has resized it.
const APPLIED_EVENT: &str = "resize-annotation-applied";
/// The page's own region, so a failure can say whether a sheet was drawn.
const PAGE_REGION: &str = "page";

/// Where the shape is drawn, as fractions of the page.
const SHAPE: ((f64, f64), (f64, f64)) = ((0.30, 0.30), (0.50, 0.45));
/// How many frames to wait for the Tool panel to swap its armed block for its
/// idle one, after the select tool has been armed from the ribbon.
///
/// ★★ **This is a poll, not a retry.** Nothing is pressed again inside the loop
/// — the invoke is already confirmed against the shell's own trace — so the
/// only question left is *which frame does the panel redraw in*, and a fixed
/// settle would be a guess at it. It was `DISARM_TRIES` while the step pressed
/// Escape five times, and the rename is the point: a bound on waiting and a
/// bound on pressing are different things, and conflating them is what let a
/// step that never worked look like a step that was merely slow.
///
/// Bounded so a build where the idle block genuinely never draws fails in
/// seconds rather than hanging.
const IDLE_TRIES: usize = 5;

/// How far the bottom-right grip travels, as a fraction of **the shape's own
/// size** — so the two scale factors come out equal.
///
/// ## ★★★ Three wrong answers before this one, and each was wrong differently
///
/// | attempt | value | why it was not uniform |
/// |---|---|---|
/// | 1 | `0.10` of the **page**, both axes | equal fractions of a 1584 × 1224 page are 158.4 pt and 122.4 pt |
/// | 2 | `90.0` **points**, both axes | equal distances grow a wide box less, in ratio, than a tall one |
/// | 3 | `0.25` of the **shape**, both axes | ✔ `sx = (w + 0.25w)/w = 1.25 = sy` |
///
/// ⇒ **A uniform scale is equal RATIOS, not equal distances**, and the space
/// the travel must be expressed in is the operand's, not the page's and not
/// the screen's. Both earlier values produced a working resize that the
/// check's own guard then declined to read — the guard was right each time,
/// and this constant was the thing that was wrong.
///
/// ★ The check asserts `uniform=true` off the trace rather than trusting this
/// arithmetic, which is what turned two silent mis-measurements into two
/// specific, self-describing skips.
const GRIP_TRAVEL_OF_SHAPE: f64 = 0.25;

/// See the module documentation.
pub struct TheLineWeightSwitchReachesTheResize;

impl Check for TheLineWeightSwitchReachesTheResize {
    fn name(&self) -> &'static str {
        "the_line_weight_switch_reaches_the_resize"
    }

    fn defect(&self) -> &'static str {
        "the operator's Scale line weight switch cannot reach the engine — `annots::resize` \
         DERIVES `scale_stroke_width` from whether the drag was proportional, so the switch is \
         drawn, stores its value, and is overridden on exactly the resizes where somebody was \
         most likely to have an opinion"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(skip) => report.skip(skip.to_string()),
        }
    }
}

#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check draws a shape, selects it, ticks a \
             checkbox in a dock panel and drags a grip. Every one is a real pointer gesture.",
        ));
    }
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx
        .pdf
        .clone()
        .ok_or_else(|| Error::new("no --pdf. This check needs a page to draw a shape on."))?;
    let page: PageGeometry = match ctx.page_size {
        Some((w, h)) => PageGeometry {
            width_pt: w,
            height_pt: h,
        },
        None => crate::fixture::page_geometry(&pdf).ok_or_else(|| {
            Error::new(format!(
                "could not read a page size from {}, and this check places its shape in page \
                 fractions. Pass --page-size.",
                pdf.display()
            ))
        })?,
    };
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("scale-switch.trace.txt"));
    spec.pdf = Some(pdf.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env.push((
        "PDFCER_DIAG_INVOKE".to_owned(),
        format!("{INVOKE},{PANEL_COMMAND}"),
    ));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!(
        "launched {} as pid {} with the Tool panel open",
        exe.display(),
        session.pid()
    ));
    session.settle(45);
    let driver = Driver::new(session.window());

    // ★★★ **NORMALISE: the panel command is a TOGGLE, and the layout persists.**
    //
    // `PDFCER_DIAG_INVOKE` presses `view.panel_tool` at launch on the assumption
    // that it OPENS the panel. It does not — it flips it — and the dock layout
    // is written to disk, so whether this check starts with the panel open
    // depends on what the previous check left behind.
    //
    // ★★ It became reliably wrong on 2026-08-31, and by a fix rather than a
    // break. `OPERATOR_REQUESTS.md` O80 wired `LayoutStore::flush` to an exit
    // hook that had never existed — before that, a layout change made in the
    // last 750 ms before a process exited was lost to the debounce, so the
    // panel state usually did NOT carry over and this check usually got away
    // with it. Making persistence work surfaced the latent order-dependence.
    //
    // ⇒ The lesson is the one `D:\dev\rag\egui` already records: **a driven
    // check that mutates persisted state must normalise at the start.** This
    // one asserted its precondition in its report text ("with the Tool panel
    // open") and never checked it.
    //
    // So: look, and press the ribbon item if the panel is not there. One
    // correction, not a loop — if a single press does not open it the panel is
    // broken, which is a different check's subject and must not be papered
    // over here.
    if declared(&session.trace()?, ui_rect, PANEL_REGION).is_none() {
        report.note(
            "★ the Tool panel was not open after the launch invoke — the toggle closed a \
             panel the persisted layout had already opened. Pressing the ribbon item to \
             put it back.",
        );
        let trace = session.trace()?;
        let Some(item) = declared(&trace, ui_rect, PANEL_ITEM_REGION) else {
            return Err(Error::new(format!(
                "the Tool panel is closed and `{PANEL_ITEM_REGION}` is not on the ribbon, so \
                 there is no route to reopen it. SKIPPED rather than failed: this check's \
                 subject is the scale switches, not the panel. Regions beginning \
                 `ribbon.item.view`: {}.",
                list(&declared_names(&trace, ui_rect, "ribbon.item.view"))
            )));
        };
        // ★ Resolved from the frame taken a moment ago rather than from one
        // cached earlier: the dock width changes when a panel opens, and a
        // stale coordinate is the harness hazard this project has written up
        // twice. Nothing has moved between the `declared` above and here.
        driver.click_at(session.frame()?.declared_center(item))?;
        session.settle(30);
        if declared(&session.trace()?, ui_rect, PANEL_REGION).is_none() {
            return Err(Error::new(format!(
                "pressing `{PANEL_ITEM_REGION}` did not put `{PANEL_REGION}` on screen, so the \
                 Tool panel will not open at all. That is a defect, and it is a different \
                 check's subject — this one cannot reach its own precondition. Trace: {}.",
                session.trace_path().display()
            )));
        }
    }

    if declared(&session.trace()?, ui_rect, PAGE_REGION).is_none() {
        return Err(Error::new(format!(
            "the application declared no `{PAGE_REGION}` region, so no sheet is on screen and \
             there is nothing to draw on. Regions beginning `page`: {}.",
            list(&declared_names(&session.trace()?, ui_rect, "page"))
        )));
    }

    // --- A: draw a rectangle ------------------------------------------------
    //
    // ★ The mapping is taken NOW, after the Tool panel has opened and settled.
    // A rect computed before the dock's width changed would aim at the page as
    // it used to sit — the harness-coordinate hazard, which this project has
    // already met once and written up.
    let corner = |f: (f64, f64)| DocPoint::new(0, f.0 * page.width_pt, f.1 * page.height_pt);
    let from = aim(ctx, &session, page, corner(SHAPE.0))?;
    let to = aim(ctx, &session, page, corner(SHAPE.1))?;
    driver.drag(from, to)?;
    session.settle(30);

    let trace = session.trace()?;
    if trace.events(COMMIT_EVENT).count() == 0 {
        return Ok(Some(format!(
            "THE RECTANGLE TOOL AUTHORED NOTHING: no `{COMMIT_EVENT}` line, so \
             `markup.rectangle` did not arm or the drag was not seen as one. Two steps BEFORE \
             the one under test. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note("★ a rectangle was authored");

    // --- B: put the pen down ------------------------------------------------
    //
    // ★★★ **A RIBBON CLICK, NOT A KEYSTROKE — 2026-08-28.**
    //
    // This step was `V`, then one Escape, then five polled Escapes, and it made
    // the check fail three runs out of six. The evidence, from six runs:
    //
    // | attempt | result |
    // |---|---|
    // | `V` (the `view.tool_select` chord) | never arrived — no invocation traced at all |
    // | one Escape | arrived sometimes |
    // | five Escapes, polling for the region | arrived on attempt 1, or not in five |
    //
    // ⇒ **A keystroke is not a reliable harness primitive while a dock panel
    // this check itself raised is open.** A chord is routed through whatever
    // holds keyboard focus, and this check opens the Tool panel *by
    // construction* — the switches live there. Escape is no better in kind: it
    // is the same channel, and polling it five times only converts a silent
    // wrong answer into a slow one.
    //
    // ★★ Clicking `ribbon.item.view.tool_select` does not depend on focus at
    // all. It is this harness's most exercised primitive, it has an oracle of
    // its own (the shell's `ribbon-command-invoked id=view.tool_select`), and
    // `app::dispatch`'s arm calls `canvas::tool::arm::select` — a plain write,
    // **not** a toggle like `view.tool_hand`/`view.tool_text` — so pressing it
    // when Select is already armed is a no-op rather than a flip. That is what
    // makes the step *deterministic* rather than merely more likely to work:
    // there is no state this click can be wrong about.
    //
    // ★ The View tab is clicked first because the control lives on View ▸
    // Navigate, and View is the one tab every mode is shown. Switching tabs
    // disturbs neither the dock nor the canvas, and every coordinate after this
    // point is re-derived through `aim`.
    //
    // ★★ **And no chord fallback here**, deliberately, where `markup_move` and
    // `measure_perimeter` keep one. Those two need the pen down and have been
    // passing on `V` for weeks with no panel of their own; this check raises the
    // Tool panel by construction, which is the exact condition under which `V`
    // was measured never to arrive. A fallback would put the flake back and
    // hide it behind a green run half the time.
    if !arm_select_from_ribbon(&session, &driver, ui_rect, report)? {
        return Err(Error::new(format!(
            "the select tool could not be armed from the ribbon — the note above says which \
             step of the route was missing — so the markup pen is still down and everything \
             after this would be measuring the wrong program. Reported as a SKIP rather than a \
             failure, on this suite's standing rule: a check that could not deliver a click has \
             learned nothing about the application. **Not retried with the `V` chord**: with \
             this panel raised, `V` was measured arriving zero times in six runs, and a silent \
             non-arrival is what made this check flaky in the first place. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note("★ the select tool was armed by clicking View ▸ Select, not by a keystroke");

    // ★★ The click is delivered and confirmed; the PANEL still redraws on its
    // own schedule, so the region is polled rather than read once. This is not
    // the old retry loop — nothing is pressed again — it is waiting for the
    // frame in which the idle block replaces the armed one. The first runs of
    // this check taught the distinction the hard way: the switch regions were
    // in the trace and `declared` answered `None`, because it correctly reads
    // the `ui-rect-gone` that follows a retired region.
    let mut switch = None;
    for attempt in 1..=IDLE_TRIES {
        if let Some(rect) = declared(&session.trace()?, ui_rect, SWITCH_REGION) {
            report.note(format!(
                "★ the pen went down and the Select options drew (frame poll {attempt})"
            ));
            switch = Some(rect);
            break;
        }
        session.settle(12);
    }
    let Some(switch) = switch else {
        let trace = session.trace()?;
        return Ok(Some(format!(
            "★★★ THE SWITCH IS NOT DRAWN: `{SELECT_TOOL_ID}` was invoked from the ribbon and no LIVE `{SWITCH_REGION}` region followed in {IDLE_TRIES} frame poll(s).
             **The state the feature shipped in for one afternoon** was that the switches were written into `panels::tool::armed::options`, and `super::body` calls the armed block only in its `else` arm — Select is this panel's IDLE state, so the branch was dead code that compiled and drew nothing.
 ★ The click is no longer a candidate cause: the shell traced the invoke. If the region APPEARS in the list below but is not live, that is the other failure — the panel drew it and retired it, meaning a tool re-armed after the select. Regions beginning `tool.`: {}. Trace: {}.",
            list(&declared_names(&trace, ui_rect, "tool.")),
            session.trace_path().display()
        )));
    };
    if !switch.is_substantial() {
        return Err(Error::new(format!(
            "`{SWITCH_REGION}` was declared at {switch:?}, which has no usable area to click — the dock is probably too narrow to lay the row out."
        )));
    }
    driver.click_at(session.frame()?.declared_center(switch))?;
    session.settle(20);

    let trace = session.trace()?;
    let Some(modifiers) = trace
        .events(MODIFIERS_EVENT)
        .filter(|l| l.get("stroke") == Some("true"))
        .last()
    else {
        return Ok(Some(format!(
            "★★ THE SWITCH DID NOT TAKE: a click at the centre of `{SWITCH_REGION}` produced no `{MODIFIERS_EVENT} stroke=true` line.
             That line is written only when the value CHANGES, so either the click missed the checkbox — its rect is published from the response's own rect — or the store did not take it. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("★★ the switch is on: `{}`", modifiers.raw));

    // --- C2: now select the shape, so its grips are drawn --------------------
    let centre = corner((
        f64::midpoint(SHAPE.0.0, SHAPE.1.0),
        f64::midpoint(SHAPE.0.1, SHAPE.1.1),
    ));
    driver.click_at(aim(ctx, &session, page, centre)?)?;
    session.settle(24);

    if session.trace()?.events(SELECT_EVENT).count() == 0 {
        return Ok(Some(format!(
            "THE SHAPE COULD NOT BE SELECTED: no `{SELECT_EVENT}` line after a click at its centre, so no grips are drawn and there is nothing to drag. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note("★ the shape was selected, so its grips are drawn");

    // --- D: drag the bottom-right grip, proportionally ----------------------
    let grip = aim(ctx, &session, page, corner(SHAPE.1))?;
    // ★ Each axis travels the same FRACTION OF THE SHAPE, so both scale
    // factors are `1 + GRIP_TRAVEL_OF_SHAPE` exactly. See that constant.
    let shape_w = (SHAPE.1.0 - SHAPE.0.0) * page.width_pt;
    let shape_h = (SHAPE.1.1 - SHAPE.0.1) * page.height_pt;
    let landing = aim(
        ctx,
        &session,
        page,
        DocPoint::new(
            0,
            SHAPE.1.0 * page.width_pt + shape_w * GRIP_TRAVEL_OF_SHAPE,
            SHAPE.1.1 * page.height_pt + shape_h * GRIP_TRAVEL_OF_SHAPE,
        ),
    )?;
    driver.drag(grip, landing)?;
    session.settle(40);

    let trace = session.trace()?;
    let Some(applied) = trace.events(APPLIED_EVENT).last() else {
        return Ok(Some(format!(
            "★★ THE GRIP DRAG REACHED NO RESIZE: no `{APPLIED_EVENT}` line.\n\
             Either the press missed the grip — it aimed at the shape's own bottom-right \
             corner, which is where the grip is centred — or `resize_annotation` refused. A \
             refusal traces `resize-annotation-refused`; look for that first, and note that a \
             pdfcer-authored appearance is REBUILT and should never be refused. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("★★★ the engine resized it: `{}`", applied.raw));

    // --- the oracle ---------------------------------------------------------
    if applied.get("stroke") != Some("true") {
        return Ok(Some(format!(
            "★★★ THE SWITCH DID NOT REACH THE ENGINE: `{}` reports stroke=false, and \
             `{MODIFIERS_EVENT} stroke=true` was recorded before the drag.\n\
             **This is the state the feature was in until 2026-08-28**, in the opposite \
             direction: `annots::resize` DERIVED `scale_stroke_width` from whether the drag was \
             proportional rather than from the operator's answer. Check that \
             `resizing::Frame::modifiers` is read on the commit frame, that it travels on \
             `Action::Annot(Resize)` rather than being re-read at apply time, and that \
             `Modifiers::to_options` is what builds the request. Trace: {}.",
            applied.raw,
            session.trace_path().display()
        )));
    }
    if applied.get("uniform") != Some("true") {
        return Err(Error::new(format!(
            "the drag came out NON-uniform (`{}`), and this check drags equal amounts in both \
             axes on purpose so that it is about the switch rather than about the distortion \
             refusal. Reported as SKIPPED rather than failed: the assertion above passed, and a \
             non-uniform drag means the harness's two axes disagree — probably a page whose \
             aspect ratio makes equal page-fraction travel unequal in points.",
            applied.raw
        )));
    }
    report.note("★ the resize was uniform and the engine wrote a new border width");
    Ok(None)
}
