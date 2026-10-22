//! `forms_style_sets_the_next_field` — Forms ▸ Style's width reaches the
//! next placed field, and the field's right-click *Use as default look* reads
//! it back from the document.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/forms_style.md`.

use crate::checks::driving::{
    SHELL_DIAG_ENV, click_mode_segment, declared, declared_names, declared_or_in_overflow, list,
    repo_fixture,
};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, WindowPoint};
use crate::error::{Error, Result};
use crate::geom::{LRect, Pt};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const FIXTURE: &str = "layer-assign.pdf";
const METHOD: &str = "Rebuild it with `python fixtures/layer-assign.PROVENANCE.py`.";
const OFFSCREEN: &str = "-4200,-4200,1400,900";
/// Makes the placement dialog accept itself, as every placing check does.
const ACCEPT_ENV: (&str, &str) = ("PDFCER_DIAG_FORM_ACCEPT", "1");
const FORMS_TAB: &str = "ribbon.tab.forms";
const WIDTH: &str = "forms.style.width";
const TEXT_FIELD: &str = "ribbon.item.edit.form_text_field";
const MENU_ROW: &str = "menu.item.canvas.field.edit.form_use_as_default";
/// Written by the Style band on every change.
const LOOK: &str = "form-look";
/// Written by *Use as default look*, carrying the placed field's own width.
const TAKEN: &str = "form-look-taken";
/// The placement dialog opening, with the field's name and size.
const OPENED: &str = "form-field-open";
/// The engine authored the field.
const ADDED: &str = "add-form-field";
/// Empty paper on the fixture, in page points; `layer_assign` puts a caret here.
const SPOT: (f64, f64) = (150.0, 450.0);
/// Pixels the width is dragged; at the band's 0.1 pt per pixel, about 4 pt.
const DRAG_PX: f32 = 40.0;

/// See the module documentation.
pub struct FormsStyleSetsTheNextField;

impl Check for FormsStyleSetsTheNextField {
    fn name(&self) -> &'static str {
        "forms_style_sets_the_next_field"
    }

    fn defect(&self) -> &'static str {
        "the Forms tab's Style group does not reach the next placed field — the band is not \
         drawn, a change to it is not remembered, the placed field ignores it — or the field's \
         right-click Use as default look does not read the field's look back"
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

fn launch(ctx: &CheckContext, report: &mut CheckReport) -> Result<(Session, ScriptedPointer)> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("forms_style.trace.txt"));
    spec.pdf = Some(repo_fixture(FIXTURE, METHOD)?);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push((viewport_env.to_owned(), OFFSCREEN.to_owned()));
    spec.env
        .push((ACCEPT_ENV.0.to_owned(), ACCEPT_ENV.1.to_owned()));
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("forms_style.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    report.note(format!("launched as pid {}", session.pid()));
    session.settle(40);
    Ok((session, pointer))
}

/// The `width=` of the last `event` line after `mark`.
fn width_after(session: &Session, event: &str, mark: usize) -> Result<Option<(f32, String)>> {
    Ok(session
        .trace()?
        .last_after(event, mark)
        .and_then(|l| l.get_f32("width").map(|w| (w, l.raw.clone()))))
}

/// `Ok(Err(why))` is the check's failure; `Err` is a harness error.
type Step<T> = Result<std::result::Result<T, String>>;

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let (session, pointer) = launch(ctx, report)?;
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile names no ui-rect trace event."))?;
    click_mode_segment(&session, &pointer, ui_rect, "edit")?;
    session.settle(20);
    let pdf = repo_fixture(FIXTURE, METHOD)?;
    let page = crate::fixture::page_geometry(&pdf)
        .ok_or_else(|| Error::new("could not read a page size from the fixture."))?;
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, page, 0)?;
    let (set, look) = match set_width(&session, &pointer, ui_rect, report)? {
        Ok(v) => v,
        Err(why) => return Ok(Some(why)),
    };
    let centre = match place_field(&session, &pointer, ui_rect, &mapping, report)? {
        Ok(c) => c,
        Err(why) => return Ok(Some(why)),
    };
    let (taken, line) = match read_back(&session, &pointer, ui_rect, centre, report)? {
        Ok(v) => v,
        Err(why) => return Ok(Some(why)),
    };
    pointer.gone(&session)?;
    // The engine stores the width it was handed; a field placed at the old
    // 1 pt reads back 1, which no band value above 1.5 can produce.
    if (taken - set).abs() > 0.05 {
        return Ok(Some(format!(
            "the band set {set} pt and the placed field carries {taken} pt: the next field did \
             not take the Style group's width. Band: `{look}`; field: `{line}`."
        )));
    }
    Ok(None)
}

