//! `checks::model_gltf` — **a glTF model placed from the ribbon is converted,
//! listed, and saves back out as PRC**
//!
//! Drives the window off the desktop through the scripted pointer. The picker
//! is answered by `PDFCER_DIAG_MODEL_PATH` with a one-quad glTF (two
//! triangles, embedded buffer) written into the run's output folder.
//!
//! Oracles: `model-insert-gltf triangles=2` (the engine's conversion report),
//! the `add-3d` funnel line, the `models-section count=1` census, and the
//! model saved back starting with the PRC signature — the document holds the
//! converted PRC, not the glTF text.

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
const SAVE_REGION: &str = "models.save";
const MODEL_ENV: &str = "PDFCER_DIAG_MODEL_PATH";
const SAVE_ENV: &str = "PDFCER_DIAG_ATTACHMENT_SAVE_PATH";
const DOC: &str = "D:/Dev/pdfcer/fixtures/synthetic/pageops/four-pages.pdf";
/// One unit quad as two triangles, its buffer embedded as a data URI.
const QUAD: &str = r#"{"asset":{"version":"2.0"},"buffers":[{"byteLength":60,"uri":"data:application/octet-stream;base64,AAAAAAAAAAAAAAAAAACAPwAAAAAAAAAAAACAPwAAgD8AAAAAAAAAAAAAgD8AAAAAAAABAAIAAAACAAMA"}],"bufferViews":[{"buffer":0,"byteLength":48},{"buffer":0,"byteOffset":48,"byteLength":12}],"accessors":[{"bufferView":0,"componentType":5126,"count":4,"type":"VEC3","min":[0,0,0],"max":[1,1,0]},{"bufferView":1,"componentType":5123,"count":6,"type":"SCALAR"}],"materials":[{"name":"orange","pbrMetallicRoughness":{"baseColorFactor":[1,0.4,0,1]}}],"meshes":[{"primitives":[{"attributes":{"POSITION":0},"indices":1,"material":0}]}],"nodes":[{"name":"plate","mesh":0}],"scenes":[{"nodes":[0]}],"scene":0}"#;

/// See the module documentation.
pub struct AGltfModelIsConvertedAndPlaced;

impl Check for AGltfModelIsConvertedAndPlaced {
    fn name(&self) -> &'static str {
        "a_gltf_model_is_converted_and_placed"
    }

    fn defect(&self) -> &'static str {
        "Edit ▸ 3D model refuses a glTF file, or places it unconverted, or the placed model is \
         not listed"
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
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let saved_to = ctx.out("gltf-saved.prc");
    let (session, pointer) = launch(ctx, report, &saved_to)?;
    if !crate::checks::model_view_window::press_insert(ctx, &session, &pointer)? {
        return Ok(Some(
            "the 3D model button is on neither the Edit tab nor its overflow.".to_owned(),
        ));
    }
    if let Some(failure) = placed(&session, &pointer, report, ui_rect)? {
        pointer.gone(&session)?;
        return Ok(Some(failure));
    }
    pointer.gone(&session)?;
    drop(session);
    saved_back(&saved_to)
}

/// Launch off-screen on a copy of the fixture, the picker answered by [`QUAD`].
fn launch(
    ctx: &CheckContext,
    report: &mut CheckReport,
    saved_to: &std::path::Path,
) -> Result<(Session, ScriptedPointer)> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    if !std::path::Path::new(DOC).is_file() {
        return Err(Error::new(format!(
            "the engine corpus's fixture is missing at {DOC}."
        )));
    }
    // Driven on a copy: the source belongs to the engine repository.
    let doc = ctx.out("gltf-source.pdf");
    std::fs::copy(DOC, &doc).map_err(|e| Error::new(format!("copying {DOC}: {e}")))?;
    let model = ctx.out("gltf-input.gltf");
    std::fs::write(&model, QUAD).map_err(|e| Error::new(format!("writing the model: {e}")))?;
    let _ = std::fs::remove_file(saved_to);

    let mut spec = LaunchSpec::new(&exe, ctx.out("gltf.trace.txt"));
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
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("gltf.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);
    Ok((session, pointer))
}

/// Converted to two triangles, placed, listed, and its Save pressed.
fn placed(
    session: &Session,
    pointer: &ScriptedPointer,
    report: &mut CheckReport,
    ui_rect: &str,
) -> Result<Option<String>> {
    let trace = session.trace()?;
    let converted = trace
        .events("model-insert-gltf")
        .last()
        .map(|l| l.raw.clone());
    let refused = trace.events("model-insert-gltf-refused").count();
    let listed = trace
        .events("models-section")
        .last()
        .and_then(|l| l.get("count").and_then(|c| c.parse::<usize>().ok()));
    report.note(format!(
        "converted: {converted:?}; refused lines={refused}; add-3d lines={}; listed={listed:?}",
        trace.events("add-3d").count()
    ));
    let Some(converted) = converted else {
        return Ok(Some(format!(
            "no `model-insert-gltf` line: the glTF was not converted ({refused} refusals)."
        )));
    };
    if !converted.contains("triangles=2 ") {
        return Ok(Some(format!(
            "the quad converted to the wrong mesh: `{converted}`; want triangles=2."
        )));
    }
    if trace.events("add-3d").count() == 0 || listed != Some(1) {
        return Ok(Some(format!(
            "the converted model was not placed and listed (listed {listed:?}, want 1)."
        )));
    }
    let Some((save, vp)) = declared_in(&trace, ui_rect, SAVE_REGION) else {
        return Ok(Some(format!(
            "no `{SAVE_REGION}` region beside the listed model."
        )));
    };
    pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(save))?;
    session.settle(20);
    Ok(None)
}

/// The model Save wrote is PRC: the document holds the converted model.
fn saved_back(saved_to: &std::path::Path) -> Result<Option<String>> {
    let written = std::fs::read(saved_to)
        .map_err(|e| Error::new(format!("reading {}: {e}", saved_to.display())))?;
    if !written.starts_with(b"PRC") {
        return Ok(Some(format!(
            "the placed model saved back as {} bytes not starting `PRC`: the document holds \
             something other than the converted model.",
            written.len()
        )));
    }
    Ok(None)
}
