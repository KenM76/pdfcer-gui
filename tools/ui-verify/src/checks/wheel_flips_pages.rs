//! `the_wheel_turns_pages_when_the_operator_asks_it_to` — O30, driven end to
//! end from the status-bar toggle.
//!
//! # The report
//!
//! `OPERATOR_REQUESTS.md` O30, 2026-08-24:
//!
//! > *"when in single page view there should be an option on screen near the
//! > button to scroll or flip through pages, or the current way it is now when
//! > the scroll wheel is used."*
//!
//! # ★★★ What makes this check able to fail, and it is not the page number
//!
//! The obvious check — *turn it on, roll the wheel, did the page change?* —
//! would pass against a build whose toggle wrote a preference nothing reads,
//! provided some **other** path happened to change the page. So the run does
//! three things in order and each is a separate claim:
//!
//! 1. **The default is silent.** Roll the wheel with the toggle OFF and assert
//!    the page does **not** change. Without this, a build that flipped pages
//!    unconditionally — ignoring the setting entirely — would pass everything
//!    below it. It is also the direct assertion that O30 did not change what
//!    the operator already had.
//! 2. **The toggle turns it on**, and the very next notch turns a page. ★★ The
//!    *very next* matters: the preference is a snapshot on `OpenDoc` for every
//!    other setting in the program, adopted when the Settings window is
//!    applied, and a build that let this one wait for that would look correct,
//!    write the file correctly, and change nothing on screen. That is the
//!    silently-inert control this suite exists for.
//! 3. **It goes both ways.** Roll the other way and assert the page goes back.
//!    A sign error is a viewer that works and feels wrong, which is harder to
//!    notice than one that is broken.
//!
//! # ★★ And the control is not drawn where the choice does not exist
//!
//! R9. Under a continuous display mode the wheel scrolls the whole document by
//! definition, so there is no second answer to offer and nothing is drawn —
//! not a disabled stub. The run switches to Continuous and asserts the
//! `status-wheel-paging` region **stops being declared**, which is the only
//! claim in this check that is about an absence.
//!
//! ★ An absence is admissible here under this module's rule 4 precisely
//! because the same region is shown to be present, in the same run, moments
//! earlier: the instrument that would have reported it is demonstrably
//! working.

use crate::checks::driving::{declared, declared_names, declared_or_in_overflow, list};
use crate::checks::{Check, CheckContext, CheckReport};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};

/// The canvas viewport, over which the wheel is rolled.
const CANVAS_REGION: &str = "canvas-viewport";

/// The status bar's wheel-paging toggle.
const TOGGLE_REGION: &str = "status-wheel-paging";

/// The status bar's own line, which carries `page=`.
const STATUS_EVENT: &str = "status";

/// How many notches to roll for one page turn.
///
/// One physical detent is 50 logical points on this platform and the
/// application's threshold is 40, so one notch is enough — but the harness's
/// wheel and the platform's may disagree about how much a "notch" is, and a
/// run that under-delivered would report "the wheel does not flip" for a build
/// where it does. Two notches is comfortably over one threshold and, because
/// the accumulator is **zeroed** on each turn rather than decremented, still
/// buys exactly one page.
const NOTCHES: i32 = 2;

/// See the module documentation.
pub struct TheWheelTurnsPagesWhenTheOperatorAsksItTo;

impl Check for TheWheelTurnsPagesWhenTheOperatorAsksItTo {
    fn name(&self) -> &'static str {
        "the_wheel_turns_pages_when_the_operator_asks_it_to"
    }

    fn defect(&self) -> &'static str {
        "in single-page view the wheel has nothing to scroll on a page that already fits, so it \
         does nothing at all — and the toggle that should make it turn pages is either absent, \
         or writes a preference that only takes effect after a Settings apply"
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

/// The wheel-paging setting the status bar last reported: `scroll` or `flip`.
///
/// ★★★ The instrument that makes this check **re-runnable**, and it did not
/// exist until this check needed it. The setting is PERSISTED, so a run that
/// turned it on left it on, and the second run of this check inherited the
/// first one's choice and reported the shipped default as broken — a
/// confident, specific, wrong accusation aimed at the part of the build a
/// reader can least easily check.
///
/// The standing rule this repeats: **a driven check that mutates persisted
/// state must normalise at the START**, and to normalise it must first be able
/// to read. A setting a check can change is a setting the trace must state.
fn wheel(session: &Session) -> Result<Option<String>> {
    Ok(session
        .trace()?
        .events(STATUS_EVENT)
        .last()
        .and_then(|line| line.get("wheel"))
        .map(str::to_owned))
}

