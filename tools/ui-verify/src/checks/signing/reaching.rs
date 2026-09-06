//! `checks::signing::reaching` — **how the signing check reaches the controls
//! it presses**
//!
//! Split out of [`super`] under **R2** on 2026-09-06, when phases E to H took
//! `checks/signing.rs` to 1,523 lines. The seam is between **reaching** and
//! **asserting**, and it is a real one rather than a cut at a line number:
//! nothing in this file knows what the Sign window is for, and nothing in
//! [`super`] knows how a wheel event finds a scrolled radio button.
//!
//! ## ★★★ THE THREE FINDINGS THAT LIVE HERE, because they are about the
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

use std::path::{Path, PathBuf};

use super::super::CheckContext;
use super::super::driving::{
    self, INVOKE_EVENT, ITEM_PREFIX, SHELL_DIAG_ENV, TAB_EVENT, declared, declared_names,
    declared_or_in_overflow, list, shell_trace,
};
use super::{OPENED_EVENT, REGION_BODY};
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
///
/// [`click_tab`] asserts a new `ribbon-tab-activated` line, which is the right
/// test when a check is switching away from a tab it knows is active. It is the
/// wrong test for *"make sure this tab is on top"*: a tab that is already active
/// emits nothing when clicked, and the strict form then reports a perfectly good
/// click as a failure.
///
/// ★ A missing tab is still an error. This tolerates *no change*, never *no tab*.
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
///
/// ★★★ Through [`declared_or_in_overflow`] rather than a bare rect lookup, and
/// for `checks::protect`'s reason word for word: at the harness's window width
/// the File band runs out of room, and `file.sign` is the **third** control in a
/// Security group that was already the last group added to a full band. A plain
/// `declared` would report *"the application declared no
/// `ribbon.item.file.sign` region"* — which would be true, and would be
/// reported as a missing feature when what is missing is a scroll.
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
/// ★ A degenerate rect counts as **absent**, not present. A region declared at
/// zero area is not something an operator can see.
pub(super) fn drawn(trace: &Trace, ui_rect: &str, name: &str) -> bool {
    declared(trace, ui_rect, name).is_some_and(|r| r.is_substantial())
}

