//! `the_text_tool_types_on_one_click` and `the_points_tool_shows_points_on_one_click`
//! — **the operator's own two gestures**, driven.
//!
//! # What these are for
//!
//! On 2026-08-19 the operator reported the canvas as unusable, in four sentences
//! that are all the same complaint:
//!
//! > *"How do I select and edit end points on the canvas? How do I edit text
//! > when on the canvas? I get a box and the I cursor, but I can't type
//! > anything. How do I make new text when I click on the canvas and expect to
//! > edit there? Same problem as the previous. How do I get to see the end
//! > points of an object and select them to drag and move? This doesn't work
//! > either."*
//!
//! And then the diagnosis, which was correct and is the reason both of these
//! checks exist:
//!
//! > *"The selector should be predictable like other programs. It seems a lot of
//! > ideas are getting invented instead of just using the … most common method
//! > expected."*
//!
//! Both features **existed**. Reaching them was invented:
//!
//! | to do this | the ritual, before 2026-08-19 |
//! |---|---|
//! | type one character | enter Edit mode → click the Edit tab → click *Edit text* → click the run. **Four steps.** |
//! | move an end point | click the shape → double-click to descend to its subpath → double-click again to descend to a node — with **nothing drawn at any stage** saying a deeper rung existed |
//!
//! Neither ritual is discoverable and neither resembles any other program. The
//! fix was to make the **tool the rung**: press `T`, click, type; press `A`,
//! click, see the points. That is Illustrator, Inkscape, Figma, CorelDRAW and
//! Word, and it is what these two checks assert.
//!
//! # ★★ Why the assertion is "ONE click"
//!
//! Because the count is the feature. A check that armed the tool from the
//! ribbon, clicked, and asserted a caret would pass on the **old** build too —
//! the old build could do all of that, it just needed four steps to get there.
//! So each check performs exactly one press of one key and exactly one click,
//! and asserts the outcome. Anything that needs a second click fails.
//!
//! ★ And the key is pressed as a **bare letter through the OS**, not as a
//! command dispatched by name. `V`/`A`/`T`/`H` being bare is the whole
//! convention being adopted, and a bare letter is the one chord shape that can
//! be broken by a stray focus — `canvas::keys` gates every keystroke on
//! `text_edit_focused()`, which is `DEFECTS.md` D1's guard, and D1 is this
//! project's canonical example of a keyboard rule that was right in the test
//! harness and wrong in the running window.

use crate::checks::driving::{SHELL_DIAG_ENV, click_mode_segment};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The mode whose canvas may edit content.
const MODE: &str = "edit";
/// `text-edit-caret kind=… page=… run=… len=…`.
const CARET_EVENT: &str = "text-edit-caret";
/// `text-edit-became-add reason=…` — the caret fell back to a fresh origin.
const BECAME_ADD_EVENT: &str = "text-edit-became-add";
/// `canvas-anchors total=… selected=… unselected_drawn=…`.
///
/// ★★★ **Written even when `total=0`, since 2026-08-29** — and the two checks
/// below are the reason. `overlay::draw_anchors` used to return before this line
/// when there was nothing to draw, so *"this object has no points"* and *"the
/// draw never ran"* were the same trace: nothing. Both checks have a `total == 0`
/// SKIP arm written for the first case, and neither could reach it.
const ANCHORS_EVENT: &str = "canvas-anchors";
/// `canvas-anchors-declined reason=…` — the enumeration stopped before it had a
/// count, and why.
///
/// ★★ The suffix is load-bearing twice over. It keeps `last(ANCHORS_EVENT)` from
/// ever returning one of these — which reads `total=`, and these carry no
/// `total` — and it is the convention `tools/gates/check-trace-names.py`
/// enforces against `vector_edit`'s funnel labels, for exactly that failure.
const DECLINED_EVENT: &str = "canvas-anchors-declined";
/// The decline reasons that are facts about **the aim or the fixture**, not
/// about the program, and therefore SKIP rather than FAIL.
///
/// ★★★ This list is the whole difference between the sweep of 2026-08-29 and an
/// honest one. Four checks read anchors on that run — these two plus
/// `multi_node` and `bezier_handle` — all four aimed at the same
/// `--doc-point 0,1140,62` on `SW41177.pdf`, all four saw no `canvas-anchors`
/// line, and they split two-and-two on what that meant: two SKIPPED saying *"the
/// point named a text run or an image"* and two FAILED naming specific lines of
/// `painting::draw_anchors`. The SKIPs were right — at that point
/// `the_text_tool_types_on_one_click` passes with `text-edit-caret run=426`, so
/// the aim **is** a text run, and a text run has no anchors — and the two
/// failures were reports about the aim wearing the clothes of reports about the
/// code.
///
/// ⇒ With the reason in the trace, no check has to guess. `not-entered` stays a
/// failure, because it means the click did not reach the rung and that is the
/// program's job; everything here means the driver pointed somewhere the feature
/// has nothing to say about.
const AIM_REASONS: [&str; 3] = ["nothing-selected", "leaf-in-form-xobject", "other-page"];

