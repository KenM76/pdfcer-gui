//! `recognised_text_is_in_the_saved_file` (ocrs),
//! `paddle_text_is_in_the_saved_file` and
//! `paddle_vl_text_is_in_the_saved_file` — File ▸ Recognise text… on a copy of
//! `fixtures/synthetic-image-only.pdf`, with the recogniser preference seeded,
//! run with the scripted pointer in a window placed off the desktop, puts an
//! OCR layer into the session; Ctrl+S saves it; and the saved file, reopened
//! in a second process, has text on its page: Recognise text refuses it as
//! already holding text, where the untouched fixture is recognised.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/ocr_scripted.md`.

use std::path::Path;

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_in, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::trace::Trace;

const MODE: &str = "ribbon.mode.read"; // ui-text-exempt: a trace region name, never displayed
const TAB: &str = "ribbon.tab.file"; // ui-text-exempt: a trace region name, never displayed
const ITEM: &str = "ribbon.item.file.ocr"; // ui-text-exempt: a trace region name, never displayed
/// The Recognise group when the band is too narrow to show it open.
const COLLAPSED: &str = "ribbon.group.file.recognise.collapsed"; // ui-text-exempt: a trace region name, never displayed
const RUN: &str = "ocr-run"; // ui-text-exempt: a trace region name, never displayed
/// The dialog's model choice on opening.
const MODEL_START: &str = "ocr-model-start"; // ui-text-exempt: a trace event name, never displayed
const STARTED: &str = "ocr-started"; // ui-text-exempt: a trace event name, never displayed
const RECOGNISED: &str = "ocr-recognised"; // ui-text-exempt: a trace event name, never displayed
const REFUSED: &str = "ocr-refused"; // ui-text-exempt: a trace event name, never displayed
/// The refusal a page that already holds text gets.
const HAS_TEXT: &str = "AlreadyHasText"; // ui-text-exempt: a trace token, never displayed
/// The edit funnel's line for the applied layer.
const LAYER: &str = "ocr-layer"; // ui-text-exempt: a trace event name, never displayed
const SAVED: &str = "save-in-place"; // ui-text-exempt: a trace event name, never displayed
/// Off the desktop, so no OS input can reach it and none of his is taken.
const OFFSCREEN: &str = "-4200,-4200,1400,900";
/// Settle frames to wait for one recognition, polled in steps of 20.
const RECOGNITION_FRAMES: u32 = 600;
/// The same for PaddleOCR-VL, a 1.2 GB model decoding token by token.
const VL_RECOGNITION_FRAMES: u32 = 12_000;

/// One recogniser, by its `ocr_engine` preference value.
pub struct RecognisedTextIsInTheSavedFile {
    pub engine: &'static str,
}

impl Check for RecognisedTextIsInTheSavedFile {
    fn name(&self) -> &'static str {
        match self.engine {
            "paddle" => "paddle_text_is_in_the_saved_file",
            "paddle-vl" => "paddle_vl_text_is_in_the_saved_file",
            _ => "recognised_text_is_in_the_saved_file",
        }
    }

    fn defect(&self) -> &'static str {
        "Recognise text… finds words and the document does not keep them: no layer reaches the \
         session, or the save does not carry it, so the reopened file has no text"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive(ctx, &mut report, self.engine) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn drive(ctx: &CheckContext, report: &mut CheckReport, engine: &str) -> Result<Option<String>> {
    let fixture = crate::fixture::workspace_root()
        .join("fixtures")
        .join("synthetic-image-only.pdf");
    if !fixture.is_file() {
        return Ok(Some(format!(
            "the image-only fixture is not at {}; it is committed, so this is a broken checkout.",
            fixture.display()
        )));
    }
    let copy = ctx.out(&format!("ocr-scripted-{engine}.pdf"));
    std::fs::copy(&fixture, &copy).map_err(|e| Error::new(format!("copying the fixture: {e}")))?;
    let original =
        std::fs::read(&copy).map_err(|e| Error::new(format!("reading the copy: {e}")))?;

    let (trace, path) = recognise(ctx, report, &copy, engine, "first", true)?;
    if let Some(failure) = judge_first(&trace, engine, &path) {
        return Ok(Some(failure));
    }
    let saved = std::fs::read(&copy).map_err(|e| Error::new(format!("reading the save: {e}")))?;
    if saved.len() <= original.len() || !saved.starts_with(&original) {
        return Ok(Some(format!(
            "★★ the save did not append to the original: {} bytes before, {} after, original \
             bytes kept {}. Trace: {path}.",
            original.len(),
            saved.len(),
            saved.starts_with(&original)
        )));
    }

    let (trace, path) = recognise(ctx, report, &copy, engine, "reopened", false)?;
    let refusal = trace.last(REFUSED).map(|l| l.raw.clone());
    if !refusal.as_deref().is_some_and(|l| l.contains(HAS_TEXT)) {
        return Ok(Some(format!(
            "★★★ the saved file has no text on its page: Recognise text on it traced {:?} and \
             refusal {refusal:?}, where a page holding the saved layer is refused as \
             `{HAS_TEXT}`. The layer was applied and saved, and the file does not hold it. \
             Trace: {path}.",
            trace.last(RECOGNISED).map(|l| l.raw.clone())
        )));
    }
    Ok(None)
}

