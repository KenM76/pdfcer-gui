//! Shared off-screen driving: launch a fixture with a ribbon invoke, press a
//! declared region, click a page point. The ce dimension checks use
//! `fixtures/dimension-scaled.pdf` through [`run`]; others name theirs through
//! [`run_on`].

use crate::checks::driving::{SHELL_DIAG_ENV, declared_in, repo_fixture};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry, WindowPoint};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

pub(super) const OFFSCREEN: &str = "-4200,-4200,1400,1000";
pub(super) const FIXTURE: &str = "dimension-scaled.pdf";
pub(super) const METHOD: &str = "Rebuild it with `python fixtures/dimension-scaled.PROVENANCE.py`.";
pub(super) const PAGE: PageGeometry = PageGeometry {
    width_pt: 400.0,
    height_pt: 300.0,
};

/// A checked-in fixture, how to rebuild it, and its page size.
pub(super) struct Fixture {
    pub file: &'static str,
    pub method: &'static str,
    pub page: PageGeometry,
}

const DIMENSION_SCALED: Fixture = Fixture {
    file: FIXTURE,
    method: METHOD,
    page: PAGE,
};

pub(super) type Body =
    fn(&CheckContext, &mut CheckReport, &Session, &ScriptedPointer) -> Result<Option<String>>;

pub(super) fn run(
    check: &dyn Check,
    ctx: &CheckContext,
    invoke: &str,
    stem: &str,
    body: Body,
) -> CheckReport {
    run_on(check, ctx, &DIMENSION_SCALED, invoke, stem, body)
}

pub(super) fn run_on(
    check: &dyn Check,
    ctx: &CheckContext,
    fixture: &Fixture,
    invoke: &str,
    stem: &str,
    body: Body,
) -> CheckReport {
    let mut report = CheckReport::new(check.name(), check.defect());
    let driven = launch(ctx, &mut report, fixture, invoke, stem).and_then(|(session, pointer)| {
        let outcome = body(ctx, &mut report, &session, &pointer);
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

pub(super) fn launch(
    ctx: &CheckContext,
    report: &mut CheckReport,
    fixture: &Fixture,
    invoke: &str,
    stem: &str,
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
    let doc = ctx.out(&format!("{stem}.pdf"));
    std::fs::copy(repo_fixture(fixture.file, fixture.method)?, &doc)
        .map_err(|e| Error::new(format!("copying {}: {e}", fixture.file)))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out(&format!("{stem}.trace.txt")));
    spec.pdf = Some(doc);
    for (k, v) in [
        (ctx.profile.diag_env.0, ctx.profile.diag_env.1),
        SHELL_DIAG_ENV,
        (viewport_env, OFFSCREEN),
        ("PDFCER_DIAG_INVOKE", invoke),
    ] {
        spec.env.push((k.to_owned(), v.to_owned()));
    }
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out(&format!("{stem}.pointer.txt")))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    report.note(format!("launched as pid {}", session.pid()));
    session.settle(60);
    Ok((session, pointer))
}

pub(super) fn ui_rect_event(ctx: &CheckContext) -> Result<&'static str> {
    ctx.profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))
}

/// Click `region` where it was last declared; an error naming it when it is not.
pub(super) fn press(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    region: &str,
) -> Result<()> {
    let (rect, vp) =
        declared_in(&session.trace()?, ui_rect_event(ctx)?, region).ok_or_else(|| {
            Error::new(format!(
                "`{region}` was never declared. Trace: {}.",
                session.trace_path().display()
            ))
        })?;
    pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(rect))?;
    session.settle(30);
    Ok(())
}

/// Press `opener` (a dock tab or a folded heading) when `region` has not been drawn yet.
pub(super) fn reveal(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    region: &str,
    opener: &str,
) -> Result<()> {
    if declared_in(&session.trace()?, ui_rect_event(ctx)?, region).is_none() {
        press(ctx, session, pointer, opener)?;
    }
    Ok(())
}

/// Click `at` (page points) on page 1 of `dimension-scaled.pdf`.
pub(super) fn click_page(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    at: (f64, f64),
) -> Result<()> {
    click_on(ctx, session, pointer, &DIMENSION_SCALED, 0, at)
}

/// Click `at` (page points) on page index `page` of `fixture`, through the
/// current canvas mapping.
pub(super) fn click_on(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    fixture: &Fixture,
    page: usize,
    at: (f64, f64),
) -> Result<()> {
    let mapping =
        CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, fixture.page, page)?;
    let point = mapping.doc_to_window(DocPoint::new(page, at.0, at.1))?;
    pointer.hover(session, point)?;
    session.settle(10);
    pointer.click(session, point)?;
    session.settle(20);
    Ok(())
}