/// The decline line's `reason=`, if the enumeration declined on the last frame.
fn decline_reason(trace: &crate::trace::Trace) -> Option<String> {
    trace
        .last(DECLINED_EVENT)
        .and_then(|l| l.get("reason"))
        .map(str::to_owned)
}
/// The View ▸ Display ▸ Show points control.
const TOGGLE_REGION: &str = "ribbon.item.view.show_points";
/// `canvas-selection via=node-tool …`.
const NODE_CLICK_EVENT: &str = "canvas-selection";
/// `T`, as a Windows virtual key. Letters are their ASCII uppercase code point.
const VK_T: u16 = 0x54;
/// `A`.
const VK_A: u16 = 0x41;

/// Launch, open the fixture, enter Edit. The shared preamble of both checks.
fn open_in_edit(
    ctx: &CheckContext,
    report: &mut CheckReport,
    name: &str,
) -> Result<(Session, Driver, CanvasMapping, crate::coords::DocPoint)> {
    let vocab = &ctx.profile.vocab;
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx.pdf.clone().ok_or_else(|| {
        Error::new("no --pdf. This check needs a drawing with content on page 1.")
    })?;
    let target = ctx
        .target
        .ok_or_else(|| Error::new("no --doc-point. Pass PAGE,X,Y in PDF user space."))?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check presses a letter key and clicks the \
             canvas. Reported as SKIPPED rather than passed.",
        ));
    }
    let ui_rect = vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event.",
            ctx.profile.name
        ))
    })?;
    let page: PageGeometry = match ctx.page_size {
        Some((w, h)) => PageGeometry {
            width_pt: w,
            height_pt: h,
        },
        None => crate::fixture::page_geometry(&pdf).ok_or_else(|| {
            Error::new(format!(
                "cannot read a page size from {}. Pass --page-size WxH.",
                pdf.display()
            ))
        })?,
    };

    let mut spec = LaunchSpec::new(&exe, ctx.out(&format!("{name}.trace.txt")));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!("launched as pid {}", session.pid()));
    session.settle(40);
    let driver = Driver::new(session.window());

    click_mode_segment(&session, &driver, ui_rect, MODE)?;
    session.settle(20);

    let trace = session.trace()?;
    let mapping = CanvasMapping::from_trace(&trace, vocab, page, target.page)?;
    Ok((session, driver, mapping, target))
}

// ===========================================================================
// T — click text, type
// ===========================================================================

/// See the module documentation.
pub struct TheTextToolTypesOnOneClick;

impl Check for TheTextToolTypesOnOneClick {
    fn name(&self) -> &'static str {
        "the_text_tool_types_on_one_click"
    }

    fn defect(&self) -> &'static str {
        "pressing T and clicking text gives an I-beam and no caret, because the text tool SWEEPS \
         text and the tool that types is a different one reachable only through Edit ▸ Content — \
         four steps of ritual before a character can be typed, with nothing on screen saying so"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive_text(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(skip) => report.skip(skip.to_string()),
        }
    }
}

