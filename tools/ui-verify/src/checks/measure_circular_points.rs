//! `three_clicks_round_a_hole_measure_the_hole` — the operator's own report,
//! driven, on a fixture built to reproduce it.
//!
//! # ★★★ The defect, in his words and then in numbers
//!
//! `OPERATOR_REQUESTS.md` O105, 2026-09-03:
//!
//! > *"can you check our radius/diameter dimensioning tool? selecting a point
//! > sometimes makes a big circle, and selecting more points around a hole
//! > doesn't always get it to narrow down to the size of the hole."*
//!
//! The cause was measured, not guessed. The tool hit-tested for a **PDF path
//! object** and fed *every anchor of every subpath of that object* to the
//! circle fit. On his own drawing — `SW41177.pdf` page 1, read with
//! `pdfcer object-list` — three objects carry **4,405**, **4,972** and
//! **6,681** anchors, the largest holding 1,194 subpaths across a 550 × 500 pt
//! region. One click anywhere on it handed the fit six thousand points
//! scattered over half the sheet, and the circle through them is enormous.
//!
//! # ★★★ Why this check pins its own fixture and ignores `--pdf`
//!
//! Because on a document with one tidy circle in its own object, **the defect
//! cannot occur**: a click contributes that circle's anchors, the fit is the
//! circle, and the broken build passes. A check that can only pass is not
//! evidence.
//!
//! So `fixtures/hole-in-a-big-object.pdf` is built by
//! `tools/gen-hole-in-a-big-object-fixture.py` to have exactly the shape that
//! produced the report: **one** path object holding a 30 pt-radius circle *and*
//! forty unrelated segments spread across the page. Under the old build a
//! single click on the rim fits a circle through 85 scattered anchors —
//! hundreds of points of radius. Under the new one, three clicks on the rim fit
//! the rim. The two builds are told apart by a number.
//!
//! ⇒ This is the same discipline `ocr` follows for the same reason, and it is
//! why the check reports the fixture it used in its own notes: a sweep that
//! passed `--pdf` and had it ignored must be able to see that happen.
//!
//! # What it asserts, and why each one is separately necessary
//!
//! | Phase | Asserts | The build it fails |
//! |---|---|---|
//! | A | three clicks on the rim produce three `measure-circular-point action=add` lines, `n=1,2,3` | one where a click is still an object toggle — the second click on the same object would REMOVE, and `n` would read 1, 0, 1 |
//! | B | after the third, `r` is the hole's radius | one where a click contributes the whole object's anchors. This is the operator's sentence, as a number |
//! | C | the Tool panel lists one row per point | O107's panel half, which nothing else can substitute — a pick set on a dense sheet is invisible |
//! | D | clicking a row removes that point | O107's literal ask |
//! | E | clicking the canvas near a picked point removes it too | O107's other route — *"we should be able to unselect points/clicked locations"* |
//!
//! ★★ **B is the load-bearing one and A alone would be worthless.** A build
//! that accepted three clicks and fitted them to the wrong geometry satisfies A
//! completely. The count says the clicks registered; only the radius says the
//! tool measured the thing under them.
//!
//! # ★ What it deliberately does NOT assert
//!
//! That a **free** position works — O106. The fixture is vector geometry and
//! every rim click snaps, which is the correct behaviour on a drawing that has
//! geometry; asserting the free path here would mean aiming somewhere the snap
//! declines, which is a different fixture (a raster) and a different check.
//! `canvas::measure::circpick`'s unit tests cover the composition; the driven
//! half of O106 is unbuilt and is named here rather than left implied.

