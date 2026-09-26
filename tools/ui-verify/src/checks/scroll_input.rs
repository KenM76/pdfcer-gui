//! `scrolling_far_keeps_the_canvas_its_pointer_input` — the experiment that
//! decides whether `O23` is a feature problem or a defect the operator already
//! meets.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/scroll_input.md`.

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

/// The canvas's own line saying it has no page to show.
///
/// Read as a **stop condition**, not as a defect. It is what an honest
/// canvas says when the operator has scrolled into the pasteboard past the end
/// of a short document, and reading it as the failure under test is what made
/// this check file A3 against the application.
const UNAVAILABLE_EVENT: &str = "canvas-unavailable";

/// The page's own declared region — read to answer *is there still a sheet
/// under the pointer?* after every step of the wheel.
const PAGE_REGION: &str = "page";

/// How far to wheel **per step**.
///
/// Small enough that a short document is not carried from "the page fills the
/// viewport" to "the page is gone" in one movement, which is what a single
/// forty-notch turn did.
const NOTCHES_PER_STEP: i32 = -6;

/// How many steps to take at most.
///
/// `STEPS * NOTCHES_PER_STEP` is the old single turn plus a margin, so a long
/// document still reaches a deep offset; a short one stops early on its own,
/// at the last step that kept a page on screen.
const STEPS: usize = 10;

