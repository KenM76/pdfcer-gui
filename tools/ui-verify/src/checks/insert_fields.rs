//! `inserted_form_pages_bring_their_fields` — **a form's pages inserted into
//! a document with no form arrive as fillable fields, and the receipt says
//! so.**
//!
//! Drives the window off the desktop through the scripted pointer's `drop`
//! step, in Edit mode on a copy of the engine corpus's `four-pages.pdf`,
//! dropping a copy of `fixtures/all-field-kinds.pdf` (two pages, every field
//! kind, no shared names) and choosing Insert.
//!
//! Oracle: the funnel's `insert-pages` line. Its disclosures must say the
//! fields came across and that the document is now a fillable form
//! (`InsertOutcome::fields_carried`, `acroform_created`), and must NOT say any
//! control needs re-registering: whole fields carried leave no orphans.

use super::dropped_pdf::{INSERT, ask, press};
use super::os_image_paste as osp;
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;
use std::path::{Path, PathBuf};

const STEM: &str = "insert-fields";
const FIXTURE: &str = "fixtures/all-field-kinds.pdf";
const INSERTED: &str = "insert-pages";
/// Said by `text::insertfields::carried` for a carry of two or more.
const CARRIED: &str = "form fields came across with the pages";
/// Said by the same sentence when the carry created `/AcroForm`.
const FORM: &str = "now a fillable form";
/// Said by `text::pages::inserted` for an orphaned control.
const ORPHANED: &str = "re-registering";

/// See the module documentation.
pub struct InsertedFormPagesBringTheirFields;

impl Check for InsertedFormPagesBringTheirFields {
    fn name(&self) -> &'static str {
        "inserted_form_pages_bring_their_fields"
    }

    fn defect(&self) -> &'static str {
        "a form's pages inserted into another document arrive as boxes that are no longer \
         fields, or arrive as fields with nothing telling the operator so"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = fixture(ctx).and_then(|file| {
            osp::launch(ctx, &mut report, STEM)
                .and_then(|(session, pointer)| drive(ctx, &mut report, &session, &pointer, &file))
        });
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// A copy of the form, so the drop never names a file in the repository.
fn fixture(ctx: &CheckContext) -> Result<PathBuf> {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join(FIXTURE);
    let path = ctx.out("insert-fields-form.pdf");
    std::fs::copy(&source, &path).map_err(|e| {
        Error::new(format!(
            "cannot copy {} to {}: {e}",
            source.display(),
            path.display()
        ))
    })?;
    Ok(path)
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    file: &Path,
) -> Result<Option<String>> {
    if let Some(failure) = ask(ctx, session, pointer, file)? {
        return Ok(Some(failure));
    }
    let before = osp::count(session, INSERTED)?;
    press(session, pointer, INSERT)?;
    let trace = session.trace()?;
    let path = session.trace_path().display().to_string();
    let Some(line) = trace.events(INSERTED).nth(before) else {
        return Ok(Some(format!(
            "Insert made no `{INSERTED}` line. Trace: {path}."
        )));
    };
    report.note(format!("receipt: {}", line.raw));
    let failure = if !line.raw.contains(CARRIED) {
        Some(format!(
            "the insert's receipt does not say the form's fields came across (`{CARRIED}`). \
             Line: `{}`.",
            line.raw
        ))
    } else if !line.raw.contains(FORM) {
        Some(format!(
            "the document had no form and the receipt does not say it now is one (`{FORM}`). \
             Line: `{}`.",
            line.raw
        ))
    } else if line.raw.contains(ORPHANED) {
        Some(format!(
            "whole fields were carried and the receipt still asks for controls to be \
             re-registered. Line: `{}`.",
            line.raw
        ))
    } else {
        None
    };
    let parked = pointer.gone(session);
    match failure {
        Some(failure) => Ok(Some(failure)),
        None => parked.map(|_| None),
    }
}