use crate::checks::driving::{
    self, SHELL_DIAG_ENV, TAB_EVENT, declared, declared_names, list, shell_trace,
};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The mode the Measure tab lives in.
const MODE: &str = "review";
/// The Measure tab's region and its shell id.
const TAB: &str = "ribbon.tab.measure";
/// The tab id the shell reports when it is activated.
const TAB_ID: &str = "measure";
/// The control that arms the radius/diameter tool.
const SUBJECT: &str = "ribbon.item.measure.radius_diameter";
/// The View-tab control that mounts the Tool panel.
const TOOL_PANEL_ITEM: &str = "ribbon.item.view.panel_tool";
/// The View tab, which carries that control.
const VIEW_TAB: &str = "ribbon.tab.view";
/// The Tool panel's body region.
const TOOL_PANEL_BODY: &str = "panel:tool";
/// The Tool panel's dock tab header.
const TOOL_PANEL_TAB: &str = "dock.tab.tool";
/// The picked-point list's region.
const POINT_LIST: &str = "tool.measure_points";
/// The prefix of one picked point's row.
const POINT_ROW_PREFIX: &str = "tool.measure_point.";
/// The line the tool traces on every pick and every removal.
const PICK_EVENT: &str = "measure-circular-point";

/// The fixture, pinned. See the module header for why `--pdf` is ignored.
const FIXTURE: &str = "fixtures/hole-in-a-big-object.pdf";

/// The hole's centre and radius in the fixture, PDF user space.
///
/// Hard-coded here **and** in the generator, which is a duplication with a
/// reason: the generator is the source and this is the *expectation*, and a
/// check that read its expectation out of the thing it is checking would pass
/// on any fixture at all. If the two ever disagree, the failure message below
/// names both numbers.
const HOLE: (f64, f64, f64) = (306.0, 500.0, 30.0);

/// How far the fitted radius may be from the hole's, in points.
///
/// ★ Two points, not two per cent, and the difference matters. The failure this
/// separates from a pass is an order of magnitude — the broken build fits a
/// radius in the **hundreds** — so any threshold between "a few points" and
/// "half the page" tells the two apart. Two points is the loosest value that
/// still fails a build fitting the rim of anything other than this hole, and it
/// absorbs the two or three points of aim a real pointer costs at fit-page
/// zoom.
const RADIUS_TOLERANCE_PT: f64 = 2.0;

/// See the module documentation.
pub struct ThreeClicksRoundAHoleMeasureTheHole;

