//! `a_document_that_phones_home_says_so` — the driven proof of
//! `OPERATOR_REQUESTS.md` **O61**.
//!
//! # What this is about
//!
//! **Ken, 2026-08-30:** *"I think pdfcer added support for several button
//! features and protections for outgoing submits."*
//!
//! Half right. pdfcer cannot yet **author** a button action — still planned, and
//! a policy decision rather than a missing verb. What it did ship is the
//! **detection**, and this shell was not asking for it.
//!
//! `Pass 133.0`, the engine's own words about why that gap was urgent rather
//! than merely incomplete:
//!
//! > a push button that submits a form to a web server reported
//! > `js_network_actions=0` … three surfaces, none disclosing it. **A check
//! > that under-reports reads as a clean bill of health**, because silence and
//! > safety are indistinguishable to the reader.
//!
//! ⇒ They fixed their scanner. This shell never called it, so the whole finding
//! stopped at the crate boundary: pdfcer could tell an operator that the drawing
//! somebody just sent them will post data to a web server, and nothing on
//! screen said so.
//!
//! # ★★★ Why this needs its own fixture, and why that is the finding
//!
//! **Nothing in either corpus has a submit action.** Checked on 2026-08-30:
//! every PDF under the engine's `fixtures/synthetic` reports
//! `js_network_actions=0`, and this shell's own fixtures are CAD drawings.
//!
//! That absence is worth stating rather than routing around: **the one document
//! shape this disclosure exists for is the one nobody had a copy of.** A check
//! written against a document with no submit action would have asserted that a
//! silent program stayed silent — a green result reporting nothing, which is
//! precisely what this harness exists to remove.
//!
//! So `tools/gen-submit-fixture.py` builds one: 622 bytes, one push button, one
//! `/SubmitForm` pointing at `example.invalid` — a host **RFC 2606 §2
//! guarantees will never resolve**, because a fixture that exists to be opened
//! by a test must not be able to contact anything even if every guarantee above
//! it fails.
//!
//! # The oracle, and the negative half that matters more
//!
//! | assertion | what it catches |
//! |---|---|
//! | the status row carries the disclosure | the shell asked and said so |
//! | the sentence names **submitting**, not just "actions" | a disclosure too vague to act on |
//! | it says pdfcer **does not** do it | an alarm about something that cannot happen here |
//! | ★ a **clean** document says **nothing** | the failure that costs every future disclosure |
//!
//! ★★ The last one is a second launch, on an ordinary drawing, and it is the
//! assertion this check would be worthless without. A build that warned on
//! every document would pass every positive assertion above and train the
//! operator to ignore the status row — after which the sentence that matters is
//! one they have learned not to read.

use crate::checks::driving::SHELL_DIAG_ENV;
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The generated fixture: one push button carrying a `/SubmitForm`.
const FIXTURE: &str = "fixtures/submit-button.pdf";

/// The shell's own line: what the scan found.
const SCAN: &str = "reach-out";

/// The line carrying the operator-facing sentence.
///
/// ★ `record_note` puts prose on the status bar and traces nothing, so the
/// shell emits this beside it. Without it a check could prove the scan ran and
/// could not prove the operator was TOLD — which is the whole subject.
const NOTE: &str = "reach-out-disclosed";

/// See the module documentation.
pub struct ADocumentThatPhonesHomeSaysSo;

impl Check for ADocumentThatPhonesHomeSaysSo {
    fn name(&self) -> &'static str {
        "a_document_that_phones_home_says_so"
    }

    fn defect(&self) -> &'static str {
        "a drawing whose push button posts data to a web server opens with nothing said — the \
         engine detects it and the shell never asks, so silence and safety are indistinguishable \
         to the operator who is about to hand the file on"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(skip) => report.skip(skip.to_string()),
        }
    }
}

