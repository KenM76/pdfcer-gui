//! Keys typed into a run whose font lacks them go into the draft, commit set
//! in the nearest face that has them through the engine's fallback, and come
//! out again with one Undo. Two checks share the drive:
//!
//! - `a_key_the_font_lacks_is_set_in_the_nearest_face` — `ABC` in one `Tj`,
//!   keys appended, so the change lies in one show operator;
//! - `a_change_across_show_operators_is_set_in_the_nearest_face` — `ABC` one
//!   `Tj` per glyph in one text object, a key at each end, so the change
//!   spans all three operators.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/reface_commit.md`.

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
/// The centre of the run `ABC` in both fixtures, 0-based page, PDF points.
const AIM: (f64, f64) = (115.2, 612.0);
const PAGE: PageGeometry = PageGeometry {
    width_pt: 612.0,
    height_pt: 792.0,
};
const INVOKE: &str = "mode.edit,edit.text";
const RUN_TEXT: &str = "ABC";
/// The two keys both fixtures' subset lacks, as the trace names them.
const KEYS: &str = "characters=U+0071,U+007A";
const KEY_REFUSED: &str = "text-edit-key-refused"; // ui-text-exempt: a trace event name, never displayed
const PLANNED: &str = "text-edit-reface-planned"; // ui-text-exempt: a trace event name, never displayed
const FALLBACK: &str = "text-edit-fallback"; // ui-text-exempt: a trace event name, never displayed
const READBACK: &str = "text-edit-reface-readback"; // ui-text-exempt: a trace event name, never displayed
const SHAPED: &str = "text-edit-shaped"; // ui-text-exempt: a trace event name, never displayed
const CARET: &str = "text-edit-caret"; // ui-text-exempt: a trace event name, never displayed
const UNDONE: &str = "undo-applied"; // ui-text-exempt: a trace event name, never displayed

/// One fixture and the keys typed into it.
struct Case {
    name: &'static str,
    defect: &'static str,
    /// `ABC` in a subset font that carries only those three glyphs.
    fixture: &'static str,
    /// The prefix of the copied document and its trace.
    stem: &'static str,
    /// `(key pressed first, text typed after it)`, in order.
    typing: &'static [(&'static str, &'static str)],
    /// The committed text.
    committed: &'static str,
    /// The engine's `operators=` on the fallback line.
    operators: &'static str,
}

const ONE_OPERATOR: Case = Case {
    name: "a_key_the_font_lacks_is_set_in_the_nearest_face",
    defect: "a key the line's font lacks is refused at the keystroke, so a letter the page has \
             no glyph for can only be typed by changing the whole line's font",
    fixture: "subset-font-floor.pdf",
    stem: "reface-commit",
    typing: &[("End", "qz")],
    committed: "ABCqz",
    operators: "1",
};

const ACROSS_OPERATORS: Case = Case {
    name: "a_change_across_show_operators_is_set_in_the_nearest_face",
    defect: "a change to a line written one show operator per letter, typing keys its font \
             lacks, is not set in the nearest face by the engine in one undo entry",
    fixture: "subset-glyph-per-show.pdf",
    stem: "reface-spanning",
    typing: &[("Home", "q"), ("End", "z")],
    committed: "qABCz",
    operators: "3",
};

/// See the module documentation.
pub struct AKeyTheFontLacksIsSetInTheNearestFace;

/// See the module documentation.
pub struct AChangeAcrossShowOperatorsIsSetInTheNearestFace;

impl Check for AKeyTheFontLacksIsSetInTheNearestFace {
    fn name(&self) -> &'static str {
        ONE_OPERATOR.name
    }

    fn defect(&self) -> &'static str {
        ONE_OPERATOR.defect
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run(ctx, &ONE_OPERATOR)
    }
}

impl Check for AChangeAcrossShowOperatorsIsSetInTheNearestFace {
    fn name(&self) -> &'static str {
        ACROSS_OPERATORS.name
    }

    fn defect(&self) -> &'static str {
        ACROSS_OPERATORS.defect
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run(ctx, &ACROSS_OPERATORS)
    }
}

fn run(ctx: &CheckContext, case: &Case) -> CheckReport {
    let mut report = CheckReport::new(case.name, case.defect);
    match drive(ctx, case, &mut report) {
        Ok(Some(failure)) => report.fail(failure),
        Ok(None) => report.pass(),
        Err(why) => report.from_error(&why),
    }
}

