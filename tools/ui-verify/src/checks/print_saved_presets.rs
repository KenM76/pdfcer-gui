//! `print_settings_save_as_a_preset_and_come_back` (O418): the Print window's
//! Save as… writes a `.pdfcerprint` file to the settings folder's
//! `print-presets`, and choosing a preset from the list puts its settings into
//! the window.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/print_saved_presets.md`.

use std::path::{Path, PathBuf};

use crate::checks::driving::{SHELL_DIAG_ENV, declared_in, repo_fixture};
use crate::checks::region_press::press_region;
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const OFFSCREEN: &str = "-4200,-4200,1400,900";
const FIXTURE: &str = "print-colour.pdf";
const METHOD: &str = "Rebuild it with `python fixtures/print-colour.PROVENANCE.py`.";
const OPEN: &str = "print-open";
const LIST: &str = "print-preset-list";
const SAVED: &str = "print-preset-saved";
const APPLIED: &str = "print-preset-applied";
const FAMILY: &str = "print.preset.";
/// The name typed into Save as.
const TYPED: &str = "Driven preset";
/// A preset seeded before launch, holding a copy count the window's default is not.
const SEEDED: &str = "Seeded four copies";
const SEEDED_COPIES: &str = "4";

/// See the module documentation.
pub struct PrintSettingsSaveAsAPreset;

impl Check for PrintSettingsSaveAsAPreset {
    fn name(&self) -> &'static str {
        "print_settings_save_as_a_preset_and_come_back"
    }

    fn defect(&self) -> &'static str {
        "the Print window cannot save its settings as a named preset, or choosing a preset \
         does not put its settings back"
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

/// The default preset folder of the binary under test.
fn folder(exe: &Path) -> Result<PathBuf> {
    exe.parent()
        .map(|dir| dir.join("userdata").join("print-presets"))
        .ok_or_else(|| Error::new("the executable under test has no parent directory."))
}

/// Seed [`SEEDED`] in an emptied preset folder.
fn seed(presets: &Path) -> Result<()> {
    let _ = std::fs::remove_dir_all(presets);
    std::fs::create_dir_all(presets)
        .map_err(|e| Error::new(format!("creating {}: {e}", presets.display())))?;
    std::fs::write(
        presets.join(format!("{SEEDED}.pdfcerprint")),
        format!("print_copies = {SEEDED_COPIES}\n"),
    )
    .map_err(|e| Error::new(format!("seeding a preset: {e}")))
}

fn launch(
    ctx: &CheckContext,
    report: &mut CheckReport,
) -> Result<(Session, ScriptedPointer, PathBuf)> {
    let exe = ctx
        .resolve_exe()
        .ok_or_else(|| Error::new("no binary to drive. Pass --exe, or build the default."))?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let presets = folder(&exe)?;
    seed(&presets)?;
    let doc = ctx.out(FIXTURE);
    std::fs::copy(repo_fixture(FIXTURE, METHOD)?, &doc)
        .map_err(|e| Error::new(format!("copying {FIXTURE}: {e}")))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("print_saved_presets.trace.txt"));
    spec.pdf = Some(doc);
    for (k, v) in [
        (ctx.profile.diag_env.0, ctx.profile.diag_env.1),
        SHELL_DIAG_ENV,
        (viewport_env, OFFSCREEN),
        ("PDFCER_DIAG_INVOKE", "file.print"),
        ("PDFCER_DIAG_PRINT_DRY_RUN", "1"),
    ] {
        spec.env.push((k.to_owned(), v.to_owned()));
    }
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("print_saved_presets.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(60);
    Ok((session, pointer, presets))
}

/// Save as, type [`TYPED`], Save; a failure when no file holding the window's
/// settings results.
fn save(session: &Session, pointer: &ScriptedPointer, presets: &Path) -> Result<Option<String>> {
    press_region(session, pointer, "print.preset.save-as", FAMILY)?;
    let trace = session.trace()?;
    let Some((_, viewport)) = declared_in(&trace, "ui-rect", "print.preset.name") else {
        return Ok(Some(
            "★★★ Save as… was pressed and no name box was drawn.".to_owned(),
        ));
    };
    pointer.type_text(session, viewport.as_deref(), TYPED)?;
    session.settle(10);
    press_region(session, pointer, "print.preset.commit", FAMILY)?;
    let trace = session.trace()?;
    let Some(line) = trace.last(SAVED) else {
        return Ok(Some(format!(
            "★★★ Save was pressed and no `{SAVED}` line followed."
        )));
    };
    let file = presets.join(format!("{TYPED}.pdfcerprint"));
    if line.get("ok") != Some("true") || !line.raw.contains(TYPED) || !file.is_file() {
        return Ok(Some(format!(
            "★★★ saving traced `{}`, and {} exists: {}. The name typed was {TYPED:?}.",
            line.raw,
            file.display(),
            file.is_file()
        )));
    }
    let text = std::fs::read_to_string(&file).unwrap_or_default();
    if !text.contains("print_copies = 1") {
        return Ok(Some(format!(
            "★★ the saved preset does not hold the window's copy count of 1: {text:?}"
        )));
    }
    Ok(None)
}

/// Open the list and choose [`SEEDED`], second by name after [`TYPED`].
fn choose(session: &Session, pointer: &ScriptedPointer) -> Result<Option<String>> {
    press_region(session, pointer, "print.preset.combo", FAMILY)?;
    press_region(session, pointer, "print.preset.item.1", FAMILY)?;
    let trace = session.trace()?;
    let Some(line) = trace.last(APPLIED) else {
        return Ok(Some(format!(
            "★★★ a preset was chosen from the list and no `{APPLIED}` line followed."
        )));
    };
    if !line.raw.contains(SEEDED) || line.get("copies") != Some(SEEDED_COPIES) {
        return Ok(Some(format!(
            "★★★ choosing {SEEDED:?}, which holds {SEEDED_COPIES} copies, traced `{}`.",
            line.raw
        )));
    }
    Ok(None)
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let (session, pointer, presets) = launch(ctx, report)?;
    let trace = session.trace()?;
    if trace.last(OPEN).is_none() {
        return Err(Error::new(format!(
            "`file.print` was rung and no `{OPEN}` line followed. That is \
             `print_dialog_reaches_the_spooler`'s subject."
        )));
    }
    let listed = trace.last(LIST).map(|l| l.raw.clone());
    if trace.last(LIST).and_then(|l| l.get("count")) != Some("1") {
        return Ok(Some(format!(
            "★★★ the window listed {listed:?} with one preset seeded in {}.",
            presets.display()
        )));
    }
    if let Some(failure) = save(&session, &pointer, &presets)? {
        return Ok(Some(failure));
    }
    if let Some(failure) = choose(&session, &pointer)? {
        return Ok(Some(failure));
    }
    for event in [SAVED, APPLIED] {
        if let Some(line) = session.trace()?.last(event) {
            report.note(line.raw.clone());
        }
    }
    pointer.gone(&session)?;
    let _ = std::fs::remove_dir_all(&presets);
    Ok(None)
}
