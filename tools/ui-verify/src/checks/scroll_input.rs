//! `scrolling_far_keeps_the_canvas_its_pointer_input` — the experiment that
//! decides whether `O23` is a feature problem or a defect the operator already
//! meets.
//!
//! # Why this exists
//!
//! `OPERATOR_REQUESTS.md` O23's pasteboard was bisected on 2026-08-21 down to
//! two literals:
//!
//! ```text
//! .scroll_offset(vec2(100, 100))   → canvas keeps its pointer input
//! .scroll_offset(vec2(484, 492))   → canvas receives NOTHING
//! ```
//!
//! No pasteboard arithmetic is involved in either. The page's rect settles at
//! one stable value wholly inside the viewport, a click computed from it lands
//! comfortably inside both, and no `canvas-pointer` event is ever emitted.
//!
//! That leaves one question, and everything else waits on it:
//!
//! > **Does a scroll offset the OPERATOR reaches — with the wheel — break input
//! > the same way?**
//!
//! | if | then |
//! |---|---|
//! | input dies | this is a **pre-existing defect in today's shell**, met whenever he scrolls a long way down a drawing. It outranks O23 entirely, and O23 has been getting the blame |
//! | input survives | the difference is that the offset was **forced on the frame the content was first laid out**, and the fix is to force it one frame later |
//!
//! # ★★ Why the assertion is `canvas-pointer` events and not a selection
//!
//! Because the symptom is the *absence of input*, not a bad hit test. Asserting
//! a selection would need an object under a point that survives an arbitrary
//! scroll — a fact about the fixture — and would fail for reasons that have
//! nothing to do with the question. `canvas-pointer` is emitted whenever the
//! pointer is over the page, needs no object, and is exactly the line that went
//! to zero.
//!
//! # ★ The control comes first
//!
//! It moves the pointer and counts events **before** scrolling. Without that,
//! "no events after the scroll" is indistinguishable from "this build never
//! emits them", "the window was not focused", and "the pointer never reached
//! the canvas" — three things that look identical in a trace and none of which
//! is the defect.
//!
//! Read mode, deliberately: it is the default, it defaults to continuous
//! scrolling, and the question has nothing to do with editing. One fewer click
//! is one fewer thing that can go wrong before the measurement.

use crate::checks::driving;
use crate::checks::{Check, CheckContext, CheckReport};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};

/// The canvas viewport's declared region — what the pointer is aimed into.
const CANVAS_REGION: &str = "canvas-viewport";

/// The line the canvas emits whenever the pointer is over a page.
const POINTER_EVENT: &str = "canvas-pointer";

/// The canvas's own state line, read for the scroll offset it reports.
const CANVAS_EVENT: &str = "canvas";

/// How far to wheel. Large enough to pass the offset that reproduced the
/// failure with a forced value, on a 36-page continuous strip.
const NOTCHES: i32 = -40;

/// The offset below which this check has not actually tested anything.
///
/// ★ Reported as SKIPPED rather than passed. A wheel that moved the view by 30
/// points has not reached the condition under test, and calling that a pass
/// would certify the opposite of what was measured.
const OFFSET_FLOOR: f32 = 300.0;

/// See the module documentation.
pub struct ScrollingFarKeepsTheCanvasItsPointerInput;

impl Check for ScrollingFarKeepsTheCanvasItsPointerInput {
    fn name(&self) -> &'static str {
        "scrolling_far_keeps_the_canvas_its_pointer_input"
    }

    fn defect(&self) -> &'static str {
        "after scrolling a long way down a document the canvas stops receiving pointer input \
         altogether — the page is still drawn in the right place and nothing responds to the \
         mouse"
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
    let vocab = &ctx.profile.vocab;
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx.pdf.clone().ok_or_else(|| {
        Error::new("no --pdf. This check needs a document long enough to scroll through.")
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check moves the pointer and turns the wheel. \
             Reported as SKIPPED rather than passed.",
        ));
    }
    let ui_rect = vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event.",
            ctx.profile.name
        ))
    })?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("scroll-input.trace.txt"));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);
    let driver = Driver::new(session.window());

    // Where the canvas is. Read once — the viewport does not move in this
    // check, since no mode is changed and no panel is opened.
    let trace = session.trace()?;
    let canvas = driving::declared(&trace, ui_rect, CANVAS_REGION).ok_or_else(|| {
        Error::new(format!(
            "the application never declared `{CANVAS_REGION}`, so there is nowhere to aim. Either \
             no document is open or the region was renamed."
        ))
    })?;
    let frame = session.frame()?;
    let centre = frame.declared_center(canvas);

    // --- the control ---------------------------------------------------------
    driver.move_to(centre)?;
    session.settle(20);
    let before = session.trace()?.events(POINTER_EVENT).count();
    if before == 0 {
        return Err(Error::new(
            "the pointer produced no `canvas-pointer` events BEFORE any scrolling, so this build \
             never emits them, the window was not focused, or the pointer never reached the \
             canvas. Any of those makes the measurement after the scroll meaningless. SKIPPED \
             rather than reported as a defect.",
        ));
    }
    report.note(format!("before scrolling: {before} pointer event(s)"));

    // --- scroll a long way ---------------------------------------------------
    driver.scroll_at(centre, NOTCHES)?;
    session.settle(30);

    let trace = session.trace()?;
    let offset = trace
        .events(CANVAS_EVENT)
        .last()
        .and_then(|l| l.get_vec2("off"))
        .map(|o| o.y.abs())
        .unwrap_or(0.0);
    if offset < OFFSET_FLOOR {
        return Err(Error::new(format!(
            "the wheel moved the view to an offset of only {offset:.0} pt, below the {OFFSET_FLOOR:.0} \
             pt this check needs in order to be testing anything. The fixture may be too short, \
             or the wheel may not be reaching the canvas. SKIPPED rather than passed."
        )));
    }
    report.note(format!("scrolled to an offset of {offset:.0} pt"));

    // --- does the canvas still see the pointer? ------------------------------
    //
    // ★ Moved to a DIFFERENT point. `canvas-pointer` is emitted on movement, so
    // re-issuing the same position could legitimately produce nothing and would
    // read as the defect.
    let elsewhere = frame.declared_at(canvas, 0.35, 0.6);
    driver.move_to(elsewhere)?;
    session.settle(20);
    let after = session.trace()?.events(POINTER_EVENT).count();

    if after <= before {
        return Ok(Some(format!(
            "★★★ THE CANVAS STOPPED SEEING THE POINTER AFTER SCROLLING. {before} pointer \
             event(s) before the wheel and {after} after it, at a scroll offset of {offset:.0} pt. \
             The page is still drawn and its rect is still published — only the input is gone. \
             ★ This is the condition `OPERATOR_REQUESTS.md` O23 was blaming its pasteboard for: \
             it reproduces with NO pasteboard in the build, from an ordinary wheel scroll, which \
             makes it a defect the operator meets whenever they scroll a long way down a \
             drawing."
        )));
    }

    report.note(format!("after scrolling: {after} pointer event(s)"));
    Ok(None)
}