/// The file token the application writes for "the wheel scrolls the page" —
/// `WheelPaging::Scroll::key`, and the shipped default.
const SCROLL: &str = "scroll";

/// The page the status bar last reported, 0-based.
fn page(session: &Session) -> Result<Option<usize>> {
    Ok(session
        .trace()?
        .events(STATUS_EVENT)
        .last()
        .and_then(|line| line.get_usize("page")))
}

/// Click a View-tab command by its ribbon item id.
fn invoke(session: &Session, driver: &Driver, ui_rect: &str, item: &str) -> Result<()> {
    let trace = session.trace()?;
    let tab = declared(&trace, ui_rect, "ribbon.tab.view").ok_or_else(|| {
        Error::new(format!(
            "no `ribbon.tab.view` region. Tabs declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(tab))?;
    session.settle(14);
    let region = declared_or_in_overflow(session, driver, ui_rect, item)?.ok_or_else(|| {
        Error::new(format!(
            "no `{item}` region on the View tab or in its overflow. Items declared: {}.",
            list(&declared_names(
                &session.trace().unwrap_or_default(),
                ui_rect,
                "ribbon.item.view."
            ))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(region))?;
    session.settle(24);
    Ok(())
}

#[allow(
    clippy::too_many_lines,
    reason = "one linear driven sequence, narrated"
)] // ui-text-exempt: clippy lint justification, never displayed
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let vocab = &ctx.profile.vocab;
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx
        .pdf
        .clone()
        .ok_or_else(|| Error::new("no --pdf. There are no pages to turn."))?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check rolls the wheel and presses a status-bar \
             control. Reported as SKIPPED rather than passed.",
        ));
    }
    let ui_rect = vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event.",
            ctx.profile.name
        ))
    })?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("wheel-flips-pages.trace.txt"));
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

    // --- the precondition: a single-page display, with pages to turn --------
    //
    // ★ Established THROUGH the ribbon, so the run also proves the control
    // that sets it. A check that reached in and set the mode would be testing
    // its own fixture.
    invoke(&session, &driver, ui_rect, "ribbon.item.view.page_single")?;
    let trace = session.trace()?;
    let canvas = declared(&trace, ui_rect, CANVAS_REGION)
        .ok_or_else(|| Error::new(format!("no `{CANVAS_REGION}`; is a document open?")))?;
    let frame = session.frame()?;
    let at = frame.declared_at(canvas, 0.5, 0.5);
    let Some(start) = page(&session)? else {
        return Err(Error::new(
            "the status bar never reported a page. SKIPPED.".to_owned(),
        ));
    };
    report.note(format!(
        "single-page display, on page {} of the document",
        start + 1
    ));

    // --- 0. NORMALISE, before anything is measured --------------------------
    //
    // ★★★ At the START, not at the end. Restoring it afterwards only runs when
    // the check PASSED — the case that did not need it — and every early
    // return leaks, which are the runs most likely to be repeated immediately.
    // This check found that out the hard way: its second run inherited its
    // first run's toggle and failed at claim 1, accusing the shipped default.
    //
    // ★ Normalised THROUGH the control, not by writing the file, so the run
    // also proves the control can turn the setting off as well as on. A build
    // whose toggle only ever set `FlipPages` would fail here rather than
    // silently pass the rest.
    match wheel(&session)?.as_deref() {
        Some(SCROLL) => {}
        Some(other) => {
            report.note(format!(
                "the wheel setting was left on `{other}` by an earlier run; normalising it back to `{SCROLL}` through the toggle before anything is measured"
            ));
            let toggle = declared(&session.trace()?, ui_rect, TOGGLE_REGION).ok_or_else(|| {
                Error::new(format!(
                    "the wheel setting is `{other}` and there is no `{TOGGLE_REGION}` region to put it back with. SKIPPED rather than measured from an unknown state."
                ))
            })?;
            driver.click_at(session.frame()?.declared_center(toggle))?;
            session.settle(20);
            if wheel(&session)?.as_deref() != Some(SCROLL) {
                return Err(Error::new(
                    "pressing the toggle did not return the wheel setting to `scroll`, so this run cannot establish its own starting state. SKIPPED."
                        .to_owned(),
                ));
            }
        }
        None => {
            return Err(Error::new(
                "the status bar does not report `wheel=`, so this check cannot normalise the persisted setting it is about to change. Build a binary that publishes it. SKIPPED rather than measured from an unknown state."
                    .to_owned(),
            ));
        }
    }

    // --- 1. the default is silent -------------------------------------------
    driver.scroll_at(at, -NOTCHES)?;
    session.settle(20);
    let after_default = page(&session)?.unwrap_or(start);
    if after_default != start {
        return Ok(Some(format!(
            "★★★ THE WHEEL TURNED A PAGE WITH THE TOGGLE OFF. It went from page {} to page {} \
             before anything was switched on. O30's default is `WheelPaging::Scroll` — today's \
             behaviour — precisely so that an upgrade does not change what the operator already \
             had; a build that flips unconditionally would pass every claim below this one.",
            start + 1,
            after_default + 1
        )));
    }
    report.note(
        "with the toggle off the wheel turned no page, which is the shipped default".to_owned(),
    );

    // --- 2. the toggle is there, and the very next notch obeys --------------
    let toggle = declared(&session.trace()?, ui_rect, TOGGLE_REGION).ok_or_else(|| {
        Error::new(format!(
            "no `{TOGGLE_REGION}` region in a single-page display. O30 asks for the option to be \
             ON SCREEN near the page buttons, so a build without this region has not delivered \
             it, whatever the preference file says. Status regions declared: {}.",
            list(&declared_names(
                &session.trace().unwrap_or_default(),
                ui_rect,
                "status-"
            ))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(toggle))?;
    session.settle(20);
    report.note("pressed the wheel-paging toggle beside the page buttons".to_owned());

    driver.scroll_at(at, -NOTCHES)?;
    session.settle(20);
    let after_on = page(&session)?.unwrap_or(start);
    if after_on <= start {
        return Ok(Some(format!(
            "★★★ THE WHEEL DID NOT TURN A PAGE. Still on page {} after the toggle was pressed \
             and the wheel rolled forward. The most likely cause is the one this check was \
             written for: `OpenDoc::prefs` is a SNAPSHOT adopted when the Settings window is \
             applied, so a wheel preference left to it writes the file correctly, draws the \
             control correctly, and changes nothing until Settings is opened and applied. \
             `app::frame` pushes this one into every open document the moment it changes.",
            after_on + 1
        )));
    }
    report.note(format!(
        "one notch forward turned page {} into page {}",
        start + 1,
        after_on + 1
    ));

    // --- 3. and it goes back ------------------------------------------------
    driver.scroll_at(at, NOTCHES)?;
    session.settle(20);
    let after_back = page(&session)?.unwrap_or(after_on);
    if after_back >= after_on {
        return Ok(Some(format!(
            "★★★ THE WHEEL ONLY GOES ONE WAY. Forward took the view from page {} to page {}, and \
             rolling back left it on page {}. `egui`'s scroll delta is POSITIVE when the operator \
             scrolls toward the start of the document, so a positive accumulation is a PREVIOUS \
             page — see `canvas::paging::flip`. A sign error here is a viewer that works and \
             feels wrong, which is harder to notice than one that is broken.",
            start + 1,
            after_on + 1,
            after_back + 1
        )));
    }
    report.note(format!(
        "and one notch back returned to page {}",
        after_back + 1
    ));

    // --- 4. and the control is absent where the choice does not exist -------
    invoke(
        &session,
        &driver,
        ui_rect,
        "ribbon.item.view.page_continuous",
    )?;
    session.settle(20);
    // ★★★ `live_names`, not `declared` — the `ui-rect` trace is a CHANGE LOG,
    // so a region declared before the mode switch is still in the file for
    // ever and `declared` would keep finding its fossil. `live_names` is the
    // helper written for exactly this question, and its own doc comment
    // carries the incident that produced it: on 2026-08-19 a check reported
    // that a delete had not worked, over a trace containing the deletion.
    //
    // ★ A first draft of this check used `declared_since` with an EVENT COUNT
    // where that helper wants a LINE NUMBER, and it reported this control as
    // still drawn when it was not. Kept in the record because it is the same
    // failure in a third costume: **a confident, specific, wrong report,
    // produced by an instrument being asked a question it does not answer.**
    session.settle(20);
    let live = crate::checks::driving::live_names(&session.trace()?, ui_rect, "status-");
    if live.iter().any(|name| name == TOGGLE_REGION) {
        return Ok(Some(format!(
            "★★★ THE TOGGLE IS STILL DRAWN UNDER A CONTINUOUS DISPLAY. There is no choice to \
             offer there — the wheel scrolls the whole document by definition — so R9 says the \
             control renders NOTHING, not a disabled stub. An operator who pressed it and saw no \
             difference would reasonably conclude it was broken. Status regions live now: {}.",
            list(&live)
        )));
    }
    report.note(
        "★★ and under a continuous display the toggle is not drawn at all, which is R9: the \
         choice does not exist there"
            .to_owned(),
    );

    Ok(None)
}
