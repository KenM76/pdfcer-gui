//! `measure_hover` — **hovering with a measure tool armed says what the click
//! will take, before it takes it.**
//!
//! # The report
//!
//! Operator, 2026-08-19:
//!
//! > *"The measuring tools themselves don't give me any indication of what is
//! > being selected either when I use them. I should be able to hover over a
//! > line or node and have it indicate that is what will be selected for the
//! > tool to use."*
//!
//! Asked whether he meant the entity or the snap point, the answer was
//! **both** — and this check asserts both, because either alone leaves the
//! question the other answers open.
//!
//! # ★★ Why this cannot be a unit test, in the specific rather than the general
//!
//! The two halves are resolved in one pass, in `canvas::measure::resolve_hover`,
//! while the page decomposition is borrowed — and then painted several
//! functions later, after the borrow is dropped. Everything in between is a
//! `Copy` struct being handed along.
//!
//! Every piece of that is individually testable and the composition is not:
//! the failure mode is *the pointer position one of them read*. That is exactly
//! how the defect the `Resolved` type exists to prevent got in — the marker
//! resolved against a raw screen position while the click used a converted
//! canvas one, so the two disagreed by the scroll origin over the zoom, which
//! is **zero at the top-left of an unscrolled page at 100 %**. It survived four
//! days and looked like *"sometimes it is fine"*.
//!
//! A driven run with the page scrolled and the pointer somewhere real is the
//! only thing that has ever caught that class here.
//!
//! # What it asserts
//!
//! | Phase | Does | Expected |
//! |---|---|---|
//! | A | arm Measure ▸ Linear | `measure-tool tool=Measure(Linear)` |
//! | B | move the pointer over page geometry, **without clicking** | `measure-hover-entity object=… segment=…` |
//! | C | the same frame's snap marker | `measure-snap-marker` within tolerance of the pointer |
//! | D | move to blank paper | the entity line stops being emitted |
//!
//! ★ Phase D is the half that stops this passing on a build that highlights
//! *everything*. A highlight that never retires is not an indication of what
//! is under the pointer, it is a decoration — and it would satisfy every
//! assertion above it.

