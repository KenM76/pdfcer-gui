//! `a_text_object_splits_into_lines` — the canvas object menu's *Split into
//! lines* cuts one three-line text object into three, Ctrl+Z rejoins them, and
//! a text object the engine refuses to cut is greyed before the press.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/split_lines.md`.

use crate::checks::driving::{
    SHELL_DIAG_ENV, click_mode_segment, declared, declared_names, list, repo_fixture,
};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, WindowPoint};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const OFFSCREEN: &str = "-4200,-4200,1200,900";
const MENU_ROW: &str = "menu.item.canvas.object.format.split_text_lines";
const APPLIED: &str = "split-text-lines-applied";
const DECLINED: &str = "split-text-lines-declined";
const PREFLIGHT: &str = "split-preflight";

/// One driven document: its fixture, how to rebuild it, and a point on its text.
struct Case {
    fixture: &'static str,
    method: &'static str,
    /// A point on the text object's middle (or first) line, in page points.
    at: (f64, f64),
}

const STACKED: Case = Case {
    fixture: "stacked-labels.pdf",
    method: "Rebuild it with `python fixtures/stacked-labels.PROVENANCE.py`.",
    at: (110.0, 684.0),
};

const QUOTED: Case = Case {
    fixture: "quote-operator.pdf",
    method: "Rebuild it with `python fixtures/quote-operator.PROVENANCE.py`.",
    at: (100.0, 704.0),
};

/// See the module documentation.
pub struct ATextObjectSplitsIntoLines;

impl Check for ATextObjectSplitsIntoLines {
    fn name(&self) -> &'static str {
        "a_text_object_splits_into_lines"
    }

    fn defect(&self) -> &'static str {
        "a multi-line text object's right-click menu offers no Split into lines, the press never \
         reaches split_text_object, the split leaves no undo entry, or a refused split says nothing"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = drive(ctx, &mut report, &STACKED, splits).and_then(|first| match first {
            Some(failure) => Ok(Some(failure)),
            None => drive(ctx, &mut report, &QUOTED, refuses),
        });
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

type Steps = fn(&Session, &ScriptedPointer, usize, &mut CheckReport) -> Result<Option<String>>;

fn launch(
    ctx: &CheckContext,
    report: &mut CheckReport,
    case: &Case,
) -> Result<(Session, ScriptedPointer)> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    // Driven on a copy, so a stray save never touches the repository's fixture.
    let doc = ctx.out(case.fixture);
    std::fs::copy(repo_fixture(case.fixture, case.method)?, &doc)
        .map_err(|e| Error::new(format!("copying {}: {e}", case.fixture)))?;
    let stem = case.fixture.trim_end_matches(".pdf");
    let mut spec = LaunchSpec::new(&exe, ctx.out(&format!("split_lines.{stem}.trace.txt")));
    spec.pdf = Some(doc);
    for (k, v) in [
        (ctx.profile.diag_env.0, ctx.profile.diag_env.1),
        SHELL_DIAG_ENV,
        (viewport_env, OFFSCREEN),
    ] {
        spec.env.push((k.to_owned(), v.to_owned()));
    }
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(
        &mut spec,
        ctx.out(&format!("split_lines.{stem}.pointer.txt")),
    )?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    report.note(format!(
        "{}: launched as pid {}",
        case.fixture,
        session.pid()
    ));
    session.settle(40);
    Ok((session, pointer))
}

