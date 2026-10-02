//! `the_text_tool_selects_as_a_word_processor_does` — with the text tool on a
//! page's own text, a double click takes the word, a triple click the line,
//! Shift+Down selects to the line's end without leaving it, a drag that starts
//! on text selects instead of drawing a box, and a click on rotated text puts
//! the caret where it landed.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/selection_gestures.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, repo_fixture};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Off every monitor, unfocused: the drive needs neither mouse nor keyboard.
const OFFSCREEN: &str = "-4200,-4200,1400,900";
const INVOKE: &str = "mode.edit,edit.text";
const LETTER: PageGeometry = PageGeometry {
    width_pt: 612.0,
    height_pt: 792.0,
};
/// First line `Date Premises Required____ `, 27 characters at y=700 from x=72.
const LINES: &str = "word-fragmented-lines.pdf";
/// Inside `Date`, the first word, PDF points.
const IN_DATE: (f64, f64) = (80.0, 704.0);
/// On the second line, PDF points: where the drag ends, below the first.
const LINE_TWO: (f64, f64) = (110.0, 674.0);
const LINE_CHARS: usize = 27;
/// `UPWARD` set at 90 degrees from (100, 300) in 12 pt Helvetica.
const ROTATED: &str = "rotated-text.pdf";
/// Two points into `W`, the third glyph (U and P advance 16.668 pt), and to
/// the ascender side of the baseline: the caret goes before `W`, index 2.
const IN_W: (f64, f64) = (97.0, 318.7);
const SELECT: &str = "text-select"; // ui-text-exempt: a trace event name, never displayed
const BOX_OPEN: &str = "text-box-open"; // ui-text-exempt: a trace event name, never displayed
const CARET: &str = "text-edit-caret"; // ui-text-exempt: a trace event name, never displayed

/// See the module documentation.
pub struct TheTextToolSelectsAsAWordProcessorDoes;

impl Check for TheTextToolSelectsAsAWordProcessorDoes {
    fn name(&self) -> &'static str {
        "the_text_tool_selects_as_a_word_processor_does"
    }

    fn defect(&self) -> &'static str {
        "a double click on page text selects nothing, a triple click does nothing, Shift+Down \
         leaves the line, a drag across text draws a new text box or stops short of the line's end, and a click on rotated text \
         puts the caret at its first letter"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let outcome = lines(ctx, &mut report).and_then(|flat| {
            let turned = rotated(ctx, &mut report)?;
            let failed: Vec<String> = flat.into_iter().chain(turned).collect();
            Ok((!failed.is_empty()).then(|| failed.join("; ")))
        });
        match outcome {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn launch(ctx: &CheckContext, fixture: &str, tag: &str) -> Result<(Session, ScriptedPointer)> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let source = repo_fixture(fixture, "The fixture is committed under fixtures/.")?;
    let doc = ctx.out(&format!("{tag}.pdf"));
    std::fs::copy(&source, &doc).map_err(|e| Error::new(format!("copying {fixture}: {e}")))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out(&format!("{tag}.trace.txt")));
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
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out(&format!("{tag}.pointer.txt")))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    Ok((session, pointer))
}

/// The last `text-select` line, or the empty string.
fn last_select(session: &Session) -> Result<String> {
    Ok(session
        .trace()?
        .events(SELECT)
        .last()
        .map(|l| l.raw.clone())
        .unwrap_or_default())
}

fn count(session: &Session, event: &str) -> Result<usize> {
    Ok(session.trace()?.events(event).count())
}

/// Double click, triple click, Shift+Down and a drag, on the Word-style lines.
fn lines(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let (session, pointer) = launch(ctx, LINES, "selection-gestures-lines")?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, LETTER, 0)?;
    let at = |p: (f64, f64)| mapping.doc_to_window(DocPoint::new(0, p.0, p.1));
    let key = |name: &str, mods: Option<&str>| -> Result<()> {
        pointer.key(&session, None, name, mods)?;
        session.settle(15);
        Ok(())
    };

    pointer.double_click(&session, at(IN_DATE)?)?;
    session.settle(20);
    let double = last_select(&session)?;
    pointer.triple_click(&session, at(IN_DATE)?)?;
    session.settle(20);
    let triple = last_select(&session)?;
    key("Escape", None)?;
    let carets = count(&session, CARET)?;
    pointer.click(&session, at(IN_DATE)?)?;
    session.settle(20);
    key("Home", None)?;
    key("ArrowDown", Some("shift"))?;
    let shift_down = last_select(&session)?;
    let left_line = count(&session, CARET)? > carets + 1;
    key("Escape", None)?;
    let boxes = count(&session, BOX_OPEN)?;
    pointer.drag(&session, at(IN_DATE)?, at(LINE_TWO)?, 8)?;
    session.settle(20);
    let swept = last_select(&session)?;
    let boxed = count(&session, BOX_OPEN)? > boxes;
    report.note(format!(
        "double `{double}`; triple `{triple}`; Shift+Down `{shift_down}` (left the line: \
         {left_line}); drag `{swept}` (opened a box: {boxed})"
    ));
    let mut wrong = Vec::new();
    if !double.contains("from=0 to=4 ") {
        wrong.push("a double click on `Date` did not select that word (from=0 to=4)".to_owned());
    }
    if !triple.contains(&format!("from=0 to={LINE_CHARS} ")) {
        wrong.push(format!(
            "a triple click did not select the line (from=0 to={LINE_CHARS})"
        ));
    }
    if left_line || !shift_down.contains(&format!("from=0 to={LINE_CHARS} ")) {
        wrong.push("Shift+Down from Home did not select to the line's end in place".to_owned());
    }
    if boxed || !swept.contains(&format!(" to={LINE_CHARS} ")) {
        wrong.push(format!(
            "a drag from text down onto the next line did not select to the line's end              (to={LINE_CHARS}), or drew a box"
        ));
    }
    Ok((!wrong.is_empty()).then(|| wrong.join("; ")))
}

/// A click two points into the third glyph of a 90-degree line.
fn rotated(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let (session, pointer) = launch(ctx, ROTATED, "selection-gestures-rotated")?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, LETTER, 0)?;
    pointer.click(
        &session,
        mapping.doc_to_window(DocPoint::new(0, IN_W.0, IN_W.1))?,
    )?;
    session.settle(20);
    let placed = last_select(&session)?;
    report.note(format!("rotated click `{placed}`"));
    if placed.contains("none caret=2") {
        return Ok(None);
    }
    Ok(Some(format!(
        "a click before `W` in the 90-degree `UPWARD` put the caret elsewhere: `{placed}`"
    )))
}
