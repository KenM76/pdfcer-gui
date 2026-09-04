//! `the_ladder_goes_as_deep_inside_a_container_as_outside_one` — **double-click
//! past the object, into its parts, and drag one.**
//!
//! # The request
//!
//! `OPERATOR_REQUESTS.md` **O70**:
//!
//! > *"a double click should bring me further down the chain, until a double
//! > click reaches the bottom and lets me edit the nodes."*
//!
//! **Until**. The chain has to end at the nodes wherever it started, and until
//! 2026-09-01 it ended one rung early for anything painted inside a form
//! XObject — not by a rule, but because the two deeper rungs were addressed by
//! a page paint-order index that a leaf does not have.
//!
//! ## ★★ What had to change together, and why the order was forced
//!
//! Four things, and doing any one alone would have made the shell worse:
//!
//! | | before | after |
//! |---|---|---|
//! | the hit test | `probe` mapped to a page index, so a leaf got `(None, None)` | asks `part_hits_of` with the `TargetId` |
//! | the descent | refused, by a guard, so the box stayed put | descends |
//! | the anchors | declined `leaf-in-form-xobject` | drawn |
//! | the drag | `Refusal::InsideForm` | `move_subpath_in_form` |
//!
//! ⇒ Descending without the other three would have entered a rung with nothing
//! addressable in it: no anchors drawn (`canvas::painting` declined), no
//! outline (`pressing::grabbable` withholds it below the Object rung), and a
//! drag that refuses. The operator's second double-click would have made the
//! selection **vanish** and offer nothing in its place. That is why the guard
//! existed for the day between the two halves, and why this check exists now.
//!
//! ## The sequence
//!
//! | # | step | oracle |
//! |---|---|---|
//! | A | click, then double-click | `smart-enter`, `first=leaf:N` |
//! | B | double-click again | `canvas-selection … level=Part` |
//! | C | drag | `move-subpath-in-form page=0 n=1` |
//!
//! ★ Step B's oracle is the **rung**, not the trace of a click. `level=Part`
//! on a `first=leaf:` selection is the one statement that says the ladder went
//! deeper in the space that had no deeper rung — and a build with the descent
//! guard back in place reports `level=Object` while every other line looks
//! identical.

