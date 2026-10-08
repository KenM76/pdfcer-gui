//! `thumbnails_zoom_without_blanking` — each thumbnail lands as a quick draft
//! and then a finer picture for its tile's width, the Pages panel's size
//! buttons resize the tiles, and every tile keeps a picture through the
//! change. Driven off the desktop
//! with the scripted pointer, on `fixtures/four-pages.pdf`.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/thumbnail_zoom.md`.

use crate::checks::driving::{
    SHELL_DIAG_ENV, declared, declared_in, declared_names, declared_since, list, repo_fixture,
};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::trace::Trace;

const OFFSCREEN: &str = "-4200,-4200,1400,900";
const FIXTURE: &str = "four-pages.pdf";
const UI_RECT: &str = "ui-rect";
const SMALLER: &str = "panel-pages-zoom-out"; // ui-text-exempt: a trace region name, never displayed
const LARGER: &str = "panel-pages-zoom-in"; // ui-text-exempt: a trace region name, never displayed
const TILE: &str = "panel-pages-tile.0"; // ui-text-exempt: a trace region name, never displayed
const TILES: &str = "pages-tiles"; // ui-text-exempt: a trace event name, never displayed
const THUMB: &str = "pages-thumbnail"; // ui-text-exempt: a trace event name, never displayed
const ZOOM: &str = "pages-zoom"; // ui-text-exempt: a trace event name, never displayed
/// The fixture's page count.
const PAGES: usize = 4;
/// Polls of ten frames each before a fill counts as never arriving.
const POLLS: usize = 60;

/// See the module documentation.
pub struct ThumbnailsZoomWithoutBlanking;

