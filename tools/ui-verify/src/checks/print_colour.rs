//! `print_in_grey_or_black_only` — the Print dialog's colour choice recolours
//! the preview, the same way the sheets sent to the printer are recoloured.
//!
//! Driven off the desktop with the scripted pointer: `file.print` is rung on
//! `print-colour.pdf`, a Letter page about 63 % saturated red. The check opens
//! the Copies tab, clicks In grey, then Black only. Oracle: after each click a
//! `print-colour of=preview` line naming that mode and more than half the
//! raster's pixels changed. Red is never white and never already grey, so
//! both modes must change every red pixel.

use crate::checks::driving::{SHELL_DIAG_ENV, declared_in, declared_names, list, repo_fixture};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const OFFSCREEN: &str = "-4200,-4200,1400,900";
const FIXTURE: &str = "print-colour.pdf";
const METHOD: &str = "Rebuild it with `python fixtures/print-colour.PROVENANCE.py`.";
const UI_RECT: &str = "ui-rect";
const OPEN: &str = "print-open";
const COLOUR: &str = "print-colour";
const TAB: &str = "print.tab.copies";
const PREFIX: &str = "print.colour.";

/// See the module documentation.
pub struct PrintInGreyOrBlackOnly;

impl Check for PrintInGreyOrBlackOnly {
    fn name(&self) -> &'static str {
        "print_in_grey_or_black_only"
    }

    fn defect(&self) -> &'static str {
        "the Print dialog cannot print a colour document in grey or black only, or its preview \
         does not show the result"
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
    let mut spec = LaunchSpec::new(&exe, ctx.out("print_colour.trace.txt"));
    spec.pdf = Some(doc);
    for (k, v) in [
        (ctx.profile.diag_env.0, ctx.profile.diag_env.1),
        SHELL_DIAG_ENV,
        (viewport_env, OFFSCREEN),
        ("PDFCER_DIAG_INVOKE", "file.print"),
    ] {
        spec.env.push((k.to_owned(), v.to_owned()));
    }
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("print_colour.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(60);
    Ok((session, pointer))
}

/// Click the region `name`, in whichever window declared it.
fn press(session: &Session, pointer: &ScriptedPointer, name: &str, family: &str) -> Result<()> {
    let trace = session.trace()?;
    let (rect, viewport) = declared_in(&trace, UI_RECT, name).ok_or_else(|| {
        Error::new(format!(
            "no `{name}` region. Declared under `{family}`: {}.",
            list(&declared_names(&trace, UI_RECT, family))
        ))
    })?;
    pointer.click_in(session, viewport.as_deref(), WindowPoint::centre_of(rect))?;
    session.settle(30);
    Ok(())
}

/// Choose `mode` and read the preview's recolour line it caused.
fn choose(session: &Session, pointer: &ScriptedPointer, mode: &str) -> Result<Option<String>> {
    let before = session.trace()?.events(COLOUR).count();
    press(session, pointer, &format!("{PREFIX}{mode}"), PREFIX)?;
    let trace = session.trace()?;
    let Some(line) = trace
        .events(COLOUR)
        .skip(before)
        .find(|l| l.get("of") == Some("preview"))
    else {
        return Ok(Some(format!(
            "★★★ choosing `{mode}` traced no `{COLOUR} of=preview` line: the preview was not \
             redrawn in the chosen colour."
        )));
    };
    let (changed, pixels) = (
        line.get_usize("changed").unwrap_or(0),
        line.get_usize("pixels").unwrap_or(0),
    );
    if line.get("mode") != Some(mode) || changed * 2 <= pixels {
        return Ok(Some(format!(
            "★★★ choosing `{mode}` recoloured the preview as `{}`: {changed} of {pixels} pixels \
             changed, on a page more than half red.",
            line.raw
        )));
    }
    Ok(None)
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let (session, pointer) = launch(ctx, report)?;
    let trace = session.trace()?;
    if trace.last(OPEN).is_none() {
        return Err(Error::new(format!(
            "`file.print` was rung and no `{OPEN}` line followed. That is \
             `print_dialog_reaches_the_spooler`'s subject."
        )));
    }
    press(&session, &pointer, TAB, "print.tab.")?;
    for mode in ["grey", "black"] {
        if let Some(failure) = choose(&session, &pointer, mode)? {
            return Ok(Some(failure));
        }
    }
    for line in session.trace()?.events(COLOUR) {
        report.note(line.raw.clone());
    }
    pointer.gone(&session)?;
    Ok(None)
}