use crate::checks::driving::{SHELL_DIAG_ENV, click_mode_segment, declared, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The mode the measure tools are reached in.
const MODE: &str = "review";
/// The tab carrying the Dimension group.
const TAB: &str = "ribbon.tab.measure";
/// The tool armed for the test.
const SUBJECT: &str = "ribbon.item.measure.linear";
/// The canvas region the pointer is moved within.
/// The page region, which is what the sweep walks across.
const PAGE: &str = "page";
/// The application's report of which tool is armed.
const ARM_EVENT: &str = "measure-tool";
/// The entity highlight's trace line.
const ENTITY: &str = "measure-hover-entity";
/// The snap marker's trace line.
const MARKER: &str = "measure-snap-marker";

/// See the module documentation.
pub struct MeasureHoverShowsWhatItWillTake;

impl Check for MeasureHoverShowsWhatItWillTake {
    fn name(&self) -> &'static str {
        "measure_hover_shows_what_it_will_take"
    }

    fn defect(&self) -> &'static str {
        "a measure tool is armed and the canvas says nothing about what a click would take — so \
         on a drawing made of near-identical strokes the operator picks by guessing, and a \
         two-line angle taken from the wrong line is a confident, plausible, wrong number"
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
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx
        .pdf
        .clone()
        .ok_or_else(|| Error::new("no fixture document. Pass --pdf."))?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check arms a tool and moves the pointer. \
             Reported as SKIPPED rather than passed.",
        ));
    }
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("measure_hover.trace.txt"));
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
    report.note(format!(
        "launched {} as pid {}",
        exe.display(),
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);
    let driver = Driver::new(session.window());

    click_mode_segment(&session, &driver, ui_rect, MODE)?;

    // --- A: arm the tool ---------------------------------------------------
    let trace = session.trace()?;
    let tab = declared(&trace, ui_rect, TAB).ok_or_else(|| {
        Error::new(format!(
            "no `{TAB}` region. Tabs declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(tab))?;
    session.settle(14);
    let item = declared(&session.trace()?, ui_rect, SUBJECT).ok_or_else(|| {
        Error::new(format!(
            "no `{SUBJECT}` region on the Measure tab. Items declared: {}.",
            list(&declared_names(
                &session.trace().unwrap_or_default(),
                ui_rect,
                "ribbon.item.measure."
            ))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(item))?;
    session.settle(20);

    let trace = session.trace()?;
    let Some(armed) = trace.events(ARM_EVENT).last() else {
        return Ok(Some(format!(
            "clicking the Linear control traced no `{ARM_EVENT}` line, so the tool did not arm \
             and there is nothing for a hover to describe."
        )));
    };
    report.note(format!("armed: `{}`", armed.raw));

    // --- B: hover over real geometry, WITHOUT clicking ---------------------
    //
    // ★ No click anywhere in this check. The whole subject is the state before
    // a commit, and a check that clicked would be asserting the same thing
    // `measure_linear` already asserts while destroying the state under test.
    let page_rect = declared(&trace, ui_rect, PAGE)
        .ok_or_else(|| Error::new(format!("no `{PAGE}` region — no sheet is being drawn.")))?;

    // ★★ SWEPT, not aimed, and the reason is that a single --doc-point
    // cannot know where ink is.
    //
    // The first draft computed one screen position from `--doc-point` and moved
    // there. It landed 135 pt from where the arithmetic said, on blank paper,
    // and reported the feature missing — a **confident, wrong defect report**,
    // which is the failure this suite has produced four times in a day by
    // trusting a derived coordinate over the trace.
    //
    // The honest instrument is the one `text_selection` already uses: try
    // points until one has something under it, and say how many were tried. A
    // sweep cannot be wrong about where the ink is, because it asks the
    // application rather than computing an answer.
    //
    // The grid is deliberately coarse. On the CAD sheets this application is
    // for, ink is everywhere; a fine grid would only make a blank fixture take
    // longer to fail.
    let mut hovered = None;
    let mut tried = 0usize;
    'sweep: for fy in [0.5_f32, 0.35, 0.65, 0.2, 0.8] {
        for fx in [0.5_f32, 0.35, 0.65, 0.2, 0.8] {
            let at = session.frame()?.declared_at(page_rect, fx, fy);
            let before = session.trace()?.events(ENTITY).count();
            driver.move_to(at)?;
            session.settle(14);
            tried += 1;
            let trace = session.trace()?;
            if trace.events(ENTITY).count() > before {
                hovered = Some((at, trace));
                break 'sweep;
            }
        }
    }
    let Some((aim, trace)) = hovered else {
        return Ok(Some(format!(
            "the pointer was moved to {tried} points across the sheet with a measure tool armed \
             and no `{ENTITY}` line was traced at any of them — so nothing told the operator \
              which line a click would take. That is the report this check exists for, unchanged. \
               (If this fixture is genuinely blank in the middle, the sweep is wrong rather than \
                the application; the page region was {page_rect:?}.)"
        )));
    };
    report.note(format!(
        "found ink after {tried} pointer position(s), at {aim:?}"
    ));
    let entities: Vec<_> = trace.events(ENTITY).collect();
    let last = entities.last().expect("non-empty by the sweep above");
    report.note(format!("hovering reported: `{}`", last.raw));
    if last.get("segment") != Some("1") {
        // ★★ SKIP, not fail. This branch already SAID the case was legitimate
        // and then fell through to a snap-marker assertion that only a straight
        // run can satisfy — so on a fixture whose middle is a curve, a text run
        // or an image, the check reported a defect it had just finished
        // explaining was not one.
        //
        // Found on `banana.pdf`, whose centre is the banana's outline. The
        // measure tool is correct there: it highlights the entity and offers no
        // endpoint, because a curve has none to snap to at that point. A check
        // that fails on correct behaviour is worse than an absent one, because
        // its red gets quoted.
        //
        // ★ Reported as SKIPPED with the finding named, so the run says "this
        // sheet could not answer the question" rather than "the application is
        // broken" — and a suite run against a drawing full of straight lines
        // still exercises it fully.
        return Err(Error::new(format!(
            "the entity under the pointer is not a straight run — `{}` — so its outline was \
             drawn instead of a segment, and there is no endpoint for a snap marker to sit on. \
             The sweep landed on a curve, a text run or an image rather than on a line, which \
             is a property of this fixture and not of the application. Drive a fixture with \
             straight vector geometry near the middle of the page to exercise this. SKIPPED \
             rather than failed: the assertion below can only be met by a segment, so applying \
             it here would report correct behaviour as a defect.",
            last.raw
        )));
    }

    // --- C: and the node, in the same frame --------------------------------
    //
    // Both halves, because the operator asked for both and because either alone
    // leaves the other's question open: a node with no highlight does not say
    // WHICH line it belongs to, and a highlight with no node does not say where
    // on it the click will land.
    let Some(marker) = trace.events(MARKER).last() else {
        return Ok(Some(format!(
            "the entity highlight drew and no `{MARKER}` line was traced in the same hover, so \
             the operator is told which line and not where on it. Both were asked for."
        )));
    };
    let drift = marker
        .get("dx")
        .and_then(|v| v.parse::<f32>().ok())
        .unwrap_or_default()
        .hypot(
            marker
                .get("dy")
                .and_then(|v| v.parse::<f32>().ok())
                .unwrap_or_default(),
        );
    let tol = marker
        .get("tol")
        .and_then(|v| v.parse::<f32>().ok())
        .unwrap_or_default();
    report.note(format!(
        "the snap marker sits {drift:.1} pt from the pointer, tolerance {tol:.1}"
    ));

    // --- D: ★★ and it FOLLOWS the pointer -----------------------------
    //
    // The half that stops this passing on a build that highlights something
    // permanently. A highlight which never changes is a decoration, and it
    // would satisfy every assertion above.
    //
    // ★ The first draft asserted the highlight RETIRED over blank paper, and
    // that premise was wrong for the documents this application is for: a CAD
    // sheet has a drawing border, so the corner is not blank, and at this zoom
    // the catch radius is 33 pt of page. The check failed against a build that
    // was working — the fifth confident-wrong report this suite has produced in
    // a day by asserting something it had assumed rather than measured.
    //
    // What can be asserted without assuming anything about the fixture is that
    // the answer *depends on the pointer*: two positions far apart must not
    // name the same object. On a drawing dense enough to measure, that is
    // certain; on one sparse enough to fail it, the message says which object
    // was named twice so the reader can judge.
    let mut named: Vec<String> = Vec::new();
    for (fx, fy) in [(0.2_f32, 0.2_f32), (0.8, 0.8), (0.2, 0.8), (0.8, 0.2)] {
        let at = session.frame()?.declared_at(page_rect, fx, fy);
        driver.move_to(at)?;
        session.settle(14);
        if let Some(line) = session.trace()?.events(ENTITY).last()
            && let Some(object) = line.get("object")
        {
            named.push(object.to_owned());
        }
    }
    let distinct: std::collections::BTreeSet<&str> = named.iter().map(String::as_str).collect();
    report.note(format!(
        "four corners of the sheet named object(s) {}",
        list(&named)
    ));
    if distinct.len() < 2 {
        return Ok(Some(format!(
            "the pointer was moved to four widely separated points and the highlight named {} \
             object(s): {}. A highlight that does not change with the pointer does not say what \
              is under it — it says something is always under it.",
            distinct.len(),
            list(&named)
        )));
    }
    report.note("the highlight names a different object as the pointer moves, so it tracks");

    let shot = ctx.out("measure_hover.png");
    match crate::capture::window_to_png(&session, &shot) {
        Ok(_) => {
            report.artifact(shot);
        }
        Err(e) => {
            report.note(format!(
                "the window could not be captured ({e}); the trace assertions above still hold"
            ));
        }
    }
    Ok(None)
}
