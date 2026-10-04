//! `an_annotation_can_be_hidden_and_shown_again` — the Properties panel's
//! Print and Show on screen switches change a selected annotation's `/F`, a
//! hidden mark stops being clickable, and the Comments list's *Show on screen*
//! brings it back.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/annot_flags.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, declared_in, repo_fixture};
use crate::checks::properties_pane;
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, WindowPoint};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const OFFSCREEN: &str = "-4200,-4200,1200,1350";
/// Properties last, so it is the active tab when both share a stack.
const INVOKE: &str = "mode.edit,markup.comments,file.properties";
const FIXTURE: &str = "layer-assign.pdf";
const METHOD: &str = "Rebuild it with `python fixtures/layer-assign.PROVENANCE.py`.";
/// A point inside the fixture's `/Square` (`/Rect [480 350 720 520]`, no `/F`).
const ANNOT: (f64, f64) = (600.0, 435.0);
const PRINTS: &str = "properties.annot_flags.prints";
const ON_SCREEN: &str = "properties.annot_flags.on_screen";
const COMMENTS_TAB: &str = "dock.tab.markup.comments";
const SHOW_AGAIN: &str = "comments.show_again";
const SELECT: &str = "annot-select";
const APPLIED: &str = "set-annotation-flag-applied";
/// `/F` bits: Print 4, NoView 32.
const PRINT: u32 = 4;
const NO_VIEW: u32 = 32;

/// See the module documentation.
pub struct AnAnnotationCanBeHiddenAndShownAgain;

impl Check for AnAnnotationCanBeHiddenAndShownAgain {
    fn name(&self) -> &'static str {
        "an_annotation_can_be_hidden_and_shown_again"
    }

    fn defect(&self) -> &'static str {
        "an annotation's Print and Show on screen flags cannot be changed from the Properties \
         panel, or a hidden mark can never be found again because the canvas no longer selects \
         it and the Comments list offers no way back"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = launch(ctx, &mut report).and_then(|(session, pointer)| {
            let outcome = steps(ctx, &mut report, &session, &pointer);
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
    // Driven on a copy, so a stray save never touches the repository's fixture.
    let doc = ctx.out("annot-flags.pdf");
    std::fs::copy(repo_fixture(FIXTURE, METHOD)?, &doc)
        .map_err(|e| Error::new(format!("copying {FIXTURE}: {e}")))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("annot_flags.trace.txt"));
    spec.pdf = Some(doc);
    for (k, v) in [
        (ctx.profile.diag_env.0, ctx.profile.diag_env.1),
        SHELL_DIAG_ENV,
        (viewport_env, OFFSCREEN),
        ("PDFCER_DIAG_INVOKE", INVOKE),
    ] {
        spec.env.push((k.to_owned(), v.to_owned()));
    }
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("annot_flags.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    report.note(format!("launched as pid {}", session.pid()));
    session.settle(60);
    Ok((session, pointer))
}

/// The `set-annotation-flag-applied` line after `mark`, checked for its
/// switch, its direction, and `/F` bit `bit` being `set` afterwards.
fn applied(
    session: &Session,
    report: &mut CheckReport,
    mark: usize,
    (switch, on): (&str, &str),
    (bit, set): (u32, bool),
) -> Result<Option<String>> {
    let trace = session.trace()?;
    let Some(line) = trace.last_after(APPLIED, mark) else {
        return Ok(Some(format!(
            "★★★ the {switch} switch was pressed and no `{APPLIED}` line followed: the engine \
             was never asked. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("{switch}: {}", line.raw));
    let after = line
        .get("after")
        .and_then(|v| u32::from_str_radix(v.trim_start_matches("0x"), 16).ok());
    let right = line.get("switch") == Some(switch)
        && line.get("on") == Some(on)
        && after.is_some_and(|f| (f & bit != 0) == set);
    Ok((!right).then(|| {
        format!(
            "★★★ `{}` must be switch={switch} on={on} with /F bit {bit} {}.",
            line.raw,
            if set { "set" } else { "clear" }
        )
    }))
}

/// Click `region` where it was last declared; `false` when it is not declared.
fn press(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    region: &str,
) -> Result<bool> {
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let Some((rect, vp)) = declared_in(&session.trace()?, ui_rect, region) else {
        return Ok(false);
    };
    pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(rect))?;
    session.settle(30);
    Ok(true)
}

/// Select the mark, switch Print on and Show on screen off, prove the canvas
/// no longer selects it, and bring it back from the Comments list.
fn steps(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let pdf = repo_fixture(FIXTURE, METHOD)?;
    let page = crate::fixture::page_geometry(&pdf)
        .ok_or_else(|| Error::new("could not read a page size from the fixture."))?;
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, page, 0)?;
    let at = mapping.doc_to_window(DocPoint::new(0, ANNOT.0, ANNOT.1))?;
    pointer.click(session, at)?;
    session.settle(20);
    if session.trace()?.last(SELECT).is_none() {
        return Ok(Some(
            "a click inside the /Square annotation traced no `annot-select`.".to_owned(),
        ));
    }

    let mark = session.trace()?.mark();
    properties_pane::press(ctx, session, pointer, PRINTS, 30)?;
    if let Some(failure) = applied(session, report, mark, ("prints", "true"), (PRINT, true))? {
        return Ok(Some(failure));
    }
    let mark = session.trace()?.mark();
    properties_pane::press(ctx, session, pointer, ON_SCREEN, 30)?;
    if let Some(failure) = applied(
        session,
        report,
        mark,
        ("on-screen", "false"),
        (NO_VIEW, true),
    )? {
        return Ok(Some(failure));
    }

    let mark = session.trace()?.mark();
    pointer.click(session, at)?;
    session.settle(20);
    if let Some(line) = session.trace()?.last_after(SELECT, mark) {
        return Ok(Some(format!(
            "★★★ a click on the hidden mark still selected it: `{}`.",
            line.raw
        )));
    }

    let mark = session.trace()?.mark();
    if !press(ctx, session, pointer, SHOW_AGAIN)? {
        press(ctx, session, pointer, COMMENTS_TAB)?;
        if !press(ctx, session, pointer, SHOW_AGAIN)? {
            return Ok(Some(format!(
                "★★★ the Comments list offers no `{SHOW_AGAIN}` for the hidden mark, so it \
                 cannot be brought back. Trace: {}.",
                session.trace_path().display()
            )));
        }
    }
    applied(
        session,
        report,
        mark,
        ("on-screen", "true"),
        (NO_VIEW, false),
    )
}