/// The first run: the seeded recogniser ran, found words on a page with no
/// text, the layer went through the edit funnel, and Ctrl+S saved it.
fn judge_first(trace: &Trace, engine: &str, path: &str) -> Option<String> {
    let ran = trace
        .last(STARTED)
        .and_then(|l| l.get("engine").map(str::to_owned));
    if ran.as_deref() != Some(engine) {
        return Some(format!(
            "`ocr_engine = {engine}` was seeded and `{STARTED}` names {ran:?}. Trace: {path}."
        ));
    }
    let Some(line) = trace.last(RECOGNISED) else {
        return Some(format!(
            "Recognise was pressed and no `{RECOGNISED}` line followed. Refusal: {:?}. Trace: \
             {path}.",
            trace.last(REFUSED).map(|l| l.raw.clone())
        ));
    };
    if line.get("pages") != Some("1") || line.get_usize("recognised").unwrap_or(0) == 0 {
        return Some(format!(
            "the fixture's one page should be recognised with words on it; the run traced `{}`.",
            line.raw
        ));
    }
    let Some(layer) = trace.last(LAYER) else {
        return Some(format!(
            "★ `{}` and no `{LAYER}` line followed: the recognition never reached the session. \
             Refused: {:?}. Trace: {path}.",
            line.raw,
            trace.last("ocr-layer-refused").map(|l| l.raw.clone())
        ));
    };
    let saved = trace.last(SAVED).map(|l| l.raw.clone());
    if !saved.as_deref().is_some_and(|l| l.contains("outcome=ok")) {
        return Some(format!(
            "the layer was applied (`{}`) and Ctrl+S traced {saved:?}. Trace: {path}.",
            layer.raw
        ));
    }
    None
}

/// Seed the recogniser beside the binary, launch on `pdf`, open Recognise
/// text…, run it, and wait for its answer; with `save`, then press Ctrl+S.
fn recognise(
    ctx: &CheckContext,
    report: &mut CheckReport,
    pdf: &Path,
    engine: &str,
    tag: &str,
    save: bool,
) -> Result<(Trace, String)> {
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
    let userdata = exe
        .parent()
        .ok_or_else(|| Error::new("the binary has no parent directory"))?
        .join("userdata");
    crate::sandbox::write_prefs(
        &userdata,
        &format!("ocr_engine = {engine}\nocr_model = {engine}\n"),
    )
    .map_err(|e| Error::new(format!("could not write preferences: {e}")))?;

    let name = format!("ocr-scripted-{engine}-{tag}");
    let mut spec = LaunchSpec::new(&exe, ctx.out(&format!("{name}.trace.txt")));
    spec.pdf = Some(pdf.to_path_buf());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push((viewport_env.to_owned(), OFFSCREEN.to_owned()));
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out(&format!("{name}.pointer.txt")))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    let path = session.trace_path().display().to_string();
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);

    let click = |region: &str| -> Result<()> {
        let trace = session.trace()?;
        let (rect, viewport) = declared_in(&trace, ui_rect, region).ok_or_else(|| {
            let prefix = region.rsplit_once('.').map_or(region, |(head, _)| head);
            Error::new(format!(
                "no `{region}` region. Declared under `{prefix}`: {}.",
                list(&declared_names(&trace, ui_rect, prefix))
            ))
        })?;
        pointer.click_in(&session, viewport.as_deref(), WindowPoint::centre_of(rect))?;
        session.settle(15);
        Ok(())
    };
    click(MODE)?;
    click(TAB)?;
    let trace = session.trace()?;
    if declared(&trace, ui_rect, ITEM).is_none() && declared(&trace, ui_rect, COLLAPSED).is_some() {
        click(COLLAPSED)?;
    }
    click(ITEM)?;
    session.settle(15);
    let trace = session.trace()?;
    let chosen = trace
        .last(MODEL_START)
        .and_then(|l| l.get("chosen").map(str::to_owned));
    if chosen.as_deref() != Some(engine) {
        return Err(Error::new(format!(
            "the dialog chose {chosen:?}, not `{engine}`: this build cannot run it, or its \
             models are not beside {}. Its `ocr-model` lines say which.",
            exe.display()
        )));
    }
    if declared(&trace, ui_rect, RUN).is_none() {
        return Err(Error::new(format!(
            "the dialog drew no `{RUN}` control: no `{engine}` models are beside {}. Point --exe \
             at a packaged build.",
            exe.display()
        )));
    }
    click(RUN)?;
    let mut waited = 0;
    let limit = if engine == "paddle-vl" {
        VL_RECOGNITION_FRAMES
    } else {
        RECOGNITION_FRAMES
    };
    while waited < limit {
        let trace = session.trace()?;
        if trace.last(RECOGNISED).is_some() || trace.last(REFUSED).is_some() {
            break;
        }
        session.settle(20);
        waited += 20;
    }
    session.settle(20);
    if save {
        let viewport = declared_in(&session.trace()?, ui_rect, TAB).and_then(|(_, vp)| vp);
        pointer.key(&session, viewport.as_deref(), "S", Some("ctrl"))?;
        session.settle(40);
    }
    pointer.gone(&session)?;
    Ok((session.trace()?, path))
}
