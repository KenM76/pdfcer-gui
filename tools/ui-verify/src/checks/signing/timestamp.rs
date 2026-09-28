//! `checks::signing::timestamp` — **a signature asked to carry a timestamp
//! either carries one or is not written**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/signing/timestamp.md`.

use super::reaching::{click, click_scrolled, click_tab, drawn, engine_fixture, launch, press};
use super::{CERT, FILE_TAB, MODE, PASSPHRASE, REGION_CONFIRM, REGION_DIALOG, SIGN};
use super::{REGION_CHOOSE, REGION_OPEN_CERT, REGION_PASSPHRASE};
use crate::checks::driving::{self, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::report::CheckReport;
use crate::sys::vk;

/// The server field, declared only by a build that can ask a server.
const REGION_TIMESTAMP: &str = "sign-timestamp";

/// A server that refuses at once: the discard port on this machine, which
/// nothing listens on. Offline and deterministic — the check never reaches
/// out of the machine unless [`LIVE_TSA_ENV`] says so.
const DEAD_TSA: &str = "http://127.0.0.1:9/";

/// Opt-in: a real RFC 3161 server to sign against as well. Network contact
/// is explicit, so the live half runs only when this is set.
const LIVE_TSA_ENV: &str = "UI_VERIFY_TSA";

/// See the module documentation.
pub struct ATimestampedSignatureIsTimestampedOrNotWritten;

impl Check for ATimestampedSignatureIsTimestampedOrNotWritten {
    fn name(&self) -> &'static str {
        "a_timestamped_signature_is_timestamped_or_not_written"
    }

    fn defect(&self) -> &'static str {
        "The Sign window offers no timestamp server, or a server that does not answer produces a \
         signed file anyway — an untimestamped signature the operator asked to be timestamped"
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
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input); reported as SKIPPED rather than passed.",
        ));
    }
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let plain = super::reaching::repo_fixture("four-pages.pdf")?;
    let certificate = engine_fixture(CERT, "the signing certificate")?;

    let mut findings = Vec::new();
    // The dead server first: the refusal. Then, if asked, a live one.
    let mut runs = vec![("dead", DEAD_TSA.to_owned())];
    if let Ok(live) = std::env::var(LIVE_TSA_ENV)
        && !live.trim().is_empty()
    {
        runs.push(("live", live.trim().to_owned()));
    } else {
        report.note(format!(
            "{LIVE_TSA_ENV} not set: only the refusal half ran; no server off this machine was \
             contacted"
        ));
    }

    for (label, server) in runs {
        let out = ctx.out(&format!("signed-tsa-{label}.pdf"));
        let _ = std::fs::remove_file(&out);
        let session = launch(
            ctx,
            report,
            &plain,
            &format!("sign-tsa-{label}.trace.txt"),
            &[
                ("PDFCER_DIAG_CERTIFICATE_PATH", certificate.clone()),
                ("PDFCER_DIAG_SAVE_PATH", out.clone()),
            ],
        )?;
        let driver = Driver::new(session.window());
        driving::click_mode_segment(&session, &driver, ui_rect, MODE)?;
        session.settle(16);
        click_tab(&session, &driver, ui_rect, FILE_TAB)?;
        press(&session, &driver, ui_rect, SIGN)?;
        if !drawn(&session.trace()?, ui_rect, REGION_DIALOG) {
            return Err(Error::new(format!("{label}: `{SIGN}` opened no window.")));
        }
        click(&session, &driver, ui_rect, REGION_CHOOSE)?;
        click(&session, &driver, ui_rect, REGION_PASSPHRASE)?;
        driver.type_ascii(PASSPHRASE)?;
        session.settle(10);
        click(&session, &driver, ui_rect, REGION_OPEN_CERT)?;

        // Undeclared means the build excludes the capability: a skip with
        // that reason, never a pass. Declared but unreachable is a failure.
        if !drawn(&session.trace()?, ui_rect, REGION_TIMESTAMP) {
            return Err(Error::new(format!(
                "this build excludes the `timestamp` feature (no `{REGION_TIMESTAMP}` region \
                 declared); build with `--features timestamp` to run this check."
            )));
        }
        // Below the fold on a short window: scrolled into the body, never
        // clicked where it was declared, which may be clipped.
        click_scrolled(&session, &driver, ui_rect, REGION_TIMESTAMP, report).map_err(|e| {
            Error::new(format!(
                "{label}: no reachable `{REGION_TIMESTAMP}` field in a build with the \
                 `timestamp` feature. {e} Regions under `sign-`: {}.",
                list(&declared_names(
                    &session.trace().unwrap_or_default(),
                    ui_rect,
                    "sign-"
                ))
            ))
        })?;
        // Whatever the remembered preference seeded, replace it.
        driver.press_chord(&[vk::CONTROL], vk::A)?;
        driver.type_text(&server)?;
        session.settle(10);
        let shot = ctx.out(&format!("sign-tsa-{label}-typed.png"));
        if crate::capture::window_to_png(&session, &shot).is_ok() {
            report.artifact(shot);
        }
        click(&session, &driver, ui_rect, REGION_CONFIRM)?;
        // The live server's round trip is bounded at 30 s by the shell.
        session.settle(if label == "live" { 400 } else { 90 });

        let trace = session.trace()?;
        let requested = trace
            .events("sign-requested")
            .last()
            .and_then(|l| l.get("tsa").map(str::to_owned));
        let applied = trace
            .events("sign-applied")
            .last()
            .and_then(|l| l.get("written").map(str::to_owned));
        let stamped = trace
            .events("sign-prepared")
            .last()
            .and_then(|l| l.get("timestamped").map(str::to_owned));
        report.note(format!(
            "{label}: tsa={requested:?} written={applied:?} timestamped={stamped:?} file={}",
            out.is_file()
        ));
        if requested.as_deref() != Some("1") {
            findings.push(format!(
                "{label}: `sign-requested tsa=` was {requested:?}; the typed server did not reach \
                 the request."
            ));
        }
        match label {
            "dead" => {
                if applied.as_deref() != Some("0") || out.is_file() || stamped.is_some() {
                    findings.push(format!(
                        "dead: a server that refused the connection still produced a signature \
                         (written={applied:?}, file={}, timestamped={stamped:?}). A requested \
                         timestamp must never be dropped silently.",
                        out.is_file()
                    ));
                }
            }
            _ => {
                if applied.as_deref() != Some("1") || stamped.as_deref() != Some("1") {
                    findings.push(format!(
                        "live: signing against {server} gave written={applied:?} \
                         timestamped={stamped:?}; expected both 1."
                    ));
                }
            }
        }
        drop(session);
    }
    Ok((!findings.is_empty()).then(|| findings.join("\n")))
}
