//! `checks::signing::reopened` — a document saved with one box hand-signed
//! opens with that box counted signed and untagged, and an emptied signature
//! (a tag that paints nothing) left unsigned.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/signing/reopened.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, declared_names, list, repo_fixture};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Off every monitor, unfocused: the check never moves the pointer.
const OFFSCREEN: &str = "-4200,-4200,1400,900";
const FIXTURE: &str = "esign-one-signed.pdf";
const METHOD: &str = "Run fixtures/esign-one-signed.PROVENANCE.py.";
const BOX_PREFIX: &str = "form.sign-box.";
/// The tags wanted: every box but `ApplicantSig`, the first, which is signed.
const WANT_TAGS: [&str; 2] = ["form.sign-box.1", "form.sign-box.2"];

/// See the module documentation.
pub struct ASignedBoxStaysSignedAfterReopening;

impl Check for ASignedBoxStaysSignedAfterReopening {
    fn name(&self) -> &'static str {
        "a_signed_box_stays_signed_after_reopening"
    }

    fn defect(&self) -> &'static str {
        "a box hand-signed in an earlier session opens unsigned (tagged, counted to sign), or a \
         box whose signature was deleted opens signed"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = launch(ctx, &mut report).and_then(|(session, pointer)| {
            let outcome = verdict(ctx, &mut report, &session);
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
    let doc = ctx.out("reopened-source.pdf");
    std::fs::copy(repo_fixture(FIXTURE, METHOD)?, &doc)
        .map_err(|e| Error::new(format!("copying {FIXTURE}: {e}")))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("reopened.trace.txt"));
    spec.pdf = Some(doc);
    for (k, v) in [
        (ctx.profile.diag_env.0, ctx.profile.diag_env.1),
        SHELL_DIAG_ENV,
        (viewport_env, OFFSCREEN),
    ] {
        spec.env.push((k.to_owned(), v.to_owned()));
    }
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("reopened.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    report.note(format!("launched as pid {}", session.pid()));
    session.settle(45);
    Ok((session, pointer))
}

fn verdict(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
) -> Result<Option<String>> {
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let trace = session.trace()?;
    let read = trace
        .events("hand-signed-read")
        .last()
        .map(|l| l.raw.clone());
    let strip = trace
        .events("sign-strip")
        .last()
        .map(|l| (l.get_usize("signed"), l.get_usize("total")));
    let mut tags = declared_names(&trace, ui_rect, BOX_PREFIX);
    tags.sort();
    report.note(format!(
        "read: {}; strip (signed, total)={strip:?}; tags: {}",
        read.as_deref().unwrap_or("none"),
        list(&tags)
    ));
    let mut findings = Vec::new();
    if strip != Some((Some(1), Some(3))) {
        findings.push(format!(
            "★★★ the signing strip counted (signed, total)={strip:?}; (1, 3) was expected: the \
             signature saved into ApplicantSig was not read back from the document."
        ));
    }
    if tags != WANT_TAGS {
        findings.push(format!(
            "★★★ the boxes tagged to sign are [{}]; [{}] was expected: ApplicantSig is signed \
             and takes no tag, and CoApplicantSig's tag paints nothing, so it is unsigned.",
            list(&tags),
            WANT_TAGS.join(", ")
        ));
    }
    Ok((!findings.is_empty()).then(|| findings.join(" ")))
}
