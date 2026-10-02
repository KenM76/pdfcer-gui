//! `every_refused_key_is_named_with_a_face_that_takes_them` — keys typed into
//! existing text that its font cannot take go in, are all named beside the
//! edit with the face they will be set in, and one click changes the whole
//! line to that face instead.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/refused_keys.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, repo_fixture};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry, WindowPoint};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::trace::Trace;

/// Off every monitor, unfocused: the drive needs neither mouse nor keyboard.
const OFFSCREEN: &str = "-4200,-4200,1400,900";
/// One run, `ABC`, in a subset font that carries only those three glyphs.
const FIXTURE: &str = "subset-font-floor.pdf";
/// The centre of the run `ABC`, 0-based page, PDF points.
const AIM: (f64, f64) = (115.2, 612.0);
const PAGE: PageGeometry = PageGeometry {
    width_pt: 612.0,
    height_pt: 792.0,
};
const INVOKE: &str = "mode.edit,edit.text";
/// Two keys the subset lacks, typed as one burst.
const TYPED: &str = "qz";
/// The run's text before the edit.
const RUN_TEXT: &str = "ABC";
const TYPING_EVENT: &str = "text-edit-typing"; // ui-text-exempt: a trace event name, never displayed
const EVENT: &str = "text-edit-refused-keys"; // ui-text-exempt: a trace event name, never displayed
const STYLE_EVENT: &str = "text-style-applied"; // ui-text-exempt: a trace event name, never displayed
const REGION: &str = "textedit.refused-keys"; // ui-text-exempt: a trace region name, never displayed
const USE_REGION: &str = "textedit.refused-keys.use-face"; // ui-text-exempt: a trace region name, never displayed

/// See the module documentation.
pub struct EveryRefusedKeyIsNamed;

impl Check for EveryRefusedKeyIsNamed {
    fn name(&self) -> &'static str {
        "every_refused_key_is_named_with_a_face_that_takes_them"
    }

    fn defect(&self) -> &'static str {
        "keys the text's font cannot take are dropped, or go in unannounced, with no way to \
         a face that has them"
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
    let source = repo_fixture(
        FIXTURE,
        "The check needs a run whose font lacks `q` and `z`.",
    )?;
    let doc = ctx.out("refused-keys-source.pdf");
    std::fs::copy(&source, &doc).map_err(|e| Error::new(format!("copying {FIXTURE}: {e}")))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("refused-keys.trace.txt"));
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
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("refused-keys.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    Ok((session, pointer))
}

/// The last notice line, raw.
fn last_notice(trace: &Trace) -> Option<String> {
    trace.events(EVENT).last().map(|l| l.raw.clone())
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
    pointer.key(&session, None, "End", None)?;
    pointer.type_text(&session, None, TYPED)?;
    session.settle(30);
    let outcome = assess(ctx, &session, &pointer, report);
    pointer.key(&session, None, "Escape", None)?;
    session.settle(10);
    pointer.gone(&session)?;
    outcome
}

fn assess(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    report: &mut CheckReport,
) -> Result<Option<String>> {
    let path = session.trace_path().display().to_string();
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile names no ui-rect trace event."))?;
    let trace = session.trace()?;
    let Some(offer) = last_notice(&trace) else {
        return Ok(Some(format!(
            "★ typing `{TYPED}` into a run whose font lacks both wrote no `{EVENT}` line: the \
             refused keys were dropped without a notice. Trace: {path}."
        )));
    };
    if !offer.contains("characters=U+0071,U+007A") {
        return Ok(Some(format!(
            "★ the notice does not name both refused keys, `q` (U+0071) and `z` (U+007A): \
             `{offer}`. Trace: {path}."
        )));
    }
    if declared(&trace, ui_rect, REGION).is_none() {
        return Ok(Some(format!(
            "★ the notice `{REGION}` was never drawn although `{offer}`. Trace: {path}."
        )));
    }
    report.note(format!(
        "★ both refused keys are named, beside the edit: `{offer}`"
    ));
    // The draft's own length, written by the canvas rather than the notice, so
    // a notice that claims keys went in that did not cannot pass.
    let want = format!("len={}", RUN_TEXT.len() + TYPED.len());
    let grew = |trace: &Trace| {
        trace
            .events(TYPING_EVENT)
            .last()
            .is_some_and(|l| l.raw.ends_with(&want))
    };
    if !offer.contains("state=planned") || !grew(&trace) {
        return Ok(Some(format!(
            "★★ the keys did not go in planned for a face every standard face has: the notice \
             reads `{offer}` and the draft is not {want}. Trace: {path}."
        )));
    }
    let Some(button) = declared(&trace, ui_rect, USE_REGION) else {
        return Ok(Some(format!(
            "★★ the notice offered a face but drew no `{USE_REGION}` button. Trace: {path}."
        )));
    };
    report.note(format!(
        "★★ the keys went in, and the whole line in their face is one click away: \
         `{USE_REGION}`"
    ));
    pointer.click(session, WindowPoint::centre_of(button))?;
    session.settle(30);
    let trace = session.trace()?;
    let styled = trace.events(STYLE_EVENT).last().map(|l| l.raw.clone());
    let after = last_notice(&trace).unwrap_or_default();
    let Some(styled) = styled else {
        return Ok(Some(format!(
            "★★★ clicking the face button applied no restyle (`{STYLE_EVENT}` absent); the \
             notice now reads `{after}`. Trace: {path}."
        )));
    };
    if !after.contains("state=whole") || !grew(&trace) {
        return Ok(Some(format!(
            "★★★ the face landed (`{styled}`) but the notice reads `{after}` or the draft is \
             not {want}. Trace: {path}."
        )));
    }
    report.note(format!(
        "★★★ the whole line took the face through the font-change path, keys kept: `{styled}`"
    ));
    Ok(None)
}
