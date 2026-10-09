//! `checks::model_views_saved` — **the 3D viewer writes its named views into
//! the file, and the model then opens on the one chosen**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/model_views_saved.md`.

use std::path::PathBuf;

use crate::checks::driving::{SHELL_DIAG_ENV, declared_in};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Off every monitor, unfocused.
const OFFSCREEN: &str = "-4200,-4200,1400,900";
/// Edit mode with the Attachments panel showing, which lists the model.
const INVOKE: &str = "mode.edit,edit.attachments";
const VIEW_REGION: &str = "models.view";
const UP_REGION: &str = "model3d.up";
/// +Y in the up choice: its index in `dialogs::model3d::axes::ALL`.
const UP_Y_REGION: &str = "model3d.up.1";
const TOP_REGION: &str = "model3d.view.3";
const SAVE_REGION: &str = "model3d.save_views";
const CLOSE_REGION: &str = "model3d.close";
const MODEL: &str = "D:/Dev/pdfcer/fixtures/synthetic/prc/assembly.prc";
/// The file's one saved view, `Corner`: along `(0.6, 0, -0.8)`, `+y` up.
const C2W: &str = "-0.8 0 -0.6 0 1 0 0.6 0 -0.8 -6 0 8";
const CORNER: [f64; 3] = [0.6, 0.0, -0.8];
/// Top with `+y` up looks down `-y`.
const TOP: [f64; 3] = [0.0, -1.0, 0.0];
/// The five named views; Top, the view chosen, is one of them, so no sixth
/// is added.
const WRITTEN: &str = "after=5";
/// Cosine above which two directions are the same.
const SAME: f64 = 0.999;

/// See the module documentation.
pub struct TheViewerSavesItsViewsIntoTheFile;

impl Check for TheViewerSavesItsViewsIntoTheFile {
    fn name(&self) -> &'static str {
        "the_3d_viewer_saves_its_views_into_the_file"
    }

    fn defect(&self) -> &'static str {
        "Save views in the file writes no views, or the model does not open on the view chosen \
         when the views were saved"
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

/// One page carrying a 3D annotation whose PRC stream saves one view,
/// `Corner`, the stream's default.
fn pdf_bytes(prc: &[u8]) -> Vec<u8> {
    let mut stream = format!(
        "<< /Type /3D /Subtype /PRC /VA [<< /Type /3DView /XN (Corner) /MS /M /C2W [{C2W}] >>] \
         /DV 0 /Length {} >>\nstream\n",
        prc.len()
    )
    .into_bytes();
    stream.extend_from_slice(prc);
    stream.extend_from_slice(b"\nendstream");
    crate::fixture::pdf_of(&[
        b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_vec(),
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 400 300] /Resources << >> \
          /Annots [4 0 R] >>"
            .to_vec(),
        b"<< /Type /Annot /Subtype /3D /Rect [50 50 350 250] /P 3 0 R /3DD 5 0 R >>".to_vec(),
        stream,
    ])
}

fn fixture(ctx: &CheckContext) -> Result<PathBuf> {
    let prc = std::fs::read(MODEL).map_err(|e| {
        Error::new(format!(
            "the engine corpus's fixture is unreadable at {MODEL}: {e}"
        ))
    })?;
    let path = ctx.out("model-views-saved.pdf");
    std::fs::write(&path, pdf_bytes(&prc))
        .map_err(|e| Error::new(format!("cannot write {}: {e}", path.display())))?;
    Ok(path)
}

fn launch(ctx: &CheckContext, pdf: PathBuf) -> Result<(Session, ScriptedPointer)> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("model-views-saved.trace.txt"));
    spec.pdf = Some(pdf);
    for (key, value) in [
        ctx.profile.diag_env,
        SHELL_DIAG_ENV,
        (viewport_env, OFFSCREEN),
        ("PDFCER_DIAG_INVOKE", INVOKE),
    ] {
        spec.env.push((key.to_owned(), value.to_owned()));
    }
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("model-views-saved.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    Ok((session, pointer))
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let (session, pointer) = launch(ctx, fixture(ctx)?)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);
    let failure = steps(&session, &pointer, ui_rect, report);
    let parked = pointer.gone(&session);
    match failure? {
        Some(failure) => Ok(Some(failure)),
        None => parked.map(|_| None),
    }
}

