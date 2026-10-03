//! `a_key_the_font_lacks_is_set_in_the_nearest_face` — keys typed into a run
//! whose font lacks them go into the draft, commit set in the nearest face
//! that has them, and come out again with one Undo.

use crate::checks::driving::{SHELL_DIAG_ENV, repo_fixture};
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
/// Two keys the subset lacks.
const TYPED: &str = "qz";
const RUN_TEXT: &str = "ABC";
const KEY_REFUSED: &str = "text-edit-key-refused"; // ui-text-exempt: a trace event name, never displayed
const PLANNED: &str = "text-edit-reface-planned"; // ui-text-exempt: a trace event name, never displayed
const FALLBACK: &str = "text-edit-fallback"; // ui-text-exempt: a trace event name, never displayed
const READBACK: &str = "text-edit-reface-readback"; // ui-text-exempt: a trace event name, never displayed
const CARET: &str = "text-edit-caret"; // ui-text-exempt: a trace event name, never displayed
const UNDONE: &str = "undo-applied"; // ui-text-exempt: a trace event name, never displayed

/// See the module documentation.
pub struct AKeyTheFontLacksIsSetInTheNearestFace;

impl Check for AKeyTheFontLacksIsSetInTheNearestFace {
    fn name(&self) -> &'static str {
        "a_key_the_font_lacks_is_set_in_the_nearest_face"
    }

    fn defect(&self) -> &'static str {
        "a key the line's font lacks is refused at the keystroke, so a letter the page has no \
         glyph for can only be typed by changing the whole line's font"
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
    let doc = ctx.out("reface-commit.pdf");
    std::fs::copy(&source, &doc).map_err(|e| Error::new(format!("copying {FIXTURE}: {e}")))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("reface-commit.trace.txt"));
    spec.pdf = Some(doc);
    for (k, v) in [
        ctx.profile.diag_env,
        SHELL_DIAG_ENV,
        (viewport_env, OFFSCREEN),
        ("PDFCER_DIAG_INVOKE", INVOKE),
    ] {
        spec.env.push((k.to_owned(), v.to_owned()));
    }
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("reface-commit.pointer.txt"))?;
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
    pointer.key(&session, None, "End", None)?;
    pointer.type_text(&session, None, TYPED)?;
    session.settle(30);
    pointer.key(&session, None, "Escape", None)?;
    session.settle(40);
    let committed = caret_len(&session, &pointer, at)?;
    pointer.key(&session, None, "Z", Some("ctrl"))?;
    session.settle(40);
    let undone = caret_len(&session, &pointer, at)?;
    let outcome = judge(&session.trace()?, &session, committed, undone);
    pointer.gone(&session)?;
    outcome
}

/// Click the run and read the length of the draft the caret opened, then
/// close it unchanged.
fn caret_len(session: &Session, pointer: &ScriptedPointer, at: WindowPoint) -> Result<usize> {
    let before = session.trace()?.events(CARET).count();
    pointer.click(session, at)?;
    session.settle(25);
    let trace = session.trace()?;
    let len = trace
        .events(CARET)
        .skip(before)
        .last()
        .and_then(|l| l.get("len").and_then(|v| v.parse().ok()))
        .unwrap_or(0);
    pointer.key(session, None, "Escape", None)?;
    session.settle(15);
    Ok(len)
}

fn judge(
    trace: &Trace,
    session: &Session,
    committed: usize,
    undone: usize,
) -> Result<Option<String>> {
    let path = session.trace_path().display().to_string();
    if let Some(l) = trace.events(KEY_REFUSED).last() {
        return Ok(Some(format!(
            "★ `{TYPED}` was refused at the keystroke (`{}`) though a face on offer has both. \
             Trace: {path}.",
            l.raw
        )));
    }
    let planned = trace.events(PLANNED).last().map(|l| l.raw.clone());
    if !planned
        .as_deref()
        .is_some_and(|l| l.contains("characters=U+0071,U+007A"))
    {
        return Ok(Some(format!(
            "★ no `{PLANNED}` line names both keys: `{planned:?}`. Trace: {path}."
        )));
    }
    let Some(done) = trace.events(FALLBACK).last().map(|l| l.raw.clone()) else {
        return Ok(Some(format!(
            "★★ the engine did not set the keys in a fallback face: no `{FALLBACK}`. Trace: {path}."
        )));
    };
    if !done.contains("characters=U+0071,U+007A") {
        return Ok(Some(format!(
            "★★ the fallback set other characters than `{TYPED}`: `{done}`. Trace: {path}."
        )));
    }
    let reads = trace
        .events(READBACK)
        .last()
        .and_then(|l| l.get("reads").map(str::to_owned));
    let want = RUN_TEXT.len() + TYPED.len();
    if reads.as_deref() != Some("1") || committed != want {
        return Ok(Some(format!(
            "★★ `{done}` but the page does not read `{RUN_TEXT}{TYPED}`: read-back \
             reads={reads:?}, and the run reopened at len={committed}, not {want}. Trace: {path}."
        )));
    }
    let undos = trace.events(UNDONE).count();
    if undos != 1 || undone != RUN_TEXT.len() {
        return Ok(Some(format!(
            "★★★ one Ctrl+Z should restore `{RUN_TEXT}`: {undos} undo(s) applied, and the run \
             reopened at len={undone}. Trace: {path}."
        )));
    }
    Ok(None)
}
