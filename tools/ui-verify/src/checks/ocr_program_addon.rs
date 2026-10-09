//! `a_program_ocr_addon_runs_and_is_disclosed` — a Tesseract program add-on
//! in an extra OCR folder is listed runnable and labelled with its program,
//! runs when chosen, and the dialog names the program it started; with
//! `ocr_program_addons = refuse` the same add-on lists as refused and nothing
//! is chosen in its place.
//!
//! The program is the engine's stand-in, `pdfcer-ocr-test-engine.exe`, copied
//! in as `tesseract.exe`: it speaks Tesseract's protocol and answers with five
//! words naming how it was run (the last is the `--dpi` it was given), so the
//! check needs no Tesseract install.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/ocr_program_addon.md`.

use std::path::{Path, PathBuf};
use std::process::Command;

use super::ocr_extra_folder::{click, pdfcer_manifest_name, raw};
use crate::checks::driving::{self, SHELL_DIAG_ENV};
use crate::checks::{Check, CheckContext, CheckReport};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};

/// Off the desktop, so the check runs while the operator uses the machine.
const OFFSCREEN: &str = "-4200,-4200,1200,1350";
/// Image-only, so every recognised word came from the program.
const FIXTURE: &str = "fixtures/synthetic-image-only.pdf";
/// The stand-in, beside the binary under test unless this names it.
const STAND_IN_VAR: &str = "UI_VERIFY_OCR_TEST_ENGINE";
const STAND_IN: &str = "pdfcer-ocr-test-engine.exe";
/// The add-on the check plants.
pub(crate) const ADDON: &str = "ui-verify-tess";
const PROGRAM: &str = "tesseract.exe";
const DATA: &str = "tessdata/eng.traineddata";
/// Regions.
const COMMAND: &str = "ribbon.item.file.ocr";
const RUN: &str = "ocr-run";
/// The stand-in answers with exactly this many words when run with the
/// built-in word lists and no word file.
pub(crate) const STAND_IN_WORDS: usize = 5;
/// How many 20-frame waits recognition gets.
const RECOGNITION_POLLS: u32 = 40;

/// See the module documentation.
pub struct AProgramOcrAddonRunsAndIsDisclosed;

impl Check for AProgramOcrAddonRunsAndIsDisclosed {
    fn name(&self) -> &'static str {
        "a_program_ocr_addon_runs_and_is_disclosed"
    }

    fn defect(&self) -> &'static str {
        "a Tesseract add-on is never offered, or runs without the dialog saying which program \
         read the pages, or still runs after program add-ons were refused in Settings"
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

fn assess(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx
        .resolve_exe()
        .ok_or_else(|| Error::new("no binary to drive. Pass --exe."))?;
    let root = std::path::absolute(ctx.out("ocr-program"))
        .map_err(|e| Error::new(format!("cannot make the folder absolute: {e}")))?;
    plant(&root, &stand_in(ctx)?)?;
    report.note(format!("planted {}", root.join(ADDON).display()));

    let allowed = format!("ocr_folder = {}\nocr_model = {ADDON}\n", root.display());
    let session = launch(ctx, &exe, &allowed, "allowed", &[], report)?;
    if let Some(failure) = run_it(&session.0, &session.1, ctx, report)? {
        return Ok(Some(failure));
    }
    drop(session);

    let refused = format!("{allowed}ocr_program_addons = refuse\n");
    let session = launch(ctx, &exe, &refused, "refused", &[], report)?;
    refused_listing(&session.0, &session.1, ctx, report)
}

/// The stand-in program: [`STAND_IN_VAR`], else beside the profile's built
/// binary (the driven copy lives in a per-check directory without it).
pub(crate) fn stand_in(ctx: &CheckContext) -> Result<PathBuf> {
    let built = Path::new(ctx.profile.default_exe).with_file_name(STAND_IN);
    let path = std::env::var_os(STAND_IN_VAR).map_or(built, PathBuf::from);
    if path.is_file() {
        return Ok(path);
    }
    Err(Error::new(format!(
        "{} is not there. Build it with `cargo build --release -p pdfcer-ocr-host --bin \
         pdfcer-ocr-test-engine`, or set {STAND_IN_VAR}.",
        path.display()
    )))
}

