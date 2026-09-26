//! `the_line_weight_switch_reaches_the_resize` — **tick the switch, drag a
//! grip, and the border thickens with the shape.**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/scale_switch.md`.

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

/// Review mode and the rectangle tool.
const INVOKE: &str = "mode.review,markup.rectangle";

/// The Properties panel's body compartment, as the DOCK reports it — the oracle
/// for *"can the operator see the switches' panel?"*.
const PANEL_REGION: &str = "dock.body.file.properties";

/// The Properties panel's dock tab header, for raising it from behind a sibling.
const PANEL_TAB_REGION: &str = "dock.tab.file.properties";
/// The line the canvas writes when a shape is authored.
const COMMIT_EVENT: &str = "markup-commit";
/// The line the canvas writes when a click selects an annotation.
const SELECT_EVENT: &str = "annot-select";
/// The switch's own published rect, in its new home.
const SWITCH_REGION: &str = "properties.tool.scale.stroke";
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
const IDLE_TRIES: usize = 5;

/// How far the bottom-right grip travels, as a fraction of **the shape's own
/// size** — so the two scale factors come out equal.
const GRIP_TRAVEL_OF_SHAPE: f64 = 0.25;

/// How far the two scale factors may disagree and still count as one uniform
/// drag.
const SCALE_AXIS_TOLERANCE: f64 = 0.025;

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
            Err(why) => report.from_error(&why),
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
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), INVOKE.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!(
        "launched {} as pid {}; the switches live in Properties since O123",
        exe.display(),
        session.pid()
    ));
    session.settle(45);
    let driver = Driver::new(session.window());

    // **NORMALISE: the panel this check reads may be behind a sibling tab.**
    //
    // Since `OPERATOR_REQUESTS.md` O123 the three resize switches live in the
    // Properties panel, and Review mounts Properties in a stack it shares with
    // Comments, Fill form and Dimension groups. A tabbed stack draws only its
    // active tab, so the switches may be perfectly correct and simply not the
    // tab on screen.
    //
    //
    // ⇒ The rule that came out of it is unchanged and is why this block still
    // exists: **a driven check that depends on persisted state must normalise at
    // the start.** What has changed is the gesture — a TAB HEADER, never a ribbon
    // toggle, because a toggle would unmount a panel that is already mounted and
    // the check would then report absent switches about a panel it closed itself.
    if declared(&session.trace()?, ui_rect, PANEL_REGION).is_none() {
        report.note(
            "★ the Properties panel is mounted but not the active tab. Raising it by its \
             dock header.",
        );
        let trace = session.trace()?;
        let Some(tab) = declared(&trace, ui_rect, PANEL_TAB_REGION) else {
            return Err(Error::new(format!(
                "the Properties panel is not mounted at all — neither `{PANEL_REGION}` nor \
                 `{PANEL_TAB_REGION}` is on screen — so the resize switches, which live in \
                 it since O123, cannot be read. SKIPPED rather than failed: this check's \
                 subject is the switches, not the dock. Regions beginning `dock.tab.`: {}.",
                list(&declared_names(&trace, ui_rect, "dock.tab."))
            )));
        };
        // Resolved from the frame taken a moment ago rather than from one
        // cached earlier: the dock width changes when a panel opens, and a
        // stale coordinate is the harness hazard this project has written up
        // twice. Nothing has moved between the `declared` above and here.
        driver.click_at(session.frame()?.declared_center(tab))?;
        session.settle(30);
        if declared(&session.trace()?, ui_rect, PANEL_REGION).is_none() {
            return Err(Error::new(format!(
                "clicking `{PANEL_TAB_REGION}` did not put `{PANEL_REGION}` on screen, so the \
                 Properties panel will not raise at all. That is a defect, and it is a \
                 different check's subject — this one cannot reach its own precondition. \
                 Trace: {}.",
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
    // Clicking `ribbon.item.view.tool_select` does not depend on focus at
    // all. It is this harness's most exercised primitive, it has an oracle of
    // its own (the shell's `ribbon-command-invoked id=view.tool_select`), and
    // `app::dispatch`'s arm calls `canvas::tool::arm::select` — a plain write,
    // **not** a toggle like `view.tool_hand`/`view.tool_text` — so pressing it
    // when Select is already armed is a no-op rather than a flip. That is what
    // makes the step *deterministic* rather than merely more likely to work:
    // there is no state this click can be wrong about.
    //
    // The View tab is clicked first because the control lives on View ▸
    // Navigate, and View is the one tab every mode is shown. Switching tabs
    // disturbs neither the dock nor the canvas, and every coordinate after this
    // point is re-derived through `aim`.
    //
    // **And no chord fallback here**, deliberately, where `markup_move` and
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

    // The click is delivered and confirmed; the PANEL still redraws on its
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
    // Each axis travels the same FRACTION OF THE SHAPE, so both scale
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
    //
    // The travel is expressed as a fraction of the shape, so both scale factors
    // are `1 + GRIP_TRAVEL_OF_SHAPE` **in arithmetic**. What the application
    // sees is not that: a pointer is delivered at integer screen pixels, and the
    // shape is 87.8 px wide and 85.2 px tall at fit zoom on this fixture, so a
    // 25 % travel is 21.9 px on one axis and 21.3 on the other — 22 and 21 once
    // the OS has rounded them. Measured: `sx=1.2508 sy=1.2449 uniform=false`.
    //
    // ⇒ **No harness can deliver an exactly uniform drag on a shape whose two
    // screen extents differ**, so `uniform=true` was a condition this check
    // could not satisfy. It went unnoticed because it was **unreachable**: the
    // grip drag had never once reached the engine (the grips of an annotation
    // were live only where they overlapped its own box, `canvas::pressing`),
    // so this guard had never run in the life of the feature.
    //
    // The subject is unaffected and is asserted above: `stroke=true` means the
    // operator's switch reached the engine. What this guard is for is the
    // *other* reading of a failure there — that the drag was so lopsided the
    // engine treated it as a distortion — and one pixel of rounding is not that.
    let sx: f64 = applied
        .get("sx")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.0);
    let sy: f64 = applied
        .get("sy")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.0);
    if (sx - sy).abs() > SCALE_AXIS_TOLERANCE {
        return Err(Error::new(format!(
            "the drag came out LOPSIDED (`{}`): sx={sx:.4} against sy={sy:.4}, a disagreement of \
             {:.4} where one pixel of pointer quantisation accounts for at most \
             {SCALE_AXIS_TOLERANCE}. This check drags equal FRACTIONS OF THE SHAPE in both axes \
             on purpose, so that it is about the switch rather than about the distortion \
             refusal. Reported as SKIPPED rather than failed: the assertion above passed, and a \
             lopsided drag means the harness's two axes disagree by more than rounding.",
            applied.raw,
            (sx - sy).abs()
        )));
    }
    report.note(format!(
        "★ the resize was uniform to within pointer rounding (sx={sx:.4}, sy={sy:.4}; the shell \
         calls it uniform={}) and the engine wrote a new border width",
        applied.get("uniform").unwrap_or("?")
    ));
    Ok(None)
}
