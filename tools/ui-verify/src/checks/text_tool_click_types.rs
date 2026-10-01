//! `a_text_tool_click_on_text_types_there` — in Edit mode, a click on page
//! text with the Text tool leaves a caret that takes the next keystroke,
//! rather than opening one and closing it in the same frame.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/text_tool_click_types.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, repo_fixture};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Off every monitor, unfocused: the drive needs neither mouse nor keyboard.
const OFFSCREEN: &str = "-4200,-4200,1400,900";
/// One run, `ABC`.
const FIXTURE: &str = "subset-font-floor.pdf";
/// The centre of the run `ABC`, 0-based page, PDF points.
const AIM: (f64, f64) = (115.2, 612.0);
const PAGE: PageGeometry = PageGeometry {
    width_pt: 612.0,
    height_pt: 792.0,
};
/// Edit mode with the Text tool (sweep and click), not the caret tool.
const INVOKE: &str = "mode.edit,view.tool_text";
const CARET: &str = "text-edit-caret"; // ui-text-exempt: a trace event name, never displayed
const ABANDON: &str = "text-edit-abandon"; // ui-text-exempt: a trace event name, never displayed
const TYPING: &str = "text-edit-typing"; // ui-text-exempt: a trace event name, never displayed

/// See the module documentation.
pub struct ATextToolClickTypes;

impl Check for ATextToolClickTypes {
    fn name(&self) -> &'static str {
        "a_text_tool_click_on_text_types_there"
    }

    fn defect(&self) -> &'static str {
        "a click on page text with the Text tool opens a caret and closes it in the same \
         frame, so the caret flickers and nothing typed goes in"
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

fn launch(ctx: &CheckContext) -> Result<(Session, ScriptedPointer)> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let source = repo_fixture(FIXTURE, "The check needs one editable run of known text.")?;
    let doc = ctx.out("text-tool-click-source.pdf");
    std::fs::copy(&source, &doc).map_err(|e| Error::new(format!("copying {FIXTURE}: {e}")))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("text-tool-click.trace.txt"));
    spec.pdf = Some(doc);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push((viewport_env.to_owned(), OFFSCREEN.to_owned()));
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), INVOKE.to_owned()));
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("text-tool-click.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    Ok((session, pointer))
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let (session, pointer) = launch(ctx)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, PAGE, 0)?;
    let at = mapping.doc_to_window(DocPoint::new(0, AIM.0, AIM.1))?;
    pointer.click(&session, at)?;
    session.settle(20);
    let opened = session.trace()?;
    pointer.key(&session, None, "End", None)?;
    pointer.type_text(&session, None, "A")?;
    session.settle(20);
    let trace = session.trace()?;
    pointer.key(&session, None, "Escape", None)?;
    session.settle(10);
    pointer.gone(&session)?;
    let path = session.trace_path().display().to_string();
    let Some(caret) = opened.events(CARET).last().map(|l| l.lineno) else {
        return Ok(Some(format!(
            "★ a Text-tool click on the run `ABC` opened no caret. Trace: {path}."
        )));
    };
    if let Some(gone) = opened.events(ABANDON).find(|l| l.lineno > caret) {
        return Ok(Some(format!(
            "★ the caret opened at trace line {caret} was closed again at line {} before \
             anything was typed: the click opens a caret the frame then settles away. \
             Trace: {path}.",
            gone.lineno
        )));
    }
    report.note("★ the caret the click opened is still open twenty frames on");
    // The canvas's own draft length: `ABC` plus the typed `A`.
    let typed = trace
        .events(TYPING)
        .last()
        .is_some_and(|l| l.raw.ends_with("len=4") && l.raw.contains("draft=true"));
    if !typed {
        return Ok(Some(format!(
            "★★ the caret stayed but the typed `A` did not reach the draft: the last \
             `{TYPING}` line is {:?}. Trace: {path}.",
            trace.events(TYPING).last().map(|l| l.raw.clone())
        )));
    }
    report.note("★★ the typed key went into the draft (`len=4`)");
    Ok(None)
}