fn drive_text(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let (session, driver, mapping, target) = open_in_edit(ctx, report, "tool_text")?;

    // ★ ONE key. Not a ribbon click, not a chord — the bare letter, through the
    // OS, which is the convention being adopted and the one that a stray focus
    // could silently break.
    driver.press(VK_T)?;
    session.settle(16);

    let window_point = mapping.doc_to_window(DocPoint::new(target.page, target.x, target.y))?;
    let frame = session.frame()?;
    driver.click_at(frame.to_screen(window_point))?;
    session.settle(24);

    let trace = session.trace()?;
    let Some(caret) = trace.last(CARET_EVENT) else {
        return Ok(Some(format!(
            "★★ T THEN ONE CLICK PLACED NO CARET.\n\
             That is the operator's report exactly — *\"I get a box and the I cursor, but I \
             can't type anything\"* — and it has two candidate causes. (1) The bare `T` did not \
             arm anything: check the keymap binds it and that `canvas::keys` is not swallowing \
             it. (2) It armed the SWEEP rather than the caret: `CanvasTool::Text` must resolve \
             to a `TextEditKind` when `caps.edit_content`, which `canvas::interact`'s click \
             router does with `text_kind`. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("★ one press of T and one click: `{}`", caret.raw));

    // ★★ And the same tool, on EMPTY paper, starts new text rather than
    // refusing. That is the operator's second sentence — *"How do I make new
    // text when I click on the canvas and expect to edit there? Same problem"* —
    // and it is a different code path: `resolve_run` returns `NoRun` and
    // `textedit::click` turns it into an origin.
    //
    // Aimed at the page's top-left margin, which on a CAD sheet is inside the
    // border and outside every run. If the fixture has content there the check
    // reports what happened rather than failing: it is a fact about the fixture.
    let corner = mapping.doc_to_window(DocPoint::new(target.page, 20.0, 20.0))?;
    let frame = session.frame()?;
    driver.click_at(frame.to_screen(corner))?;
    session.settle(24);

    let trace = session.trace()?;
    if trace.last(BECAME_ADD_EVENT).is_some() {
        report.note("★★ the same tool clicked on blank paper and started NEW text");
    } else {
        report.note(
            "the second click did not fall through to a new run — the point named an existing \
             run, which is a fact about this fixture rather than about the feature",
        );
    }
    Ok(None)
}

// ===========================================================================
// A — click a shape, see its points
// ===========================================================================

/// See the module documentation.
pub struct ThePointsToolShowsPointsOnOneClick;

impl Check for ThePointsToolShowsPointsOnOneClick {
    fn name(&self) -> &'static str {
        "the_points_tool_shows_points_on_one_click"
    }

    fn defect(&self) -> &'static str {
        "the only way to see an object's points is to click it and then double-click twice to \
         descend a rung ladder nothing on screen mentions — so an operator who wants to move an \
         end point has to already know the ladder exists in order to discover it"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive_points(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(skip) => report.skip(skip.to_string()),
        }
    }
}

fn drive_points(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let (session, driver, mapping, target) = open_in_edit(ctx, report, "tool_points")?;

    driver.press(VK_A)?;
    session.settle(16);

    let window_point = mapping.doc_to_window(DocPoint::new(target.page, target.x, target.y))?;
    let frame = session.frame()?;
    driver.click_at(frame.to_screen(window_point))?;
    session.settle(24);

    let trace = session.trace()?;
    let Some(anchors) = trace.last(ANCHORS_EVENT) else {
        let routed = trace
            .events(NODE_CLICK_EVENT)
            .filter(|l| l.raw.contains("node-tool"))
            .count();
        // ★★★ The decline reason first, because it is the only thing here that
        // knows whose fault this is. See `AIM_REASONS`.
        if let Some(reason) = decline_reason(&trace)
            && AIM_REASONS.contains(&reason.as_str())
        {
            return Err(Error::new(format!(
                "the node tool armed and the click was routed to it {routed} time(s), and the \
                 anchor enumeration declined with `{DECLINED_EVENT} reason={reason}` — which \
                 is a fact about WHERE THIS RUN AIMED and not about the feature. SKIPPED \
                 rather than failed. Aim `--doc-point` at a stroked path: a text run, an \
                 image and a target inside a form XObject each have no anchors to show. \
                 Trace: {}.",
                session.trace_path().display()
            )));
        }
        let declined = decline_reason(&trace).map_or_else(
            || "no decline line either".to_owned(),
            |r| format!("`{DECLINED_EVENT} reason={r}`"),
        );
        return Ok(Some(format!(
            "★★ A THEN ONE CLICK DREW NO POINTS. The click was routed to the node tool \
             {routed} time(s), and the enumeration reported: {declined}.\n\
             If that count is zero the bare `A` armed nothing — check the keymap and that \
             `view.tool_node` is not being declined by the mode gate. If it is non-zero the \
             click reached `SelectionState::click_direct` and the selection did not end up at \
             the Part rung, which is the one line that makes the points appear: a click naming \
             a subpath must set `SelectionLevel::Part`, because `painting::draw_anchors` draws \
             from that rung up — `reason=not-entered` says exactly that happened. Trace: {}.",
            session.trace_path().display()
        )));
    };
    let total = anchors.get_usize("total").unwrap_or(0);
    let drawn = anchors.get_usize("unselected_drawn").unwrap_or(0);
    report.note(format!("★ one press of A and one click: `{}`", anchors.raw));
    if total == 0 {
        return Err(Error::new(
            "the click landed on an object with no anchors — a text run or an image, neither of \
             which has points. That is a fact about the aim point, so it is SKIPPED.",
        ));
    }
    if drawn == 0 {
        return Ok(Some(format!(
            "the points tool reported {total} anchors and drew NONE of them. Above \
             `overlay::MAX_UNSELECTED_ANCHORS` the unselected marks are suppressed \
             deliberately — but the marks are scoped to the entered SUBPATH, which is tens of \
             anchors on any real path, so hitting the cap means the scope regressed to the \
             whole object. That is the defect that made this feature blank on \
             `SW41177.pdf`, where one object carries 4,972 anchors."
        )));
    }
    report.note(format!("★★ {drawn} point(s) drawn on the first click"));
    Ok(None)
}

/// `show_points_draws_an_objects_points_without_descending` — **switch the View
/// toggle on, click once, and the anchors are there.**
///
/// # ★★★ Why this check exists, and it is not "one more toggle"
///
/// `view.show_points` was registered, drawn on View ▸ Display and **inert for
/// the whole life of the project**, behind a reason that said *"there is
/// nothing for it to show — this build draws no anchor mark at any rung"*. That
/// was true on 2026-08-15 and false four days later. Re-derived on 2026-08-28
/// as one of six stale blockers in eleven.
///
/// ★★ **The first wiring of it was ALSO inert, and no test could have caught
/// that.** The toggle was added as a disjunct to `draw_anchors`' rung guard —
/// which is correct — and the function then fell out two lines later on
/// `entered_object()`, which answers `None` at the Object rung *by
/// construction*, because "entered" means the operator descended. So the
/// control switched on, the trace said `view-chrome ShowPoints on=true`, and
/// not one anchor was drawn.
///
/// Every assertion the toggle had passed: it registered, it rendered pressed,
/// it reached `ViewState`. **What none of them asked was whether anything
/// changed on screen** — which is R1's whole subject, and the reason this file
/// gets a third member rather than the toggle getting a unit test.
///
/// # The oracle, and why it is a COMPARISON
///
/// `canvas-anchors total=… unselected_drawn=…` must be **absent** before the
/// toggle and **present** after it, on the same click at the same point. A
/// check asserting only the second half would pass on a build where anchors
/// draw at the Object rung unconditionally — which is a different program, and
/// a noisier one.
pub struct ShowPointsDrawsAnObjectsPointsWithoutDescending;

impl Check for ShowPointsDrawsAnObjectsPointsWithoutDescending {
    fn name(&self) -> &'static str {
        "show_points_draws_an_objects_points_without_descending"
    }

    fn defect(&self) -> &'static str {
        "View \u{25b8} Show points is on the ribbon and changes nothing on screen — the \
         command was drawn and inert for the life of the project, and the first wiring of it \
         was inert too, because the draw falls out on a rung guard two lines below the one \
         the toggle was added to"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive_show_points(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(skip) => report.skip(skip.to_string()),
        }
    }
}

