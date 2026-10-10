//! `an_end_point_inside_a_wrapped_drawing_can_be_dragged` — a line inside a
//! form XObject (what "Make part of the page" makes of a markup) has its end
//! point dragged, and the engine moves it.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/form_node_move.md`.

use crate::checks::driving::{self, SHELL_DIAG_ENV, click_mode_segment};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry, WindowPoint};
use crate::error::{Error, Result};
use crate::geom::{LRect, Pt};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const MODE: &str = "edit";
const SELECTION: &str = "canvas-selection"; // ui-text-exempt: a trace event name, never displayed
const ANCHORS: &str = "canvas-anchors"; // ui-text-exempt: a trace event name, never displayed
const MOVE: &str = "canvas-move"; // ui-text-exempt: a trace event name, never displayed
const MOVED: &str = "move-node-in-form"; // ui-text-exempt: a trace event name, never displayed
const PREVIEW: &str = "canvas-shape-preview"; // ui-text-exempt: a trace event name, never displayed
/// The first drawn anchor mark. Mirrors `canvas::overlay`'s region names.
const FIRST_ANCHOR: &str = "canvas.anchor.0";
/// The selected anchor. Mirrors `canvas::overlay::SELECTED_ANCHOR_REGION`.
const SELECTED_ANCHOR: &str = "canvas.selected-anchor";
const PANEL: &str = "properties-panel"; // ui-text-exempt: a trace event name, never displayed
/// Mirrors `panels::properties::REGION_LEAF`.
const LEAF_REGION: &str = "properties.leaf";

const FIXTURE: &str = "../../fixtures/form-xobject.pdf";
const FIXTURE_PAGE: PageGeometry = PageGeometry {
    width_pt: 400.0,
    height_pt: 300.0,
};
/// On the horizontal bar (page space 60..340 at y=150), clear of the others.
const ON_THE_BAR: (f64, f64) = (100.0, 150.0);
/// How far the end point is dragged, in window points, on each axis.
const DRAG_PX: f32 = 30.0;
/// The window, placed off the desktop.
const OFFSCREEN: &str = "-4200,-4200,1400,900";

pub struct AnEndPointInsideAWrappedDrawingCanBeDragged;

impl Check for AnEndPointInsideAWrappedDrawingCanBeDragged {
    fn name(&self) -> &'static str {
        "an_end_point_inside_a_wrapped_drawing_can_be_dragged"
    }

    fn defect(&self) -> &'static str {
        "an end point of a line inside a wrapped drawing (a markup made part of the page) can be \
         picked and dragged, the shape previews, and the release changes nothing"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive(ctx, &mut report, descend_and_drag) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

pub struct ALineInsideAPlacedDrawingShowsItsProperties;

