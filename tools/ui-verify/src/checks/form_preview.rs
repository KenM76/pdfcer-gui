//! `form_preview_draws_a_dynamic_form` — opening a dynamic XFA form says so on
//! the status line, and View ▸ Panels ▸ Form preview lays the form out and
//! draws its own page instead of the "Please wait..." stand-in.
//!
//! Driven off the desktop with no input at all: the command is rung through
//! `PDFCER_DIAG_INVOKE`. Oracles: the `xfa-dynamic-disclosed` line at open,
//! the dialog's body region declared, and `form-preview pages=1 warnings=0`
//! for the fixture's one-page template.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, repo_fixture};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const OFFSCREEN: &str = "-4200,-4200,1400,900";
const FIXTURE: &str = "dynamic-xfa.pdf";
const METHOD: &str = "Rebuild it with `python fixtures/dynamic-xfa.PROVENANCE.py`.";
const DISCLOSED: &str = "xfa-dynamic-disclosed";
const DRAWN: &str = "form-preview";
const REGION: &str = "dialog:form-preview";

/// See the module documentation.
pub struct FormPreviewDrawsADynamicForm;

impl Check for FormPreviewDrawsADynamicForm {
    fn name(&self) -> &'static str {
        "form_preview_draws_a_dynamic_form"
    }

    fn defect(&self) -> &'static str {
        "a dynamic XFA form opens at its \"Please wait...\" stand-in with nothing said, or Form \
         preview opens no window, draws no page, or fails"
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
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let doc = ctx.out("form-preview.pdf");
    std::fs::copy(repo_fixture(FIXTURE, METHOD)?, &doc)
        .map_err(|e| Error::new(format!("copying {FIXTURE}: {e}")))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("form_preview.trace.txt"));
    spec.pdf = Some(doc);
    for (k, v) in [
        (ctx.profile.diag_env.0, ctx.profile.diag_env.1),
        SHELL_DIAG_ENV,
        (viewport_env, OFFSCREEN),
        ("PDFCER_DIAG_INVOKE", "view.form_preview"),
    ] {
        spec.env.push((k.to_owned(), v.to_owned()));
    }
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    session.settle(90);

    let trace = session.trace()?;
    if trace.last(DISCLOSED).is_none() {
        return Ok(Some(format!(
            "★★★ the dynamic form opened and no `{DISCLOSED}` line was traced: the status line \
             never said the pages are a stand-in. Trace: {}.",
            session.trace_path().display()
        )));
    }
    let Some(line) = trace.last(DRAWN) else {
        return Ok(Some(format!(
            "★★★ `view.form_preview` was rung and no `{DRAWN}` line followed: the form was never \
             drawn. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(line.raw.clone());
    if line.get("pages") != Some("1") || line.get("warnings") != Some("0") {
        return Ok(Some(format!(
            "★★★ `{}` must be pages=1 warnings=0 for the fixture's one-page template.",
            line.raw
        )));
    }
    if declared(&trace, ui_rect, REGION).is_none() {
        return Ok(Some(format!(
            "★★★ the form was drawn but no `{REGION}` region was declared: the window never showed."
        )));
    }
    Ok(None)
}
