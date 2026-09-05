//! `a_marquee_over_a_table_takes_its_text_as_well_as_its_lines` — **the
//! operator drew a box round a table and could not move it.**
//!
//! # The report
//!
//! Ken, 2026-09-01, on `TR-0461-1500-copy.pdf`:
//!
//! > *"I can't box select the tables in the left or right top corners using the
//! > mouse — it only picks up the lines of each table, so I can't drag the
//! > entire thing and move it somewhere else, or cut/copy and paste it
//! > elsewhere."*
//!
//! ## ★★★ What "only the lines" would mean, and why it needs measuring
//!
//! A CAD-exported table is two kinds of object drawn in one place: **paths**
//! (the rules and the border) and **text** (every cell's contents). They are
//! separate objects in the content stream and nothing in the file says they
//! belong together.
//!
//! So *"it only picks up the lines"* has three candidate causes and they want
//! opposite fixes:
//!
//! | | what would be wrong | how this check tells |
//! |---|---|---|
//! | the marquee excludes **text objects** | the hit test, or a filter above it | the selection has paths and no text |
//! | the marquee is **`Enclosed`** and the table touches the page edge, so it cannot be surrounded | the gesture, not the hit test | a marquee that fits INSIDE the page selects both |
//! | the selection is right and the **drag** refuses a mixed set | `canvas::moving`, not selection at all | both kinds selected, and no move line |
//!
//! ⇒ This check settles the first two by drawing a band that fits comfortably
//! inside the page around a table that does **not** touch the edge, and asking
//! what came back. A green result here moves the investigation to the third,
//! which is a different module and a different report.
//!
//! ★★ It is deliberately NOT a screenshot. Two objects selected and one object
//! selected draw the same blue outline round the same table; the distinguishing
//! fact is the census, and `canvas-selection` carries it.
//!
//! ## The fixture is the operator's own drawing
//!
//! Copied to scratch first. The check writes nothing — a marquee is a read —
//! but the application persists layout and recent-file state beside whatever it
//! opens, and this project's standing rule is that the suite's side effects do
//! not land in the operator's own folder.