impl Check for ALineInsideAPlacedDrawingShowsItsProperties {
    fn name(&self) -> &'static str {
        "a_line_inside_a_placed_drawing_shows_its_properties"
    }

    fn defect(&self) -> &'static str {
        "a line inside a placed drawing (a markup made part of the page) is selected and the \
         Properties panel shows nothing for it"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive(ctx, &mut report, enter_and_read_properties) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn enter_and_read_properties(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    page: PageGeometry,
) -> Result<Option<String>> {
    let (ui_rect, _) = enter_leaf(ctx, session, pointer, page)?;
    session.settle(20);
    let trace = session.trace()?;
    let described = trace
        .events(PANEL)
        .filter(|l| l.get("leaf").is_some())
        .last();
    let Some(line) = described else {
        return Ok(Some(format!(
            "THE LEAF IS SELECTED AND PROPERTIES DESCRIBES NOTHING: no `{PANEL} leaf=` line \
             (last `{PANEL}`: {}). `panels::properties::leaf_section` must describe \
             `PageObjects::leaves[i]` when the selection holds no page object. Trace: {}.",
            trace.last(PANEL).map_or("none", |l| l.raw.as_str()),
            session.trace_path().display()
        )));
    };
    if line.get("kind") != Some("Path") {
        return Ok(Some(format!(
            "Properties described the leaf as something other than a path: `{}`. The bar is a \
             stroked line. Trace: {}.",
            line.raw,
            session.trace_path().display()
        )));
    }
    if driving::declared(&trace, ui_rect, LEAF_REGION).is_none() {
        return Ok(Some(format!(
            "the leaf was described and `{LEAF_REGION}` was never on screen: the section drew \
             outside the panel's visible area. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note(format!("Properties described the leaf: `{}`", line.raw));
    Ok(None)
}

/// Launch off the desktop on a copy of `fixture` (relative to this crate),
/// with the trace and pointer files named after `stem`.
pub(super) fn launch(
    ctx: &CheckContext,
    report: &mut CheckReport,
    fixture: &str,
    stem: &str,
    invoke: Option<&str>,
) -> Result<(Session, ScriptedPointer, PageGeometry)> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let pdf = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(fixture);
    if !pdf.is_file() {
        return Err(Error::new(format!(
            "the fixture {fixture} is missing. Its PROVENANCE file beside it builds it."
        )));
    }
    let page = crate::fixture::page_geometry(&pdf).unwrap_or(FIXTURE_PAGE);
    let doc = ctx.out(&format!("{stem}.pdf"));
    std::fs::copy(&pdf, &doc).map_err(|e| Error::new(format!("copying {fixture}: {e}")))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out(&format!("{stem}.trace.txt")));
    spec.pdf = Some(doc);
    for (k, v) in [
        (ctx.profile.diag_env.0, ctx.profile.diag_env.1),
        SHELL_DIAG_ENV,
        (viewport_env, OFFSCREEN),
    ] {
        spec.env.push((k.to_owned(), v.to_owned()));
    }
    if let Some(invoke) = invoke {
        spec.env
            .push(("PDFCER_DIAG_INVOKE".to_owned(), invoke.to_owned()));
    }
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out(&format!("{stem}.pointer.txt")))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    report.note(format!(
        "launched {} as pid {}",
        exe.display(),
        session.pid()
    ));
    session.settle(45);
    Ok((session, pointer, page))
}

/// What a check does once the window is up; `Some` is a FAIL.
pub(super) type Body = fn(
    &CheckContext,
    &mut CheckReport,
    &Session,
    &ScriptedPointer,
    PageGeometry,
) -> Result<Option<String>>;

fn drive(ctx: &CheckContext, report: &mut CheckReport, body: Body) -> Result<Option<String>> {
    run_body(ctx, report, FIXTURE, "form-node-move", body)
}

/// Launch on `fixture`, run `body`, then park the pointer whatever it found.
pub(super) fn run_body(
    ctx: &CheckContext,
    report: &mut CheckReport,
    fixture: &str,
    stem: &str,
    body: Body,
) -> Result<Option<String>> {
    run_body_with(ctx, report, fixture, stem, None, body)
}

/// [`run_body`], with `invoke` rung through `PDFCER_DIAG_INVOKE` at launch
/// (for example `file.properties`, to open the Properties panel).
pub(super) fn run_body_with(
    ctx: &CheckContext,
    report: &mut CheckReport,
    fixture: &str,
    stem: &str,
    invoke: Option<&str>,
    body: Body,
) -> Result<Option<String>> {
    let (session, pointer, page) = launch(ctx, report, fixture, stem, invoke)?;
    let outcome = body(ctx, report, &session, &pointer, page);
    let parked = pointer.gone(&session);
    let found = outcome?;
    parked?;
    Ok(found)
}

/// Edit mode, then a click and a double-click on the bar: the selection is
/// the bar, a leaf of the form. Returns the `ui-rect` event name and the
/// point clicked.
fn enter_leaf(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    page: PageGeometry,
) -> Result<(&'static str, WindowPoint)> {
    enter_leaf_at(ctx, session, pointer, page, ON_THE_BAR)
}

/// Edit mode, then a click and a double-click at `on` (page space): the
/// selection is the leaf of the form painted there.
pub(super) fn enter_leaf_at(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    page: PageGeometry,
    on: (f64, f64),
) -> Result<(&'static str, WindowPoint)> {
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    click_mode_segment(session, pointer, ui_rect, MODE)?;
    session.settle(20);
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, page, 0)?;
    let at = mapping.doc_to_window(DocPoint::new(0, on.0, on.1))?;
    pointer.click(session, at)?;
    session.settle(25);
    pointer.double_click(session, at)?;
    session.settle(30);
    let trace = session.trace()?;
    let inside = trace
        .last(SELECTION)
        .and_then(|l| l.get("first").map(str::to_owned))
        .is_some_and(|first| first.starts_with("leaf:"));
    if !inside {
        return Err(Error::new(format!(
            "the descent left no leaf selected; that is \
             `a_click_selects_the_whole_drawing_and_a_double_click_goes_inside`'s subject. \
             Trace: {}.",
            session.trace_path().display()
        )));
    }
    Ok((ui_rect, at))
}

