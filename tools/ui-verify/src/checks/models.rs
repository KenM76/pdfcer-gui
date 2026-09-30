//! `checks::models` — **a 3D model placed from the ribbon is listed in the
//! Attachments panel and saves back out byte for byte**
//!
//! Drives the window off the desktop through the scripted pointer (no OS mouse
//! or keyboard). The pickers are answered by `PDFCER_DIAG_MODEL_PATH`,
//! `PDFCER_DIAG_ATTACHMENT_SAVE_PATH` and `PDFCER_DIAG_MESH_SAVE_PATH`. The
//! model is the engine corpus's `square.prc`, a real PRC triangle mesh.
//!
//! Oracles: the `add-3d` funnel line (the engine wrote it), the
//! `models-section count=1` census (the panel reads it back from the session),
//! `model-saved` and the saved file's bytes against the source, then
//! `mesh-saved` and the STL's triangle count agreeing with its length.

use crate::checks::driving::{SHELL_DIAG_ENV, declared_in};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Off every monitor, unfocused.
const OFFSCREEN: &str = "-4200,-4200,1400,900";
/// Edit mode with the Attachments panel showing.
const INVOKE: &str = "mode.edit,edit.attachments";
const RIBBON_TAB: &str = "ribbon.tab.edit";
const RIBBON_ITEM: &str = "ribbon.item.edit.insert_3d";
const SAVE_REGION: &str = "models.save";
const MODEL_ENV: &str = "PDFCER_DIAG_MODEL_PATH";
const SAVE_ENV: &str = "PDFCER_DIAG_ATTACHMENT_SAVE_PATH";
const MESH_ENV: &str = "PDFCER_DIAG_MESH_SAVE_PATH";
const MESH_REGION: &str = "models.mesh";
const DOC: &str = "D:/Dev/pdfcer/fixtures/synthetic/pageops/four-pages.pdf";
const MODEL: &str = "D:/Dev/pdfcer/fixtures/synthetic/prc/square.prc";

/// See the module documentation.
pub struct AModelIsPlacedListedAndSavedBack;

impl Check for AModelIsPlacedListedAndSavedBack {
    fn name(&self) -> &'static str {
        "a_3d_model_is_placed_listed_and_saved_back"
    }

    fn defect(&self) -> &'static str {
        "Edit ▸ 3D model places nothing, or the placed model is not listed in the Attachments \
         panel, or Save model writes different bytes, or Save as mesh writes no well-formed STL"
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

