//! `the_display_buttons_stack_in_two_rows` — the View tab's page-display group
//! is laid out two-high, not four-across.
//!
//! # The operator's ask, `OPERATOR_REQUESTS.md` O97
//!
//! > *"our display buttons should be on two rows to save space."*
//!
//! Four small commands — single page, continuous, facing, facing continuous —
//! that had been sitting in one long row and pushing everything to their right
//! toward the overflow.
//!
//! # ★★★ Why this is a DRIVEN check and not a unit test, when the layout is
//! pure arithmetic
//!
//! It looks like the ideal unit-test subject: `egui_shell::ribbon::plan`
//! computes the wrap, it takes a group and returns rows, and it is tested. But
//! the thing that can break is not the arithmetic — it is whether the **hint
//! reaches it**, and there are four places the chain can be cut, none of which
//! the planner's own tests can see:
//!
//! 1. `prefer_rows: 2` is absent from the group in `built_in.ron`, or the
//!    regenerated manifest silently drops it;
//! 2. the RON round-trip loses the field, because `Group`'s `Default` supplies
//!    `None` and a missing key is indistinguishable from an unset one;
//! 3. the band builder reads the group's items but never asks for its hint;
//! 4. the hint arrives and the **fits-already short-circuit runs first** — the
//!    planner's ordinary rule is "one row if it fits", and four small buttons
//!    fit at almost any width, so a `prefer_rows` that is consulted *after*
//!    that test is a `prefer_rows` that never does anything.
//!
//! ★★ The fourth is the one that matters, and it is why this check pins the
//! window to a **fixed wide viewport** before measuring. At a narrow width the
//! group might wrap for the ordinary reason and the check would pass over a
//! build where the hint is dead. At 2560 pt there is abundant room, so **the
//! only reason these four can be on two rows is that somebody asked for it.**
//!
//! ★ A fixed viewport rather than "maximise", because maximise gives whatever
//! monitor the run happens to be on — reproducible on one machine and not
//! across two, and silently weaker on a laptop.
//!
//! # The oracle is geometry, not pixels
//!
//! Every ribbon item publishes its rectangle. Four rectangles falling into
//! exactly two distinct vertical bands, two per band, is the whole assertion —
//! no screenshot, no colour, nothing that a theme change could disturb. That is
//! unusually cheap for a layout claim and is worth saying, because the
//! surrounding notes on this feature had assumed it would need a rendered
//! image.
//!
//! # What a passing run does NOT prove
//!
//! That the rows are in a sensible **order**, or that the group is narrower than
//! it was. Order is a manifest question and is asserted where the manifest is
//! tested; width is what two rows buys and is not independently measured here,
//! because "narrower than the one-row version" would need a second run of a
//! build that does not exist.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::geom::LRect;
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The mode whose ribbon carries the View tab.
const MODE: &str = "review";
/// The four commands, in manifest order.
const ITEMS: [&str; 4] = [
    "ribbon.item.view.page_single",
    "ribbon.item.view.page_continuous",
    "ribbon.item.view.page_facing",
    "ribbon.item.view.page_facing_continuous",
];
/// How many rows the group asks for.
const WANTED_ROWS: usize = 2;
/// Where and how large the window is placed, as `PDFCER_DIAG_VIEWPORT` takes
/// it: `x,y,w,h`.
///
/// ★★★ **The width IS the precondition of the assertion**, not a convenience.
/// See the module header's point 4. It also does not steal the desktop:
/// `PDFCER_DIAG_VIEWPORT` switches `with_active` off, so the window lays out
/// fully without taking focus.
const VIEWPORT: &str = "0,0,2560,1000";
/// How far apart two rectangles' tops may be and still count as the same row,
/// in logical points.
///
/// ★ Deliberately small. Buttons on one row share a `y` exactly in `egui`'s
/// layout, so any tolerance at all is generous; 4 pt allows for the harness
/// rounding a scaled coordinate and nothing else. A large tolerance here would
/// quietly merge two genuinely-stacked rows on a compact theme and report the
/// feature missing on a correct build.
const SAME_ROW_PT: f32 = 4.0;

/// See the module documentation.
pub struct TheDisplayButtonsStackInTwoRows;

