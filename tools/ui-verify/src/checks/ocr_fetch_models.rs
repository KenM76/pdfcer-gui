//! `ocr_models_download_into_the_models_folder` — File ▸ Recognise ▸ Download
//! OCR models… fetches the engine's pinned `ocrs` files into `models/ocrs`
//! beside the program, and the window then shows the licence line. Runs a
//! copy of the binary in the check's output folder, so the download lands
//! there; needs the network.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/ocr_fetch_models.md`.

use std::path::{Path, PathBuf};

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_in, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const OFFSCREEN: &str = "-4200,-4200,1400,900";
const TAB: &str = "ribbon.tab.file"; // ui-text-exempt: a trace region name, never displayed
const ITEM: &str = "ribbon.item.file.fetch_ocr_models"; // ui-text-exempt: a trace region name, never displayed
const COLLAPSED: &str = "ribbon.group.file.recognise.collapsed"; // ui-text-exempt: a trace region name, never displayed
const DOWNLOAD: &str = "fetch-models.download.ocrs"; // ui-text-exempt: a trace region name, never displayed
const RESULT: &str = "fetch-models.result"; // ui-text-exempt: a trace region name, never displayed
const DONE: &str = "fetch-models-done"; // ui-text-exempt: a trace event name, never displayed
const FAILED: &str = "fetch-models-failed"; // ui-text-exempt: a trace event name, never displayed
/// The engine's pinned `ocrs` file names, which the build's own
/// `models/ocrs` holds as the reference copies.
const FILES: [&str; 2] = ["text-detection.rten", "text-rec-checkpoint.rten"];

/// See the module documentation.
pub struct OcrModelsDownload;

impl Check for OcrModelsDownload {
    fn name(&self) -> &'static str {
        "ocr_models_download_into_the_models_folder"
    }

    fn defect(&self) -> &'static str {
        "Download OCR models… is missing, writes nowhere Recognise text looks, or \
         writes files that differ from the pinned ones"
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
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let reference = exe
        .parent()
        .map(|d| d.join("models").join("ocrs"))
        .ok_or_else(|| Error::new("the binary has no folder"))?;
    if FILES.iter().any(|f| !reference.join(f).is_file()) {
        return Err(Error::new(format!(
            "the reference copies are not in {}; package or copy the ocrs models there.",
            reference.display()
        )));
    }
    let program = ctx.out("fetch-models");
    std::fs::create_dir_all(&program)
        .map_err(|e| Error::new(format!("making {}: {e}", program.display())))?;
    let _ = std::fs::remove_dir_all(program.join("models"));
    let copy = program.join(exe.file_name().unwrap_or_default());
    std::fs::copy(&exe, &copy).map_err(|e| Error::new(format!("copying the binary: {e}")))?;

    let (session, pointer, ui_rect) = launch(ctx, &copy)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);
    let outcome = download(report, &session, &pointer, ui_rect);
    let parked = pointer.gone(&session);
    let outcome = outcome?;
    parked?;
    if outcome.is_some() {
        return Ok(outcome);
    }
    Ok(compare(
        report,
        &program.join("models").join("ocrs"),
        &reference,
    ))
}

fn launch(ctx: &CheckContext, exe: &Path) -> Result<(Session, ScriptedPointer, &'static str)> {
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let mut spec = LaunchSpec::new(exe, ctx.out("fetch-models.trace.txt"));
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
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("fetch-models.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    Ok((session, pointer, ui_rect))
}

fn download(
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
) -> Result<Option<String>> {
    let path = session.trace_path().display().to_string();
    let click = |region: &str| -> Result<()> {
        let trace = session.trace()?;
        let (rect, viewport) = declared_in(&trace, ui_rect, region).ok_or_else(|| {
            Error::new(format!(
                "no `{region}` region. Declared under `ribbon.item.file`: {}.",
                list(&declared_names(&trace, ui_rect, "ribbon.item.file"))
            ))
        })?;
        pointer.click_in(session, viewport.as_deref(), WindowPoint::centre_of(rect))?;
        session.settle(15);
        Ok(())
    };
    click(TAB)?;
    if declared(&session.trace()?, ui_rect, ITEM).is_none()
        && declared(&session.trace()?, ui_rect, COLLAPSED).is_some()
    {
        click(COLLAPSED)?;
    }
    if declared(&session.trace()?, ui_rect, ITEM).is_none() {
        return Ok(Some(format!(
            "★ File ▸ Recognise has no `{ITEM}`. Items: {}. Trace: {path}.",
            list(&declared_names(
                &session.trace()?,
                ui_rect,
                "ribbon.item.file."
            ))
        )));
    }
    click(ITEM)?;
    session.settle(20);
    if declared(&session.trace()?, ui_rect, DOWNLOAD).is_none() {
        return Ok(Some(format!(
            "★ the window offers no `{DOWNLOAD}` button. Trace: {path}."
        )));
    }
    click(DOWNLOAD)?;

    // Two files of about 12 MB together; the engine bounds each request.
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(240);
    let (done, ended) = loop {
        session.settle(30);
        let trace = session.trace()?;
        if let Some(line) = trace.last(DONE) {
            break (true, line.clone());
        }
        if let Some(line) = trace.last(FAILED) {
            break (false, line.clone());
        }
        if std::time::Instant::now() > deadline {
            return Ok(Some(format!(
                "no `{DONE}` or `{FAILED}` within four minutes. Trace: {path}."
            )));
        }
    };
    report.note(format!("download: `{}`", ended.raw));
    if !done {
        return Ok(Some(format!(
            "★★ the download did not finish: `{}`. Trace: {path}.",
            ended.raw
        )));
    }
    if ended.get("files") != Some("2") {
        return Ok(Some(format!(
            "★★ the download wrote {:?} files, not the two `ocrs` pins. Trace: {path}.",
            ended.get("files")
        )));
    }
    session.settle(10);
    if declared(&session.trace()?, ui_rect, RESULT).is_none() {
        return Ok(Some(format!(
            "★★ the download ended and the window shows no result line (the licence's \
             attribution). Trace: {path}."
        )));
    }
    Ok(None)
}

/// Each downloaded file is byte-for-byte the build's reference copy.
fn compare(report: &mut CheckReport, got: &Path, reference: &Path) -> Option<String> {
    for name in FILES {
        let a: PathBuf = got.join(name);
        let (Ok(mine), Ok(theirs)) = (std::fs::read(&a), std::fs::read(reference.join(name)))
        else {
            return Some(format!(
                "★★★ the download reported success and {} is not there: it wrote \
                 somewhere Recognise text does not look.",
                a.display()
            ));
        };
        if mine != theirs {
            return Some(format!(
                "★★★ {} differs from the reference copy in {}.",
                a.display(),
                reference.display()
            ));
        }
    }
    report.note(format!(
        "both files are in {} and match the reference copies",
        got.display()
    ));
    None
}