/// Click a region's centre, refusing when it was never drawn.
///
/// ★★★ Through [`driving::frame_of`] rather than `session.frame()`, and the
/// first driven run of this check is why. **A dialog is an OS window**
/// (`ui-conventions/dialogs.md` G1), so every region inside the Sign window is
/// declared in a CHILD viewport with its own origin; `session.frame()` is the
/// application window's, and clicking `declared_center` against it aims the
/// real pointer hundreds of points away — at whatever happens to be there.
///
/// The symptom was silence: the trace showed `certificate-picked chosen=1` and
/// then nothing at all, because the press on *Open certificate* landed outside
/// the button. ⇒ **Ask what the check AIMED AT**, which is the same finding
/// this project has recorded about the rotation buttons and about
/// `panning_past_the_overscan`, arriving a third way.
///
/// ★ `frame_of` is safe on a main-window region too — an untagged one answers
/// with `session.frame()`, unchanged — so there is no reason for a call site to
/// use the other form.
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
///
/// ★★★ Written for phase E, and the run that forced it is the point. The form
/// grew by one section when `Pass 10.12`'s certification option landed, and the
/// placement radios went below the fold: the harness aimed at
/// `(943, 889)` — a point **outside the application's window entirely** — and
/// reported it, correctly, as *"there is simply nothing of the application
/// there."*
///
/// ⇒ **A region declared inside a `ScrollArea` is a position in the scrolled
/// CONTENT, not on screen.** [`click`] is right for anything above the fold and
/// silently wrong below it; a check written against today's form length is a
/// check that breaks the next time a section is added, which is exactly what
/// happened here.
///
/// ★★ The wheel goes at the window's own centre rather than at a fraction of
/// it, and that is a difference from `reaching::bring_into_body`, which aims
/// three-quarters down because a dock body carries fixed furniture above its
/// scroll area. This window's furniture is **below** — the separator and the
/// button row — so its centre is inside the scrolling region, and aiming lower
/// would risk the wheel landing on the footer.
///
/// ★ Everything goes through [`driving::frame_of`], never `session.frame()`: a
/// dialog is an OS window with its own origin, and this module's header records
/// what aiming at the wrong frame cost the first time.
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
///
/// ⚠⚠ **`panels::signatures` writes `field={:?}` over an `Option<String>`, so
/// the value arrives as the literal text `Some("SignHere")`** — and on a
/// document with no field name, as `None`. That is the *"never Debug-format a
/// value a check parses"* trap in the trace this check's whole verdict rests
/// on, and it was found by reading phase D's own note, which had been printing
/// `field=Some("Signature1")` since the day it was written without anybody
/// noticing that the quotes and the wrapper were not the field's name.
///
/// ★ It is normalised **here** rather than fixed at the emitter, and the reason
/// is ownership rather than preference: `crates/pdfcer-gui/src/panels/` belongs
/// to no track this session and a concurrent edit would lose one of them. The
/// one-line fix — a bare token, `none` for the absent case, spelled by a `const
/// fn` the way `dialogs::sign::refusal_token` is — is reported to the operator
/// instead. **This function is the workaround, not the remedy**, and it is
/// deliberately tolerant of the fixed form so that it keeps working the day the
/// emitter is corrected.
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
/// ★ A token the application spells as a `const fn`, never a `{:?}` of a domain
/// type — `dialogs::sign::refusal_token` says why at its definition, and the
/// reason is that Debug-formatting a value a check parses produced two false
/// failure reports on 2026-09-05.
pub(super) fn last_refusal(session: &Session) -> Result<Option<String>> {
    Ok(session
        .trace()?
        .events(OPENED_EVENT)
        .last()
        .and_then(|l| l.get("refusal").map(str::to_owned)))
}

/// Resolve a fixture from this repository.
pub(super) fn repo_fixture(name: &str) -> Result<PathBuf> {
    // Resolved from this crate's manifest directory at COMPILE time, not from
    // `--source-root`, for the reason `checks::protect::repo_fixture` records:
    // `--source-root` is the staleness comparison's root and defaults to
    // `crates`, so joining `fixtures` onto it produced a path that does not
    // exist and a check that SKIPPED for ever while looking healthy.
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("fixtures")
        .join(name);
    if !path.is_file() {
        return Err(Error::new(format!(
            "the fixture {} is missing.",
            path.display()
        )));
    }
    Ok(path)
}

/// Resolve something from the engine repository's synthetic corpus.
///
/// ★ The path is derived, not configured. `D:\Dev\pdfcer` is READ-ONLY to this
/// project and its corpus is the only place these shapes exist, so the check
/// reads from it and writes nowhere near it — `checks::adopt_widget`'s
/// precedent, unchanged.
///
/// A missing corpus is a hard error naming the path, not a SKIP: a SKIP reads
/// as *"this build does not have the feature"*, and this is a fact about the
/// checkout rather than about the program.
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
///
/// Lifted out of phase D on 2026-09-06 when phase F needed the identical five
/// steps, and the duplication would have been the third copy of a sequence whose
/// every line is load-bearing.
///
/// ★★★ The fallback is not optional, and the first full re-run of this check is
/// why. `raise_dock_tab` succeeded once purely because a PREVIOUS launch had
/// left the panel selected and the shell had saved that layout — so the verdict
/// was resting on inherited state. On a machine whose saved layout has it behind
/// another tab, the phase would have had no oracle and would have SKIPPED,
/// reporting nothing, in green.
///
/// ★★ The tab click is TOLERANT, unlike [`click_tab`]: the View tab may already
/// be active, in which case a correct click emits no new
/// `ribbon-tab-activated` line and the strict form reports a click that landed
/// as one that did not.
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