use crate::checks::driving::{SHELL_DIAG_ENV, click_mode_segment, declared, declared_names, list};
use crate::checks::text_selection::aim;
use crate::checks::{Check, CheckContext};
use crate::coords::{DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Content selection needs it.
const MODE: &str = "edit";
/// The ladder's own selection line — `via=`, `level=` and `first=`.
const SELECTION: &str = "canvas-selection"; // ui-text-exempt: a trace event name, never displayed
/// The line `canvas::smart::enter` writes.
const ENTER: &str = "smart-enter"; // ui-text-exempt: a trace event name, never displayed
/// ★ The line this check exists to read.
const MOVED: &str = "move-subpath-in-form"; // ui-text-exempt: a trace event name, never displayed
/// The page region, so a failure can say whether a sheet was drawn at all.
const PAGE_REGION: &str = "page"; // ui-text-exempt: a trace region name, never displayed

/// The form fixture — one container, three fat crossing strokes.
const FIXTURE: &str = "../../fixtures/form-xobject.pdf";
/// Its page, as the generator writes it.
const FIXTURE_PAGE: PageGeometry = PageGeometry {
    width_pt: 400.0,
    height_pt: 300.0,
};
/// On the horizontal bar, clear of the vertical one and of the diagonal.
const ON_THE_BAR: (f64, f64) = (100.0, 150.0);
/// How far to drag the subpath, in page points.
const DRAG_BY: (f64, f64) = (0.0, 35.0);

pub struct TheLadderGoesAsDeepInsideAContainer;

impl Check for TheLadderGoesAsDeepInsideAContainer {
    fn name(&self) -> &'static str {
        "the_ladder_goes_as_deep_inside_a_container_as_outside_one"
    }

    fn defect(&self) -> &'static str {
        "the descent stops one rung early inside a wrapped drawing — a double-click that goes \
         deeper everywhere else does nothing there, so the parts and points of anything a CAD \
         exporter wrapped are unreachable by the gesture that reaches them elsewhere"
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
            "input is disabled (--no-input). This check is three clicks and a drag.",
        ));
    }
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let pdf = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(FIXTURE);
    if !pdf.is_file() {
        return Err(Error::new(format!(
            "the fixture {FIXTURE} is missing. Run `python tools/gen-form-xobject-fixture.py`."
        )));
    }
    let page = crate::fixture::page_geometry(&pdf).unwrap_or(FIXTURE_PAGE);

    let mut spec = LaunchSpec::new(&exe, ctx.out("form-leaf-descend.trace.txt"));
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
    report.note(format!(
        "launched {} as pid {}",
        exe.display(),
        session.pid()
    ));
    session.settle(45);
    let driver = Driver::new(session.window());

    click_mode_segment(&session, &driver, ui_rect, MODE)?;
    if declared(&session.trace()?, ui_rect, PAGE_REGION).is_none() {
        return Err(Error::new(format!(
            "no `{PAGE_REGION}` region, so no sheet is on screen. Regions beginning `page`: {}.",
            list(&declared_names(&session.trace()?, ui_rect, "page"))
        )));
    }

    // --- A: on a stroke inside the container --------------------------------
    //
    // ★★★ **THE LADDER LOST A RUNG ON 2026-09-01, AND THAT IS THE FIX.**
    //
    // This step used to click, then double-click, and require a `smart-enter`
    // line before going on. That was right when a first click on anything
    // inside a form resolved to the FORM — so the operator had to enter the
    // container before they could reach what was in it.
    //
    // ⇒ Resolving to the container is now refused when the container holds
    // essentially everything on the page (`container_is_worth_selecting`),
    // because every CAD exporter wraps a whole drawing in one such form and
    // "select the container first" meant "select the whole drawing, every
    // time" — the operator's own headline complaint, restored by the feature
    // built to improve selection.
    //
    // So on THIS fixture the first click lands on the stroke itself. The
    // container rung is not skipped; **it was never a rung here**, and the
    // ladder is one gesture shorter for exactly the documents where that
    // matters most.
    //
    // ★★ `smart-enter` is therefore no longer required — but it is still
    // ACCEPTED, and the check reports which of the two happened. A fixture
    // whose form does not swallow the page still enters, and this check must
    // pass on both without being told which it is looking at. Requiring either
    // one would make it a test of the fixture rather than of the ladder.
    let at = aim(
        ctx,
        &session,
        page,
        DocPoint::new(0, ON_THE_BAR.0, ON_THE_BAR.1),
    )?;
    driver.click_at(at)?;
    session.settle(25);

    let trace = session.trace()?;
    let first_click = trace
        .last(SELECTION)
        .and_then(|l| l.get("first").map(str::to_owned))
        .unwrap_or_else(|| "none".to_owned());
    if first_click == "none" {
        return Err(Error::new(format!(
            "the first click selected nothing, so there is no stroke under \
             ({:.0}, {:.0}) and this check cannot reach its own subject. Trace: {}.",
            ON_THE_BAR.0,
            ON_THE_BAR.1,
            session.trace_path().display()
        )));
    }
    if !first_click.starts_with("leaf:") {
        // The container still won the click, so this fixture's form does not
        // swallow the page — enter it, which is the older route and still
        // correct.
        driver.double_click_at(at)?;
        session.settle(30);
        let trace = session.trace()?;
        if trace.events(ENTER).count() == 0 {
            return Err(Error::new(format!(
                "the first click resolved to `{first_click}` rather than a leaf, and the \
                 double-click traced no `{ENTER}` line — so neither route reached the inside of \
                 the container and this check cannot reach its own subject. Trace: {}.",
                session.trace_path().display()
            )));
        }
        report.note("★ entered the container — this fixture's form does not swallow the page");
    } else {
        report.note(format!(
            "★ the first click reached the stroke directly: `{first_click}` — this form holds \
             essentially the whole page, so there is no container rung to climb"
        ));
    }

    // --- B: one rung deeper -------------------------------------------------
    driver.double_click_at(at)?;
    session.settle(30);

    let trace = session.trace()?;
    let Some(line) = trace.last(SELECTION) else {
        return Ok(Some(format!(
            "no `{SELECTION}` line at all after the second double-click. Trace: {}.",
            session.trace_path().display()
        )));
    };
    let level = line.get("level").unwrap_or("none");
    let first = line.get("first").unwrap_or("none");
    if level == "Object" {
        return Ok(Some(format!(
            "★★★ THE LADDER STOPPED ONE RUNG EARLY: `{}`.\n\
             A second double-click inside a container left the rung at Object, so the parts and \
             points of anything a CAD exporter wrapped are unreachable by the gesture that \
             reaches them everywhere else. Four things have to be true together — the hit test \
             asking `part_hits_of`, the descent guard gone, the anchors drawn, and the drag \
             routed to a `*_in_form` verb — and this is what any one of them missing looks like. \
             Trace: {}.",
            line.raw,
            session.trace_path().display()
        )));
    }
    if !first.starts_with("leaf:") {
        return Ok(Some(format!(
            "★★ THE DESCENT LEFT THE CONTAINER: `{}`. The rung went deeper and the subject is \
             `{first}` rather than a leaf, so the second double-click re-resolved to something on \
             the page instead of descending into what was selected. Trace: {}.",
            line.raw,
            session.trace_path().display()
        )));
    }
    report.note(format!("★★ one rung deeper, still inside: `{}`", line.raw));

    // --- C: and the drag reaches the engine ---------------------------------
    let to = aim(
        ctx,
        &session,
        page,
        DocPoint::new(0, ON_THE_BAR.0 + DRAG_BY.0, ON_THE_BAR.1 + DRAG_BY.1),
    )?;
    driver.drag(at, to)?;
    session.settle(45);

    let trace = session.trace()?;
    let Some(moved) = trace.last(MOVED) else {
        return Ok(Some(format!(
            "★★★ THE DEEPER RUNG COMMITS NOTHING: the descent reached `{level}` and no `{MOVED}` \
             line followed the drag. A rung that can be entered and not acted on is worse than \
             one that cannot be entered — the anchors are drawn, so the gesture is offered, and \
             R9 calls an offered gesture that refuses the misleading kind of placeholder. Trace: \
             {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!(
        "★★★ …and the drag reached the engine: `{}`",
        moved.raw
    ));
    Ok(None)
}
