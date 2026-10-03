//! `an_extra_ocr_folder_adds_its_models_to_the_dropdown` — a folder added on
//! Settings ▸ OCR models puts its models in Recognise text's drop-down, a
//! PaddleOCR-VL add-on there is listed and cannot be chosen, a remembered
//! model that is gone is named rather than replaced, and the copy chosen is
//! the one that runs.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/ocr_extra_folder.md`.

use std::path::{Path, PathBuf};

use crate::checks::driving::{self, SHELL_DIAG_ENV};
use crate::checks::{Check, CheckContext, CheckReport};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::trace::Trace;

/// Off the desktop, so the check runs while the operator uses the machine.
const OFFSCREEN: &str = "-4200,-4200,1400,900";
/// Image-only, so any recognised word came from the model under test.
const FIXTURE: &str = "fixtures/synthetic-image-only.pdf";
/// The seam answering the folder picker, and the one opening Settings.
const FOLDER_ENV: &str = "PDFCER_DIAG_OCR_FOLDER";
const INVOKE_ENV: &str = "PDFCER_DIAG_INVOKE";
const OPEN_SETTINGS: &str = "file.settings";
/// Where the ocrs weights are copied from, when set; else the engine checkout.
const OCRS_MODELS_VAR: &str = "UI_VERIFY_OCRS_MODELS";
const OCRS_FILES: [&str; 2] = ["text-detection.rten", "text-rec-checkpoint.rten"];

/// The add-ons the check plants, and the remembered model it seeds.
const COPY: &str = "ui-verify-ocrs";
const VL: &str = "ui-verify-vl";
const GONE: &str = "ui-verify-gone";

/// Regions.
const OCR_PAGE: &str = "settings.heading.ocr";
const ADD: &str = "settings.ocr.add";
const SAVE: &str = "dialog:settings.save";
const COMMAND: &str = "ribbon.item.file.ocr";
const COMBO: &str = "ocr-model";
const ITEM_PREFIX: &str = "ocr-model.item.";
const RUN: &str = "ocr-run";

/// How many 20-frame waits recognition gets.
const RECOGNITION_POLLS: u32 = 40;

/// See the module documentation.
pub struct AnExtraOcrFolderAddsItsModelsToTheDropdown;

impl Check for AnExtraOcrFolderAddsItsModelsToTheDropdown {
    fn name(&self) -> &'static str {
        "an_extra_ocr_folder_adds_its_models_to_the_dropdown"
    }

    fn defect(&self) -> &'static str {
        "a folder of OCR models added in Settings is saved and never searched, so its models \
         never reach Recognise text's list; or an add-on this build cannot run is offered and \
         fails at Run; or a remembered model that has gone is silently swapped for another"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match assess(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// The engine checkout's ocrs weights, from the `file:///` git URL the GUI's
/// manifest builds against.
fn ocrs_source() -> Result<PathBuf> {
    if let Some(dir) = std::env::var_os(OCRS_MODELS_VAR) {
        return Ok(PathBuf::from(dir));
    }
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../crates/pdfcer-gui/Cargo.toml");
    let text = std::fs::read_to_string(&manifest)
        .map_err(|e| Error::new(format!("cannot read {}: {e}", manifest.display())))?;
    let engine = text
        .lines()
        .filter(|l| !l.trim_start().starts_with('#'))
        .find_map(|l| {
            let at = l.find("git = \"file:///")? + "git = \"file:///".len();
            l[at..].split('"').next().map(PathBuf::from)
        })
        .ok_or_else(|| {
            Error::new(format!(
                "{} names no file:/// engine; set {OCRS_MODELS_VAR}.",
                manifest.display()
            ))
        })?;
    Ok(engine.join("crates/pdfcer-core/assets/models/ocrs"))
}

/// Plant `root/uv-ocrs` (a runnable copy of ocrs) and `root/uv-vl` (a
/// PaddleOCR-VL manifest with no weights).
fn plant(root: &Path) -> Result<()> {
    let from = ocrs_source()?;
    let _ = std::fs::remove_dir_all(root);
    let copy = root.join("uv-ocrs");
    let vl = root.join("uv-vl");
    for dir in [&copy, &vl] {
        std::fs::create_dir_all(dir)
            .map_err(|e| Error::new(format!("cannot create {}: {e}", dir.display())))?;
    }
    for file in OCRS_FILES {
        let source = from.join(file);
        std::fs::copy(&source, copy.join(file)).map_err(|e| {
            Error::new(format!(
                "cannot copy {} (set {OCRS_MODELS_VAR}): {e}",
                source.display()
            ))
        })?;
    }
    let manifest = pdfcer_manifest_name();
    let write = |dir: &Path, body: String| {
        std::fs::write(dir.join(manifest), body)
            .map_err(|e| Error::new(format!("cannot write a manifest in {}: {e}", dir.display())))
    };
    write(
        &copy,
        format!("name = {COPY}\nengine = ocrs\nlabel = ui-verify copy of ocrs\n"),
    )?;
    write(
        &vl,
        format!("name = {VL}\nengine = paddle-vl\nlabel = ui-verify VL stub\n"),
    )
}

/// `pdfcer_core::ocr::addons::MANIFEST_FILE`, which this harness does not link.
const fn pdfcer_manifest_name() -> &'static str {
    "pdfcer-ocr-model.txt"
}