use crate::checks::driving::{SHELL_DIAG_ENV, click_mode_segment, declared, declared_names, list};
use crate::checks::text_selection::aim;
use crate::checks::{Check, CheckContext};
use crate::coords::{DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Content selection needs Edit.
const RIBBON_MODE: &str = "edit";
/// ★★ Single page, fitted, BEFORE anything is aimed at.
///
/// The first run of this check drove a band to screen y=2 — above the canvas
/// entirely — because the file is ten pages shown continuously and the view had
/// been scrolled by the layout it inherited. `aim` faithfully computed where the
/// table WOULD be and the drag went there, off the canvas, selecting nothing.
///
/// A region off the top of the canvas looks exactly like a hit test that
/// excluded everything, which is the fourth instance of that shape in this
/// harness. Fitting the page first makes the aim a statement about the document
/// rather than about the scroll position the run happened to start from.
const INVOKE: &str = "view.page_single,view.zoom_fit_page";
/// The selection census the marquee writes.
const SELECTION: &str = "canvas-selection"; // ui-text-exempt: a trace event name, never displayed
/// The page region, so a failure can say whether a sheet was drawn at all.
const PAGE_REGION: &str = "page"; // ui-text-exempt: a trace region name, never displayed

/// The operator's drawing.
/// The operator's drawing, **by file name** — see [`crate::fixture::operator_file`]
/// for why this stopped being an absolute path on 2026-09-05.
const FIXTURE: &str = "TR-0461-1500-copy.pdf";
/// Its page, in points — 1224 × 792, measured off a 1.2× raster.
const FIXTURE_PAGE: PageGeometry = PageGeometry {
    width_pt: 1224.0,
    height_pt: 792.0,
};

/// **Where the band STARTS, and it is on measured-empty paper.**
///
/// # ★★★ This constant is the whole repair, 2026-09-02
///
/// The previous origin was `(14, 618)` — just outside the table, "comfortably
/// inside the sheet", and **on top of an object**. Three runs of this check
/// reported *"THE BAND SELECTED NOTHING AT ALL"*, which reads as a hit test
/// that excluded everything. It was not. The trace carried
/// `selection-set page=0 object=23 via=press` and **no marquee line of any
/// kind**: the press had selected the object under it and the drag had become a
/// MOVE.
///
/// `canvas::presspick` documents that behaviour and its first stated
/// non-disturbance is the rule this check broke: *"A press on empty paper still
/// marquees."* Pressing on ink does not.
///
/// ★★ And "empty" is a much larger radius than it looks. The pick tolerance is
/// `SELECT_SCREEN_TOLERANCE_PX` converted to page units, so at the fitted zoom
/// this check drives (about 0.38×) a **4-pixel** screen tolerance is over **ten
/// page points**. The old origin sat 6 pt from the sheet border — visually in
/// the margin, and inside the catch radius.
///
/// ★ Chosen by rendering page 1 at scale 1.0 (1 px = 1 pt) and looking: this
/// point is in the blank field below the INSPECTION STATUS table and left of
/// the isometric view, about **80 pt** from the nearest ink in any direction.
const BAND_FROM: (f64, f64) = (300.0, 560.0);

/// Where it ENDS — up and to the **left**, inside the table.
///
/// ★★★ Right-to-left, so this is a **crossing window** and takes anything it
/// touches. That is the point: it is the gesture `OPERATOR_REQUESTS.md` O88
/// added, and it is the only one that can reach this table at all.
///
/// An **enclosing** band cannot be driven here and the reason is the operator's
/// own complaint rather than a harness limitation: to surround a table hard
/// against the sheet edge the band must start outside the page, and every
/// corner from which it could be started is on ink. So the window direction is
/// deliberately **not** driven by this check, and that absence is reported
/// rather than left for a reader of a PASS to assume away.
const BAND_TO: (f64, f64) = (60.0, 765.0);

/// `marquee-mode crossing=… mode=… hits=…` — the shell's record of WHICH
/// rule the band was resolved by.
///
/// ★★ Asserted as well as the selection census, and the pair is the point. A
/// build that ignored the drag direction and ran `Enclosed` for everything
/// would select nothing here and fail on the count — but so would a build
/// whose hit test was broken outright, and the two want opposite fixes. This
/// line separates them.
const MODE: &str = "marquee-mode";

pub struct AMarqueeOverATableTakesItsTextAsWellAsItsLines;

impl Check for AMarqueeOverATableTakesItsTextAsWellAsItsLines {
    fn name(&self) -> &'static str {
        "a_marquee_over_a_table_takes_its_text_as_well_as_its_lines"
    }

    fn defect(&self) -> &'static str {
        "a box drawn round a table selects its rules and not its words, so dragging the \
         selection moves the grid and leaves the contents behind — and a cut takes half a table"
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
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). The subject is a rubber-band drag.",
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
            "{}. This check is about a table in THAT file; a substitute would be measuring a different document.",
            crate::fixture::operator_file_complaint(FIXTURE)
        )));
    };
    // A scratch copy. A marquee writes nothing, but the application persists
    // layout and recent-file state beside what it opens.
    let pdf = ctx.out("marquee-table.pdf");
    if let Some(dir) = pdf.parent() {
        std::fs::create_dir_all(dir).map_err(|e| Error::new(e.to_string()))?;
    }
    std::fs::copy(&source, &pdf).map_err(|e| Error::new(e.to_string()))?;

    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let page = crate::fixture::page_geometry(&pdf).unwrap_or(FIXTURE_PAGE);

    let mut spec = LaunchSpec::new(&exe, ctx.out("marquee-table.trace.txt"));
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
        "launched {} as pid {} on a scratch copy of the operator's drawing",
        exe.display(),
        session.pid()
    ));
    session.settle(50);
    let driver = Driver::new(session.window());

    click_mode_segment(&session, &driver, ui_rect, RIBBON_MODE)?;
    if declared(&session.trace()?, ui_rect, PAGE_REGION).is_none() {
        return Err(Error::new(format!(
            "no `{PAGE_REGION}` region, so no sheet is on screen. Regions beginning `page`: {}.",
            list(&declared_names(&session.trace()?, ui_rect, "page"))
        )));
    }

    // ★★★ **ARM THE SELECT TOOL FIRST**, and this line is the repair that made
    // this check able to run at all — 2026-09-02.
    //
    // Its first three runs reported "THE BAND SELECTED NOTHING AT ALL: no
    // `canvas-selection … via=pv.marquee` line", which reads as a hit test
    // that excluded everything. It was not: the trace carried **no**
    // `canvas-selection` line of any kind and only sixteen `canvas-pointer`
    // ones, so no rubber band had ever begun. The press belongs to whichever
    // tool is armed, and nothing here had armed one.
    //
    // ★★ Two of the three failures were previously written off as the harness
    // "driving the band above the canvas" — true of the first run, and it
    // masked this. A check that fails for two different reasons in two runs is
    // one whose SECOND diagnosis nobody looked for.
    //
    // A `false` return means the pointer route to the tool is unavailable, which
    // is a SKIP rather than a failure: the band could not be started, so nothing
    // about what a band selects was measured.
    if !crate::checks::driving::arm_select_from_ribbon(&session, &driver, ui_rect, report)? {
        return Err(Error::new(
            "the select tool could not be armed from the ribbon, so no rubber band could be \
             started. Nothing about what a band selects was measured.",
        ));
    }

    // --- the band ------------------------------------------------------------
    let from = aim(
        ctx,
        &session,
        page,
        DocPoint::new(0, BAND_FROM.0, BAND_FROM.1),
    )?;
    let to = aim(ctx, &session, page, DocPoint::new(0, BAND_TO.0, BAND_TO.1))?;
    report.note(format!(
        "band ({:.0}, {:.0}) → ({:.0}, {:.0}) in page points — right to left, so a CROSSING \
         window, started on measured-empty paper about 80 pt from the nearest ink",
        BAND_FROM.0, BAND_FROM.1, BAND_TO.0, BAND_TO.1
    ));
    driver.drag(from, to)?;
    session.settle(60);

    let trace = session.trace()?;
    let Some(line) = trace
        .events(SELECTION)
        .filter(|l| l.get("via") == Some("pv.marquee"))
        .last()
    else {
        return Ok(Some(format!(
            "★★★ NO RUBBER BAND EVER BEGAN: no `{SELECTION} via=pv.marquee` line followed the \
             drag.\n\n\
             ★★ Before reading this as a broken hit test, look for `selection-set via=press` \
             in the trace. That is what it was for the three runs before 2026-09-02: the press \
             landed on an object, `canvas::presspick` selected it, and the drag became a MOVE. \
             That module's own header states the rule — *\"a press on empty paper still \
             marquees\"* — and \"empty\" is wider than it looks, because the pick tolerance is \
             over ten PAGE points at this zoom. If that line is present, the band's origin is \
             on ink and this check is aimed wrongly rather than failing.\n\n\
             If it is absent, the band genuinely did not start: check that the Select tool is \
             armed, and that nothing registered above the page widget claimed the press. \
             Trace: {}.",
            session.trace_path().display()
        )));
    };
    // ★★★ **WHICH RULE RESOLVED IT** — asserted before the census, and the pair
    // is the point.
    //
    // A build that ignored the drag direction and ran `Enclosed` for everything
    // would select nothing here and fail on the count below — but so would a
    // build whose hit test was broken outright, and the two want opposite
    // fixes. This line separates them, and it is the only evidence that O88's
    // direction rule is wired at all rather than merely written.
    let Some(mode) = trace.events(MODE).last() else {
        return Ok(Some(format!(
            "★★ THE BAND SELECTED, AND NEVER SAID WHICH RULE IT USED: a \
             `{SELECTION} via=pv.marquee` line is present and no `{MODE}` line is. \
             `canvas::marquee::select` emits one on every band, so its absence means the \
             selection reached the state by some other route — which this check cannot \
             interpret. Trace: {}.",
            session.trace_path().display()
        )));
    };
    if mode.get("crossing") != Some("true") || mode.get("mode") != Some("touched") {
        return Ok(Some(format!(
            "★★★ A RIGHT-TO-LEFT DRAG WAS NOT READ AS A CROSSING WINDOW: `{}`.\n\n\
             The band was dragged from x={:.0} to x={:.0}, which is leftward, so `crossing` \
             must be `true` and the mode `touched`. AutoCAD's rule, and O88's whole subject. \
             If `crossing=false`, look at `Drag::outcome` — the comparison is \
             `latest.x < origin.x` and it is deliberately strict, so a perfectly vertical drag \
             is a window. If `crossing=true` and the mode is `enclosed`, `mode_for` and this \
             arm disagree.",
            mode.raw, BAND_FROM.0, BAND_TO.0
        )));
    }
    report.note(format!("★ the direction was read: `{}`", mode.raw));

    let count: usize = line
        .get("sel")
        .and_then(|s| s.parse().ok())
        .unwrap_or_default();
    report.note(format!(
        "★ the band selected {count} object(s): `{}`",
        line.raw
    ));

    if count == 0 {
        return Ok(Some(format!(
            "★★★ THE BAND ENCLOSED THE TABLE AND SELECTED NOTHING: `{}`.\n\
             Both corners of the band are well inside the sheet, so this is not the operator's \
             page-edge case. Look at `hit_test_rect` and at whether `MarqueeMode::Enclosed` is \
             being asked about objects whose bounds are in a different space. Trace: {}.",
            line.raw,
            session.trace_path().display()
        )));
    }

    // ★★★ **THE ORACLE IS THE KINDS, NOT THE COUNT** — rewritten 2026-09-02,
    // and the previous oracle was an assumption that measurement refuted.
    //
    // It read: *"a table's rules are ONE path object per line in most CAD
    // exports and its words are one text object per cell, so a band over this
    // table should return well into double figures"*, and failed anything under
    // four.
    //
    // ★★ Measured on this very sheet: `objects n=25 paths=19 text=6`. The
    // **whole drawing** — two tables, a title block, an isometric view, dozens
    // of labels — is twenty-five objects. A band returning three is a large
    // fraction of the page, and the threshold was rejecting a correct result
    // while calling it the operator's defect.
    //
    // ★★★ And it could never have expressed his complaint anyway. *"It only
    // picks up the lines of each table"* is a claim about a **kind being
    // missing**. One path and one text is a pass; nine paths and no text is the
    // defect — and a count ranks those two the wrong way round at every
    // threshold.
    //
    // So `canvas::marquee::select` now reports the breakdown from the
    // provider's own classifier, and this is the assertion:
    let paths = mode.get_usize("paths").unwrap_or_default();
    let text = mode.get_usize("text").unwrap_or_default();
    report.note(format!(
        "★ the band took {paths} path(s) and {text} text object(s) of {count} — \
         against a page that decomposes into 25 objects in total, 19 paths and 6 \
         text"
    ));
    if paths == 0 {
        return Ok(Some(format!(
            "THE BAND TOOK NO PATHS: `{}`. A band across a ruled table that returns no path \
             object has missed the rules, which are the one thing the operator said it DID \
             pick up.",
            mode.raw
        )));
    }
    if text == 0 {
        return Ok(Some(format!(
            "★★★ THE BAND TOOK LINES AND NO TEXT: `{}`.\n\n\
             This is the operator's report reproduced exactly — *\"it only picks up the lines \
             of each table, so I can't drag the entire thing and move it somewhere else\"*. The \
             rules came and the words did not, so a drag would move the grid and leave the \
             contents behind, and a cut would take half a table.\n\n\
             A CAD table is paths AND text as separate page objects and the engine's marquee \
             filters by neither. Look first at the PICK FILTER — `PickClass::Text` can be \
             switched off from the status bar and is persisted between sessions, so a filter \
             set once will keep this failing. Then at whether the marquee is asking the same \
             filter a click asks. Trace: {}.",
            mode.raw,
            session.trace_path().display()
        )));
    }

    report.note(
        "★★★ …so the band takes a table's words as well as its rules. The kind that \
         was reported missing is present, which moves the operator's remaining \
         complaint downstream of selection — to the drag itself, or to the fact that \
         an ENCLOSING band cannot surround a table hard against the sheet edge, which \
         is what the crossing window this check drives exists to solve",
    );
    Ok(None)
}