impl Check for ThumbnailsZoomWithoutBlanking {
    fn name(&self) -> &'static str {
        "thumbnails_zoom_without_blanking"
    }

    fn defect(&self) -> &'static str {
        "the page thumbnails cannot be made larger, a tile goes blank while they resize, or \
         the larger tiles keep only the small picture stretched"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let outcome = launch(ctx, &mut report).and_then(|(session, pointer)| {
            let outcome = drive(&mut report, &session, &pointer);
            let parked = pointer.gone(&session);
            let outcome = outcome?;
            parked?;
            Ok(outcome)
        });
        match outcome {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// Wait for a full grid and its finer pass; press Smaller twice, then Larger
/// twice; judge the sizes and the census through both changes.
fn drive(
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    if !wait(session, |t| filled(t, 0))? {
        return Ok(Some(format!(
            "the thumbnails never filled at launch: last `{TILES}` was {:?}.",
            session.trace()?.last(TILES).map(|l| l.raw.clone())
        )));
    }
    if let Some(failure) = two_passes(report, session)? {
        return Ok(Some(failure));
    }
    let trace = session.trace()?;
    let mark = trace.mark();
    let Some(start) = declared(&trace, UI_RECT, TILE) else {
        return Ok(Some(format!(
            "no `{TILE}` region; tiles declared: {}.",
            list(&declared_names(&trace, UI_RECT, "panel-pages-tile."))
        )));
    };
    for (button, grows) in [(SMALLER, false), (LARGER, true)] {
        let from = session
            .trace()
            .ok()
            .and_then(|t| declared(&t, UI_RECT, TILE))
            .unwrap_or(start);
        let since = session.trace()?.mark();
        press(session, pointer, button)?;
        press(session, pointer, button)?;
        let moved = wait(session, |t| {
            declared_since(t, UI_RECT, TILE, since).is_some_and(|r| {
                if grows {
                    r.width() > from.width() * 1.3
                } else {
                    r.width() < from.width() * 0.7
                }
            })
        })?;
        let to = declared_since(&session.trace()?, UI_RECT, TILE, since).map(|r| r.width());
        report.note(format!(
            "{button} twice: tile {:.0} -> {to:?}",
            from.width()
        ));
        if !moved {
            return Ok(Some(format!(
                "`{button}` was pressed twice and the first tile went from {:.0} pt wide to \
                 {to:?}, not markedly {}.",
                from.width(),
                if grows { "wider" } else { "narrower" }
            )));
        }
    }
    let trace = session.trace()?;
    let zooms: Vec<String> = trace
        .events(ZOOM)
        .filter(|l| l.lineno > mark)
        .map(|l| l.raw.clone())
        .collect();
    report.note(format!("zoom: {}", list(&zooms)));
    let blanks: Vec<String> = trace
        .events(TILES)
        .filter(|l| l.lineno > mark)
        .filter(|l| l.get_usize("blank").unwrap_or(0) > never_drawn(&trace, l.lineno))
        .map(|l| l.raw.clone())
        .collect();
    if !blanks.is_empty() {
        return Ok(Some(format!(
            "a tile that had a picture was drawn without one while the thumbnails \
             resized (more blank tiles than pages never drawn): {}.",
            list(&blanks)
        )));
    }
    Ok(None)
}

/// Page 1 lands a draft first and a fine picture after it: the dock's tile
/// is wider than the draft.
fn two_passes(report: &mut CheckReport, session: &Session) -> Result<Option<String>> {
    let fine = wait(session, |t| first_landing(t, true).is_some())?;
    let trace = session.trace()?;
    report.note(format!("page 1 landings: {}", list(&page_one(&trace))));
    let draft = first_landing(&trace, false);
    let Some(fine_at) = first_landing(&trace, true).filter(|_| fine) else {
        return Ok(Some(format!(
            "page 1's tile is wider than the draft and no fine picture followed: {}.",
            list(&page_one(&trace))
        )));
    };
    if draft.is_none_or(|d| d > fine_at) {
        return Ok(Some(format!(
            "page 1 had no quick draft before its fine picture: {}.",
            list(&page_one(&trace))
        )));
    }
    Ok(None)
}

/// The line number of page 1's first landing at the fine grade (`fine`) or
/// the draft.
fn first_landing(trace: &Trace, fine: bool) -> Option<usize> {
    trace
        .events(THUMB)
        .filter(|l| l.get("page") == Some("1"))
        .find(|l| {
            l.get("grade")
                .is_some_and(|g| g.starts_with("fine:") == fine)
        })
        .map(|l| l.lineno)
}

/// How many of the fixture's pages had no picture land before `lineno`. A
/// page scrolled into view for the first time is blank until its draft lands;
/// a blank count above this is a picture taken away.
fn never_drawn(trace: &Trace, lineno: usize) -> usize {
    (1..=PAGES)
        .filter(|p| {
            let page = p.to_string();
            !trace
                .events(THUMB)
                .any(|l| l.lineno < lineno && l.get("page") == Some(page.as_str()))
        })
        .count()
}

fn page_one(trace: &Trace) -> Vec<String> {
    trace
        .events(THUMB)
        .filter(|l| l.get("page") == Some("1"))
        .map(|l| l.raw.clone())
        .collect()
}

/// The newest census after `mark` shows visible tiles, all with a picture
/// and none pending.
fn filled(trace: &Trace, mark: usize) -> bool {
    trace.last_after(TILES, mark).is_some_and(|l| {
        l.get_usize("visible").is_some_and(|v| v > 0)
            && l.get_usize("pending") == Some(0)
            && l.get_usize("blank") == Some(0)
    })
}

fn wait(session: &Session, done: impl Fn(&Trace) -> bool) -> Result<bool> {
    for _ in 0..POLLS {
        if done(&session.trace()?) {
            return Ok(true);
        }
        session.settle(10);
    }
    Ok(false)
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
    let mut spec = LaunchSpec::new(&exe, ctx.out("thumbnail_zoom.trace.txt"));
    spec.pdf = Some(repo_fixture(FIXTURE, "It is a checked-in fixture.")?);
    for (k, v) in [
        (ctx.profile.diag_env.0, ctx.profile.diag_env.1),
        SHELL_DIAG_ENV,
        (viewport_env, OFFSCREEN),
        ("PDFCER_DIAG_INVOKE", "mode.edit"),
    ] {
        spec.env.push((k.to_owned(), v.to_owned()));
    }
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("thumbnail_zoom.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!("launched as pid {}", session.pid()));
    session.settle(45);
    Ok((session, pointer))
}

fn press(session: &Session, pointer: &ScriptedPointer, name: &str) -> Result<()> {
    let trace = session.trace()?;
    let (rect, viewport) = declared_in(&trace, UI_RECT, name).ok_or_else(|| {
        Error::new(format!(
            "no `{name}` region. Pages-panel regions: {}.",
            list(&declared_names(&trace, UI_RECT, "panel-pages-"))
        ))
    })?;
    pointer.click_in(session, viewport.as_deref(), WindowPoint::centre_of(rect))?;
    session.settle(15);
    Ok(())
}