fn assess(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx
        .resolve_exe()
        .ok_or_else(|| Error::new("no binary to drive. Pass --exe."))?;
    let viewport_env = ctx
        .profile
        .viewport_env
        .ok_or_else(|| Error::new("the profile has no viewport variable."))?;
    let ui_rect = ctx.profile.vocab.ui_rect_event.unwrap_or("ui-rect");
    let pdf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(FIXTURE);
    if !pdf.is_file() {
        return Err(Error::new(format!("the fixture {FIXTURE} is not on disk.")));
    }
    // Absolute, as the native picker answers.
    let root = std::path::absolute(ctx.out("ocr-extra"))
        .map_err(|e| Error::new(format!("cannot make the folder absolute: {e}")))?;
    plant(&root)?;
    let userdata = exe
        .parent()
        .ok_or_else(|| Error::new("the binary has no parent directory"))?
        .join("userdata");
    std::fs::create_dir_all(&userdata)
        .map_err(|e| Error::new(format!("cannot create {}: {e}", userdata.display())))?;
    crate::sandbox::write_prefs(&userdata, &format!("ocr_model = {GONE}\n"))
        .map_err(|e| Error::new(format!("could not write preferences: {e}")))?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("ocr_extra_folder.trace.txt"));
    spec.pdf = Some(pdf);
    for (k, v) in [
        (ctx.profile.diag_env.0, ctx.profile.diag_env.1.to_owned()),
        (SHELL_DIAG_ENV.0, SHELL_DIAG_ENV.1.to_owned()),
        (FOLDER_ENV, root.display().to_string()),
        (INVOKE_ENV, OPEN_SETTINGS.to_owned()),
        (viewport_env, OFFSCREEN.to_owned()),
    ] {
        spec.env.push((k.to_owned(), v));
    }
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("ocr_extra_folder.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(pointer.path().to_path_buf());
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!(
        "planted {} and seeded ocr_model = {GONE}",
        root.display()
    ));
    session.settle(40);

    if let Some(failure) = add_the_folder(&session, &pointer, ui_rect, report)? {
        return Ok(Some(failure));
    }
    if let Some(failure) = read_the_list(&session, &pointer, ui_rect, report)? {
        return Ok(Some(failure));
    }
    choose_and_run(&session, &pointer, ui_rect, report)
}

/// Settings ▸ OCR models ▸ Add…, answered by the seam, then Save.
fn add_the_folder(
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
    report: &mut CheckReport,
) -> Result<Option<String>> {
    click(session, pointer, ui_rect, OCR_PAGE)?;
    click(session, pointer, ui_rect, ADD)?;
    let trace = session.trace()?;
    if trace.last("ocr-folder-added").is_none() {
        return Ok(Some(format!(
            "Add… on the OCR models page was clicked and no `ocr-folder-added` followed. Picker: \
             {}.",
            raw(&trace, "ocr-folder-picked")
        )));
    }
    report.note(format!("added: `{}`", raw(&trace, "ocr-folder-added")));
    click(session, pointer, ui_rect, SAVE)?;
    Ok(None)
}

/// Open Recognise text and read what its list holds.
fn read_the_list(
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
    report: &mut CheckReport,
) -> Result<Option<String>> {
    crate::checks::ocr::click_tab(session, pointer, ui_rect, "file")?;
    let item = driving::declared_or_in_overflow(session, pointer, ui_rect, COMMAND)?
        .ok_or_else(|| Error::new(format!("the File tab declares no `{COMMAND}`.")))?;
    crate::input::Click::click_rect(pointer, session, item)?;
    session.settle(30);
    let trace = session.trace()?;
    let Some(start) = trace.last("ocr-model-start") else {
        return Ok(Some(
            "Recognise text was clicked and its model list traced no `ocr-model-start`.".into(),
        ));
    };
    report.note(format!("`{}`", start.raw));
    let listed = |name: &str| {
        trace
            .events("ocr-model")
            .find(|l| l.get("name") == Some(name))
    };
    let Some(copy) = listed(COPY) else {
        return Ok(Some(format!(
            "★ the folder was added and saved, and Recognise text does not list `{COPY}`. It \
             searched {} root(s). The extra folders are not reaching discovery.",
            start.get("roots").unwrap_or("?")
        )));
    };
    if copy.get("runnable") != Some("yes") {
        return Ok(Some(format!(
            "the copy of ocrs is listed as unrunnable: `{}`.",
            copy.raw
        )));
    }
    let vl_why = listed(VL).and_then(|l| l.get("why").map(str::to_owned));
    if vl_why.as_deref() != Some("no-vl-runner") {
        return Ok(Some(format!(
            "the PaddleOCR-VL stub should list as unrunnable with why=no-vl-runner; it says \
             {vl_why:?}."
        )));
    }
    if start.get("chosen") != Some("none") || start.get("remembered") != Some(GONE) {
        return Ok(Some(format!(
            "`{GONE}` was remembered and is not on disk, so nothing may be chosen in its place; \
             the dialog started on `{}`.",
            start.raw
        )));
    }
    report.note("the copy lists runnable, the VL stub lists with no-vl-runner, nothing replaced");
    Ok(None)
}

