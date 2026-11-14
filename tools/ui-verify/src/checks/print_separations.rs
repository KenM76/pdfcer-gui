//! `print_separates_into_plates` — Colour ▸ Separations prints each plain
//! page as one sheet per ink.
//!
//! Opens Print (dry run, off the desktop, scripted pointer) on
//! `fixtures/transparency-cmyk.pdf`, a one-page CMYK drawing, opens the Colour
//! tab, chooses Separations and presses Print. Oracle: a `print-plates` line
//! with `refused=false`, more than one plate and its inks named, and a
//! `print-spool-done` line that sent that many sheets for the one page.
//! Ignores `--pdf`; the fixture is the method.

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
const OPEN: &str = "print-open";
const PLATES: &str = "print-plates";
const SPOOLED: &str = "print-spool-done";
const TAB: &str = "print.tab.colour";
const SEPARATIONS: &str = "print.colour.separations";
const PRINT: &str = "dialog.buttons.accept";

pub struct PrintSeparatesIntoPlates;

impl Check for PrintSeparatesIntoPlates {
    fn name(&self) -> &'static str {
        "print_separates_into_plates"
    }

    fn defect(&self) -> &'static str {
        "Separations is chosen and the page still prints as one composite sheet"
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
    let mut spec = LaunchSpec::new(&exe, ctx.out("print_separations.trace.txt"));
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
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("print_separations.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(60);
    Ok((session, pointer))
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let (session, pointer) = launch(ctx, report)?;
    if session.trace()?.last(OPEN).is_none() {
        return Err(Error::new(format!(
            "`file.print` was rung and no `{OPEN}` line followed. That is \
             `print_dialog_reaches_the_spooler`'s subject."
        )));
    }
    press_region(&session, &pointer, TAB, "print.tab.")?;
    press_region(&session, &pointer, SEPARATIONS, "print.colour.")?;
    press_region(&session, &pointer, PRINT, "dialog.")?;
    session.settle(240);
    let trace = session.trace()?;
    let spool = trace.last(SPOOLED).map(|l| l.raw.clone()).ok_or_else(|| {
        Error::new(format!(
            "Print was pressed and no `{SPOOLED}` line followed."
        ))
    })?;
    let plates = trace.last(PLATES).map(|l| l.raw.clone());
    pointer.gone(&session)?;
    report.note(spool.clone());
    if !spool.contains("printed=false") {
        return Ok(Some(format!("★★★ a spool started a real job: `{spool}`.")));
    }
    let Some(plates) = plates else {
        return Ok(Some(format!(
            "★★★ Separations was chosen and no `{PLATES}` line was traced: the page went through \
             the ordinary render. Spool: `{spool}`."
        )));
    };
    report.note(plates.clone());
    let field = |line: &str, key: &str| {
        line.split(&format!(" {key}="))
            .nth(1)
            .and_then(|rest| rest.split(' ').next())
            .map(str::to_owned)
    };
    if field(&plates, "refused").as_deref() == Some("true") {
        return Err(Error::new(format!(
            "the engine declined to separate the fixture (`{plates}`), so the shell's plate path \
             was not reached. That is a fact about the fixture or the engine, not the dialog."
        )));
    }
    let count: usize = field(&plates, "plates")
        .and_then(|n| n.parse().ok())
        .unwrap_or(0);
    let inks = field(&plates, "inks").unwrap_or_default();
    if count < 2 || inks.split('|').filter(|i| !i.is_empty()).count() != count {
        return Ok(Some(format!(
            "★★★ the page separated into `{plates}`: a CMYK page has more than one ink and each \
             plate names one."
        )));
    }
    let sent: usize = field(&spool, "pages")
        .and_then(|n| n.parse().ok())
        .unwrap_or(0);
    if sent != count {
        return Ok(Some(format!(
            "★★★ {count} plates were rendered and the spool sent {sent} sheets: `{spool}`."
        )));
    }
    Ok(None)
}
