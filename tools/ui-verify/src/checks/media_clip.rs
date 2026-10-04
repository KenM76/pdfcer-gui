//! `a_clip_plays_from_a_region` — Markup ▸ Media clip, a drag on the page,
//! the picker's clip, the page-open trigger, the always-temporary-copy
//! permission and Add put a `/Screen` region on the page carrying the clip
//! under its suggested type.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/media_clip.md`.

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
const MEDIA_ENV: &str = "PDFCER_DIAG_MEDIA_PATH"; // ui-text-exempt: an environment variable name
/// The clip's bytes. The engine embeds them unread, so any content serves;
/// the `.mp4` name is what must yield the suggested type.
const PAYLOAD: &[u8] = b"not really a video, but embedded byte for byte";
/// The drag, corner to corner, in an empty part of the fixture's 800 x 600 page.
const SWEEP: ((f64, f64), (f64, f64)) = ((150.0, 450.0), (300.0, 350.0));
const TAB: &str = "ribbon.tab.markup";
const ITEM: &str = "ribbon.item.markup.screen";
const PAGE_OPEN: &str = "text-annot.screen-trigger.PageOpen";
const ALWAYS: &str = "text-annot.screen-temp.TEMPALWAYS";
const ACCEPT: &str = "text-annot.accept";

/// See the module documentation.
pub struct AClipPlaysFromARegion;

impl Check for AClipPlaysFromARegion {
    fn name(&self) -> &'static str {
        "a_clip_plays_from_a_region"
    }

    fn defect(&self) -> &'static str {
        "Markup > Media clip does not arm, a drag on the page never asks for a clip or never \
         opens the window, the suggested type, the trigger or the temporary-file choice is \
         lost, or Add never reaches add_screen_annotation"
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
    let payload = ctx.out("media-clip.payload.mp4");
    std::fs::write(&payload, PAYLOAD)
        .map_err(|e| Error::new(format!("cannot write {}: {e}", payload.display())))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("media_clip.trace.txt"));
    spec.pdf = Some(repo_fixture(FIXTURE, METHOD)?);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push((viewport_env.to_owned(), OFFSCREEN.to_owned()));
    spec.env
        .push((MEDIA_ENV.to_owned(), payload.to_string_lossy().into_owned()));
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("media_clip.pointer.txt"))?;
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
        .any(|l| l.get("tool").is_some_and(|t| t.contains("Screen")))
    {
        return Ok(Some(format!(
            "Markup > Media clip traced no `markup-tool tool=…Screen…` line, so it armed \
             nothing. Trace: {}.",
            session.trace_path().display()
        )));
    }
    let pdf = repo_fixture(FIXTURE, METHOD)?;
    let page = crate::fixture::page_geometry(&pdf)
        .ok_or_else(|| Error::new("could not read a page size from the fixture."))?;
    let mapping = CanvasMapping::from_trace(&trace, &ctx.profile.vocab, page, 0)?;
    let at = |p: (f64, f64)| mapping.doc_to_window(DocPoint::new(0, p.0, p.1));
    pointer.drag(session, at(SWEEP.0)?, at(SWEEP.1)?, 8)?;
    session.settle(30);
    let trace = session.trace()?;
    let Some(open) = trace.events("screen-annot-open").next() else {
        return Ok(Some(format!(
            "the drag on the page traced no `screen-annot-open`: the clip was never asked for \
             or the window never opened. Picker: {}.",
            trace
                .events("media-picked")
                .last()
                .map_or("no `media-picked` line", |l| l.raw.as_str())
        )));
    };
    report.note(open.raw.clone());
    step!(press(session, pointer, ui_rect, PAGE_OPEN, "text-annot.")?);
    step!(press(session, pointer, ui_rect, ALWAYS, "text-annot.")?);
    step!(press(session, pointer, ui_rect, ACCEPT, "text-annot.")?);
    session.settle(20);
    placed(session, report)
}

/// The read line must carry the payload's length, the type the `.mp4` name
/// suggests, and the two non-default choices pressed; the placed line a
/// non-zero id. Both choices differ from the engine's defaults, so a
/// dropped choice reads as the default and fails.
fn placed(session: &Session, report: &mut CheckReport) -> Result<Option<String>> {
    let trace = session.trace()?;
    let Some(read) = trace.events("screen-annot-read").next() else {
        return Ok(Some(format!(
            "Add traced no `screen-annot-read`: the window's accept never reached the place \
             action. Unreadable: {}.",
            trace
                .events("screen-annot-unreadable")
                .next()
                .map_or("none", |l| l.raw.as_str())
        )));
    };
    report.note(read.raw.clone());
    let bytes = PAYLOAD.len().to_string();
    let want = [
        ("bytes", bytes.as_str()),
        ("type", "video/mp4"),
        ("trigger", "PageOpen"),
        ("temp", "TEMPALWAYS"),
    ];
    if want.iter().any(|(k, v)| read.get(k) != Some(*v)) {
        return Ok(Some(format!(
            "★★★ the clip read `{}`; it must carry bytes={bytes} type=video/mp4 \
             trigger=PageOpen temp=TEMPALWAYS.",
            read.raw
        )));
    }
    let Some(line) = trace.events("screen-annot-placed").next() else {
        return Ok(Some(
            "★★★ the clip was read and no `screen-annot-placed` followed, so the engine never \
             authored the region."
                .to_owned(),
        ));
    };
    report.note(line.raw.clone());
    Ok(line
        .get("id")
        .is_none_or(|id| id == "0")
        .then(|| format!("★★★ the placed line `{}` names no object.", line.raw)))
}
