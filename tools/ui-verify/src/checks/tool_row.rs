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
            Err(why) => report.from_error(&why),
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

    // ★★★ The blank-paper half of this gesture is now a check of its own —
    // [`AClickOnBlankPaperStartsNewText`], 2026-09-07 — and it is worth saying
    // here why it left, because the six lines that used to sit at this spot are
    // the reason `O142` went unnoticed for two days.
    //
    // They clicked the page's bottom-left margin and then read
    // `trace.last(BECAME_ADD_EVENT)`. Present → a congratulatory note. Absent →
    // a note reading *"the point named an existing run, which is a fact about
    // this fixture rather than about the feature"* — **asserted from nothing.**
    // Neither branch could fail, and the excuse branch was a guess about the
    // fixture offered in the voice of a measurement.
    //
    // ⇒ So while the feature genuinely was dead — the engine's `hit_test` had
    // no distance bound, and answered *"the nearest run"* for a click a hundred
    // thousand points off the sheet — this check went green on every run and
    // printed the excuse. It is the `line_weights` shape exactly: an outcome
    // that cannot go red is not evidence, and a check that explains an absence
    // without measuring it is worse than one that says nothing.
    //
    // ★ Nothing about the second gesture is asserted from this function any
    // more, deliberately. Two places asserting one behaviour is how the weaker
    // one comes to be the only one that ever runs.
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
            Err(why) => report.from_error(&why),
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
            Err(why) => report.from_error(&why),
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

// ===========================================================================
// T on blank paper — click nothing, start something
// ===========================================================================

/// `a_click_on_blank_paper_starts_new_text` — the operator's **second**
/// 2026-08-19 sentence, driven, and the first check in this file that can fail
/// for the right reason.
///
/// > *"How do I make new text when I click on the canvas and expect to edit
/// > there? Same problem as the previous."* — 2026-08-19
///
/// One text tool, two outcomes: click **in** text and the caret lands in that
/// run; click on **blank paper** and a fresh run starts where the pointer is.
/// [`TheTextToolTypesOnOneClick`] asserts the first. This asserts the second,
/// which is a different code path — `place::resolve_run` returns
/// `Refusal::NoRun` and `place::click`'s fall-through arm converts it into an
/// `Anchor::Origin` at the click point.
///
/// # ★★★ Why this is a separate check, and it is a story about instruments
///
/// It used to be six lines at the end of `drive_text`, and those six lines let
/// `O142` sit undetected for **two days**. They read
/// `trace.last("text-edit-became-add")` and, when it was absent, printed *"the
/// point named an existing run, which is a fact about this fixture rather than
/// about the feature"*. Nothing measured that. **The absent branch could not
/// fail and the excuse was invented**, so a completely dead feature and a
/// badly-aimed click produced the same green line.
///
/// And the feature *was* dead. `EditableTextModel::hit_test` had no distance
/// bound: asked about a point with nothing near it, it returned the nearest
/// line of text at **any** distance — measured at the time as a click 100,000
/// points to the right of a 612-point page still landing in a run, and *"nothing
/// here"* answered zero times across two documents and thirty probes. The
/// fall-through arm was therefore unreachable, and this check's ancestor
/// reported success-or-shrug the whole time.
///
/// ⇒ `pdfcer-core` `8670523` (2026-09-05 19:13) bounded it to one line-height,
/// and this shell pinned an engine containing it at `eafe88f` **the same
/// evening, three hours later**. Nothing re-measured, so the operator row went
/// on saying BROKEN for two days after it was fixed. That is the cost being
/// paid for here.
///
/// # The three outcomes, all evidenced, none guessed
///
/// Every one is read from lines emitted **after** an anchor taken immediately
/// before the blank click, via [`crate::trace::Trace::last_after`] — never
/// `last()` over the whole capture, which would return the *first* click's
/// caret and report it as this click's answer. That fossil is the exact failure
/// `last_after`'s own doc comment was written to prevent.
///
/// | what the trace says after the blank click | verdict |
/// |---|---|
/// | `text-edit-caret … origin=X,Y` | **PASS** — a fresh run started |
/// | `text-edit-caret … run=N` | **SKIP** — the aim landed in real text |
/// | nothing at all | **FAIL** — the click reached no caret code at all |
///
/// ★★ The SKIP arm is the old excuse **with its evidence attached**: it can
/// only be reached by a trace line that names the run it hit, so it is a
/// measurement of the aim rather than a story about it. That is the whole
/// difference, and it is why the arm is allowed to exist at all.
///
/// ★★★ And `origin=` is checked against **where the pointer actually went**,
/// not merely observed to exist. `place.rs`'s own trace comment sets this
/// standard — *"a trace line must carry the number a wrong build would get
/// wrong"* — and an origin anchor that ignored the click and used, say, the
/// page corner or the previous caret would satisfy a bare presence test while
/// putting the operator's text somewhere he did not point. The tolerance is
/// [`ORIGIN_TOLERANCE_PT`].
///
/// # ★★ All three arms were FALSIFIED before this check was believed
///
/// A green check is not evidence that a check works; it is evidence that one
/// arm of it was reachable. On 2026-09-07 each arm was driven into
/// deliberately, against the same fixture and the same binary:
///
/// | planted | outcome | proves |
/// |---|---|---|
/// | nothing — the real thing | **PASS**, `origin=18.5,18.8` for a click asked at (20, 20) | the feature, and that the tolerance is doing rounding and not hiding a mistake |
/// | `BLANK_AIM_PT` moved onto known text at (1140, 62) | **SKIP**, quoting `run=426` | the excuse arm now carries the evidence the old one invented |
/// | `VK_A` pressed instead of `VK_T`, so no caret code runs at all | **FAIL** | the check can go red, which is the whole point of splitting it out |
///
/// ⚠ One arm is *not* independently reachable and that is deliberate: a
/// `BLANK_AIM_PT` outside the page box is refused by `CanvasMapping` before any
/// click is sent (*"document point (-260, -260) is outside the 1584x1224 pt page
/// box"*), so the harness's own geometry guard fires first. That is correct —
/// it means the FAIL arm can only be reached by the application failing — but
/// it does mean the off-page road to it is closed, and the `VK_A` plant above is
/// what proves the arm at all.
pub struct AClickOnBlankPaperStartsNewText;

