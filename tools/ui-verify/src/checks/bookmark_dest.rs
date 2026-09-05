//! `a_bookmark_lands_on_the_detail_it_names` — **the destination, not just the
//! page.**
//!
//! # The report
//!
//! Ken, 2026-09-01:
//!
//! > *"in Acrobat clicking on the nested bookmarks in the drawing package takes
//! > you to a zoomed in area of the page for the drawing bookmark that was
//! > clicked on. when we click on ours it just jumps us to the correct page,
//! > but doesn't send us to the spot on the page the bookmark actually points
//! > to."*
//!
//! ## ★★★ Why his own drawing is the fixture
//!
//! `TR-0461-1500-copy.pdf` is the case exactly, and its outline says so:
//!
//! ```text
//! bookmark level=0 title="TR-0461-1500"   dest=Page { page_index: 0, view: Fit }
//!   bookmark level=1 title="Drawing View64" dest=… FitR { left: 493, bottom: 119, right: 1104, top: 558 }
//!   bookmark level=1 title="Drawing View65" dest=… FitR { left:  76, bottom: 119, right:  687, top: 558 }
//! ```
//!
//! **Two nested bookmarks, one page, different rectangles.** Under the old
//! behaviour — page only — clicking either arrived in the same place, which is
//! indistinguishable from both being broken. That is why a check that asserted
//! "the page changed" would have passed against the defect, and why this one
//! asserts the ZOOM changed instead.
//!
//! ## The oracle, and why it is the zoom
//!
//! `Fit` on an A1 sheet is about 0.39×; a `FitR` around one detail is several
//! times that. The `canvas` line carries `zoom=`, so *"did clicking a detail
//! bookmark actually take me to the detail"* reduces to *"did the zoom rise"* —
//! which no page-only navigation can produce.
//!
//! ★★ Asserted as a RATIO against the zoom before the click rather than against
//! an absolute number. The absolute depends on the window size, and a check
//! that pinned it would fail on a different monitor while the feature worked.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Edit is not needed; the panel is, and Read is where an operator reads.
const INVOKE: &str = "view.panel_bookmarks";
/// The per-row line, carrying `title=`, `page=` and the row's own `rect=`.
const ROW: &str = "bookmark-row"; // ui-text-exempt: a trace event name, never displayed
/// The line the panel writes when a row is actually PRESSED — the other half
/// of , which only says where rows are. See the assertion that reads it.
const PICK: &str = "bookmark-pick"; // ui-text-exempt: a trace event name, never displayed
/// The canvas's per-frame line, carrying `zoom=`.
const CANVAS: &str = "canvas"; // ui-text-exempt: a trace event name, never displayed
/// The page region, so a failure can say whether a sheet was drawn at all.
const PAGE_REGION: &str = "page"; // ui-text-exempt: a trace region name, never displayed
/// The operator's own drawing package.
/// The operator's drawing package, **by file name** — see [`crate::fixture::operator_file`]
/// for why this stopped being an absolute path on 2026-09-05.
const FIXTURE: &str = "TR-0461-1500-copy.pdf";
/// The nested bookmark to click — a detail on page 1, not the page itself.
const DETAIL: &str = "Drawing View64";
/// How much bigger a detail must be than the page fit to count as arrival.
///
/// ★ 1.5×, deliberately loose. The point is to separate "framed a detail" from
/// "did not move at all", and the exact ratio depends on the window; a tight
/// bound would be a test of the monitor.
const AT_LEAST: f64 = 1.5;

pub struct ABookmarkLandsOnTheDetailItNames;

impl Check for ABookmarkLandsOnTheDetailItNames {
    fn name(&self) -> &'static str {
        "a_bookmark_lands_on_the_detail_it_names"
    }

    fn defect(&self) -> &'static str {
        "clicking a bookmark goes to its page and stops there, so every nested drawing-view \
         bookmark on one sheet arrives in the same place — which is indistinguishable from all \
         of them being broken"
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

