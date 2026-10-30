//! `form_preview_draws_a_dynamic_form` — opening a dynamic XFA form says so on
//! the status line, and View ▸ Panels ▸ Form preview lays the form out and
//! draws its own page instead of the "Please wait..." stand-in.
//!
//! Driven off the desktop: the command is rung through `PDFCER_DIAG_INVOKE`
//! and the one choice is clicked with the scripted pointer. Oracles: the
//! `xfa-dynamic-disclosed` line at open, the dialog's body region declared,
//! and `form-preview pages=1 warnings=1 values=2 merge_warnings=1
//! approximated=0` for the fixture's one-page template, whose data fills two
//! fields and lacks the third's. Clicking "3-D borders: Plain lines" must
//! redraw with `approximated=1 choices=00010`: the lowered `tenant` border is
//! drawn plain and disclosed, so a choice the window shows but the draw
//! ignores is caught.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_in, repo_fixture};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const OFFSCREEN: &str = "-4200,-4200,1400,900";
const FIXTURE: &str = "dynamic-xfa.pdf";
const METHOD: &str = "Rebuild it with `python fixtures/dynamic-xfa.PROVENANCE.py`.";
const DISCLOSED: &str = "xfa-dynamic-disclosed";
const DRAWN: &str = "form-preview";
const REGION: &str = "dialog:form-preview";
/// "3-D borders: Plain lines" (`dialogs::xfa_preview::choices_ui`, choice 3, answer 1).
const PLAIN: &str = "form-preview.choice.3.1";

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
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("form_preview.pointer.txt"))?;
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
    let expect = [
        ("pages", "1"),
        ("warnings", "1"),
        ("values", "2"),
        ("merge_warnings", "1"),
        ("approximated", "0"),
        ("choices", "00000"),
    ];
    if let Some((k, v)) = expect.iter().find(|(k, v)| line.get(k) != Some(*v)) {
        return Ok(Some(format!(
            "★★★ `{}` must carry {k}={v}: the fixture's data fills tenant and rent, lacks deposit, and draws its lowered border bevelled by default.",
            line.raw
        )));
    }
    if declared(&trace, ui_rect, REGION).is_none() {
        return Ok(Some(format!(
            "★★★ the form was drawn but no `{REGION}` region was declared: the window never showed."
        )));
    }
    plain_borders(ui_rect, &session, &pointer)
}

/// Click "3-D borders: Plain lines" and require the redraw to disclose the
/// one approximated border.
fn plain_borders(
    ui_rect: &str,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let trace = session.trace()?;
    let Some((rect, vp)) = declared_in(&trace, ui_rect, PLAIN) else {
        return Ok(Some(format!(
            "★★★ no `{PLAIN}` region was declared: the reading choices never drew."
        )));
    };
    let before = trace.events(DRAWN).count();
    pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(rect))?;
    session.settle(60);
    let trace = session.trace()?;
    let Some(line) = trace.events(DRAWN).nth(before) else {
        return Ok(Some(format!(
            "★★★ clicking `{PLAIN}` drew nothing new: the choice never reached the draw."
        )));
    };
    if line.get("approximated") != Some("1") || line.get("choices") != Some("00010") {
        return Ok(Some(format!(
            "★★★ `{}` after choosing plain 3-D borders must carry approximated=1 choices=00010: the lowered tenant border is drawn plain and that is disclosed.",
            line.raw
        )));
    }
    Ok(None)
}
