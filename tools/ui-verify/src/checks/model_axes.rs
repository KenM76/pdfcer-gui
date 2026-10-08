//! `a_3d_model_opens_from_the_page_and_turns_about_the_chosen_up` — a click on
//! a placed 3D model in Read opens the viewer, and choosing +Y as up turns
//! every named view about it.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/model_axes.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, click_mode_segment, declared_in};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, WindowPoint};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Off every monitor, unfocused.
const OFFSCREEN: &str = "-4200,-4200,1400,900";
/// Edit mode with the Attachments panel showing, to place the model.
const INVOKE: &str = "mode.edit,edit.attachments";
const RIBBON_TAB: &str = "ribbon.tab.edit";
const RIBBON_ITEM: &str = "ribbon.item.edit.insert_3d";
const FRONT: &str = "model3d.view.1";
const TOP: &str = "model3d.view.3";
const UP: &str = "model3d.up";
/// +Y in the up choice: its index in `dialogs::model3d::axes::ALL`.
const UP_Y: &str = "model3d.up.1";
const MODEL_ENV: &str = "PDFCER_DIAG_MODEL_PATH";
const DOC: &str = "D:/Dev/pdfcer/fixtures/synthetic/pageops/four-pages.pdf";
const MODEL: &str = "D:/Dev/pdfcer/fixtures/synthetic/prc/assembly.prc";
/// How close a traced unit direction must come to the expected one.
const TOLERANCE: f64 = 0.02;

/// See the module documentation.
pub struct AModelOpensFromThePageAndTurnsAboutTheChosenUp;

impl Check for AModelOpensFromThePageAndTurnsAboutTheChosenUp {
    fn name(&self) -> &'static str {
        "a_3d_model_opens_from_the_page_and_turns_about_the_chosen_up"
    }

    fn defect(&self) -> &'static str {
        "Clicking a 3D model on the page in Read does not open the viewer, or choosing which axis \
         is up does not change the way the named views look"
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

fn launch(ctx: &CheckContext) -> Result<(Session, ScriptedPointer)> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    for needed in [DOC, MODEL] {
        if !std::path::Path::new(needed).is_file() {
            return Err(Error::new(format!(
                "the engine corpus's fixture is missing at {needed}."
            )));
        }
    }
    // Driven on copies: the sources belong to the engine repository.
    let doc = ctx.out("model-axes-source.pdf");
    std::fs::copy(DOC, &doc).map_err(|e| Error::new(format!("copying {DOC}: {e}")))?;
    let model = ctx.out("model-axes-input.prc");
    std::fs::copy(MODEL, &model).map_err(|e| Error::new(format!("copying {MODEL}: {e}")))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("model-axes.trace.txt"));
    spec.pdf = Some(doc);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push((viewport_env.to_owned(), OFFSCREEN.to_owned()));
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), INVOKE.to_owned()));
    spec.env
        .push((MODEL_ENV.to_owned(), model.to_string_lossy().into_owned()));
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("model-axes.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    Ok((session, pointer))
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let (session, pointer) = launch(ctx)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);
    let outcome =
        opened_from_page(ctx, report, &session, &pointer, ui_rect).and_then(|f| match f {
            Some(failure) => Ok(Some(failure)),
            None => turns_about_up(report, &session, &pointer, ui_rect),
        });
    pointer.gone(&session)?;
    outcome
}

/// Place the model in Edit, switch to Read, click the page's centre where
/// the model sits, and require the viewer to open from that click.
fn opened_from_page(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
) -> Result<Option<String>> {
    let mut trace = session.trace()?;
    if declared_in(&trace, ui_rect, RIBBON_ITEM).is_none()
        && let Some((tab, vp)) = declared_in(&trace, ui_rect, RIBBON_TAB)
    {
        pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(tab))?;
        session.settle(15);
        trace = session.trace()?;
    }
    let Some((item, vp)) = declared_in(&trace, ui_rect, RIBBON_ITEM) else {
        return Ok(Some(format!(
            "no `{RIBBON_ITEM}` region: the 3D model button is not on the Edit tab."
        )));
    };
    pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(item))?;
    session.settle(30);
    if session
        .trace()?
        .events("model-insert-requested")
        .next()
        .is_none()
    {
        return Ok(Some(
            "the 3D model button placed nothing (no `model-insert-requested`).".to_owned(),
        ));
    }
    click_mode_segment(session, pointer, ui_rect, "read")?;
    session.settle(20);
    let trace = session.trace()?;
    let page = crate::fixture::page_geometry(std::path::Path::new(DOC))
        .ok_or_else(|| Error::new("could not read a page size from the fixture."))?;
    let mapping = CanvasMapping::from_trace(&trace, &ctx.profile.vocab, page, 0)?;
    let centre = DocPoint::new(0, page.width_pt / 2.0, page.height_pt / 2.0);
    pointer.click(session, mapping.doc_to_window(centre)?)?;
    session.settle(30);
    let trace = session.trace()?;
    let click = trace
        .events("model-page-click")
        .last()
        .map(|l| l.raw.clone());
    let opened = trace
        .events("model-view-opened")
        .last()
        .map(|l| l.raw.clone());
    report.note(format!("page click: {click:?}; opened: {opened:?}"));
    if click.is_none() || opened.is_none() {
        return Ok(Some(format!(
            "a click on the placed model in Read opened no viewer: `model-page-click` {click:?}, \
             `model-view-opened` {opened:?}."
        )));
    }
    Ok(None)
}

/// Front looks along +y with z up; with +Y chosen as up it looks along -z,
/// and Top looks along -y.
fn turns_about_up(
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
) -> Result<Option<String>> {
    if let Some(failure) =
        press_and_look(report, session, pointer, ui_rect, &[FRONT], [0.0, 1.0, 0.0])?
    {
        return Ok(Some(failure));
    }
    if let Some(failure) = press_and_look(
        report,
        session,
        pointer,
        ui_rect,
        &[UP, UP_Y],
        [0.0, 0.0, -1.0],
    )? {
        return Ok(Some(failure));
    }
    let axes = session
        .trace()?
        .events("model-view-axes")
        .last()
        .map(|l| l.raw.clone());
    report.note(format!("axes: {axes:?}"));
    press_and_look(report, session, pointer, ui_rect, &[TOP], [0.0, -1.0, 0.0])
}

/// Press each region in turn, then require the last picture's look
/// direction to be `expected`.
fn press_and_look(
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
    regions: &[&str],
    expected: [f64; 3],
) -> Result<Option<String>> {
    for region in regions {
        let trace = session.trace()?;
        let Some((rect, vp)) = declared_in(&trace, ui_rect, region) else {
            return Ok(Some(format!("no `{region}` region in the viewer.")));
        };
        pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(rect))?;
        session.settle(20);
    }
    let trace = session.trace()?;
    let dir = trace
        .events("model-view-rendered")
        .last()
        .and_then(|l| l.get("dir").map(str::to_owned));
    report.note(format!("{}: dir {dir:?}", regions.join(" > ")));
    let parsed: Option<Vec<f64>> = dir
        .as_deref()
        .map(|d| d.split(',').filter_map(|c| c.parse().ok()).collect());
    let close = parsed.is_some_and(|v| {
        v.len() == 3
            && v.iter()
                .zip(expected)
                .all(|(a, b)| (a - b).abs() < TOLERANCE)
    });
    if close {
        return Ok(None);
    }
    Ok(Some(format!(
        "after {} the view looks along {dir:?}; it should look along {expected:?}.",
        regions.join(" then ")
    )))
}
