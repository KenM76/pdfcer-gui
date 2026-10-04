//! `a_caret_marks_an_insertion` — Markup ▸ Insert text, a click on the page,
//! typed words, New paragraph and Add put a `/Caret` on the page carrying the
//! words and the paragraph mark.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/insert_text.md`.

use crate::checks::driving::{
    SHELL_DIAG_ENV, click_mode_segment, declared_in, declared_names, list, repo_fixture,
};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, WindowPoint};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const FIXTURE: &str = "layer-assign.pdf";
const METHOD: &str = "Rebuild it with `python fixtures/layer-assign.PROVENANCE.py`.";
const OFFSCREEN: &str = "-4200,-4200,1400,900";
/// The words typed; the caret reports their trimmed character count.
const WORDS: &str = "inserted words ";
const WORDS_CHARS: &str = "14";
/// An empty spot on the fixture's 800 x 600 page, clear of its box and annotation.
const SPOT: (f64, f64) = (150.0, 450.0);
const TAB: &str = "ribbon.tab.markup";
const ITEM: &str = "ribbon.item.markup.insert_text";
const PARAGRAPH: &str = "text-annot.caret-paragraph";
const ACCEPT: &str = "text-annot.accept";

/// See the module documentation.
pub struct ACaretMarksAnInsertion;

impl Check for ACaretMarksAnInsertion {
    fn name(&self) -> &'static str {
        "a_caret_marks_an_insertion"
    }

    fn defect(&self) -> &'static str {
        "Markup > Insert text does not arm, a click on the page never opens the window, the \
         typed words or the New paragraph choice are lost, or Add never reaches \
         add_caret_annotation"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = launch(ctx, &mut report).and_then(|(session, pointer)| {
            let outcome = drive(ctx, &mut report, &session, &pointer);
            let parked = pointer.gone(&session);
            match outcome? {
                Some(failure) => Ok(Some(failure)),
                None => parked.map(|_| None),
            }
        });
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

macro_rules! step {
    ($e:expr) => {
        match $e {
            Ok(v) => v,
            Err(why) => return Ok(Some(why)),
        }
    };
}

fn launch(ctx: &CheckContext, report: &mut CheckReport) -> Result<(Session, ScriptedPointer)> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("insert_text.trace.txt"));
    spec.pdf = Some(repo_fixture(FIXTURE, METHOD)?);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push((viewport_env.to_owned(), OFFSCREEN.to_owned()));
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("insert_text.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    report.note(format!("launched as pid {}", session.pid()));
    session.settle(40);
    Ok((session, pointer))
}

/// Click a declared region, or the failure naming the regions under `family`.
fn press(
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
    region: &str,
    family: &str,
) -> Result<std::result::Result<(), String>> {
    let trace = session.trace()?;
    let Some((r, viewport)) = declared_in(&trace, ui_rect, region) else {
        return Ok(Err(format!(
            "no `{region}` region. Regions beginning `{family}`: {}. Trace: {}.",
            list(&declared_names(&trace, ui_rect, family)),
            session.trace_path().display()
        )));
    };
    pointer.click_in(session, viewport.as_deref(), WindowPoint::centre_of(r))?;
    session.settle(20);
    Ok(Ok(()))
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile names no ui-rect trace event."))?;
    click_mode_segment(session, pointer, ui_rect, "review")?;
    session.settle(20);
    step!(press(session, pointer, ui_rect, TAB, "ribbon.tab.")?);
    step!(press(
        session,
        pointer,
        ui_rect,
        ITEM,
        "ribbon.item.markup."
    )?);
    let trace = session.trace()?;
    if !trace
        .events("markup-tool")
        .any(|l| l.get("tool").is_some_and(|t| t.contains("Caret")))
    {
        return Ok(Some(format!(
            "Markup > Insert text traced no `markup-tool tool=…Caret…` line, so it armed \
             nothing. Trace: {}.",
            session.trace_path().display()
        )));
    }
    let pdf = repo_fixture(FIXTURE, METHOD)?;
    let page = crate::fixture::page_geometry(&pdf)
        .ok_or_else(|| Error::new("could not read a page size from the fixture."))?;
    let mapping = CanvasMapping::from_trace(&trace, &ctx.profile.vocab, page, 0)?;
    pointer.click(
        session,
        mapping.doc_to_window(DocPoint::new(0, SPOT.0, SPOT.1))?,
    )?;
    session.settle(30);
    let trace = session.trace()?;
    let Some(open) = trace
        .events("text-annot-open")
        .find(|l| l.get("kind") == Some("Caret"))
    else {
        return Ok(Some(format!(
            "the click on the page traced no `text-annot-open kind=Caret`: the window never \
             opened. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(open.raw.clone());
    let Some((_, viewport)) = declared_in(&trace, ui_rect, ACCEPT) else {
        return Ok(Some(format!(
            "the insert-text window declared no `{ACCEPT}`. Regions beginning `text-annot.`: {}.",
            list(&declared_names(&trace, ui_rect, "text-annot."))
        )));
    };
    pointer.type_text(session, viewport.as_deref(), WORDS)?;
    session.settle(20);
    step!(press(session, pointer, ui_rect, PARAGRAPH, "text-annot.")?);
    step!(press(session, pointer, ui_rect, ACCEPT, "text-annot.")?);
    session.settle(20);
    placed(session, report)
}

/// The read line must carry the typed words' count and the paragraph mark,
/// and the placed line a non-zero id.
fn placed(session: &Session, report: &mut CheckReport) -> Result<Option<String>> {
    let trace = session.trace()?;
    let Some(read) = trace.events("caret-annot-read").next() else {
        return Ok(Some(format!(
            "Add traced no `caret-annot-read`: the window's accept never reached the place \
             action. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(read.raw.clone());
    if read.get("chars") != Some(WORDS_CHARS) || read.get("paragraph") != Some("true") {
        return Ok(Some(format!(
            "★★★ the caret read `{}`; it must carry chars={WORDS_CHARS} and paragraph=true.",
            read.raw
        )));
    }
    let Some(line) = trace.events("caret-annot-placed").next() else {
        return Ok(Some(
            "★★★ the caret was read and no `caret-annot-placed` followed, so the engine never \
             authored it."
                .to_owned(),
        ));
    };
    report.note(line.raw.clone());
    Ok(line
        .get("id")
        .is_none_or(|id| id == "0")
        .then(|| format!("★★★ the placed line `{}` names no object.", line.raw)))
}
