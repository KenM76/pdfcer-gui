//! `signature_names_where_revocation_lives` — **the Signatures panel lists the
//! revocation URLs a signer's certificate names**, one CRL, one OCSP responder
//! and one issuer location on `fixtures/signed-revocation-urls.pdf`.
//!
//! The control is `signed-two-pages.pdf`, whose certificates name none: a panel
//! that invented URLs, or counted every certificate, fails there.

use crate::checks::driving::{SHELL_DIAG_ENV, click_mode_segment, repo_fixture};
use crate::checks::trust_store::show_signatures;
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const EVENT: &str = "signature-revocation-sources";
/// What the named fixture's one certificate carries.
const WITH_URLS: (&str, &str) = (
    "signed-revocation-urls.pdf",
    "certs=1 crl=1 ocsp=1 issuers=1 unreadable=0",
);
/// What a certificate naming no revocation location yields.
const WITHOUT_URLS: (&str, &str) = (
    "signed-two-pages.pdf",
    "certs=0 crl=0 ocsp=0 issuers=0 unreadable=0",
);

/// See the module documentation.
pub struct SignatureNamesWhereRevocationLives;

impl Check for SignatureNamesWhereRevocationLives {
    fn name(&self) -> &'static str {
        "signature_names_where_revocation_lives"
    }

    fn defect(&self) -> &'static str {
        "the Signatures panel does not say where a signer's certificate keeps its revocation \
         status, so an operator who wants to check it by hand has nowhere to start"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let mut outcome = Ok(None);
        for (fixture, want) in [WITH_URLS, WITHOUT_URLS] {
            outcome = drive(ctx, &mut report, fixture, want);
            if !matches!(outcome, Ok(None)) {
                break;
            }
        }
        match outcome {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    fixture: &str,
    want: &str,
) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new("no binary to drive. Pass --exe, or build the release binary.")
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input); this check clicks a ribbon control. SKIPPED.",
        ));
    }
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let pdf = repo_fixture(fixture, "This check pins a signed fixture.")?;

    let mut spec = LaunchSpec::new(&exe, ctx.out(&format!("revocation-{fixture}.trace.txt")));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);
    let driver = Driver::new(session.window());
    click_mode_segment(&session, &driver, ui_rect, "read")?;
    if !show_signatures(&session, &driver, ui_rect)? {
        return Err(Error::new(format!(
            "the Signatures panel never came to the front on {fixture}. SKIPPED, not passed."
        )));
    }
    let trace = session.trace()?;
    let Some(line) = trace.last(EVENT) else {
        return Ok(Some(format!(
            "★ NO `{EVENT}` LINE on {fixture}: the panel drew and never reached the revocation \
             lines. Trace: {}.",
            session.trace_path().display()
        )));
    };
    let got = line
        .raw
        .split_once(EVENT)
        .map_or("", |(_, rest)| rest.trim());
    if got != want {
        return Ok(Some(format!(
            "★★ {fixture}: the panel reported `{got}`, expected `{want}`. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note(format!("★ {fixture}: `{EVENT} {got}`"));
    Ok(None)
}