fn launch(ctx: &CheckContext, case: &Case) -> Result<(Session, ScriptedPointer)> {
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
        case.fixture,
        "The check needs a run whose font lacks `q` and `z`.",
    )?;
    let doc = ctx.out(&format!("{}.pdf", case.stem));
    std::fs::copy(&source, &doc)
        .map_err(|e| Error::new(format!("copying {}: {e}", case.fixture)))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out(&format!("{}.trace.txt", case.stem)));
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
    let pointer =
        ScriptedPointer::attach(&mut spec, ctx.out(&format!("{}.pointer.txt", case.stem)))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    Ok((session, pointer))
}

fn drive(ctx: &CheckContext, case: &Case, report: &mut CheckReport) -> Result<Option<String>> {
    let (session, pointer) = launch(ctx, case)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, PAGE, 0)?;
    let at = mapping.doc_to_window(DocPoint::new(0, AIM.0, AIM.1))?;
    pointer.click(&session, at)?;
    session.settle(20);
    for (key, text) in case.typing {
        pointer.key(&session, None, key, None)?;
        pointer.type_text(&session, None, text)?;
        session.settle(15);
    }
    session.settle(15);
    pointer.key(&session, None, "Escape", None)?;
    session.settle(40);
    let committed = caret_len(&session, &pointer, at)?;
    pointer.key(&session, None, "Z", Some("ctrl"))?;
    session.settle(40);
    let undone = caret_len(&session, &pointer, at)?;
    let outcome = judge(case, &session.trace()?, &session, committed, undone);
    pointer.gone(&session)?;
    outcome
}

/// Whether the committed draft was laid out in the run's place, the keys in
/// the fallback face, rather than in the stand-in editor box. Read before the
/// commit: the reopened run is several show operators in two fonts.
fn preview_verdict(case: &Case, trace: &Trace, path: &str) -> Option<String> {
    let want = case.committed.len().to_string();
    let commit = trace.first(FALLBACK).map_or(usize::MAX, |l| l.lineno);
    let line = trace
        .events(SHAPED)
        .filter(|l| l.lineno < commit && l.get("chars") == Some(want.as_str()))
        .last()
        .map(|l| l.raw.clone());
    match line {
        Some(l) if l.contains("shaped=1") => None,
        Some(l) => Some(format!(
            "★★ the draft `{}` was not laid out in place: `{l}`. Trace: {path}.",
            case.committed
        )),
        None => Some(format!(
            "★★ no `{SHAPED}` line for the {want}-character draft. Trace: {path}."
        )),
    }
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

/// The keystroke's verdict: no key refused, and the face confirmed by the
/// engine's fallback for both keys.
fn keystroke_verdict(trace: &Trace, path: &str) -> Option<String> {
    if let Some(l) = trace.events(KEY_REFUSED).last() {
        return Some(format!(
            "★ `q` or `z` was refused at the keystroke (`{}`) though a face on offer has both. \
             Trace: {path}.",
            l.raw
        ));
    }
    let planned = trace.events(PLANNED).last().map(|l| l.raw.clone());
    if planned
        .as_deref()
        .is_some_and(|l| l.contains(KEYS) && l.contains("route=engine"))
    {
        return None;
    }
    Some(format!(
        "★ no `{PLANNED}` line names both keys confirmed by the engine (`route=engine`): \
         `{planned:?}`. Trace: {path}."
    ))
}

fn judge(
    case: &Case,
    trace: &Trace,
    session: &Session,
    committed: usize,
    undone: usize,
) -> Result<Option<String>> {
    let path = session.trace_path().display().to_string();
    if let Some(failure) = keystroke_verdict(trace, &path) {
        return Ok(Some(failure));
    }
    if let Some(failure) = preview_verdict(case, trace, &path) {
        return Ok(Some(failure));
    }
    let Some(done) = trace.events(FALLBACK).last() else {
        return Ok(Some(format!(
            "★★ the engine did not set the keys in a fallback face: no `{FALLBACK}`. Trace: {path}."
        )));
    };
    if !done.raw.contains(KEYS) || done.get("operators") != Some(case.operators) {
        return Ok(Some(format!(
            "★★ the fallback should set `q` and `z` across {} show operator(s): `{}`. \
             Trace: {path}.",
            case.operators, done.raw
        )));
    }
    let reads = trace
        .events(READBACK)
        .last()
        .and_then(|l| l.get("reads").map(str::to_owned));
    let want = case.committed.len();
    if reads.as_deref() != Some("1") || committed != want {
        return Ok(Some(format!(
            "★★ `{}` but the page does not read `{}`: read-back reads={reads:?}, and the run \
             reopened at len={committed}, not {want}. Trace: {path}.",
            done.raw, case.committed
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