fn descend_and_drag(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    page: PageGeometry,
) -> Result<Option<String>> {
    // A second double-click at the bar enters its Part rung, which publishes
    // the anchor marks.
    let (ui_rect, at) = enter_leaf(ctx, session, pointer, page)?;
    pointer.double_click(session, at)?;
    session.settle(30);

    let trace = session.trace()?;
    let anchor = driving::declared(&trace, ui_rect, FIRST_ANCHOR).ok_or_else(|| {
        Error::new(format!(
            "no `{FIRST_ANCHOR}` after descending into the bar, so there is no end point to aim \
             at (last `{ANCHORS}`: {}). Trace: {}.",
            trace.last(ANCHORS).map_or("none", |l| l.raw.as_str()),
            session.trace_path().display()
        ))
    })?;
    pointer.double_click(session, WindowPoint::centre_of(anchor))?;
    session.settle(20);

    let trace = session.trace()?;
    let Some(picked) = driving::declared(&trace, ui_rect, SELECTED_ANCHOR) else {
        return Err(Error::new(format!(
            "the double-click on an end point selected no anchor (last `{SELECTION}`: {}). \
             Trace: {}.",
            trace.last(SELECTION).map_or("none", |l| l.raw.as_str()),
            session.trace_path().display()
        )));
    };
    report.note("an end point of the line inside the form is selected");
    let mark = trace.mark();
    let from = WindowPoint::centre_of(picked);
    let to = WindowPoint::centre_of(LRect::new(
        Pt::new(picked.min.x + DRAG_PX, picked.min.y + DRAG_PX),
        Pt::new(picked.max.x + DRAG_PX, picked.max.y + DRAG_PX),
    ));
    pointer.drag(session, from, to, 8)?;
    session.settle(40);

    let trace = session.trace()?;
    let previewed = trace
        .events(PREVIEW)
        .filter_map(|l| l.get_usize("shapes"))
        .max()
        .unwrap_or(0);
    let raised = trace.last_after(MOVE, mark);
    let Some(applied) = trace.last_after(MOVED, mark) else {
        return Ok(Some(format!(
            "THE END POINT WAS DRAGGED AND NOTHING APPLIED: no `{MOVED}` line ({previewed} \
             shape(s) previewed; `{MOVE}` after the drag: {}). `canvas::moving::drag` must give \
             `MoveSubject::NodeInForm` the anchor's position from \
             `object_node_points_of(TargetId::Leaf)`; without it `action` refuses with \
             `Refusal::NodeNotFound`, which is silent. Trace: {}.",
            raised.map_or("none", |l| l.raw.as_str()),
            session.trace_path().display()
        )));
    };
    if applied.get_usize("n") != Some(1) {
        return Ok(Some(format!(
            "the move sent the wrong operand count: `{}`. One anchor was selected. Trace: {}.",
            applied.raw,
            session.trace_path().display()
        )));
    }
    report.note(format!("the engine moved the end point: `{}`", applied.raw));
    Ok(None)
}
