//! `off_page_blank_overhang` — **the off-page census leaves out a picture
//! that shows nothing past the edge, lists one that does, and counts the one
//! it left out.**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/off_page_blank_overhang.md`.

use crate::checks::driving::SHELL_DIAG_ENV;
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::fixture::workspace_root;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const INVOKE: &str = "mode.edit,edit.offpage";
const FIXTURE: &str = "fixtures/off-page-blank-overhang.pdf";
const OPENED: &str = "offpage-opened"; // ui-text-exempt: a trace event name, never displayed
const SCANNED: &str = "offpage-scanned"; // ui-text-exempt: a trace event name, never displayed

/// See the module documentation.
pub struct BlankOverhangIsCountedNotListed;

impl Check for BlankOverhangIsCountedNotListed {
    fn name(&self) -> &'static str {
        "off_page_blank_overhang"
    }

    fn defect(&self) -> &'static str {
        "the off-page census lists a cleaned picture as off-page content, or leaves it out \
         without saying so"
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

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = workspace_root().join(FIXTURE);
    if !pdf.is_file() {
        return Ok(Some(format!(
            "the committed fixture is not at {}: a broken checkout.",
            pdf.display()
        )));
    }
    let mut spec = LaunchSpec::new(&exe, ctx.out("off_page_blank_overhang.trace.txt"));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), INVOKE.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    session.settle(60);

    if session.trace()?.last(OPENED).is_none() {
        return Ok(Some(format!(
            "`{INVOKE}` was invoked and no `{OPENED}` line followed: the census window did not \
             open."
        )));
    }
    let mut scanned = None;
    for _ in 0..30 {
        if let Some(line) = session.trace()?.last(SCANNED) {
            scanned = Some(line.clone());
            break;
        }
        session.settle(10);
    }
    let Some(scanned) = scanned else {
        return Ok(Some(format!(
            "the census opened and wrote no `{SCANNED}` line after 30 settles."
        )));
    };
    report.note(format!("census: `{}`", scanned.raw));
    let shot = ctx.out("off_page_blank_overhang.png");
    crate::capture::window_to_png(&session, &shot)?;
    report.artifact(shot);

    let count = |field: &str| scanned.get(field).and_then(|v| v.parse::<usize>().ok());
    match (count("objects"), count("blank_overhangs")) {
        (Some(1), Some(1)) => {
            report.note("the inked picture is listed, the blank one is counted and not listed");
            Ok(None)
        }
        (objects, blank) => Ok(Some(format!(
            "the fixture holds one picture inked past the edge and one blank past it; the \
             census reads objects={objects:?} blank_overhangs={blank:?}. objects=2 lists the \
             cleaned picture; blank_overhangs=0 or absent leaves it out unsaid. `{}`.",
            scanned.raw
        ))),
    }
}