/// How much the scroll offset must **grow** for this check to be testing
/// anything.
///
/// A *gain*, not an absolute floor, and the difference is measured: on
/// `a1-titleblock.pdf` the canvas rests at `off=[484.0 592.3]` because the
/// sheet is centred in a pasteboard, so the old absolute floor of 300 pt was
/// satisfied **before the wheel was touched**. A check whose precondition is
/// already true at rest is not testing the wheel.
const OFFSET_GAIN: f32 = 200.0;

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
    // Read as a PRECONDITION and then not used: the aim comes from the page
    // (below), but a run in which the canvas itself was never declared has a
    // different and much simpler explanation than the one this check reports,
    // and saying so first costs nothing.
    let _canvas = driving::declared(&trace, ui_rect, CANVAS_REGION).ok_or_else(|| {
        Error::new(format!(
            "the application never declared `{CANVAS_REGION}`, so there is nowhere to aim. Either \
             no document is open or the region was renamed."
        ))
    })?;
    let frame = session.frame()?;
    // Aimed at the PAGE, not at the middle of the viewport. `canvas-pointer`
    // is emitted when the pointer is over a **page**, so a control taken over
    // the pasteboard would count zero and abort the run with the wrong reason.
    let page_rect = driving::declared(&trace, ui_rect, PAGE_REGION).ok_or_else(|| {
        Error::new(format!(
            "the application declared no `{PAGE_REGION}` region, so no sheet is on screen and \
             there is nothing to scroll past. SKIPPED rather than reported as a defect."
        ))
    })?;
    let centre = frame.declared_center(page_rect);

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

    // --- scroll a long way, and STOP while a page is still on screen ---------
    //
    let start = offset_now(&session)?;
    let mut offset = start;
    let mut page_now = page_rect;
    let mut steps_taken = 0usize;
    for step in 1..=STEPS {
        driver.scroll_at(centre, NOTCHES_PER_STEP)?;
        session.settle(14);
        let trace = session.trace()?;
        let Some(rect) = driving::declared(&trace, ui_rect, PAGE_REGION) else {
            report.note(format!(
                "★ step {step} scrolled the last page off the viewport — the canvas said \
                 `{}`, which is the honest answer for the pasteboard past the end of a short \
                 document and NOT the defect under test. Winding back one step.",
                trace
                    .events(UNAVAILABLE_EVENT)
                    .last()
                    .map_or_else(|| "no page region".to_owned(), |l| l.raw.clone())
            ));
            driver.scroll_at(centre, -NOTCHES_PER_STEP)?;
            session.settle(14);
            break;
        };
        page_now = rect;
        offset = offset_now(&session)?;
        steps_taken = step;
    }
    let gain = (offset - start).abs();
    if gain < OFFSET_GAIN {
        return Err(Error::new(format!(
            "the wheel moved the view by only {gain:.0} pt ({start:.0} → {offset:.0}) over \
             {steps_taken} step(s) before the document ran out, below the {OFFSET_GAIN:.0} pt \
             this check needs in order to be testing anything. This fixture is too short to \
             scroll a long way down while keeping a page under the pointer — use a document \
             with several pages in a continuous display. SKIPPED rather than passed."
        )));
    }
    report.note(format!(
        "scrolled from an offset of {start:.0} pt to {offset:.0} pt over {steps_taken} step(s), \
         with the page still drawn at {page_now:?}"
    ));

    // --- does the canvas still see the pointer? ------------------------------
    //
    // Moved to a DIFFERENT point. `canvas-pointer` is emitted on movement, so
    // re-issuing the same position could legitimately produce nothing and would
    // read as the defect.
    //
    //
    //
    // With a defect planted in `canvas::trace::pointer` that suppresses the
    // line past a scroll offset of 700 pt, this check **still passed**: it
    // compared a total taken before the wheel (1) against a total taken after
    // it (16), and every one of those sixteen had been emitted by the pointer
    // sitting under the wheel *while the offset was still small*. The scroll
    // itself manufactures the evidence that the scroll did no harm.
    //
    // ⇒ The comparison has to be against the count at the moment the deep
    // offset is reached, so that only lines produced by the final move can
    // satisfy it. The pre-wheel control above still earns its keep — it is what
    // separates "the canvas stopped seeing the pointer" from "this build never
    // emits the line" — but it is not the baseline.
    let settled = session.trace()?.events(POINTER_EVENT).count();
    let elsewhere = frame.declared_at(page_now, 0.35, 0.6);
    driver.move_to(elsewhere)?;
    session.settle(20);
    let after = session.trace()?.events(POINTER_EVENT).count();
    let fresh = after.saturating_sub(settled);
    report.note(format!(
        "{settled} pointer event(s) had accumulated by the time the scroll settled; the move \
         that follows must produce its own"
    ));

    if fresh == 0 {
        return Ok(Some(format!(
            "★★★ THE CANVAS STOPPED SEEING THE POINTER AFTER SCROLLING. The pointer produced \
             {before} event(s) before the wheel was touched and **none at all** from the move \
             made after it, at a scroll offset of {offset:.0} pt ({gain:.0} pt of travel). \
             {settled} event(s) had accumulated by the time the scroll settled and not one of \
             them counts here — only lines the final move produced do. The page is still drawn \
             — its region was read back as `{page_now:?}` after the last step, and the pointer \
             was moved to a point inside THAT rectangle — so only the input is gone. \
             ★ This is the condition `OPERATOR_REQUESTS.md` O23 was blaming its pasteboard for: \
             it reproduces with NO pasteboard in the build, from an ordinary wheel scroll, which \
             makes it a defect the operator meets whenever they scroll a long way down a \
             drawing."
        )));
    }

    report.note(format!(
        "after scrolling: {fresh} fresh pointer event(s) from the final move ({after} in total)"
    ));
    Ok(None)
}

/// The canvas's current vertical scroll offset, in points.
///
/// Read from the `canvas` line's `off=` rather than accumulated by the
/// harness. The application is the only thing that knows where its own scroll
/// area ended up after a wheel event the OS delivered asynchronously, and a
/// harness-side sum would be a second opinion that disagrees at exactly the
/// clamp this check exists to sit near.
fn offset_now(session: &Session) -> Result<f32> {
    Ok(session
        .trace()?
        .events(CANVAS_EVENT)
        .last()
        .and_then(|l| l.get_vec2("off"))
        .map_or(0.0, |o| o.y.abs()))
}