fn zoom_of(session: &Session, event: &str) -> Option<f64> {
    session
        .trace()
        .ok()?
        .last(event)?
        .get("zoom")?
        .parse::<f64>()
        .ok()
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks a row.",
        ));
    }
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let Some(source) = crate::fixture::operator_file(FIXTURE) else {
        return Err(Error::new(format!(
            "{}. This check needs a document whose nested bookmarks carry `/FitR` destinations; one with page-only bookmarks cannot tell the fix from the defect.",
            crate::fixture::operator_file_complaint(FIXTURE)
        )));
    };
    let pdf = ctx.out("bookmark-dest.pdf");
    if let Some(dir) = pdf.parent() {
        std::fs::create_dir_all(dir).map_err(|e| Error::new(e.to_string()))?;
    }
    std::fs::copy(&source, &pdf).map_err(|e| Error::new(e.to_string()))?;

    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("bookmark-dest.trace.txt"));
    spec.pdf = Some(pdf);
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
        "launched {} as pid {}",
        exe.display(),
        session.pid()
    ));
    session.settle(55);
    let driver = Driver::new(session.window());

    if declared(&session.trace()?, ui_rect, PAGE_REGION).is_none() {
        return Err(Error::new(format!(
            "no `{PAGE_REGION}` region, so no sheet is on screen. Regions beginning `page`: {}.",
            list(&declared_names(&session.trace()?, ui_rect, "page"))
        )));
    }

    // The nested bookmark's row. Its rect comes from the trace, because the
    // panel is a scrolled tree and nothing else knows where a row landed.
    let trace = session.trace()?;
    let Some(row) = trace
        .events(ROW)
        .filter(|l| l.get("title").is_some_and(|t| t.contains(DETAIL)))
        .last()
    else {
        return Err(Error::new(format!(
            "no `{ROW}` line whose title contains {DETAIL:?}. The nested rows are collapsed \
             under their parent unless the outline opens them, and this check does not expand \
             anything — a document whose details are hidden behind a closed disclosure cannot \
             be clicked. Rows seen: {}.",
            trace
                .events(ROW)
                .filter_map(|l| l.get("title").map(str::to_owned))
                .collect::<Vec<_>>()
                .join(", ")
        )));
    };

    let before = zoom_of(&session, CANVAS).unwrap_or_default();
    report.note(format!("★ the fitted page is at zoom {before:.3}"));

    // ★ The row's rect comes off its own trace line, not off a `ui-rect`
    // region: the panel is a scrolled tree and publishes no per-row region, so
    // the line the panel already writes for the harness is the only thing that
    // knows where a row landed.
    let rect = row.get_rect("rect").ok_or_else(|| {
        Error::new("the row line carries no readable `rect=`, so there is nowhere to click.")
    })?;
    driver.click_at(session.frame()?.declared_center(rect))?;
    session.settle(60);

    let after = zoom_of(&session, CANVAS).unwrap_or_default();
    report.note(format!(
        "★★ after clicking {DETAIL:?} the zoom is {after:.3}"
    ));

    // ★★★ WHICH ROW WAS ACTUALLY PRESSED — asked BEFORE the zoom is judged.
    //
    // `zoom unchanged` has two causes and they want opposite fixes: the
    // destination was not applied (a defect in the shell), or the click landed
    // on a different row or on none at all (a defect in this check's aim, or a
    // panel that moved between the trace read and the press). Until the panel
    // traced the press there was no way to tell, and the failure message had to
    // hedge — which is how an intermittent harness problem came to read as a
    // regression in a shipped feature.
    let picked = session.trace()?.events(PICK).last().map(|l| l.raw.clone());
    let Some(picked) = picked else {
        return Ok(Some(format!(
            "the row for {DETAIL:?} was clicked at its own traced rectangle and the panel \
             recorded NO press at all. The click did not land on a bookmark row — the \
             panel scrolled between the trace read and the press, or the row moved under \
             it. This is not a finding about destinations. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("the panel recorded the press: `{picked}`"));
    if !picked.contains(DETAIL) {
        return Ok(Some(format!(
            "the click landed on the WRONG ROW: the panel recorded `{picked}` when \
             {DETAIL:?} was aimed at. Again not a finding about destinations — the \
             rectangle read from the trace was stale by the time the pointer arrived."
        )));
    }

    if after < before * AT_LEAST {
        return Ok(Some(format!(
            "★★★ THE BOOKMARK WENT TO THE PAGE AND STOPPED: zoom {before:.3} → {after:.3}, when \
             {DETAIL:?} names a `/FitR` rectangle covering about a third of the sheet.\n\
             This is the operator's report exactly. `Destination::Page` carries BOTH a \
             `page_index` and a `view`, and a `..` in the pattern that reads it is where the \
             view goes — after which every nested drawing-view bookmark on one sheet arrives in \
             the same place. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note("★★★ …which is the detail framed, not the page fitted");
    Ok(None)
}