/// Plant `root/ui-verify-tess`: the stand-in as `tesseract.exe`, one
/// language file, and a program manifest hashing both.
pub(crate) fn plant(root: &Path, program: &Path) -> Result<()> {
    let _ = std::fs::remove_dir_all(root);
    let dir = root.join(ADDON);
    let io = |what: &str, e: std::io::Error| Error::new(format!("cannot {what}: {e}"));
    std::fs::create_dir_all(dir.join("tessdata")).map_err(|e| io("create the add-on", e))?;
    std::fs::copy(program, dir.join(PROGRAM)).map_err(|e| io("copy the stand-in", e))?;
    std::fs::write(dir.join(DATA), b"eng").map_err(|e| io("write the language file", e))?;
    let mut manifest =
        format!("name = {ADDON}\nengine = tesseract\nkind = program\nprogram = {PROGRAM}\n");
    for file in [PROGRAM, DATA] {
        let hex = sha256(&dir.join(file))?;
        manifest.push_str(&format!("sha256 = {file} {hex}\n"));
    }
    std::fs::write(dir.join(pdfcer_manifest_name()), manifest)
        .map_err(|e| io("write the manifest", e))
}

/// A file's SHA-256 in lowercase hex, from `certutil`, which every Windows
/// carries; this harness links no hashing crate.
fn sha256(path: &Path) -> Result<String> {
    let out = Command::new("certutil")
        .args(["-hashfile", &path.display().to_string(), "SHA256"])
        .output()
        .map_err(|e| Error::new(format!("cannot run certutil: {e}")))?;
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(|l| l.replace(' ', "").to_ascii_lowercase())
        .find(|l| l.len() == 64 && l.bytes().all(|b| b.is_ascii_hexdigit()))
        .ok_or_else(|| Error::new(format!("certutil gave no SHA-256 for {}", path.display())))
}

/// Seed the preferences and open the fixture, with `env` set as well.
pub(crate) fn launch(
    ctx: &CheckContext,
    exe: &Path,
    prefs: &str,
    tag: &str,
    env: &[(&str, &str)],
    report: &mut CheckReport,
) -> Result<(Session, ScriptedPointer)> {
    let viewport_env = ctx
        .profile
        .viewport_env
        .ok_or_else(|| Error::new("the profile has no viewport variable."))?;
    let pdf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(FIXTURE);
    let userdata = exe
        .parent()
        .ok_or_else(|| Error::new("the binary has no parent directory"))?
        .join("userdata");
    crate::sandbox::write_prefs(&userdata, prefs)
        .map_err(|e| Error::new(format!("could not write preferences: {e}")))?;
    let mut spec = LaunchSpec::new(exe, ctx.out(&format!("ocr_program_addon.{tag}.trace.txt")));
    spec.pdf = Some(pdf);
    for (k, v) in [
        (ctx.profile.diag_env.0, ctx.profile.diag_env.1),
        (SHELL_DIAG_ENV.0, SHELL_DIAG_ENV.1),
        (viewport_env, OFFSCREEN),
    ]
    .into_iter()
    .chain(env.iter().copied())
    {
        spec.env.push((k.to_owned(), v.to_owned()));
    }
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(
        &mut spec,
        ctx.out(&format!("ocr_program_addon.{tag}.pointer.txt")),
    )?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(pointer.path().to_path_buf());
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);
    Ok((session, pointer))
}

/// Open Recognise text and return the add-on's `ocr-model` line.
pub(crate) fn open_dialog(
    session: &Session,
    pointer: &ScriptedPointer,
    ctx: &CheckContext,
) -> Result<Option<crate::trace::TraceLine>> {
    let ui_rect = ctx.profile.vocab.ui_rect_event.unwrap_or("ui-rect");
    crate::checks::ocr::click_tab(session, pointer, ui_rect, "file")?;
    let item = driving::declared_or_in_overflow(session, pointer, ui_rect, COMMAND)?
        .ok_or_else(|| Error::new(format!("the File tab declares no `{COMMAND}`.")))?;
    crate::input::Click::click_rect(pointer, session, item)?;
    session.settle(30);
    Ok(session
        .trace()?
        .events("ocr-model")
        .find(|l| l.get("name") == Some(ADDON))
        .cloned())
}

