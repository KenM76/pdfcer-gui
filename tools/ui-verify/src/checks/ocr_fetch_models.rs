//! File > Recognise > Manage OCR models… offers Download for each pinned
//! set not installed, fetches its files into `models/<folder>` beside the
//! program with the set's manifest, and the window then shows the licence
//! line and lists the set by name. Runs a copy of the binary in the check's
//! output folder, so the download lands there; needs the network.
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
const ITEM: &str = "ribbon.item.file.ocr_models"; // ui-text-exempt: a trace region name, never displayed
const COLLAPSED: &str = "ribbon.group.file.recognise.collapsed"; // ui-text-exempt: a trace region name, never displayed
const RESULT: &str = "ocr-models.result"; // ui-text-exempt: a trace region name, never displayed
const DONE: &str = "fetch-models-done"; // ui-text-exempt: a trace event name, never displayed
const FAILED: &str = "fetch-models-failed"; // ui-text-exempt: a trace event name, never displayed
const LISTED: &str = "ocr-models-listed"; // ui-text-exempt: a trace event name, never displayed
const MANIFEST: &str = "pdfcer-ocr-model.txt";

/// One of the engine's pinned sets, as this check drives it.
struct Set {
    /// `FetchableModels::name`, which is also its folder.
    name: &'static str,
    /// `FetchableModels::files`' names.
    files: &'static [&'static str],
    /// Whether the build's own `models/<name>` holds byte-identical copies.
    reference: bool,
}

const OCRS: Set = Set {
    name: "ocrs",
    files: &["text-detection.rten", "text-rec-checkpoint.rten"],
    reference: true,
};

const PADDLE_LATIN: Set = Set {
    name: "ppocrv5-latin",
    files: &["det.onnx", "rec.onnx", "inference.yml"],
    reference: false,
};

/// `ocrs`, compared byte for byte with the packaged copies.
pub struct OcrModelsDownload;

impl Check for OcrModelsDownload {
    fn name(&self) -> &'static str {
        "ocr_models_download_into_the_models_folder"
    }

    fn defect(&self) -> &'static str {
        "Manage OCR models… offers no Download for a missing ocrs, or its download is missing, writes nowhere Recognise text looks, or \
         writes files that differ from the pinned ones"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run(self.name(), self.defect(), ctx, &OCRS)
    }
}

/// The PaddleOCR Latin set, which the build does not package.
pub struct PaddleModelsDownload;

impl Check for PaddleModelsDownload {
    fn name(&self) -> &'static str {
        "a_paddle_set_downloads_and_is_listed_by_name"
    }

    fn defect(&self) -> &'static str {
        "Manage OCR models… offers no Download for a pinned PaddleOCR set, or the download writes no manifest, so the window \
         does not list the set by name and offers it again"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run(self.name(), self.defect(), ctx, &PADDLE_LATIN)
    }
}

fn run(name: &'static str, defect: &'static str, ctx: &CheckContext, set: &Set) -> CheckReport {
    let mut report = CheckReport::new(name, defect);
    match drive(ctx, &mut report, set) {
        Ok(Some(failure)) => report.fail(failure),
        Ok(None) => report.pass(),
        Err(why) => report.from_error(&why),
    }
}

fn drive(ctx: &CheckContext, report: &mut CheckReport, set: &Set) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let reference = exe
        .parent()
        .map(|d| d.join("models").join(set.name))
        .ok_or_else(|| Error::new("the binary has no folder"))?;
    if set.reference && set.files.iter().any(|f| !reference.join(f).is_file()) {
        return Err(Error::new(format!(
            "the reference copies are not in {}; package or copy the {} models there.",
            reference.display(),
            set.name
        )));
    }
    let program = ctx.out(&format!("fetch-models-{}", set.name));
    std::fs::create_dir_all(&program)
        .map_err(|e| Error::new(format!("making {}: {e}", program.display())))?;
    let _ = std::fs::remove_dir_all(program.join("models"));
    let copy = program.join(exe.file_name().unwrap_or_default());
    std::fs::copy(&exe, &copy).map_err(|e| Error::new(format!("copying the binary: {e}")))?;

    let (session, pointer, ui_rect) = launch(ctx, &copy, set.name)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);
    let outcome = download(report, &session, &pointer, ui_rect, set);
    let parked = pointer.gone(&session);
    let outcome = outcome?;
    parked?;
    if outcome.is_some() {
        return Ok(outcome);
    }
    let got = program.join("models").join(set.name);
    Ok(manifest(&got, set).or_else(|| {
        set.reference
            .then(|| compare(report, &got, &reference, set))?
    }))
}

