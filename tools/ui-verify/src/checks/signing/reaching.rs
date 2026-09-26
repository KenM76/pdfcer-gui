//! `checks::signing::reaching` — **how the signing check reaches the controls
//! it presses**
//!
//!
//! ## THE THREE FINDINGS THAT LIVE HERE, because they are about the
//! HARNESS and will bite the next check as well
//!
//! 1. **A dialog is an OS window**, so `session.frame()` is the wrong frame for
//!    anything inside one. Everything here goes through
//!    [`super::super::driving::frame_of`]. The first driven run of this check
//!    aimed every in-dialog click hundreds of points away and the symptom was
//!    *silence*.
//! 2. **A region declared inside a `ScrollArea` is a position in the scrolled
//!    CONTENT.** [`click`] is right above the fold and silently wrong below it;
//!    [`click_scrolled`] is the one to use for anything on the form, and the
//!    form grew past the fold the day a section was added.
//! 3. **The scroll body is not the window.** A control scrolled to just above
//!    the footer is inside the window rectangle and *clipped out of the scroll
//!    area*, so egui reports its position and refuses the click — which reads
//!    as "the control is there and pressing it does nothing". [`click_scrolled`]
//!    measures against `sign-body`, and the footer's own controls ([`click`] on
//!    `sign-confirm`) are never inside it.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/signing/reaching.md`.

use std::path::{Path, PathBuf};

use super::super::CheckContext;
use super::super::driving::{
    self, INVOKE_EVENT, ITEM_PREFIX, SHELL_DIAG_ENV, TAB_EVENT, declared, declared_names,
    declared_or_in_overflow, list, shell_trace,
};
use super::{DISCLOSED_EVENT, OPENED_EVENT, REGION_BODY};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::trace::Trace;