/// Allowed: listed runnable, started from start, run, and disclosed.
fn run_it(
    session: &Session,
    pointer: &ScriptedPointer,
    ctx: &CheckContext,
    report: &mut CheckReport,
) -> Result<Option<String>> {
    let ui_rect = ctx.profile.vocab.ui_rect_event.unwrap_or("ui-rect");
    let Some(listed) = open_dialog(session, pointer, ctx)? else {
        return Ok(Some(format!(
            "★ the add-on is in an extra folder and Recognise text does not list `{ADDON}`."
        )));
    };
    report.note(format!("allowed: `{}`", listed.raw));
    if listed.get("runnable") != Some("yes") || listed.get("engine") != Some("tesseract") {
        return Ok(Some(format!(
            "★ a hashed Tesseract add-on with program add-ons allowed lists as `{}`.",
            listed.raw
        )));
    }
    let trace = session.trace()?;
    if trace.last("ocr-model-start").and_then(|l| l.get("chosen")) != Some(ADDON) {
        return Ok(Some(format!(
            "`{ADDON}` was remembered and is runnable, and the dialog started on `{}`.",
            raw(&trace, "ocr-model-start")
        )));
    }
    click(session, pointer, ui_rect, RUN)?;
    let mut trace = session.trace()?;
    for _ in 0..RECOGNITION_POLLS {
        if trace.last("ocr-applied").is_some() || trace.last("ocr-refused").is_some() {
            break;
        }
        session.settle(20);
        trace = session.trace()?;
    }
    report.note(format!("`{}`", raw(&trace, "ocr-started")));
    let started = trace.last("ocr-started");
    if started.and_then(|l| l.get("engine")) != Some("tesseract") {
        return Ok(Some(format!(
            "the run traced `{}`; it should have started engine=tesseract.",
            raw(&trace, "ocr-started")
        )));
    }
    let Some(applied) = trace.last("ocr-applied") else {
        return Ok(Some(format!(
            "★ the add-on was chosen and Run pressed, and nothing was applied: `{}`.",
            raw(&trace, "ocr-refused")
        )));
    };
    report.note(format!("`{}`", applied.raw));
    let program = applied.raw.split(" program=").nth(1).unwrap_or("none");
    let expected = Path::new(PROGRAM);
    if applied.get_usize("words") != Some(STAND_IN_WORDS)
        || applied.get("scored") != Some("true")
        || applied.get_usize("disclosed").unwrap_or(0) == 0
        || !Path::new(program).ends_with(Path::new(ADDON).join(expected))
    {
        return Ok(Some(format!(
            "★ the stand-in answers {STAND_IN_WORDS} scored words and the run must name \
             {ADDON}\\{PROGRAM} as the program that read them; the dialog traced `{}`.",
            applied.raw
        )));
    }
    Ok(None)
}

/// Refused: the same add-on lists unrunnable with `refused-by-policy`, and
/// the remembered choice is not replaced.
fn refused_listing(
    session: &Session,
    pointer: &ScriptedPointer,
    ctx: &CheckContext,
    report: &mut CheckReport,
) -> Result<Option<String>> {
    let listed = open_dialog(session, pointer, ctx)?;
    if let Some(l) = &listed {
        report.note(format!("refused: `{}`", l.raw));
    }
    let Some(listed) = listed else {
        return Ok(Some(format!(
            "with program add-ons refused, `{ADDON}` vanished from the list; it must stay listed \
             with its reason."
        )));
    };
    if listed.get("runnable") != Some("no") || listed.get("why") != Some("refused-by-policy") {
        return Ok(Some(format!(
            "★ `ocr_program_addons = refuse` is set and the add-on lists as `{}`; it must be \
             runnable=no why=refused-by-policy.",
            listed.raw
        )));
    }
    let trace = session.trace()?;
    if trace.last("ocr-model-start").and_then(|l| l.get("chosen")) != Some("none") {
        return Ok(Some(format!(
            "the remembered add-on is refused, so nothing may be chosen in its place; the dialog \
             started on `{}`.",
            raw(&trace, "ocr-model-start")
        )));
    }
    Ok(None)
}