/// Try the VL entry, which must not take; choose the copy; run it.
fn choose_and_run(
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
    report: &mut CheckReport,
) -> Result<Option<String>> {
    let index_of = |trace: &Trace, name: &str| {
        trace
            .events("ocr-model")
            .position(|l| l.get("name") == Some(name))
    };
    let trace = session.trace()?;
    let (copy, vl) = (index_of(&trace, COPY), index_of(&trace, VL));
    let (Some(copy), Some(vl)) = (copy, vl) else {
        return Err(Error::new("the list lost an entry between reads."));
    };
    pick(session, pointer, ui_rect, vl)?;
    if session.trace()?.last("ocr-model-chosen").is_some() {
        return Ok(Some(format!(
            "★ the PaddleOCR-VL entry was clicked and it was chosen: `{}`. An entry this build \
             cannot run must not be choosable.",
            raw(&session.trace()?, "ocr-model-chosen")
        )));
    }
    pick(session, pointer, ui_rect, copy)?;
    let trace = session.trace()?;
    if trace.last("ocr-model-chosen").and_then(|l| l.get("name")) != Some(COPY) {
        return Ok(Some(format!(
            "the copy's entry was clicked and the choice traced `{}`.",
            raw(&trace, "ocr-model-chosen")
        )));
    }
    click(session, pointer, ui_rect, RUN)?;
    let mut trace = session.trace()?;
    for _ in 0..RECOGNITION_POLLS {
        if trace.last("ocr-recognised").is_some() || trace.last("ocr-refused").is_some() {
            break;
        }
        session.settle(20);
        trace = session.trace()?;
    }
    let started = trace.last("ocr-started");
    report.note(format!("`{}`", raw(&trace, "ocr-started")));
    if started.and_then(|l| l.get("model")) != Some(COPY)
        || started.and_then(|l| l.get("source")) != Some("extra-folder")
    {
        return Ok(Some(format!(
            "★ the copy in the extra folder was chosen and the run traced `{}`.",
            raw(&trace, "ocr-started")
        )));
    }
    let Some(done) = trace.last("ocr-recognised") else {
        return Ok(Some(format!(
            "recognition with the copy did not finish: `{}`.",
            raw(&trace, "ocr-refused")
        )));
    };
    if done.get_usize("recognised").unwrap_or(0) == 0 {
        return Ok(Some(format!(
            "the copy recognised no words: `{}`.",
            done.raw
        )));
    }
    report.note(format!("`{}`", done.raw));
    Ok(None)
}

/// Open the drop-down and click entry `index`.
fn pick(session: &Session, pointer: &ScriptedPointer, ui_rect: &str, index: usize) -> Result<()> {
    let entry = format!("{ITEM_PREFIX}{index}");
    for _ in 0..3 {
        click(session, pointer, ui_rect, COMBO)?;
        if driving::declared(&session.trace()?, ui_rect, &entry).is_some() {
            return click(session, pointer, ui_rect, &entry);
        }
    }
    Err(Error::new(format!(
        "the model list did not open with `{entry}`. Entries: {}.",
        driving::list(&driving::declared_names(
            &session.trace()?,
            ui_rect,
            ITEM_PREFIX
        ))
    )))
}

/// Click a declared region in whichever viewport declared it.
fn click(session: &Session, pointer: &ScriptedPointer, ui_rect: &str, name: &str) -> Result<()> {
    let trace = session.trace()?;
    let (rect, viewport) = driving::declared_in(&trace, ui_rect, name).ok_or_else(|| {
        Error::new(format!(
            "the application declared no `{name}` region. Trace: {}.",
            session.trace_path().display()
        ))
    })?;
    pointer.click_in(session, viewport.as_deref(), WindowPoint::centre_of(rect))?;
    session.settle(20);
    Ok(())
}

/// The last `event` line, or `none`.
fn raw(trace: &Trace, event: &str) -> String {
    trace
        .last(event)
        .map_or_else(|| "none".to_owned(), |l| l.raw.clone())
}
