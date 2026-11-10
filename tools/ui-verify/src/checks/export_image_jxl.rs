//! `export_image_writes_jpeg_xl` — File ▸ Export image offers JPEG XL and the
//! press writes a JPEG XL file, not a PNG under a `.jxl` name.
//!
//! Driven off the desktop with the scripted pointer: `file.export_image` rung
//! on `four-pages-unrotated.pdf`, the JPEG XL radio pressed, Export pressed,
//! and `PDFCER_DIAG_SAVE_PATH` answering the save dialog. The plan must say
//! `format=jxl` and the file must open with a JPEG XL signature: the bare
//! codestream's `FF 0A` or the container's `00 00 00 0C 4A 58 4C 20`. The
//! build without the encoder falls back to PNG bytes, which this rejects.

use crate::checks::driving::{SHELL_DIAG_ENV, repo_fixture};
use crate::checks::region_press::press_region;
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const OFFSCREEN: &str = "-4200,-4200,1400,900";
const FIXTURE: &str = "four-pages-unrotated.pdf";
const METHOD: &str = "Rebuild it from its PROVENANCE file in `fixtures/`.";
const OPENED: &str = "export-image-open";
const REQUESTED: &str = "export-image-requested";
const WROTE: &str = "export-image";
const RADIO: &str = "export-image.format.jxl";
const EXPORT: &str = "export-image.export";
const CODESTREAM: &[u8] = &[0xFF, 0x0A];
const CONTAINER: &[u8] = &[0, 0, 0, 0x0C, b'J', b'X', b'L', b' '];

/// See the module documentation.
pub struct ExportImageWritesJpegXl;

impl Check for ExportImageWritesJpegXl {
    fn name(&self) -> &'static str {
        "export_image_writes_jpeg_xl"
    }

    fn defect(&self) -> &'static str {
        "File > Export image does not offer JPEG XL, or its press writes no file, or writes \
         bytes that are not JPEG XL under a .jxl name"
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
) -> Result<(Session, ScriptedPointer)> {
    let exe = ctx
        .resolve_exe()
        .ok_or_else(|| Error::new("no binary to drive. Pass --exe, or build the default."))?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let doc = ctx.out(FIXTURE);
    std::fs::copy(repo_fixture(FIXTURE, METHOD)?, &doc)
        .map_err(|e| Error::new(format!("copying {FIXTURE}: {e}")))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("export_image_jxl.trace.txt"));
    spec.pdf = Some(doc);
    let save = target.display().to_string();
    for (k, v) in [
        (ctx.profile.diag_env.0, ctx.profile.diag_env.1),
        SHELL_DIAG_ENV,
        (viewport_env, OFFSCREEN),
        ("PDFCER_DIAG_INVOKE", "file.export_image"),
        ("PDFCER_DIAG_SAVE_PATH", save.as_str()),
    ] {
        spec.env.push((k.to_owned(), v.to_owned()));
    }
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("export_image_jxl.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(60);
    Ok((session, pointer))
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let target = ctx.out("export_image.jxl");
    let _ = std::fs::remove_file(&target);
    if target.exists() {
        return Err(Error::new(format!(
            "cannot clear {} before the run.",
            target.display()
        )));
    }
    let (session, pointer) = launch(ctx, report, &target)?;
    if session.trace()?.last(OPENED).is_none() {
        return Err(Error::new(format!(
            "`file.export_image` was rung and no `{OPENED}` line followed."
        )));
    }
    press_region(&session, &pointer, RADIO, "export-image.format")?;
    press_region(&session, &pointer, EXPORT, "export-image.")?;
    session.settle(240);
    let trace = session.trace()?;
    let Some(requested) = trace.last(REQUESTED) else {
        return Ok(Some(format!(
            "Export was pressed and no `{REQUESTED}` line followed."
        )));
    };
    report.note(requested.raw.clone());
    if requested.get("format") != Some("jxl") {
        return Ok(Some(format!(
            "★★★ the JPEG XL radio was pressed and the plan says otherwise: `{}`.",
            requested.raw
        )));
    }
    if let Some(wrote) = trace.last(WROTE) {
        report.note(wrote.raw.clone());
    }
    pointer.gone(&session)?;
    let bytes = std::fs::read(&target).map_err(|e| {
        Error::new(format!(
            "no file at {} after the export: {e}",
            target.display()
        ))
    })?;
    if !(bytes.starts_with(CODESTREAM) || bytes.starts_with(CONTAINER)) {
        return Ok(Some(format!(
            "★★★ {} is {} bytes and has no JPEG XL signature; it starts {:02X?}, and a PNG \
             would start [89, 50, 4E, 47].",
            target.display(),
            bytes.len(),
            &bytes[..bytes.len().min(8)]
        )));
    }
    report.note(format!("{} bytes of JPEG XL", bytes.len()));
    Ok(None)
}