fn drive_show_points(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let (session, driver, mapping, target) = open_in_edit(ctx, report, "show_points")?;

    // --- 1: click WITHOUT the toggle. The control case. ---------------------
    let window_point = mapping.doc_to_window(DocPoint::new(target.page, target.x, target.y))?;
    let frame = session.frame()?;
    driver.click_at(frame.to_screen(window_point))?;
    session.settle(24);

    let before = session.trace()?.events(ANCHORS_EVENT).count();
    if before > 0 {
        return Ok(Some(format!(
            "a plain click at the Object rung already drew anchors, before Show points was \
             switched on: `{ANCHORS_EVENT}` appeared {before} time(s).\n\
             That is a different program from the one this check describes, and a noisier one — \
             the Object rung deliberately draws no marks, because an object's anchors are not \
             the operator's subject there and thousands of hollow squares over something they \
             are about to move as a whole is noise with a rendering cost. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note("★ a plain click drew no anchors, which is the Object rung behaving as designed");

    // --- 2: switch the toggle on and click again ----------------------------
    //
    // ★ Through the ribbon rather than the harness's invoke seam, deliberately.
    // `PDFCER_DIAG_INVOKE` runs before the document is open and would prove the
    // dispatch arm rather than the control; what is under test is a toggle an
    // operator presses, and the whole defect class here is *the control changes
    // state and nothing happens*.
    // ★ The View TAB first. `open_in_edit` leaves the Edit tab active, and a
    // ribbon control only publishes a rect on the tab that is drawn — so
    // hunting for the toggle without switching tabs finds nothing and reports
    // it as absent, which is what the first run of this check did. The band
    // draws one tab; a region is a fact about what was drawn, not about what
    // exists.
    let ui_rect = ctx.profile.vocab.ui_rect_event.unwrap_or_default();
    let trace = session.trace()?;
    let view_tab = crate::checks::driving::declared(&trace, ui_rect, "ribbon.tab.view")
        .ok_or_else(|| {
            Error::new(format!(
                "no `ribbon.tab.view` region, so the View tab cannot be reached. Trace: {}.",
                session.trace_path().display()
            ))
        })?;
    driver.click_at(session.frame()?.declared_center(view_tab))?;
    session.settle(16);

    let trace = session.trace()?;
    let toggle = crate::checks::driving::declared(
        &trace,
        ctx.profile.vocab.ui_rect_event.unwrap_or_default(),
        TOGGLE_REGION,
    )
    .ok_or_else(|| {
        Error::new(format!(
            "no `{TOGGLE_REGION}` region — Show points is not drawn on the View tab in this \
                 build, so there is nothing to press. Trace: {}.",
            session.trace_path().display()
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(toggle))?;
    session.settle(20);
    driver.click_at(session.frame()?.to_screen(window_point))?;
    session.settle(24);

    let trace = session.trace()?;
    let Some(anchors) = trace.last(ANCHORS_EVENT) else {
        // ★★★ Same three-way split as `drive_points`, and it matters more here,
        // because this check's control case has already established that the
        // click lands and selects. If the enumeration declines for an aim
        // reason, what this run measured is the aim.
        if let Some(reason) = decline_reason(&trace)
            && AIM_REASONS.contains(&reason.as_str())
        {
            return Err(Error::new(format!(
                "Show points was switched on — the toggle's own region was clicked and the \
                 shell traced it — and the anchor enumeration declined with `{DECLINED_EVENT} \
                 reason={reason}`, which is a fact about WHERE THIS RUN AIMED and not about \
                 the toggle. SKIPPED rather than failed. Aim `--doc-point` at a stroked path. \
                 Trace: {}.",
                session.trace_path().display()
            )));
        }
        let declined = decline_reason(&trace).map_or_else(
            || {
                format!(
                    " There is no `{DECLINED_EVENT}` line either, so the draw ran and the \
                        census was suppressed, or the paint pass did not reach it at all."
                )
            },
            |r| format!(" The enumeration reported `{DECLINED_EVENT} reason={r}`."),
        );
        return Ok(Some(format!(
            "★★ SHOW POINTS WAS SWITCHED ON AND NOTHING WAS DRAWN: no `{ANCHORS_EVENT}` line \
             after the toggle and a second click.{declined}\n\
             This is the exact state the first wiring shipped in. `painting::draw_anchors` has \
             a rung guard the toggle was added to, and TWO LINES BELOW IT a `let else` on \
             `entered_object()` — which answers `None` at the Object rung by construction, \
             because \"entered\" means the operator descended. Check that the `None` arm reads \
             the selected object when `view.show_points` is on. Trace: {}.",
            session.trace_path().display()
        )));
    };
    let total = anchors.get_usize("total").unwrap_or(0);
    let drawn = anchors.get_usize("unselected_drawn").unwrap_or(0);
    report.note(format!("★★ the toggle drew points: `{}`", anchors.raw));

    if total == 0 {
        return Err(Error::new(
            "the click landed on an object with no anchors \u{2014} a text run or an image, neither \
             of which has points. That is a fact about the aim point, so it is SKIPPED.",
        ));
    }
    // ★★ `drawn == 0` is NOT a failure here, and that is the difference from
    // the points-tool check above. Above the cap, `overlay::draw_anchors`
    // suppresses the unselected marks deliberately — and at the Object rung the
    // list is the WHOLE object, which on a CAD path is thousands. That is
    // expected, it is why the disclosure exists, and the check asserts the
    // disclosure instead of calling the cap a defect.
    if drawn == 0 {
        let told = trace
            .events(ctx.profile.vocab.ui_rect_event.unwrap_or_default())
            .count();
        let _ = told;
        report.note(format!(
            "the object carries {total} anchors, past the 400 cap, so none were drawn \u{2014} which \
             is the cap behaving correctly and is why the status bar is told to say so"
        ));
        return Ok(None);
    }
    report.note(format!(
        "★★★ {drawn} of {total} points drawn at the Object rung, with no descent \u{2014} which is \
         what the toggle is for"
    ));
    Ok(None)
}