impl Check for TheDisplayButtonsStackInTwoRows {
    fn name(&self) -> &'static str {
        "the_display_buttons_stack_in_two_rows"
    }

    fn defect(&self) -> &'static str {
        "the View tab's four page-display buttons sit in one long row, so they eat horizontal \
         space the ribbon does not have and push the groups to their right into the overflow"
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

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check must click the View \
             tab, because the manifest's first tab is `file` and the group it \
             measures is not on screen at launch. Reported as SKIPPED rather \
             than passed: a check that did not run has learned nothing.",
        ));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the application cannot say \
             where its controls are.",
            ctx.profile.name
        ))
    })?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("display_two_rows.trace.txt"));
    spec.pdf = ctx.pdf.clone();
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    // A real, laid-out window at a known width, without taking the desktop.
    if let Some(name) = ctx.profile.viewport_env {
        spec.env.push((name.to_owned(), VIEWPORT.to_owned()));
    } else {
        return Err(Error::new(format!(
            "the `{}` profile declares no viewport override, so this check \
             cannot fix the window width — and the width is what makes two \
             rows mean anything. Reported as SKIP rather than measured at \
             whatever width the window happened to open at.",
            ctx.profile.name
        )));
    }

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.note(format!(
        "launched {} as pid {}",
        exe.display(),
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);

    let driver = Driver::new(session.window());
    crate::checks::driving::click_mode_segment(&session, &driver, ui_rect, MODE)?;
    session.settle(14);

    // ★ The View tab has to be CLICKED. The manifest's first tab is `file`, so
    // the group this check measures is not on screen at launch — which is why
    // the `--no-input` path is a SKIP rather than a best-effort measure.
    // "Measure it if the right tab happens to be showing" is how a check comes
    // to pass without having looked at anything.
    let trace = session.trace()?;
    let tab = declared(&trace, ui_rect, "ribbon.tab.view").ok_or_else(|| {
        Error::new(format!(
            "no `ribbon.tab.view` region. Tabs declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(tab))?;
    session.settle(20);

    let trace = session.trace()?;
    let mut rects: Vec<(&str, LRect)> = Vec::new();
    for item in ITEMS {
        let Some(rect) = declared(&trace, ui_rect, item) else {
            return Err(Error::new(format!(
                "no `{item}` region on screen, so the display group is not being drawn — the \
                 View tab may not be the active one, or the group may have collapsed into the \
                 overflow. Items declared on the View tab: {}. Reported as SKIP rather than \
                 FAIL: this check has not measured the layout, and 'I could not see it' is \
                 not 'it is wrong'.",
                list(&declared_names(&trace, ui_rect, "ribbon.item.view."))
            )));
        };
        rects.push((item, rect));
    }

    // Group by top edge. `egui` gives buttons on one row an identical `y`, so
    // this is a partition rather than a clustering; SAME_ROW_PT exists only to
    // absorb harness rounding.
    let mut bands: Vec<Vec<&str>> = Vec::new();
    let mut tops: Vec<f32> = Vec::new();
    for (name, rect) in &rects {
        let top = rect.min.y;
        match tops.iter().position(|t| (t - top).abs() <= SAME_ROW_PT) {
            Some(i) => bands[i].push(name),
            None => {
                tops.push(top);
                bands.push(vec![name]);
            }
        }
    }
    report.note(format!(
        "the four display buttons fall into {} vertical band(s) at y = {:?}",
        bands.len(),
        tops.iter().map(|t| t.round() as i32).collect::<Vec<_>>()
    ));

    if bands.len() != WANTED_ROWS {
        return Ok(Some(format!(
            "the four display buttons occupy {} row(s) and the group asks for {WANTED_ROWS}. \
             pinned to 2560 pt wide there is room for all four side by side, \
             so one row means the `prefer_rows` hint never reached the \
             means the `prefer_rows` hint never reached the planner — check that the group \
             still carries `prefer_rows: 2` in `built_in.ron`, that the RON round-trip keeps \
             it (a missing key and an unset one look the same), and that the planner consults \
             it BEFORE its ordinary 'one row if it fits' short-circuit rather than after. \
             Tops measured: {:?}.",
            bands.len(),
            tops.iter().map(|t| t.round() as i32).collect::<Vec<_>>()
        )));
    }

    // Two rows of two. A 3+1 split is also "two rows" and is not what was
    // asked for — it would leave the group as wide as three buttons and save
    // almost nothing, which is the thing the request was about.
    let mut sizes: Vec<usize> = bands.iter().map(Vec::len).collect();
    sizes.sort_unstable();
    if sizes != vec![2, 2] {
        return Ok(Some(format!(
            "the four display buttons are split {sizes:?} across two rows rather than 2 and 2. \
             Two rows of three and one is still two rows, and it is still as wide as three \
             buttons — which saves almost nothing, and saving width is what was asked for. \
             Bands: {bands:?}."
        )));
    }

    // ★ The rows must be genuinely STACKED, not two bands a few points apart
    // that overlap on screen. The separation has to be at least a button's own
    // height or the second row is drawn over the first.
    let height = rects[0].1.height();
    let separation = (tops[1] - tops[0]).abs();
    if height > 0.0 && separation < height {
        return Ok(Some(format!(
            "the two rows are {separation:.0} pt apart and a button is {height:.0} pt tall, so \
             the second row overlaps the first. The wrap produced two bands and the layout did \
             not advance the cursor between them — which draws as a smudge rather than as two \
             rows, and no test of the planner's arithmetic would notice."
        )));
    }
    report.note(format!(
        "two rows of two, {separation:.0} pt apart, buttons {height:.0} pt tall"
    ));

    Ok(None)
}
