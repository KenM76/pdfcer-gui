//! Reaching a control inside the Properties panel's scroll area with the
//! scripted pointer: wheel the panel until the control's region is declared
//! visible, then click its centre.

use crate::checks::CheckContext;
use crate::checks::driving::declared_in;
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;

/// The Properties panel's dock body, the region the wheel turns over.
pub const PANEL_BODY: &str = "dock.body.file.properties";

/// How many wheel turns before a control is called unreachable.
const MAX_SCROLL: usize = 12;

/// Scroll the Properties panel until `region` is declared, click it, and
/// settle `settle` frames; returns the viewport it was declared in.
///
/// # Errors
///
/// No ui-rect event in the profile, no Properties panel, or `region` still
/// undeclared after [`MAX_SCROLL`] turns.
pub fn press(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    region: &str,
    settle: u32,
) -> Result<Option<String>> {
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    for _ in 0..=MAX_SCROLL {
        let trace = session.trace()?;
        if let Some((rect, vp)) = declared_in(&trace, ui_rect, region) {
            pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(rect))?;
            session.settle(settle);
            return Ok(vp);
        }
        let (panel, vp) = declared_in(&trace, ui_rect, PANEL_BODY).ok_or_else(|| {
            Error::new(format!(
                "no `{PANEL_BODY}` region, so the Properties panel is not open."
            ))
        })?;
        pointer.wheel_in(session, vp.as_deref(), WindowPoint::centre_of(panel), -3.0)?;
        session.settle(10);
    }
    Err(Error::new(format!(
        "no visible `{region}` after {MAX_SCROLL} wheel turns. Trace: {}",
        session.trace_path().display()
    )))
}

/// Off every monitor, unfocused; tall so most rows need little scrolling.
const OFFSCREEN: &str = "-4200,-4200,1200,1350";
/// Edit mode, then the Properties panel.
const INVOKE: &str = "mode.edit,file.properties";

/// Launch on a copy of the repository fixture `fixture` with the form field
/// `field` selected through the seam and the Properties panel open, off
/// screen, with a scripted pointer. `label` names the copy and the artifacts.
///
/// # Errors
///
/// No binary, no viewport variable, the fixture missing, or the launch failing.
pub fn launch_on_field(
    ctx: &CheckContext,
    report: &mut crate::report::CheckReport,
    (fixture, method): (&str, &str),
    field: &str,
    label: &str,
) -> Result<(Session, ScriptedPointer)> {
    launch_on_field_invoking(ctx, report, (fixture, method), (field, INVOKE), label)
}

/// [`launch_on_field`] with `invoke` run on opening instead of Edit mode and
/// the Properties panel.
///
/// # Errors
///
/// As [`launch_on_field`].
pub fn launch_on_field_invoking(
    ctx: &CheckContext,
    report: &mut crate::report::CheckReport,
    (fixture, method): (&str, &str),
    (field, invoke): (&str, &str),
    label: &str,
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
    let doc = ctx.out(&format!("{label}.pdf"));
    std::fs::copy(crate::checks::driving::repo_fixture(fixture, method)?, &doc)
        .map_err(|e| Error::new(format!("copying {fixture}: {e}")))?;
    let mut spec = crate::launch::LaunchSpec::new(&exe, ctx.out(&format!("{label}.trace.txt")));
    spec.pdf = Some(doc);
    for (k, v) in [
        (ctx.profile.diag_env.0, ctx.profile.diag_env.1),
        crate::checks::driving::SHELL_DIAG_ENV,
        (viewport_env, OFFSCREEN),
        ("PDFCER_DIAG_INVOKE", invoke),
        ("PDFCER_DIAG_SELECT_FIELD", field),
    ] {
        spec.env.push((k.to_owned(), v.to_owned()));
    }
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out(&format!("{label}.pointer.txt")))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    report.note(format!("launched as pid {}", session.pid()));
    session.settle(60);
    Ok((session, pointer))
}

/// The newest `event` line whose `field` is `field` must carry every pair in
/// `want`; `Some(failure)` names what it lacks, or that there was no line.
///
/// # Errors
///
/// The trace cannot be read.
pub fn reads(
    session: &Session,
    report: &mut crate::report::CheckReport,
    (event, field): (&str, &str),
    after: &str,
    want: &[(&str, &str)],
) -> Result<Option<String>> {
    let trace = session.trace()?;
    let Some(line) = trace
        .events(event)
        .filter(|l| l.get("field") == Some(field))
        .last()
    else {
        return Ok(Some(format!(
            "{after}: no `{event} field={field}` line, so the control never drew for the \
             field. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("{after}: {}", line.raw));
    let missing: Vec<String> = want
        .iter()
        .filter(|(k, v)| line.get(k) != Some(*v))
        .map(|(k, v)| format!("{k}={v}"))
        .collect();
    Ok((!missing.is_empty())
        .then(|| format!("★★★ {after}, `{}` lacks {}.", line.raw, missing.join(" "))))
}