/// Opens the Forms tab and drags the Style band's width to the right.
fn set_width(
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
    report: &mut CheckReport,
) -> Step<(f32, String)> {
    let trace = session.trace()?;
    let Some(tab) = declared(&trace, ui_rect, FORMS_TAB) else {
        return Ok(Err(format!(
            "Edit mode has no `{FORMS_TAB}`. Tabs: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab."))
        )));
    };
    pointer.click(session, WindowPoint::centre_of(tab))?;
    session.settle(20);
    let Some(width) = declared_or_in_overflow(session, pointer, ui_rect, WIDTH)? else {
        return Ok(Err(format!(
            "the Forms tab draws no `{WIDTH}`: the Style group is missing. Regions under \
             `forms.style.`: {}. Trace: {}.",
            list(&declared_names(&session.trace()?, ui_rect, "forms.style.")),
            session.trace_path().display()
        )));
    };
    let mark = session.trace()?.mark();
    let from = WindowPoint::centre_of(width);
    let to = WindowPoint::centre_of(LRect::new(
        Pt::new(width.min.x + DRAG_PX, width.min.y),
        Pt::new(width.max.x + DRAG_PX, width.max.y),
    ));
    pointer.drag(session, from, to, 8)?;
    session.settle(20);
    let Some((set, look)) = width_after(session, LOOK, mark)? else {
        return Ok(Err(format!(
            "dragging `{WIDTH}` {DRAG_PX} px traced no `{LOOK}` line: the band did not change \
             the remembered look. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("the band: `{look}`"));
    if set <= 1.5 {
        return Ok(Err(format!(
            "dragging the width {DRAG_PX} px right left it at {set}, not above the 1 pt a new \
             field starts with: `{look}`."
        )));
    }
    Ok(Ok((set, look)))
}

/// Places a text field from the Forms tab at [`SPOT`]; returns its centre.
/// The field is laid out from the click as its lower-left corner.
fn place_field(
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
    mapping: &CanvasMapping,
    report: &mut CheckReport,
) -> Step<WindowPoint> {
    let Some(item) = declared_or_in_overflow(session, pointer, ui_rect, TEXT_FIELD)? else {
        return Ok(Err(format!("the Forms tab declares no `{TEXT_FIELD}`.")));
    };
    pointer.click(session, WindowPoint::centre_of(item))?;
    session.settle(20);
    let mark = session.trace()?.mark();
    pointer.click(
        session,
        mapping.doc_to_window(DocPoint::new(0, SPOT.0, SPOT.1))?,
    )?;
    session.settle(40);
    let trace = session.trace()?;
    let Some(open) = trace.last_after(OPENED, mark) else {
        return Ok(Err(format!(
            "a click with Text field armed opened no placement dialog (`{OPENED}`). Trace: {}.",
            session.trace_path().display()
        )));
    };
    let field = open.get("name").unwrap_or("?").to_owned();
    let (w, h) = (
        f64::from(open.get_f32("w").unwrap_or(0.0)),
        f64::from(open.get_f32("h").unwrap_or(0.0)),
    );
    if trace.last_after(ADDED, mark).is_none() {
        return Ok(Err(format!(
            "the placement dialog for {field} accepted and no `{ADDED}` followed: the field \
             was never authored. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note(format!("placed {field}, {w} x {h} pt"));
    Ok(Ok(mapping.doc_to_window(DocPoint::new(
        0,
        SPOT.0 + w / 2.0,
        SPOT.1 + h / 2.0,
    ))?))
}

/// Right-clicks the placed field and runs *Use as default look*.
fn read_back(
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
    centre: WindowPoint,
    report: &mut CheckReport,
) -> Step<(f32, String)> {
    pointer.right_click(session, centre)?;
    session.settle(20);
    let trace = session.trace()?;
    let Some(row) = declared(&trace, ui_rect, MENU_ROW) else {
        return Ok(Err(format!(
            "the field's right-click menu has no `{MENU_ROW}`. Rows: {}. Trace: {}.",
            list(&declared_names(&trace, ui_rect, "menu.item.canvas.")),
            session.trace_path().display()
        )));
    };
    let mark = session.trace()?.mark();
    pointer.click(session, WindowPoint::centre_of(row))?;
    session.settle(20);
    let Some((taken, line)) = width_after(session, TAKEN, mark)? else {
        return Ok(Err(format!(
            "Use as default look traced no `{TAKEN}` line. Declines: {}. Trace: {}.",
            session
                .trace()?
                .last_after("command-declined", mark)
                .map_or("none", |l| l.raw.as_str()),
            session.trace_path().display()
        )));
    };
    report.note(format!("read back: `{line}`"));
    Ok(Ok((taken, line)))
}