fn launch(
    ctx: &CheckContext,
    exe: &Path,
    tag: &str,
) -> Result<(Session, ScriptedPointer, &'static str)> {
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let mut spec = LaunchSpec::new(exe, ctx.out(&format!("fetch-models-{tag}.trace.txt")));
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
    let pointer = ScriptedPointer::attach(
        &mut spec,
        ctx.out(&format!("fetch-models-{tag}.pointer.txt")),
    )?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    Ok((session, pointer, ui_rect))
}

fn download(
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
    set: &Set,
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
    let button = format!("ocr-models.download.{}", set.name);
    if declared(&session.trace()?, ui_rect, &button).is_none() {
        return Ok(Some(format!(
            "★ the window offers no `{button}` button. Offered: {}. Trace: {path}.",
            list(&declared_names(
                &session.trace()?,
                ui_rect,
                "ocr-models.download."
            ))
        )));
    }
    let before = session.trace()?.mark();
    click(&button)?;
    if let Some(failure) = finished(report, session, set, &path)? {
        return Ok(Some(failure));
    }
    session.settle(10);
    let trace = session.trace()?;
    if declared(&trace, ui_rect, RESULT).is_none() {
        return Ok(Some(format!(
            "★★ the download ended and the window shows no result line (the licence's \
             attribution). Trace: {path}."
        )));
    }
    let listed = trace
        .last_after(LISTED, before)
        .and_then(|l| l.get("names"));
    if !listed.is_some_and(|names| names.split(',').any(|n| n == set.name)) {
        return Ok(Some(format!(
            "★★ after the download the window lists {listed:?}, not `{}`: the set is not \
             recognised as installed and would be offered again. Trace: {path}.",
            set.name
        )));
    }
    Ok(None)
}

/// Waits for the download's end; a failure sentence when it did not finish
/// with every pinned file.
fn finished(
    report: &mut CheckReport,
    session: &Session,
    set: &Set,
    path: &str,
) -> Result<Option<String>> {
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
    if ended.get("name") != Some(set.name) || ended.get_usize("files") != Some(set.files.len()) {
        return Ok(Some(format!(
            "★★ the download reported `{}`, not `{}` with its {} pinned files. Trace: {path}.",
            ended.raw,
            set.name,
            set.files.len()
        )));
    }
    Ok(None)
}

/// The set's files, and its manifest naming the set, are where discovery
/// looks.
fn manifest(got: &Path, set: &Set) -> Option<String> {
    if let Some(absent) = set.files.iter().find(|f| !got.join(f).is_file()) {
        return Some(format!(
            "★★★ the download reported success and {} is not there: it wrote \
             somewhere Recognise text does not look.",
            got.join(absent).display()
        ));
    }
    let text = std::fs::read_to_string(got.join(MANIFEST)).unwrap_or_default();
    let named = format!("name = {}", set.name);
    if !text.lines().any(|l| l.trim() == named) {
        return Some(format!(
            "★★★ {} has no line `{named}`, so discovery cannot name the folder after the set.",
            got.join(MANIFEST).display()
        ));
    }
    None
}

/// Each downloaded file is byte-for-byte the build's reference copy.
fn compare(report: &mut CheckReport, got: &Path, reference: &Path, set: &Set) -> Option<String> {
    for name in set.files {
        let a: PathBuf = got.join(name);
        let (Ok(mine), Ok(theirs)) = (std::fs::read(&a), std::fs::read(reference.join(name)))
        else {
            return Some(format!("★★★ {} could not be read back.", a.display()));
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
        "every file is in {} and matches the reference copies",
        got.display()
    ));
    None
}