impl Check for ThreeClicksRoundAHoleMeasureTheHole {
    fn name(&self) -> &'static str {
        "three_clicks_round_a_hole_measure_the_hole"
    }

    fn defect(&self) -> &'static str {
        "the radius/diameter tool picks whole PDF path objects rather than points, so one click \
         on a hole inside a large object hands the circle fit every anchor in that object — \
         thousands of them, spread over half the sheet — and the fitted circle is enormous; \
         adding more picks makes it worse, and a second click on the same object removes it \
         again (OPERATOR_REQUESTS.md O105)"
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
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check is several real clicks on the page. \
             Reported as SKIPPED rather than passed.",
        ));
    }
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;

    // ★ The fixture is PINNED, and any `--pdf` is ignored. See the module
    // header: on a document whose circles are their own objects the defect
    // under test cannot occur, so a sweep's fixture would make this check
    // unable to fail.
    let pdf = ctx.source_root.clone().unwrap_or_default().join(FIXTURE);
    let pdf = if pdf.exists() {
        pdf
    } else {
        std::path::PathBuf::from(FIXTURE)
    };
    if !pdf.exists() {
        return Err(Error::new(format!(
            "the fixture {FIXTURE} is not on disk. Regenerate it with \
             `python tools/gen-hole-in-a-big-object-fixture.py`; this check cannot use an \
             arbitrary document, because a page whose circles are their own objects cannot \
             exhibit the defect."
        )));
    }
    if ctx.pdf.is_some() {
        report.note(format!(
            "--pdf was supplied and is IGNORED: this check pins {FIXTURE}, which is built to \
             carry the operator's geometry (one object holding a small circle and forty \
             unrelated segments)"
        ));
    }

    let page = crate::coords::PageGeometry {
        width_pt: 612.0,
        height_pt: 792.0,
    };

    let mut spec = LaunchSpec::new(&exe, ctx.out("measure_circular_points.trace.txt"));
    spec.pdf = Some(pdf.clone());
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
        "launched {} as pid {} on {}",
        exe.display(),
        session.pid(),
        pdf.display()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);
    // Maximised so the Measure tab's groups are on the band and the right dock
    // has room for the Tool panel's list — this check reads a panel, and a
    // panel squeezed to nothing publishes rows nobody can press.
    session.maximize();
    session.settle(20);
    let driver = Driver::new(session.window());

    // --- arm the tool ------------------------------------------------------
    driving::click_mode_segment(&session, &driver, ui_rect, MODE)?;
    let trace = session.trace()?;
    let tab = declared(&trace, ui_rect, TAB).ok_or_else(|| {
        Error::new(format!(
            "no `{TAB}` region after switching to {MODE}. Tabs declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(tab))?;
    session.settle(14);
    if !shell_trace(&session)?
        .events(TAB_EVENT)
        .any(|l| l.get("tab") == Some(TAB_ID))
    {
        return Err(Error::new(format!(
            "the click on `{TAB}` produced no `{TAB_EVENT} tab={TAB_ID}` line, so the Measure \
             tab never activated and there is nothing to arm."
        )));
    }
    let Some(item) = driving::declared_or_in_overflow(&session, &driver, ui_rect, SUBJECT)? else {
        return Err(Error::new(format!(
            "the Measure tab declares no `{SUBJECT}`, on the band or anywhere the band search \
             can reach. Controls declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.item.measure."))
        )));
    };
    driver.click_at(session.frame()?.declared_center(item))?;
    session.settle(16);
    report.note("Measure ▸ radius/diameter armed");

    // --- aim three points on the rim ---------------------------------------
    let trace = session.trace()?;
    let mapping = CanvasMapping::from_trace(&trace, &ctx.profile.vocab, page, 0)?;
    let frame = session.frame()?;
    let (cx, cy, r) = HOLE;
    // North, east and west. Three quadrant anchors: they are 90° apart, so no
    // two are within a snap radius of each other, and they are the points the
    // fixture's four cubics actually place anchors at — see the generator.
    let rim = [(cx, cy + r), (cx + r, cy), (cx - r, cy)];
    let mut aimed = Vec::with_capacity(rim.len());
    for (x, y) in rim {
        let point = DocPoint::new(0, x, y);
        aimed.push((point, frame.to_screen(mapping.doc_to_window(point)?)));
    }
    report.note(format!(
        "the hole is r={r:.0} at ({cx:.0}, {cy:.0}); aiming at its north, east and west quadrants"
    ));

    let mut failures: Vec<String> = Vec::new();

    // --- A + B: the picks, and the radius they converge on -----------------
    for (n, (point, screen)) in aimed.iter().enumerate() {
        let before = session.trace()?.events(PICK_EVENT).count();
        driver.click_at(*screen)?;
        session.settle(14);
        let trace = session.trace()?;
        let lines: Vec<&crate::trace::TraceLine> = trace.events(PICK_EVENT).collect();
        let Some(line) = lines.get(before) else {
            return Ok(Some(format!(
                "click {} of {} at document ({:.0}, {:.0}) produced no `{PICK_EVENT}` line, so \
                 the pick never reached the tool. Lines so far: {}.",
                n + 1,
                aimed.len(),
                point.x,
                point.y,
                lines.len()
            )));
        };
        let action = line.get("action").unwrap_or("?");
        let count = line.get_usize("n").unwrap_or(0);
        report.note(format!(
            "click {} → action={action} origin={} n={count} r={} resid={}",
            n + 1,
            line.get("origin").unwrap_or("?"),
            line.get("r").unwrap_or("?"),
            line.get("resid").unwrap_or("?")
        ));
        if action != "add" {
            failures.push(format!(
                "click {} on the rim was taken as `{action}`, not `add`. Three clicks at three \
                 different quadrants must each ADD a point — an `add` followed by a `remove` is \
                 the signature of the object toggle this check exists to catch, where clicking \
                 twice on the same shape puts it in and takes it out again.",
                n + 1
            ));
        }
        if count != n + 1 {
            failures.push(format!(
                "after click {} the set holds {count} point(s), not {}.",
                n + 1,
                n + 1
            ));
        }
        // ★★ THE assertion, on the last click. See the module header: A alone
        // is satisfied by a build that accepts three clicks and fits them to
        // the wrong geometry.
        if n + 1 == aimed.len() {
            match line.get("r").and_then(|v| v.parse::<f64>().ok()) {
                Some(fitted) if (fitted - r).abs() <= RADIUS_TOLERANCE_PT => {
                    report.note(format!(
                        "the fit is r={fitted:.2} against the hole's {r:.2} — within \
                         {RADIUS_TOLERANCE_PT:.1} pt"
                    ));
                }
                Some(fitted) => failures.push(format!(
                    "three clicks on the rim of a {r:.0} pt hole fitted a circle of radius \
                     {fitted:.2}. That is the operator's report — *\"selecting a point sometimes \
                     makes a big circle\"* — as a number. The fixture's hole shares its path \
                     object with forty unrelated segments spanning the page, so a radius in the \
                     hundreds means the fit is being handed the OBJECT's anchors rather than the \
                     points that were clicked."
                )),
                None => failures.push(format!(
                    "three points are in the set and the trace reports r={}, so no circle was \
                     fitted through them at all. Three non-collinear points always fit one; \
                     `none` here means the picks did not land where they were aimed, or fewer \
                     than three reached the set.",
                    line.get("r").unwrap_or("absent")
                )),
            }
        }
    }

    // --- C: the panel lists them -------------------------------------------
    //
    // ★ Brought to the FRONT before it is read. A docked pane behind another
    // tab publishes nothing, which is indistinguishable from a panel with
    // nothing to say — the finding `RESUME.md` records after a check spent an
    // evening being reported as a defect in the Properties pane.
    raise_tool_panel(&session, &driver, ui_rect, report)?;
    let trace = session.trace()?;
    if declared(&trace, ui_rect, POINT_LIST).is_none() {
        failures.push(format!(
            "the Tool panel is on screen and declares no `{POINT_LIST}` region, so the picked \
             points are not listed anywhere. That is O107's panel half: the canvas markers say \
             WHERE the points are and cannot say how many, and on a dense sheet a marker on a \
             junction is not distinguishable from the junction. Regions declared under `tool.`: \
             {}.",
            list(&declared_names(&trace, ui_rect, "tool."))
        ));
    }
    let rows = declared_names(&trace, ui_rect, POINT_ROW_PREFIX)
        .into_iter()
        .filter(|n| declared(&trace, ui_rect, n).is_some())
        .collect::<Vec<_>>();
    if rows.len() != aimed.len() {
        failures.push(format!(
            "the panel lists {} row(s) for {} picked point(s): {}. One row per point is what \
             makes the set countable, which is the whole reason the list exists.",
            rows.len(),
            aimed.len(),
            list(&rows)
        ));
    } else {
        report.note(format!(
            "the panel lists {} rows, one per point",
            rows.len()
        ));
    }

    // --- D: a row removes its point ----------------------------------------
    if let Some(row) = rows.first() {
        let Some(rect) = declared(&trace, ui_rect, row) else {
            return Ok(Some(format!("`{row}` was listed and then retired")));
        };
        let before = session.trace()?.events(PICK_EVENT).count();
        driver
            .click_at(driving::frame_of(&session, &trace, ui_rect, row)?.declared_center(rect))?;
        session.settle(16);
        let trace = session.trace()?;
        let lines: Vec<&crate::trace::TraceLine> = trace.events(PICK_EVENT).collect();
        match lines.get(before) {
            Some(line)
                if line.get("via") == Some("panel") && line.get("action") == Some("remove") =>
            {
                report.note(format!(
                    "clicking `{row}` removed that point: n={}",
                    line.get("n").unwrap_or("?")
                ));
                if line.get_usize("n") != Some(aimed.len() - 1) {
                    failures.push(format!(
                        "after removing one row the set holds {}, not {}.",
                        line.get("n").unwrap_or("?"),
                        aimed.len() - 1
                    ));
                }
            }
            Some(line) => failures.push(format!(
                "clicking the row `{row}` traced `{}` rather than a panel removal.",
                line.raw
            )),
            None => failures.push(format!(
                "clicking the row `{row}` traced no `{PICK_EVENT}` line at all, so the row is \
                 drawn and inert — which is exactly the placeholder R9 forbids, and exactly the \
                 shape of the operator's original complaint that a control did nothing."
            )),
        }
    }

    // --- E: the canvas removes one too -------------------------------------
    //
    // The other route O107 asks for. Aimed at the SECOND rim point, which is
    // still in the set after D removed the first.
    let (_, screen) = aimed[1];
    let before = session.trace()?.events(PICK_EVENT).count();
    driver.click_at(screen)?;
    session.settle(14);
    let trace = session.trace()?;
    let lines: Vec<&crate::trace::TraceLine> = trace.events(PICK_EVENT).collect();
    match lines.get(before) {
        Some(line) if line.get("action") == Some("remove") && line.get("via").is_none() => {
            report.note(format!(
                "clicking the canvas on a point already picked removed it: n={}",
                line.get("n").unwrap_or("?")
            ));
        }
        Some(line) => failures.push(format!(
            "clicking the canvas on a point ALREADY in the set traced `{}`. It must remove it — \
             O107, *\"we should be able to unselect points/clicked locations\"* — and a build \
             that adds a second point on top of the first instead is one where a mis-picked \
             point can only be cleared by abandoning the whole measurement.",
            line.raw
        )),
        None => failures.push(format!(
            "clicking the canvas on a point already in the set traced no `{PICK_EVENT}` line."
        )),
    }

    let shot = ctx.out("measure-circular-points.png");
    match crate::capture::window_to_png(&session, &shot) {
        Ok(_) => {
            report.artifact(shot);
        }
        Err(e) => {
            report.note(format!("the window could not be captured ({e})"));
        }
    }

    if failures.is_empty() {
        Ok(None)
    } else {
        Ok(Some(failures.join("  |  ")))
    }
}

/// Put the Tool panel in front, whatever state the dock is in.
///
/// Three cases, and each really occurs:
///
/// 1. **Already the active tab** — its body publishes, nothing to do.
/// 2. **Mounted but behind another tab** — its `dock.tab.tool` header
///    publishes while its body does not. Clicking the header raises it. ★ The
///    ribbon toggle must NOT be used here: it would *unmount* a panel that is
///    already there, and the check would then report an absent list about a
///    panel it closed itself.
/// 3. **Not mounted** — neither region publishes; the View tab's toggle mounts
///    it.
///
/// # Errors
///
/// SKIPs the check when the panel cannot be reached at all, because a check
/// that could not open the surface it reads has learned nothing about it.
fn raise_tool_panel(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    report: &mut CheckReport,
) -> Result<()> {
    let trace = session.trace()?;
    if declared(&trace, ui_rect, TOOL_PANEL_BODY).is_some() {
        report.note("the Tool panel is already the active tab in its dock");
        return Ok(());
    }
    if let Some(tab) = declared(&trace, ui_rect, TOOL_PANEL_TAB) {
        driver.click_at(
            driving::frame_of(session, &trace, ui_rect, TOOL_PANEL_TAB)?.declared_center(tab),
        )?;
        session.settle(16);
        if declared(&session.trace()?, ui_rect, TOOL_PANEL_BODY).is_some() {
            report.note("the Tool panel was behind another tab; raised it by its header");
            return Ok(());
        }
    }

    let trace = session.trace()?;
    let view = declared(&trace, ui_rect, VIEW_TAB).ok_or_else(|| {
        Error::new(format!(
            "no `{VIEW_TAB}` region, so the Tool panel's toggle cannot be reached. Tabs \
             declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(view))?;
    session.settle(14);
    let Some(item) = driving::declared_or_in_overflow(session, driver, ui_rect, TOOL_PANEL_ITEM)?
    else {
        return Err(Error::new(format!(
            "the View tab declares no `{TOOL_PANEL_ITEM}`, so the Tool panel cannot be mounted."
        )));
    };
    driver.click_at(session.frame()?.declared_center(item))?;
    session.settle(20);
    if declared(&session.trace()?, ui_rect, TOOL_PANEL_BODY).is_none() {
        return Err(Error::new(format!(
            "pressing `{TOOL_PANEL_ITEM}` declared no `{TOOL_PANEL_BODY}` region, so the Tool \
             panel did not mount and its picked-point list cannot be read."
        )));
    }
    report.note("mounted the Tool panel from View");
    Ok(())
}
