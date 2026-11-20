//! `print_preview_fits_width_and_height` — the print preview's Fit width and
//! Fit height buttons, pressed off-screen with the scripted pointer, fill the
//! preview with the sheet's width or height as Fit fills it with the whole
//! sheet. Read from the preview's own `print-preview` trace line: the canvas
//! rect, the sheet in PDF points and the drawn scale.

use crate::checks::driving::{SHELL_DIAG_ENV, declared_in, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const OFFSCREEN: &str = "-4200,-4200,1400,900";
const REGION_WIDTH: &str = "print.preview.fit-width";
const REGION_HEIGHT: &str = "print.preview.fit-height";
/// The share of the canvas Fit fills (`FIT_MARGIN` in the preview).
const FIT_MARGIN: f32 = 0.92;
/// How far the filled length may be from the target, in points.
const TOLERANCE_PTS: f32 = 1.0;

pub struct PrintPreviewFitsWidthAndHeight;

impl Check for PrintPreviewFitsWidthAndHeight {
    fn name(&self) -> &'static str {
        "print_preview_fits_width_and_height"
    }

    fn defect(&self) -> &'static str {
        "the print preview's Fit width or Fit height button is missing, does nothing, or sizes \
         the sheet to something other than the preview's width or height"
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

/// The last `print-preview` line's canvas size, sheet size and scale.
struct Drawn {
    canvas: (f32, f32),
    sheet: (f32, f32),
    scale: f32,
}

fn pair(raw: &str, key: &str, open: &str, close: char) -> Option<(f32, f32)> {
    let after = raw.split_once(&format!("{key}={open}"))?.1;
    let inside = after.split(close).next()?;
    let size = inside.rsplit(' ').next()?;
    let (w, h) = size.split_once('x')?;
    Some((w.parse().ok()?, h.parse().ok()?))
}

fn drawn(session: &Session) -> Result<Drawn> {
    let trace = session.trace()?;
    let line = trace.events("print-preview").last().ok_or_else(|| {
        Error::new("the print dialog drew no preview, so no fit could be measured.".to_owned())
    })?;
    let raw = &line.raw;
    let scale = raw
        .split_once("scale=")
        .and_then(|(_, r)| r.split(' ').next()?.parse().ok());
    match (
        pair(raw, "canvas", "[", ']'),
        pair(raw, "sheet", "", ' '),
        scale,
    ) {
        (Some(canvas), Some(sheet), Some(scale)) => Ok(Drawn {
            canvas,
            sheet,
            scale,
        }),
        _ => Err(Error::new(format!("could not read `{raw}`."))),
    }
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx
        .pdf
        .clone()
        .ok_or_else(|| Error::new("no --pdf: the print dialog needs a document.".to_owned()))?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("print-preview-fit.trace.txt"));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), "file.print".to_owned()));
    spec.env
        .push((viewport_env.to_owned(), OFFSCREEN.to_owned()));
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("print-preview-fit.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);

    let mut failures = Vec::new();
    // Height first: on a sheet whose Fit is already width-limited, Fit width
    // pressed first would pass without doing anything.
    for (region, label, along) in [
        (REGION_HEIGHT, "Fit height", 1),
        (REGION_WIDTH, "Fit width", 0),
    ] {
        let trace = session.trace()?;
        let (rect, viewport) = declared_in(&trace, ui_rect, region).ok_or_else(|| {
            Error::new(format!(
                "no `{region}` region. Declared under `print.preview`: {}.",
                list(&declared_names(&trace, ui_rect, "print.preview"))
            ))
        })?;
        pointer.click_in(&session, viewport.as_deref(), WindowPoint::centre_of(rect))?;
        session.settle(15);
        let d = drawn(&session)?;
        let (sheet, canvas) = if along == 0 {
            (d.sheet.0, d.canvas.0)
        } else {
            (d.sheet.1, d.canvas.1)
        };
        let filled = sheet * d.scale;
        let target = canvas * FIT_MARGIN;
        report.note(format!(
            "{label}: the sheet's {sheet} pt drawn {filled:.1} pt across a {canvas:.0} pt \
             preview, target {target:.1} pt"
        ));
        if (filled - target).abs() > TOLERANCE_PTS {
            failures.push(format!(
                "{label} drew the sheet's {sheet} pt as {filled:.1} pt where the preview's \
                 {canvas:.0} pt calls for {target:.1} pt"
            ));
        }
    }
    Ok((!failures.is_empty()).then(|| failures.join("; ")))
}