/// Launch one process, optionally with the certificate and save-path seams set.
pub(super) fn launch(
    ctx: &CheckContext,
    report: &mut CheckReport,
    pdf: &Path,
    trace_name: &str,
    env: &[(&str, PathBuf)],
) -> Result<Session> {
    let mut spec = LaunchSpec::new(
        ctx.resolve_exe().ok_or_else(|| {
            Error::new(format!(
                "no binary to drive. Pass --exe, or build the profile's default at {}.",
                ctx.profile.default_exe
            ))
        })?,
        ctx.out(trace_name),
    );
    spec.pdf = Some(pdf.to_path_buf());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    for (key, value) in env {
        spec.env
            .push(((*key).to_owned(), value.display().to_string()));
    }
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.note(format!(
        "launched {} on {} as pid {}",
        spec.exe.display(),
        pdf.display(),
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);

    if !session.trace()?.started(ctx.profile.vocab.start_event) {
        return Err(Error::new(format!(
            "the trace has no `{}` line, so the diagnostic switch {}={} did not reach the process \
             and this check has no oracle. Captured stderr is at {}.",
            ctx.profile.vocab.start_event,
            ctx.profile.diag_env.0,
            ctx.profile.diag_env.1,
            session.trace_path().display()
        )));
    }
    Ok(session)
}

/// Click a ribbon tab and confirm the shell reported it.
pub(super) fn click_tab(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    (region, id): (&str, &str),
) -> Result<()> {
    let trace = session.trace()?;
    let rect = declared(&trace, ui_rect, region).ok_or_else(|| {
        Error::new(format!(
            "the application declared no `{region}` region. Tabs declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    let before = shell_trace(session)?
        .events(TAB_EVENT)
        .filter(|l| l.get("tab") == Some(id))
        .count();
    driver.click_at(session.frame()?.declared_center(rect))?;
    session.settle(14);
    if shell_trace(session)?
        .events(TAB_EVENT)
        .filter(|l| l.get("tab") == Some(id))
        .count()
        <= before
    {
        return Err(Error::new(format!(
            "the click on `{region}` produced no new `{TAB_EVENT} tab={id}` line."
        )));
    }
    Ok(())
}

/// **Click a ribbon tab without requiring that it CHANGED.**
fn click_tab_tolerant(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    region: &str,
) -> Result<()> {
    let trace = session.trace()?;
    let rect = declared(&trace, ui_rect, region).ok_or_else(|| {
        Error::new(format!(
            "the application declared no `{region}` region. Tabs declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(rect))?;
    session.settle(14);
    Ok(())
}

/// **Find a ribbon control and press it**, through the overflow if it is there.
pub(super) fn press(session: &Session, driver: &Driver, ui_rect: &str, id: &str) -> Result<()> {
    let name = format!("{ITEM_PREFIX}{id}");
    let found = declared_or_in_overflow(session, driver, ui_rect, &name)?;
    let items = list(&declared_names(&session.trace()?, ui_rect, ITEM_PREFIX));
    let rect = found.ok_or_else(|| {
        Error::new(format!(
            "`{id}` is on no band, in no collapsed group's popup and behind no overflow button — \
             so an operator cannot reach it. ⚠ If this build was compiled WITHOUT the `signing` \
             feature that is the correct behaviour and this check should not have been run \
             against it. Ribbon items declared: {items}."
        ))
    })?;
    let before = invokes(session, id)?;
    driver.click_at(session.frame()?.declared_center(rect))?;
    session.settle(24);
    if invokes(session, id)? <= before {
        return Err(Error::new(format!(
            "the click on `{id}` produced no new `{INVOKE_EVENT} id={id}` line, so the control was \
             found and did not fire. Every assertion below would then be measuring a window that \
             never opened."
        )));
    }
    Ok(())
}

/// How many times the shell has reported `id` invoked.
pub(super) fn invokes(session: &Session, id: &str) -> Result<usize> {
    Ok(shell_trace(session)?
        .events(INVOKE_EVENT)
        .filter(|l| l.get("id") == Some(id))
        .count())
}

/// Whether the application declared `name` at a usable rectangle.
///
/// A degenerate rect counts as **absent**, not present. A region declared at
/// zero area is not something an operator can see.
pub(super) fn drawn(trace: &Trace, ui_rect: &str, name: &str) -> bool {
    declared(trace, ui_rect, name).is_some_and(|r| r.is_substantial())
}

/// Click a region's centre, refusing when it was never drawn.
pub(super) fn click(session: &Session, driver: &Driver, ui_rect: &str, name: &str) -> Result<()> {
    let trace = session.trace()?;
    let rect = declared(&trace, ui_rect, name).ok_or_else(|| {
        Error::new(format!(
            "no `{name}` region to click. Regions declared under `sign-`: {}.",
            list(&declared_names(&trace, ui_rect, "sign-"))
        ))
    })?;
    let frame = driving::frame_of(session, &trace, ui_rect, name)?;
    driver.click_at(frame.declared_center(rect))?;
    session.settle(18);
    Ok(())
}

/// **Scroll the Sign window until `name` is wholly inside it, then click it.**
pub(super) fn click_scrolled(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    name: &str,
    report: &mut CheckReport,
) -> Result<()> {
    /// How far this check is willing to say it looked. The Sign form is six
    /// sections; six notches reaches the bottom of it from the top with room
    /// to spare, and a larger number would turn *"the control is not there"*
    /// into a slower way of saying the same thing.
    const NOTCHES: usize = 8;

    for attempt in 0..=NOTCHES {
        let trace = session.trace()?;
        let rect = declared(&trace, ui_rect, name).ok_or_else(|| {
            Error::new(format!(
                "no `{name}` region to click. Regions declared under `sign-`: {}.",
                list(&declared_names(&trace, ui_rect, "sign-"))
            ))
        })?;
        let window = declared(&trace, ui_rect, REGION_BODY).ok_or_else(|| {
            Error::new(format!(
                "the Sign window declared no `{REGION_BODY}` region, so there is nothing to \
                 measure `{name}` against."
            ))
        })?;
        let frame = driving::frame_of(session, &trace, ui_rect, name)?;
        if window.contains_rect(rect) {
            if attempt > 0 {
                report.note(format!(
                    "`{name}` was below the Sign window's fold; {attempt} scroll notch(es) \
                     brought it into view"
                ));
            }
            driver.click_at(frame.declared_center(rect))?;
            session.settle(18);
            return Ok(());
        }
        driver.scroll_at(frame.declared_center(window), -1)?;
        session.settle(12);
    }
    Err(Error::new(format!(
        "`{name}` is declared and never came wholly inside `{REGION_BODY}` after {NOTCHES} \
         scroll notch(es), so a click at its centre would be clipped out of the scroll area. ⚠ A control in the window's FOOTER (the confirm and cancel row) is never inside the body and must be clicked with `click`, not this. Otherwise: either the \
         wheel is landing somewhere that does not scroll, or the form is longer than the window \
         can ever show."
    )))
}

/// **A field name read out of `signature-row`, normalised.**
pub(super) fn field_name_of(raw: &str) -> String {
    raw.trim()
        .strip_prefix("Some(")
        .and_then(|s| s.strip_suffix(')'))
        .unwrap_or(raw)
        .trim_matches('"')
        .to_owned()
}

/// The last `sign-opened` line's `refusal=` token.
///
pub(super) fn last_refusal(session: &Session) -> Result<Option<String>> {
    Ok(session
        .trace()?
        .events(OPENED_EVENT)
        .last()
        .and_then(|l| l.get("refusal").map(str::to_owned)))
}

/// Resolve a fixture from this repository, via
/// [`crate::checks::driving::repo_fixture`].
pub(super) fn repo_fixture(name: &str) -> Result<PathBuf> {
    crate::checks::driving::repo_fixture(
        name,
        "The signing checks pin the documents they open, because a signature is a property of \
         particular bytes and no suite-wide fixture has one.",
    )
}

/// Resolve something from the engine repository's synthetic corpus.
pub(super) fn engine_fixture(rel: &str, what: &str) -> Result<PathBuf> {
    let path = Path::new("D:/Dev/pdfcer/fixtures/synthetic").join(rel);
    if !path.is_file() {
        return Err(Error::new(format!(
            "{what} is missing at {}. It lives in `pdfcer-core`'s own synthetic corpus, which this check READS — see this module's header for why nothing like it is committed into this repository.",
            path.display()
        )));
    }
    Ok(path)
}

/// **Bring the Signatures panel to the front, mounting it if it is not there.**
pub(super) fn raise_signatures(session: &Session, driver: &Driver, ui_rect: &str) -> Result<()> {
    if !super::super::reaching::raise_dock_tab(session, driver, ui_rect, "view.panel_signatures")? {
        click_tab_tolerant(session, driver, ui_rect, "ribbon.tab.view")?;
        press(session, driver, ui_rect, "view.panel_signatures")?;
        session.settle(24);
        let _ = super::super::reaching::raise_dock_tab(
            session,
            driver,
            ui_rect,
            "view.panel_signatures",
        )?;
    }
    session.settle(30);
    Ok(())
}

/// **Whether the text the signature box will show was put in front of the
/// operator**, read after a signing.
pub(super) fn disclosed_appearance(
    trace: &Trace,
    phase: &str,
    report: &mut CheckReport,
    findings: &mut Vec<String>,
) {
    let Some(line) = trace.events(DISCLOSED_EVENT).last() else {
        findings.push(format!(
            "{phase}: no `{DISCLOSED_EVENT}` line after a signing. Either the build predates the \
             appearance disclosure or the sentence was never composed."
        ));
        return;
    };
    let composed = line.get("appearance_lines").unwrap_or_default().to_owned();
    let shown = line.get("appearance_shown").unwrap_or_default().to_owned();
    report.note(format!(
        "{phase}: {DISCLOSED_EVENT} appearance_lines={composed} appearance_shown={shown}"
    ));
    if composed != "0" && shown != composed {
        findings.push(format!(
            "{phase}: the engine composed {composed} line(s) of appearance text into the \
             signature box and the report the operator reads carries {shown} of them. That text \
             is on the page, it is not in the document they still have open, and rule 4 owes them \
             every line of it."
        ));
    }
}
