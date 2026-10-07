//! `security_notes_say_old_passwords_are_kept` — a form whose earlier saved
//! version stored a password field's value says so in Document properties ▸
//! Security notes, naming the field and the version and never the value.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/security_passwords.md`.

use crate::checks::driving::declared;
use crate::checks::security_notes::{await_line, launch_on};
use crate::checks::{Check, CheckContext};
use crate::error::Result;
use crate::launch::Session;
use crate::report::CheckReport;

const FIXTURE: &str = "password-history.pdf";
const METHOD: &str = "Rebuild it with `python fixtures/password-history.PROVENANCE.py`.";
const EVENT: &str = "security-passwords"; // ui-text-exempt: a trace event name, never displayed
const UI_RECT: &str = "ui-rect";
const DOCK: &str = "dock.right.frame";
/// `panels::docprops::security::PASSWORDS_REGION`.
const REGION: &str = "docprops.security-notes.passwords"; // ui-text-exempt: a trace region name, never displayed

/// See the module documentation.
pub struct SecurityNotesSayOldPasswordsAreKept;

impl Check for SecurityNotesSayOldPasswordsAreKept {
    fn name(&self) -> &'static str {
        "security_notes_say_old_passwords_are_kept"
    }

    fn defect(&self) -> &'static str {
        "a form still holding a password typed into an earlier saved version opens with nothing \
         said, so the operator sends it on believing the field is empty"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let outcome = launch_on(ctx, &mut report, FIXTURE, METHOD, "security_passwords")
            .and_then(|session| steps(&mut report, &session));
        match outcome {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn steps(report: &mut CheckReport, session: &Session) -> Result<Option<String>> {
    let Some(line) = await_line(session, EVENT)? else {
        return Ok(Some(format!(
            "Document properties was opened on {FIXTURE} and no `{EVENT}` line followed: the \
             file's saved versions were never checked."
        )));
    };
    report.note(format!("scan: `{}`", line.raw));
    // Version 1 of 2 stores the value; version 2, the one a reader opens,
    // does not.
    let owed = [("revisions", "2"), ("earlier", "1"), ("current", "0")];
    if owed.iter().any(|(k, v)| line.get(k) != Some(v)) {
        return Ok(Some(format!(
            "{FIXTURE} stores a password value in version 1 of 2 and none in version 2; the \
             scan said `{}`.",
            line.raw
        )));
    }
    let trace = session.trace()?;
    let Some(drawn) = declared(&trace, UI_RECT, REGION) else {
        return Ok(Some(format!(
            "the value was found and no visible `{REGION}` region was published: the operator is \
             never told."
        )));
    };
    let Some(dock) = declared(&trace, UI_RECT, DOCK) else {
        return Ok(Some(format!(
            "no `{DOCK}` region to measure the lines against."
        )));
    };
    report.note(format!(
        "lines right edge {:.1}, dock right edge {:.1}",
        drawn.max.x, dock.max.x
    ));
    if drawn.max.x > dock.max.x + 0.5 {
        return Ok(Some(format!(
            "the stored-password lines run past the dock's right edge ({:.1} > {:.1}), so their \
             end is cut off.",
            drawn.max.x, dock.max.x
        )));
    }
    Ok(None)
}
