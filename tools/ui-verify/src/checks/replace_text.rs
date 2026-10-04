//! `replacing_text_strikes_and_carets` — in Review, text swept on the page,
//! Markup ▸ Replace text, typed words and Add put a `/StrikeOut` over the
//! sweep grouped under a `/Caret` that carries the words.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/replace_text.md`.

use crate::checks::driving::{
    SHELL_DIAG_ENV, click_mode_segment, declared_in, declared_names, list, repo_fixture,
};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry, WindowPoint};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const FIXTURE: &str = "word-fragmented-lines.pdf";
const METHOD: &str = "Rebuild it with `python fixtures/word-fragmented-lines.PROVENANCE.py`.";
const LETTER: PageGeometry = PageGeometry {
    width_pt: 612.0,
    height_pt: 792.0,
};
const OFFSCREEN: &str = "-4200,-4200,1400,900";
/// The sweep along the first line, `Date Premises Required____ ` at y=700
/// from x=72, PDF points.
const SWEEP: ((f64, f64), (f64, f64)) = ((74.0, 704.0), (160.0, 704.0));
/// The words typed; the read line reports their trimmed character count.
const WORDS: &str = "new words";
const WORDS_CHARS: &str = "9";
const TAB: &str = "ribbon.tab.markup";
const ITEM: &str = "ribbon.item.markup.replace_text";
const ACCEPT: &str = "text-annot.accept";

/// See the module documentation.
pub struct ReplacingTextStrikesAndCarets;

impl Check for ReplacingTextStrikesAndCarets {
    fn name(&self) -> &'static str {
        "replacing_text_strikes_and_carets"
    }

    fn defect(&self) -> &'static str {
        "Markup > Replace text acts with nothing selected, never opens its window over a text \
             selection, loses the typed words, or Add never reaches add_replace_text"
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
    let mut spec = LaunchSpec::new(&exe, ctx.out("replace_text.trace.txt"));
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
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("replace_text.pointer.txt"))?;
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
    if let Some(line) = session.trace()?.events("replace-text-open").next() {
        return Ok(Some(format!(
            "★★★ with nothing selected, Replace text opened its window: `{}`.",
            line.raw
        )));
    }
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, LETTER, 0)?;
    let at = |p: (f64, f64)| mapping.doc_to_window(DocPoint::new(0, p.0, p.1));
    pointer.drag(session, at(SWEEP.0)?, at(SWEEP.1)?, 8)?;
    session.settle(20);
    step!(press(
        session,
        pointer,
        ui_rect,
        ITEM,
        "ribbon.item.markup."
    )?);
    session.settle(20);
    let trace = session.trace()?;
    let Some(open) = trace.events("replace-text-open").next() else {
        return Ok(Some(format!(
            "after a sweep along the first line, Replace text traced no `replace-text-open`. \
             Declined: {}. Trace: {}.",
            trace
                .events("command-declined")
                .last()
                .map_or("no `command-declined` line", |l| l.raw.as_str()),
            session.trace_path().display()
        )));
    };
    report.note(open.raw.clone());
    let Some((_, viewport)) = declared_in(&trace, ui_rect, ACCEPT) else {
        return Ok(Some(format!(
            "the replace-text window declared no `{ACCEPT}`. Regions beginning `text-annot.`: \
             {}.",
            list(&declared_names(&trace, ui_rect, "text-annot."))
        )));
    };
    pointer.type_text(session, viewport.as_deref(), WORDS)?;
    session.settle(20);
    step!(press(session, pointer, ui_rect, ACCEPT, "text-annot.")?);
    session.settle(20);
    placed(session, report, open.get("quads").unwrap_or("?"))
}

/// The read line must carry the typed words' count, the opened sweep's boxes
/// and a caret right of the sweep's start; the placed line two non-zero ids.
fn placed(session: &Session, report: &mut CheckReport, quads: &str) -> Result<Option<String>> {
    let trace = session.trace()?;
    let Some(read) = trace.events("replace-text-read").next() else {
        return Ok(Some(format!(
            "Add traced no `replace-text-read`: the window's accept never reached the place \
             action. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(read.raw.clone());
    let x = read
        .get("x")
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.0);
    if read.get("chars") != Some(WORDS_CHARS) || read.get("quads") != Some(quads) || x <= SWEEP.0.0
    {
        return Ok(Some(format!(
            "★★★ the replacement read `{}`; it must carry chars={WORDS_CHARS}, quads={quads} \
             and an x right of {}.",
            read.raw, SWEEP.0.0
        )));
    }
    let Some(line) = trace.events("replace-text-placed").next() else {
        return Ok(Some(
            "★★★ the replacement was read and no `replace-text-placed` followed, so the engine \
             never authored it."
                .to_owned(),
        ));
    };
    report.note(line.raw.clone());
    let named = |k: &str| line.get(k).is_some_and(|id| id != "0");
    Ok((!named("caret") || !named("strike"))
        .then(|| format!("★★★ the placed line `{}` names no pair.", line.raw)))
}