/// How far the reported `origin=` may sit from the point the driver clicked,
/// in PDF points, before this check calls it a different place.
///
/// ★ Generous on purpose, and the generosity has a source: the driver clicks a
/// **whole screen pixel**, and at the fit zoom this shell opens at, one screen
/// pixel is a little under two PDF points on a D-size sheet. Rounding the
/// window point to an integer, converting back through the renderer's inverse
/// transform, and comparing in page space therefore cannot be exact.
///
/// ⚠ It is a bound on *rounding*, not a bound on *correctness*. Six points is
/// far below any wrong answer this check is built to catch — a page corner, the
/// previous caret's run, the page centre — every one of which is hundreds of
/// points away. If a future build lands inside six points and is still wrong,
/// widening this is the wrong repair; read the anchor out of the trace instead.
const ORIGIN_TOLERANCE_PT: f64 = 6.0;

/// Where the blank click aims, in PDF user space (origin bottom-left).
///
/// ⚠ **This is a guess about the fixture and it is labelled as one** — which is
/// exactly what the code this replaces failed to do. On a CAD sheet the very
/// corner of the media box is outside the drawn border, so it is usually blank;
/// but "usually" is not an assertion, which is why a run landing here SKIPs
/// with the run number rather than failing, and why the check reports the point
/// it used in every outcome.
const BLANK_AIM_PT: (f64, f64) = (20.0, 20.0);