/// Open on `Corner`, turn to Top about +Y, save the views, close, open
/// again: it must open looking down -y.
fn steps(
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
    report: &mut CheckReport,
) -> Result<Option<String>> {
    if let Some(missing) = press_all(session, pointer, ui_rect, &[VIEW_REGION])? {
        return Ok(Some(missing));
    }
    let first = looked(session, 0)?;
    report.note(format!("opened looking along {first:?}"));
    if !first.is_some_and(|d| same(d, CORNER)) {
        return Ok(Some(format!(
            "the viewer first looked along {first:?}, not the file's own view {CORNER:?}: the \
             starting point is not the one this check assumes."
        )));
    }
    let regions = [UP_REGION, UP_Y_REGION, TOP_REGION, SAVE_REGION];
    if let Some(missing) = press_all(session, pointer, ui_rect, &regions)? {
        return Ok(Some(missing));
    }
    let trace = session.trace()?;
    let set = trace
        .events("model-views-set")
        .last()
        .map(|l| l.raw.clone());
    report.note(format!("saved: {set:?}"));
    if !set.as_deref().is_some_and(|l| l.contains(WRITTEN)) {
        return Ok(Some(format!(
            "Save views in the file did not write the five named views: `model-views-set` was \
             {set:?}; requested {:?}.",
            trace
                .events("model-views-requested")
                .last()
                .map(|l| l.raw.clone())
        )));
    }
    // Again from the same window: its own write must not make it refuse.
    let first_set = trace.events("model-views-set").count();
    if let Some(missing) = press_all(session, pointer, ui_rect, &[SAVE_REGION])? {
        return Ok(Some(missing));
    }
    let trace = session.trace()?;
    let second = trace
        .events("model-views-set")
        .nth(first_set)
        .map(|l| l.raw.clone());
    report.note(format!("saved again: {second:?}"));
    if !second.as_deref().is_some_and(|l| l.contains("before=5")) {
        return Ok(Some(format!(
            "a second Save views in the file from the same window wrote nothing: {second:?}; declined {:?}.",
            trace
                .events("model-views-declined")
                .last()
                .map(|l| l.raw.clone())
        )));
    }
    let opened = trace.events("model-view-opened").count();
    let rendered = trace.events("model-view-rendered").count();
    let again = [CLOSE_REGION, VIEW_REGION];
    if let Some(missing) = press_all(session, pointer, ui_rect, &again)? {
        return Ok(Some(missing));
    }
    let trace = session.trace()?;
    let reopened_line = trace
        .events("model-view-opened")
        .nth(opened)
        .map(|l| l.raw.clone());
    let reopened = looked(session, rendered)?;
    report.note(format!(
        "reopened: {reopened_line:?} looking along {reopened:?}"
    ));
    if !reopened_line
        .as_deref()
        .is_some_and(|l| l.contains("file-view=1"))
    {
        return Ok(Some(format!(
            "after saving, the viewer did not open on a view from the file: {reopened_line:?}."
        )));
    }
    if !reopened.is_some_and(|d| same(d, TOP)) {
        return Ok(Some(format!(
            "after saving with Top chosen, the model opened looking along {reopened:?}; Top \
             about +Y looks along {TOP:?}."
        )));
    }
    Ok(None)
}

/// Click each region in turn, letting the window redraw; the sentence for
/// the first one not declared.
fn press_all(
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
    regions: &[&str],
) -> Result<Option<String>> {
    for region in regions {
        let trace = session.trace()?;
        let Some((rect, vp)) = declared_in(&trace, ui_rect, region) else {
            return Ok(Some(format!("no `{region}` region was declared.")));
        };
        pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(rect))?;
        session.settle(25);
    }
    Ok(None)
}

/// The look direction on the first `model-view-rendered` line after the
/// first `skip`.
fn looked(session: &Session, skip: usize) -> Result<Option<[f64; 3]>> {
    let trace = session.trace()?;
    Ok(trace
        .events("model-view-rendered")
        .nth(skip)
        .and_then(|line| {
            let parts: Vec<f64> = line
                .get("dir")?
                .split(',')
                .filter_map(|p| p.parse().ok())
                .collect();
            <[f64; 3]>::try_from(parts).ok()
        }))
}

fn same(a: [f64; 3], b: [f64; 3]) -> bool {
    let length = |v: [f64; 3]| v.iter().map(|c| c * c).sum::<f64>().sqrt();
    let dot: f64 = (0..3).map(|i| a[i] * b[i]).sum();
    dot / (length(a) * length(b)).max(f64::MIN_POSITIVE) > SAME
}