fn models_count(session: &Session) -> Result<Option<usize>> {
    Ok(session
        .trace()?
        .events("models-section")
        .last()
        .and_then(|l| l.get("count").and_then(|c| c.parse().ok())))
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    for needed in [DOC, MODEL] {
        if !std::path::Path::new(needed).is_file() {
            return Err(Error::new(format!(
                "the engine corpus's fixture is missing at {needed}."
            )));
        }
    }
    let source_model =
        std::fs::read(MODEL).map_err(|e| Error::new(format!("reading {MODEL}: {e}")))?;
    // Driven on a copy: the source belongs to the engine repository.
    let doc = ctx.out("models-source.pdf");
    std::fs::copy(DOC, &doc).map_err(|e| Error::new(format!("copying {DOC}: {e}")))?;
    let model = ctx.out("models-input.prc");
    std::fs::write(&model, &source_model)
        .map_err(|e| Error::new(format!("writing the model: {e}")))?;
    let saved_to = ctx.out("models-saved.prc");
    let _ = std::fs::remove_file(&saved_to);
    let mesh_to = ctx.out("models-mesh.stl");
    let _ = std::fs::remove_file(&mesh_to);

    let mut spec = LaunchSpec::new(&exe, ctx.out("models.trace.txt"));
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
    spec.env
        .push((SAVE_ENV.to_owned(), saved_to.to_string_lossy().into_owned()));
    spec.env
        .push((MESH_ENV.to_owned(), mesh_to.to_string_lossy().into_owned()));
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("models.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);

    let before = models_count(&session)?;
    // The Insert band is on the Edit tab; raise it if another tab is showing.
    let mut trace = session.trace()?;
    if declared_in(&trace, ui_rect, RIBBON_ITEM).is_none()
        && let Some((tab, vp)) = declared_in(&trace, ui_rect, RIBBON_TAB)
    {
        pointer.click_in(&session, vp.as_deref(), WindowPoint::centre_of(tab))?;
        session.settle(15);
        trace = session.trace()?;
    }
    let Some((item, vp)) = declared_in(&trace, ui_rect, RIBBON_ITEM) else {
        return Ok(Some(format!(
            "no `{RIBBON_ITEM}` region: the 3D model button is not on the Edit tab."
        )));
    };
    pointer.click_in(&session, vp.as_deref(), WindowPoint::centre_of(item))?;
    session.settle(30);

    let trace = session.trace()?;
    let requested = trace.events("model-insert-requested").last().is_some();
    let committed = trace.events("add-3d").count() > 0;
    let after = models_count(&session)?;
    report.note(format!(
        "models before={before:?}; insert requested={requested}; add-3d lines={}; models \
         after={after:?}",
        trace.events("add-3d").count()
    ));
    if !requested {
        pointer.gone(&session)?;
        return Ok(Some(format!(
            "clicking the 3D model button raised no `model-insert-requested`: the picker did not \
             answer from `{MODEL_ENV}`, or the file was refused (look for `model-insert-refused`)."
        )));
    }
    if !committed || after != Some(1) {
        pointer.gone(&session)?;
        return Ok(Some(format!(
            "the model was read but the panel lists {after:?} models (want 1); a refused edit \
             traces `add-3d-refused`."
        )));
    }

    let Some((save, vp)) = declared_in(&trace, ui_rect, SAVE_REGION) else {
        pointer.gone(&session)?;
        return Ok(Some(format!(
            "no `{SAVE_REGION}` region beside the listed model."
        )));
    };
    pointer.click_in(&session, vp.as_deref(), WindowPoint::centre_of(save))?;
    session.settle(20);
    let trace = session.trace()?;
    let Some((mesh, vp)) = declared_in(&trace, ui_rect, MESH_REGION) else {
        pointer.gone(&session)?;
        return Ok(Some(format!(
            "no `{MESH_REGION}` region beside the listed PRC model."
        )));
    };
    pointer.click_in(&session, vp.as_deref(), WindowPoint::centre_of(mesh))?;
    session.settle(20);
    let shot = ctx.out("models-after.png");
    if pointer.screenshot(&session, &shot).is_ok() {
        report.artifact(shot);
    }
    pointer.gone(&session)?;
    let trace = session.trace()?;
    let saved = trace.events("model-saved").last().map(|l| l.raw.clone());
    let meshed = trace.events("mesh-saved").last().map(|l| l.raw.clone());
    drop(session);
    report.note(format!("saved: {saved:?}; meshed: {meshed:?}"));
    if saved.is_none() {
        return Ok(Some(
            "Save model raised no `model-saved` line (look for `model-save-declined`).".to_owned(),
        ));
    }
    let written = std::fs::read(&saved_to)
        .map_err(|e| Error::new(format!("reading {}: {e}", saved_to.display())))?;
    if written != source_model {
        return Ok(Some(format!(
            "the saved model is {} bytes and differs from the {} placed.",
            written.len(),
            source_model.len()
        )));
    }
    if meshed.is_none() {
        return Ok(Some(
            "Save as mesh raised no `mesh-saved` line (look for `mesh-save-refused`).".to_owned(),
        ));
    }
    let stl = std::fs::read(&mesh_to)
        .map_err(|e| Error::new(format!("reading {}: {e}", mesh_to.display())))?;
    let triangles = stl
        .get(80..84)
        .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]) as usize);
    match triangles {
        Some(n) if n > 0 && stl.len() == 84 + 50 * n => Ok(None),
        _ => Ok(Some(format!(
            "the mesh file is {} bytes and is not a binary STL whose triangle count \
             ({triangles:?}) matches its length.",
            stl.len()
        ))),
    }
}