/// Open one document and return what the shell traced about it.
fn open(
    ctx: &CheckContext,
    exe: &std::path::Path,
    pdf: &std::path::Path,
    label: &str,
) -> Result<(Session, crate::trace::Trace)> {
    let mut spec = LaunchSpec::new(exe, ctx.out(&format!("reach-out-{label}.trace.txt")));
    spec.pdf = Some(pdf.to_path_buf());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    session.settle(40);
    let trace = session.trace()?;
    Ok((session, trace))
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let fixture = std::path::Path::new(FIXTURE);
    if !fixture.is_file() {
        return Err(Error::new(format!(
            "{FIXTURE} is missing. Generate it with `python tools/gen-submit-fixture.py` — this \
             check pins it because NOTHING in either corpus carries a submit action, which is \
             itself why the disclosure went unwritten for so long."
        )));
    }
    let clean = ctx.pdf.clone().ok_or_else(|| {
        Error::new(
            "no --pdf. This check needs an ORDINARY drawing as well as its own fixture: the \
             assertion that matters most is that a clean document says nothing.",
        )
    })?;

    // --- the document that phones home --------------------------------------
    let (session, trace) = open(ctx, &exe, fixture, "submit")?;
    report.artifact(session.trace_path().to_path_buf());

    let Some(scan) = trace.last(SCAN) else {
        return Ok(Some(format!(
            "opening {FIXTURE} produced no `{SCAN}` line, so the shell never asked the engine \
             what this document reaches for. `app::reachout::scan` runs once in `lifecycle::adopt`; \
             either it is not called or the open failed. Trace: {}",
            session.trace_path().display()
        )));
    };
    let network = scan.get("network").unwrap_or("?");
    report.note(format!("the scan reports network={network}"));
    if network != "1" {
        return Ok(Some(format!(
            "the scan reports `network={network}` on a fixture whose single push button carries a \
             `/SubmitForm`. `pdfcer list-fields` on the same file reports \
             `js_network_actions=1`, so the engine sees it and this shell is reading the wrong \
             field or scanning the wrong document. Trace: {}",
            session.trace_path().display()
        )));
    }

    let said = trace
        .events(NOTE)
        .any(|l| l.raw.contains("send data somewhere"));
    if !said {
        return Ok(Some(format!(
            "★★★ THE SCAN FOUND A SUBMIT ACTION AND NOTHING WAS SAID. `{SCAN} network=1` is in \
             the trace and no `{NOTE}` line names it. The operator opens a drawing that will post \
             data to a web server when a button is pressed, and the program is silent — which is \
             indistinguishable, to them, from a document that does nothing of the kind. Trace: {}",
            session.trace_path().display()
        )));
    }
    // ★ The tone, asserted rather than trusted: the sentence must say pdfcer
    // does NOT do this. Without that clause it is an alarm about something that
    // cannot happen in this program, and an operator who learns pdfcer cries
    // wolf stops reading the status row entirely.
    let reassures = trace
        .events(NOTE)
        .any(|l| l.raw.contains("pdfcer never does any of that"));
    if !reassures {
        return Ok(Some(format!(
            "the disclosure names the submit action and does not say pdfcer never performs one. \
             That makes it an alarm about something that cannot happen here — pdfcer recognises \
             actions and never executes them — and the cost is every future disclosure, not this \
             one. Trace: {}",
            session.trace_path().display()
        )));
    }
    report.note("★★ the disclosure names the submit AND says pdfcer never does it");

    // --- ★★ and an ordinary drawing says nothing ----------------------------
    let (clean_session, clean_trace) = open(ctx, &exe, &clean, "clean")?;
    report.artifact(clean_session.trace_path().to_path_buf());
    if clean_trace.last(SCAN).is_none() {
        return Err(Error::new(
            "the ordinary document produced no scan line at all, so the silence below cannot be \
             told from a scan that never ran. SKIPPED rather than passed.",
        ));
    }
    let noisy = clean_trace
        .events(NOTE)
        .any(|l| l.raw.contains("This document can"));
    if noisy {
        return Ok(Some(format!(
            "★★★ AN ORDINARY DRAWING WAS WARNED ABOUT. {} carries no submit, launch or \
             open-script action and the shell disclosed anyway. A disclosure that fires on every \
             document is one the operator learns to ignore — after which the sentence that \
             matters is one they have trained themselves not to read. That costs more than the \
             feature is worth. Trace: {}",
            clean.display(),
            clean_session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★ and {} said nothing — the disclosure is silent on an ordinary drawing",
        clean.display()
    ));

    Ok(None)
}