impl Check for AClickOnBlankPaperStartsNewText {
    fn name(&self) -> &'static str {
        "a_click_on_blank_paper_starts_new_text"
    }

    fn defect(&self) -> &'static str {
        "clicking empty paper with the text tool armed puts the caret in whatever text happened \
         to be nearest — at any distance, anywhere on the sheet — instead of starting a new run \
         where the pointer is, so the operator's one-tool gesture silently becomes an edit of \
         something he did not click"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive_blank_paper(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn drive_blank_paper(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let (session, driver, mapping, target) = open_in_edit(ctx, report, "tool_blank_paper")?;

    // One bare letter, exactly as the sibling check presses it and for the same
    // reason: the count of gestures is part of what is being asserted.
    driver.press(VK_T)?;
    session.settle(16);

    let (aim_x, aim_y) = BLANK_AIM_PT;
    let corner = mapping.doc_to_window(DocPoint::new(target.page, aim_x, aim_y))?;
    let frame = session.frame()?;

    // ★★★ The anchor is taken HERE — after the tool is armed, immediately
    // before the gesture whose effect is being read. Anchoring any earlier
    // would let a line emitted during launch, mode entry or arming satisfy a
    // question about the click.
    let mark = session.trace()?.mark();

    driver.click_at(frame.to_screen(corner))?;
    session.settle(24);

    let trace = session.trace()?;
    let Some(caret) = trace.last_after(CARET_EVENT, mark) else {
        return Ok(Some(format!(
            "★★★ A CLICK ON BLANK PAPER PRODUCED NO CARET LINE AT ALL.\n\
             The text tool was armed and the pointer was put at page {} ({aim_x}, {aim_y}) in \
             PDF user space, and `{CARET_EVENT}` was not emitted after that gesture. This is \
             NOT the old defect — the old defect emitted a caret naming the wrong run. It \
             means the click did not reach `canvas::textedit::place::click` at all.\n\
             Look at, in order: (1) whether the point converted — a `--doc-point` page that \
             does not exist makes `doc_to_window` succeed and aim at nothing; (2) whether the \
             bare `T` is still arming the caret rather than the sweep; (3) whether a panel or \
             a floating window is over that corner of the canvas and took the press, which is \
             the `Area::constrain_to` failure recorded in the egui RAG. Trace: {}.",
            target.page,
            session.trace_path().display()
        )));
    };

    // ── The aim landed in real text. Honest, evidenced, and not this check's
    //    subject — so SKIP, carrying the run number that proves it.
    if let Some(run) = caret.get("run") {
        return Err(Error::new(format!(
            "the blank click landed IN TEXT: `{}`. The point used was page {} ({aim_x}, \
             {aim_y}) in PDF user space, and run {run} is drawn there, so this fixture has \
             content where this check expects paper. That is a fact about the fixture and the \
             aim, not about the feature — SKIPPED rather than failed or passed. Re-run against \
             a document with a blank margin, or move `BLANK_AIM_PT`. Trace: {}.",
            caret.raw,
            target.page,
            session.trace_path().display()
        )));
    }

    // ── The feature. `origin=` is the fall-through arm's own anchor shape.
    let Some(origin) = caret.get("origin") else {
        return Ok(Some(format!(
            "★★ THE CARET LINE NAMES NEITHER A RUN NOR AN ORIGIN: `{}`.\n\
             `place::click` writes `run=`, `origin=` or `box=`, so a `box=` here means the \
             click was routed to the dragged-rectangle entrance rather than the point \
             entrance. Trace: {}.",
            caret.raw,
            session.trace_path().display()
        )));
    };
    let Some((got_x, got_y)) = parse_origin(origin) else {
        return Ok(Some(format!(
            "the caret line's `origin=` is unreadable: `{origin}` in `{}`. It is written as \
             `origin={{x:.1}},{{y:.1}}` by `place::click`; if that format changed, this check \
             changed with it. Trace: {}.",
            caret.raw,
            session.trace_path().display()
        )));
    };

    // ★★ Presence was never the assertion. WHERE is.
    let (dx, dy) = ((got_x - aim_x).abs(), (got_y - aim_y).abs());
    if dx > ORIGIN_TOLERANCE_PT || dy > ORIGIN_TOLERANCE_PT {
        return Ok(Some(format!(
            "★★★ A NEW RUN STARTED, BUT NOT WHERE THE POINTER WAS.\n\
             Clicked page {} at ({aim_x}, {aim_y}) in PDF user space; the caret anchored at \
             ({got_x}, {got_y}) — off by ({dx:.1}, {dy:.1}) points against a tolerance of \
             {ORIGIN_TOLERANCE_PT}.\n\
             The fall-through arm IS firing, so this is not `hit_test`'s distance bound. It is \
             the conversion: `place::click`'s `NoRun` arm calls \
             `viewer::canvas_to_pdf_space` a SECOND time, independently of `resolve_run`'s \
             call, and a page whose `/Rotate` or CropBox origin is handled differently by the \
             two would land the text somewhere the operator did not point. Trace: {}.",
            target.page,
            session.trace_path().display()
        )));
    }

    report.note(format!(
        "★★★ one press of T and ONE click on blank paper started a new run: `{}`",
        caret.raw
    ));
    report.note(format!(
        "★★ and it started WHERE THE POINTER WAS — asked for ({aim_x}, {aim_y}), anchored at \
         ({got_x}, {got_y}), within {ORIGIN_TOLERANCE_PT} pt"
    ));
    if let Some(became) = trace.last_after(BECAME_ADD_EVENT, mark) {
        report.note(format!("★ by the documented route: `{}`", became.raw));
    } else {
        // ⚠ Not a failure. The outcome is what is being asserted; this line is
        // `place.rs` explaining ITSELF, and a build that reached an origin
        // anchor by some other correct road has still done what the operator
        // asked. But it is worth saying out loud, because it means the two
        // stopped agreeing.
        report.note(
            "⚠ an origin anchor was reached WITHOUT `text-edit-became-add` — the outcome is \
             right and the route is not the documented one; read `place::click`",
        );
    }
    Ok(None)
}

/// Split `origin=X,Y` from a caret trace line into a pair of PDF-space numbers.
///
/// Returns `None` rather than a default on anything unparseable, because the
/// caller must be able to tell *"the origin is in the wrong place"* from *"the
/// trace format moved and this check is now reading noise"*. A default would
/// merge them, and the second dressed as the first is a false defect report.
fn parse_origin(value: &str) -> Option<(f64, f64)> {
    let (x, y) = value.split_once(',')?;
    Some((x.trim().parse().ok()?, y.trim().parse().ok()?))
}
