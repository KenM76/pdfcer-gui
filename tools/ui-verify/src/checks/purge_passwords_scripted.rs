//! `stored_passwords_removed_without_the_mouse` — Security ▸ Remove old
//! passwords… on `fixtures/password-history.pdf`, whose earlier version stores
//! a Password field's value, writes a one-version copy that no longer holds
//! it, on a window placed off the desktop and driven through the scripted
//! pointer.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/purge_passwords_scripted.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_in, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The line a successful purge writes.
const WROTE: &str = "purge-passwords"; // ui-text-exempt: a trace event name, never displayed
const TAB: &str = "ribbon.tab.security"; // ui-text-exempt: a trace region name, never displayed
const ITEM: &str = "ribbon.item.file.purge_password_values"; // ui-text-exempt: a trace region name, never displayed
/// The Security group when the band is too narrow to show it open.
const COLLAPSED: &str = "ribbon.group.security.security.collapsed"; // ui-text-exempt: a trace region name, never displayed
const SAVE_PATH_ENV: &str = "PDFCER_DIAG_SAVE_PATH"; // ui-text-exempt: an environment variable name
/// Off the desktop, so no OS input can reach it and none of his is taken.
const OFFSCREEN: &str = "-4200,-4200,1400,900";
/// The value the fixture's first version stores.
const SECRET: &[u8] = b"s3cret";
const EOF_MARKER: &[u8] = b"%%EOF";

pub struct StoredPasswordsRemovedWithoutTheMouse;

impl Check for StoredPasswordsRemovedWithoutTheMouse {
    fn name(&self) -> &'static str {
        "stored_passwords_removed_without_the_mouse"
    }

    fn defect(&self) -> &'static str {
        "Security ▸ Remove old passwords… does not write a copy, or the copy still holds a password value an earlier version of the file stored"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let pdf = crate::fixture::workspace_root()
        .join("fixtures")
        .join("password-history.pdf");
    if !pdf.is_file() {
        return Ok(Some(format!(
            "the password-history fixture is not at {}; it is committed, so this is a broken checkout.",
            pdf.display()
        )));
    }
    let target = ctx.out("no-passwords.pdf");
    let _ = std::fs::remove_file(&target);
    if target.exists() {
        return Err(Error::new(format!(
            "cannot clear {} before the export.",
            target.display()
        )));
    }

    let mut spec = LaunchSpec::new(&exe, ctx.out("purge-passwords-scripted.trace.txt"));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push((viewport_env.to_owned(), OFFSCREEN.to_owned()));
    spec.env
        .push((SAVE_PATH_ENV.to_owned(), target.display().to_string()));
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer =
        ScriptedPointer::attach(&mut spec, ctx.out("purge-passwords-scripted.pointer.txt"))?;

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);

    let fresh = |region: &str| -> Result<Option<crate::geom::LRect>> {
        Ok(declared(&session.trace()?, ui_rect, region))
    };
    let click = |region: &str| -> Result<()> {
        let trace = session.trace()?;
        let (rect, viewport) = declared_in(&trace, ui_rect, region).ok_or_else(|| {
            let prefix = region.rsplit_once('.').map_or(region, |(head, _)| head);
            Error::new(format!(
                "no `{region}` region. Declared under `{prefix}`: {}.",
                list(&declared_names(&trace, ui_rect, prefix))
            ))
        })?;
        pointer.click_in(&session, viewport.as_deref(), WindowPoint::centre_of(rect))?;
        session.settle(15);
        Ok(())
    };

    click(TAB)?;
    if fresh(ITEM)?.is_none() && fresh(COLLAPSED)?.is_some() {
        click(COLLAPSED)?;
    }
    click(ITEM)?;
    session.settle(20);
    pointer.gone(&session)?;

    let trace = session.trace()?;
    let Some(line) = trace.last(WROTE) else {
        return Ok(Some(format!(
            "the item was clicked and no `{WROTE}` line followed. None found: {}. Refused: {}. Failed: {}. Trace: {}.",
            trace
                .last("purge-passwords-none")
                .map_or("none".to_owned(), |l| l.raw.clone()),
            trace
                .last("purge-passwords-refused")
                .map_or("none".to_owned(), |l| l.raw.clone()),
            trace
                .last("purge-passwords-failed")
                .map_or("none".to_owned(), |l| l.raw.clone()),
            session.trace_path().display()
        )));
    };
    // One value, only in the superseded version, so the current version has
    // no field to clear; the copy has one version.
    if line.get("fields") != Some("0")
        || line.get("earlier") != Some("1")
        || line.get("latest") != Some("0")
        || line.get("after_revisions") != Some("1")
    {
        return Ok(Some(format!(
            "the fixture's first version stores one password value and its second none; the purge traced `{}`.",
            line.raw
        )));
    }
    let bytes = std::fs::read(&target).map_err(|e| {
        Error::new(format!(
            "the purge traced success and {} cannot be read: {e}",
            target.display()
        ))
    })?;
    if bytes.windows(SECRET.len()).any(|w| w == SECRET) {
        return Ok(Some(format!(
            "{} still contains the stored value; an appended save keeps every earlier version.",
            target.display()
        )));
    }
    let versions = bytes
        .windows(EOF_MARKER.len())
        .filter(|w| *w == EOF_MARKER)
        .count();
    if versions != 1 {
        return Ok(Some(format!(
            "{} has {versions} `%%EOF` markers; a single-version rewrite has one.",
            target.display()
        )));
    }
    report.artifact(target);
    report.note(format!("`{}`", line.raw));
    Ok(None)
}
