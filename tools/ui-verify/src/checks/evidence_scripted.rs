//! `validation_evidence_added_without_the_mouse` — File ▸ Security ▸ Add
//! validation evidence… embeds a supplied certificate and CRL, plus the
//! signer's own certificate, in a signed document, and refuses an unsigned one
//! with its own reason. The window is off the desktop and driven through the
//! scripted pointer.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/evidence_scripted.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_in, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const APPLIED: &str = "evidence-applied"; // ui-text-exempt: a trace event name, never displayed
const REFUSED: &str = "evidence-refused"; // ui-text-exempt: a trace event name, never displayed
const TAB: &str = "ribbon.tab.file"; // ui-text-exempt: a trace region name, never displayed
const ITEM: &str = "ribbon.item.file.add_validation_evidence"; // ui-text-exempt: a trace region name, never displayed
const COLLAPSED: &str = "ribbon.group.file.security.collapsed"; // ui-text-exempt: a trace region name, never displayed
const FILES_ENV: &str = "PDFCER_DIAG_EVIDENCE_FILES"; // ui-text-exempt: an environment variable name
const OFFSCREEN: &str = "-4200,-4200,1400,900";

pub struct ValidationEvidenceAddedWithoutTheMouse;

impl Check for ValidationEvidenceAddedWithoutTheMouse {
    fn name(&self) -> &'static str {
        "validation_evidence_added_without_the_mouse"
    }

    fn defect(&self) -> &'static str {
        "File ▸ Security ▸ Add validation evidence… is unreachable, embeds nothing in a signed document, or does not refuse an unsigned one"
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
    let fixtures = crate::fixture::workspace_root().join("fixtures");
    let files = [
        fixtures.join("evidence-ca.cer"),
        fixtures.join("evidence-crl.crl"),
    ];
    if let Some(missing) = files.iter().find(|f| !f.is_file()) {
        return Ok(Some(format!(
            "{} is committed and absent: a broken checkout.",
            missing.display()
        )));
    }
    let joined = files
        .iter()
        .map(|f| f.display().to_string())
        .collect::<Vec<_>>()
        .join(";");

    // Signed: one supplied certificate, one CRL, and the signer's own.
    let signed = run_once(
        ctx,
        report,
        &fixtures.join("signed-revocation-urls.pdf"),
        "evidence-signed",
        &joined,
    )?;
    let Some(line) = signed.last(APPLIED) else {
        return Ok(Some(format!(
            "on a signed document the item was clicked and no `{APPLIED}` line followed. \
             Refused: {}.",
            signed
                .last(REFUSED)
                .map_or("none".to_owned(), |l| l.raw.clone())
        )));
    };
    // `certs` counts the supplied CA and the signer's certificate; `own` is
    // the signer's alone.
    if line.get("certs") != Some("2")
        || line.get("crls") != Some("1")
        || line.get("own") != Some("1")
    {
        return Ok(Some(format!(
            "expected certs=2 crls=1 own=1 (one supplied certificate, one CRL, one signer); \
             traced `{}`.",
            line.raw
        )));
    }
    report.note(format!("signed: `{}`", line.raw));

    // Unsigned: refused before the picker, for its own reason.
    let unsigned = run_once(
        ctx,
        report,
        &fixtures.join("four-pages.pdf"),
        "evidence-unsigned",
        &joined,
    )?;
    if let Some(wrong) = unsigned.last(APPLIED) {
        return Ok(Some(format!(
            "an unsigned document received evidence: `{}`.",
            wrong.raw
        )));
    }
    match unsigned.last(REFUSED) {
        Some(l) if l.get("reason") == Some("unsigned") => {
            report.note(format!("unsigned: `{}`", l.raw));
            Ok(None)
        }
        other => Ok(Some(format!(
            "an unsigned document must be refused with reason=unsigned; traced {}.",
            other.map_or("nothing".to_owned(), |l| format!("`{}`", l.raw))
        ))),
    }
}

fn run_once(
    ctx: &CheckContext,
    report: &mut CheckReport,
    pdf: &std::path::Path,
    stem: &str,
    files: &str,
) -> Result<crate::trace::Trace> {
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
    if !pdf.is_file() {
        return Err(Error::new(format!(
            "{} is committed and absent: a broken checkout.",
            pdf.display()
        )));
    }

    let mut spec = LaunchSpec::new(&exe, ctx.out(&format!("{stem}.trace.txt")));
    spec.pdf = Some(pdf.to_path_buf());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push((viewport_env.to_owned(), OFFSCREEN.to_owned()));
    spec.env.push((FILES_ENV.to_owned(), files.to_owned()));
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out(&format!("{stem}.pointer.txt")))?;

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);

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
    let trace = session.trace()?;
    if declared(&trace, ui_rect, ITEM).is_none() && declared(&trace, ui_rect, COLLAPSED).is_some() {
        click(COLLAPSED)?;
    }
    click(ITEM)?;
    session.settle(20);
    pointer.gone(&session)?;
    session.trace()
}
