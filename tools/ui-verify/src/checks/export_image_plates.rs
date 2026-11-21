//! `export_image_plates` — **File ▸ Export image ▸ EMF ▸ One file per ink
//! writes a metafile for every plate, each named for its ink**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/export_image_plates.md`.

use crate::checks::driving::SHELL_DIAG_ENV;
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Off every monitor, unfocused: the drive needs neither mouse nor keyboard.
const OFFSCREEN: &str = "-4200,-4200,1400,1000";
const INVOKE: &str = "file.export_image";
const REGION_EMF: &str = "export-image.format.emf";
const REGION_PLATES: &str = "export-image.plates";
const REGION_EXPORT: &str = "export-image.export";
/// The fixture's spot inks, as `spots_flattened::spots_pdf` names them.
const SPOTS: [&str; 2] = ["Ink0", "Ink1"];
/// `ENHMETA_SIGNATURE` at offset 40 of every enhanced metafile.
const ENHMETA_SIGNATURE: &[u8] = b" EMF";

/// See the module documentation.
pub struct ExportImageWritesOneMetafilePerInk;

impl Check for ExportImageWritesOneMetafilePerInk {
    fn name(&self) -> &'static str {
        "export_image_writes_one_metafile_per_ink"
    }

    fn defect(&self) -> &'static str {
        "Export image offers no plates choice under EMF, or the choice is ignored and one \
         composite metafile is written, or a spot ink gets no file of its own, or a plate file \
         is not a metafile"
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

fn launch(
    ctx: &CheckContext,
    report: &mut CheckReport,
    target: &std::path::Path,
) -> Result<(Session, ScriptedPointer, &'static str)> {
    let exe = ctx
        .resolve_exe()
        .ok_or_else(|| Error::new("no binary to drive. Pass --exe."))?;
    let viewport_env = ctx
        .profile
        .viewport_env
        .ok_or_else(|| Error::new("the profile has no viewport variable."))?;
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let doc = ctx.out("export-plates-source.pdf");
    std::fs::write(&doc, super::spots_flattened::spots_pdf(SPOTS.len()))
        .map_err(|e| Error::new(format!("writing {}: {e}", doc.display())))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("export-plates.trace.txt"));
    spec.pdf = Some(doc);
    for (k, v) in [
        (ctx.profile.diag_env.0, ctx.profile.diag_env.1.to_owned()),
        (SHELL_DIAG_ENV.0, SHELL_DIAG_ENV.1.to_owned()),
        (viewport_env, OFFSCREEN.to_owned()),
        ("PDFCER_DIAG_INVOKE", INVOKE.to_owned()),
        ("PDFCER_DIAG_SAVE_PATH", target.display().to_string()),
    ] {
        spec.env.push((k.to_owned(), v));
    }
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("export-plates.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(60);
    Ok((session, pointer, ui_rect))
}

/// Remove every metafile an earlier run left, so none is mistaken for this run's.
fn clear(dir: &std::path::Path) -> Result<()> {
    let _ = std::fs::create_dir_all(dir);
    for entry in std::fs::read_dir(dir).map_err(|e| Error::new(e.to_string()))? {
        let path = entry.map_err(|e| Error::new(e.to_string()))?.path();
        if path.extension().is_some_and(|x| x == "emf") {
            std::fs::remove_file(&path).map_err(|e| Error::new(e.to_string()))?;
        }
    }
    Ok(())
}

/// Wheel the window's scrolled body until the plates checkbox is fully shown;
/// `false` when it never is.
fn scroll_to_plates(session: &Session, pointer: &ScriptedPointer, ui_rect: &str) -> Result<bool> {
    use crate::checks::driving::{declared, declared_in};
    let trace = session.trace()?;
    let (radio, viewport) = declared_in(&trace, ui_rect, REGION_EMF)
        .ok_or_else(|| Error::new(format!("no `{REGION_EMF}` region.")))?;
    let at = crate::coords::WindowPoint::centre_of(radio);
    for _ in 0..12 {
        if declared(&session.trace()?, ui_rect, REGION_PLATES).is_some() {
            return Ok(true);
        }
        pointer.wheel_in(session, viewport.as_deref(), at, -120.0)?;
        session.settle(8);
    }
    Ok(declared(&session.trace()?, ui_rect, REGION_PLATES).is_some())
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let dir = ctx.out("export-plates");
    clear(&dir)?;
    let target = dir.join("plates.emf");
    let (session, pointer, ui_rect) = launch(ctx, report, &target)?;
    pointer.click_region(&session, ui_rect, REGION_EMF)?;
    session.settle(15);
    if !scroll_to_plates(&session, &pointer, ui_rect)? {
        pointer.gone(&session)?;
        return Ok(Some(format!(
            "★★★ EMF is selected and `{REGION_PLATES}` was never declared: the window offers \
             no separation plates."
        )));
    }
    pointer.click_region(&session, ui_rect, REGION_PLATES)?;
    session.settle(10);
    pointer.click_region(&session, ui_rect, REGION_EXPORT)?;
    for _ in 0..40 {
        if session.trace()?.events("export-image-plate").count() >= 4 + SPOTS.len() {
            break;
        }
        session.settle(10);
    }
    let trace = session.trace()?;
    pointer.gone(&session)?;
    if let Some(r) = trace.events("export-image-requested").last() {
        report.note(format!("requested: {}", r.raw));
    }
    verdict(&trace, &dir, report)
}

/// The traced plates, the files on disk and the spot inks agree.
fn verdict(
    trace: &crate::trace::Trace,
    dir: &std::path::Path,
    report: &mut CheckReport,
) -> Result<Option<String>> {
    let plates: Vec<String> = trace
        .events("export-image-plate")
        .filter_map(|l| l.get("ink").map(str::to_owned))
        .collect();
    report.note(format!("plates traced: {plates:?}"));
    let mut files: Vec<String> = std::fs::read_dir(dir)
        .map_err(|e| Error::new(e.to_string()))?
        .filter_map(|e| e.ok().map(|e| e.file_name().to_string_lossy().into_owned()))
        .filter(|n| n.ends_with(".emf"))
        .collect();
    files.sort();
    report.note(format!("files written: {files:?}"));
    if plates.len() < 4 + SPOTS.len() || files.len() != plates.len() {
        return Ok(Some(format!(
            "★★★ {} plate(s) traced and {} metafile(s) on disk; a page with two spot inks owes \
             one per process ink and one per spot.",
            plates.len(),
            files.len()
        )));
    }
    for spot in SPOTS {
        let Some(name) = files.iter().find(|n| n.contains(spot)) else {
            return Ok(Some(format!(
                "★★★ no file is named for the spot ink {spot}."
            )));
        };
        let bytes = std::fs::read(dir.join(name)).map_err(|e| Error::new(e.to_string()))?;
        if bytes.get(40..44) != Some(ENHMETA_SIGNATURE) {
            return Ok(Some(format!("★★★ {name} is not an enhanced metafile.")));
        }
    }
    Ok(None)
}