/// Launch on `case`, select its text, right-click it, press the row, then
/// hand the trace mark taken before the press to `then`.
fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    case: &Case,
    then: Steps,
) -> Result<Option<String>> {
    let (session, pointer) = launch(ctx, report, case)?;
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile names no ui-rect trace event."))?;
    click_mode_segment(&session, &pointer, ui_rect, "edit")?;
    session.settle(20);
    let pdf = repo_fixture(case.fixture, case.method)?;
    let page = crate::fixture::page_geometry(&pdf)
        .ok_or_else(|| Error::new("could not read a page size from the fixture."))?;
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, page, 0)?;
    let point = mapping.doc_to_window(DocPoint::new(0, case.at.0, case.at.1))?;
    pointer.click(&session, point)?;
    session.settle(20);
    pointer.right_click(&session, point)?;
    session.settle(20);
    let trace = session.trace()?;
    let Some(row) = declared(&trace, ui_rect, MENU_ROW) else {
        return Ok(Some(format!(
            "{}: no `{MENU_ROW}` row after a right-click on the text. Rows: {}. Trace: {}.",
            case.fixture,
            list(&declared_names(&trace, ui_rect, "menu.item.canvas.")),
            session.trace_path().display()
        )));
    };
    let mark = trace.mark();
    pointer.click(&session, WindowPoint::centre_of(row))?;
    session.settle(30);
    let outcome = then(&session, &pointer, mark, report)?;
    pointer.gone(&session)?;
    Ok(outcome)
}

/// The three-line object became three, and Ctrl+Z rejoined them.
fn splits(
    session: &Session,
    pointer: &ScriptedPointer,
    mark: usize,
    report: &mut CheckReport,
) -> Result<Option<String>> {
    let trace = session.trace()?;
    let Some(line) = trace.last_after(APPLIED, mark) else {
        return Ok(Some(format!(
            "★★★ pressing Split into lines traced no `{APPLIED}`: the engine was never asked. \
             Declined: {}. Trace: {}.",
            trace
                .last_after(DECLINED, mark)
                .map_or("none", |l| l.raw.as_str()),
            session.trace_path().display()
        )));
    };
    report.note(line.raw.clone());
    let want = [
        ("object", "0"),
        ("cuts", "2"),
        ("pieces", "3"),
        ("disclosed", "1"),
    ];
    let missing: Vec<String> = want
        .iter()
        .filter(|(k, v)| line.get(k) != Some(*v))
        .map(|(k, v)| format!("{k}={v}"))
        .collect();
    if !missing.is_empty() {
        return Ok(Some(format!(
            "★★★ `{}` lacks {}.",
            line.raw,
            missing.join(" ")
        )));
    }
    let mark = trace.mark();
    pointer.key(session, None, "Z", Some("ctrl"))?;
    session.settle(30);
    if session.trace()?.last_after("undo-applied", mark).is_none() {
        return Ok(Some(
            "Ctrl+Z after the split traced no `undo-applied`: the split left no undo entry."
                .to_owned(),
        ));
    }
    report.note("split into three, rejoined by Ctrl+Z");
    Ok(None)
}

/// A line drawn with `'` is refused before the press: the engine's preflight
/// names it and the greyed row swallows the click.
fn refuses(
    session: &Session,
    _pointer: &ScriptedPointer,
    mark: usize,
    report: &mut CheckReport,
) -> Result<Option<String>> {
    let trace = session.trace()?;
    if let Some(line) = trace.last_after(APPLIED, mark) {
        return Ok(Some(format!(
            "★★★ a text object whose second line is drawn with `'` was split: `{}`.",
            line.raw
        )));
    }
    let Some(preflight) = trace.last_after(PREFLIGHT, 0) else {
        return Ok(Some(format!(
            "★★★ selecting the text traced no `{PREFLIGHT}`: the engine was never asked whether \
             the split would be refused. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(preflight.raw.clone());
    if preflight.get("refusal") != Some("line-show-operator") {
        return Ok(Some(format!(
            "★★★ the preflight answered `{}`, not `line-show-operator`: `{}`.",
            preflight.get("refusal").unwrap_or("nothing"),
            preflight.raw
        )));
    }
    if let Some(line) = trace.last_after(DECLINED, mark) {
        return Ok(Some(format!(
            "★★★ the row the engine refused was pressable: the click reached the press, \
             which declined it (`{}`). The row should have been greyed.",
            line.raw
        )));
    }
    report.note("the refused row was greyed; the click reached no press");
    Ok(None)
}
