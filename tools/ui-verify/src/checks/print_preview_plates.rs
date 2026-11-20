//! `print_preview_shows_plates` — under Colour ▸ Separations the print
//! preview's plates switch shows the page as its plates, and the ink arrows
//! step through them.
//!
//! Opens Print (off the desktop, scripted pointer) on
//! `fixtures/transparency-cmyk.pdf`, chooses Separations, presses the plates
//! switch, then the next-ink arrow. Oracle, from the preview's own trace:
//! before the switch the drawn raster is in colour (`grey=false`); after it,
//! plate 0 is drawn and every pixel is neutral (`grey=true`); after the arrow,
//! plate 1 is drawn, grey, under a different ink name. Ignores `--pdf`; the
//! fixture is the method.

use crate::checks::driving::{SHELL_DIAG_ENV, repo_fixture};
use crate::checks::region_press::press_region;
use crate::checks::{Check, CheckContext, CheckReport};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};

const OFFSCREEN: &str = "-4200,-4200,1400,900";
const FIXTURE: &str = "transparency-cmyk.pdf";
const METHOD: &str = "A one-page drawing painted in CMYK, so it has process plates to separate. \
                      Rebuild it from its PROVENANCE file.";
const RENDER: &str = "print-preview-render";
const PLATE: &str = "print-preview-plate";
const TAB: &str = "print.tab.colour";
const SEPARATIONS: &str = "print.colour.separations";
const SWITCH: &str = "print.preview.plates";
const NEXT: &str = "print.preview.plate-next";

pub struct PrintPreviewShowsPlates;

impl Check for PrintPreviewShowsPlates {
    fn name(&self) -> &'static str {
        "print_preview_shows_plates"
    }

    fn defect(&self) -> &'static str {
        "the print preview cannot show a separated page's plates, shows them in colour, or its \
         ink arrows do not step to the next plate"
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

fn launch(ctx: &CheckContext, report: &mut CheckReport) -> Result<(Session, ScriptedPointer)> {
    let exe = ctx
        .resolve_exe()
        .ok_or_else(|| Error::new("no binary to drive. Pass --exe, or build the default."))?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let doc = ctx.out(FIXTURE);
    std::fs::copy(repo_fixture(FIXTURE, METHOD)?, &doc)
        .map_err(|e| Error::new(format!("copying {FIXTURE}: {e}")))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("print_preview_plates.trace.txt"));
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
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("print_preview_plates.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(60);
    Ok((session, pointer))
}

/// The value of `key` on `line`.
fn field(line: &str, key: &str) -> Option<String> {
    line.split(&format!(" {key}="))
        .nth(1)
        .and_then(|rest| rest.split(' ').next())
        .map(str::to_owned)
}

/// The last line of `event`, raw.
fn last(session: &Session, event: &str) -> Result<Option<String>> {
    Ok(session.trace()?.last(event).map(|l| l.raw.clone()))
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let (session, pointer) = launch(ctx, report)?;
    press_region(&session, &pointer, TAB, "print.tab.")?;
    press_region(&session, &pointer, SEPARATIONS, "print.colour.")?;
    session.settle(30);
    let colour = last(&session, RENDER)?.ok_or_else(|| {
        Error::new(format!(
            "the print dialog drew no preview: no `{RENDER}` line."
        ))
    })?;
    report.note(format!("before the switch: {colour}"));
    if field(&colour, "grey").as_deref() != Some("false") {
        return Err(Error::new(format!(
            "the preview in colour is already grey (`{colour}`), so a grey plate proves nothing. \
             The fixture must have colour."
        )));
    }

    press_region(&session, &pointer, SWITCH, "print.preview.")?;
    session.settle(60);
    let first = last(&session, RENDER)?.unwrap_or_default();
    let first_plate = last(&session, PLATE)?;
    report.note(format!("after the switch: {first}"));
    let Some(first_plate) = first_plate else {
        return Ok(Some(format!(
            "the plates switch was pressed and no `{PLATE}` line followed: the preview did not \
             render plates. Last render: `{first}`."
        )));
    };
    report.note(first_plate.clone());
    let mut failures = Vec::new();
    if field(&first, "plate").as_deref() != Some("0")
        || field(&first, "grey").as_deref() != Some("true")
    {
        failures.push(format!(
            "after the switch the preview drew `{first}`, not plate 0 in grey"
        ));
    }
    let count: usize = field(&first_plate, "of")
        .and_then(|n| n.parse().ok())
        .unwrap_or(0);
    if count < 2 {
        return Err(Error::new(format!(
            "the fixture separated into {count} plate(s) (`{first_plate}`); stepping needs two."
        )));
    }

    press_region(&session, &pointer, NEXT, "print.preview.")?;
    session.settle(30);
    let second = last(&session, RENDER)?.unwrap_or_default();
    let second_plate = last(&session, PLATE)?.unwrap_or_default();
    pointer.gone(&session)?;
    report.note(format!(
        "after the next-ink arrow: {second} / {second_plate}"
    ));
    if field(&second, "plate").as_deref() != Some("1")
        || field(&second, "grey").as_deref() != Some("true")
    {
        failures.push(format!(
            "after the next-ink arrow the preview drew `{second}`, not plate 1 in grey"
        ));
    }
    if field(&second_plate, "ink") == field(&first_plate, "ink") {
        failures.push(format!(
            "the next-ink arrow left the ink at {:?}",
            field(&second_plate, "ink")
        ));
    }
    Ok((!failures.is_empty()).then(|| failures.join("; ")))
}
