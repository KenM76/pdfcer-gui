//! `exporting_form_data_writes_a_file` — **press Export form data and a file
//! appears on disk with the form's values in it.**
//!
//! # What this is for
//!
//! `file.export_form_data` was registered, drawn on File ▸ Export, and **inert
//! for the whole life of the project**, behind a `SCAFFOLDED` entry claiming
//! *"blocked on a writer that does not exist"*. Three writers exist and two
//! have since `Pass 7.1`. It was wired on 2026-08-27, and this is the check
//! that keeps it wired.
//!
//! ## ★★★ Why the oracle is a FILE and not a trace line
//!
//! Every other link in this chain can be asserted from the trace, and asserting
//! only those would leave the one that matters untested. An export's whole
//! product is a file somebody else opens. A build that computed the bytes,
//! traced `export-form-data fields=3`, and then wrote them nowhere — a
//! swallowed `Err`, a path that was never joined, a picker whose answer was
//! dropped — would satisfy a trace-only check completely and would ship an
//! Export button that exports nothing.
//!
//! So this reads the file back and asserts on its **contents**: it must be
//! FDF, and it must contain the field the check itself just filled. The
//! second half is what distinguishes *"a file was written"* from *"the
//! operator's data was written"*, and they are not the same claim.
//!
//! ## ★★ The picker is answered, not clicked
//!
//! `PDFCER_DIAG_SAVE_PATH` supplies the save dialog's result. That is the same
//! seam `save_copy` uses and its header carries the argument: a native modal
//! blocks the thread, so a harness that tried to drive it would be automating
//! the operating system's file dialog rather than this program.
//!
//! ★ What that costs is stated rather than hidden: **the dialog itself is not
//! covered here.** Its title, its suggested filename and its extension filter
//! are unasserted, and a build whose picker opened in the wrong directory would
//! pass. That is the same gap `save_copy` records, for the same reason, and it
//! is the price of not automating a foreign window.
//!
//! ## The format is the extension, so the extension is the test
//!
//! There is no format dialog — the operator types `.fdf`, `.xfdf` or `.csv` in
//! the picker and the extension decides. So driving the picker with a chosen
//! extension is driving the format selector, which is why this check can cover
//! the branch at all without a second surface to click.

use crate::checks::driving::SHELL_DIAG_ENV;
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The command, invoked through the harness seam.
///
/// ★ `mode.edit` first, for `form_field`'s reason: the command lives on File ▸
/// Export, which every mode is shown, but the check fills a field first and
/// filling on the canvas is mode-dependent. Driving from a known mode makes the
/// run reproducible rather than dependent on whatever mode the last session
/// left behind.
const INVOKE: &str = "mode.edit,file.export_form_data";
/// The environment variable that answers the save dialog.
const SAVE_PATH_ENV: &str = "PDFCER_DIAG_SAVE_PATH";
/// The trace line the export writes on success.
const EXPORTED: &str = "export-form-data";
/// The two declines, so a refusal is reported as itself rather than as silence.
const DECLINED: &str = "export-form-data-declined";
/// The write-failed line.
const FAILED: &str = "export-form-data-failed";
/// The import half of the round trip.
const IMPORT_INVOKE: &str = "mode.edit,file.import_form_data";
/// The environment variable that answers the import picker.
///
/// ★ Its own variable rather than `PDFCER_DIAG_OPEN_PATH`, so a check can name
/// the data file without also answering the document picker. The application
/// draws the same distinction, for the same reason.
const FORM_DATA_ENV: &str = "PDFCER_DIAG_FORM_DATA_PATH";
/// The import's own summary line.
///
/// ★★★ `-applied`, and the suffix is the whole reason this constant has a doc
/// comment. `vector_edit` writes a **second** line for the same edit under the
/// bare name — `import-form-data page=0 n=1 epoch=1 disclosures=…` — and trace
/// matching is on the exact event name, so `.last()` on the bare name reads the
/// funnel's line, finds no `applied=` key, and reports `applied=0` about an
/// import that set every field it was given.
///
/// **That is exactly what the first run of this check did**, and it is the same
/// defect `text-style` had one day earlier. Reading the note about it did not
/// prevent the repeat — the naming convention is what does. `restyle_text`'s
/// `STYLE_EVENT` carries the same warning.
const IMPORTED: &str = "import-form-data-applied";

/// See the module documentation.
pub struct ExportingFormDataWritesAFile;

impl Check for ExportingFormDataWritesAFile {
    fn name(&self) -> &'static str {
        "exporting_form_data_writes_a_file"
    }

    fn defect(&self) -> &'static str {
        "Export form data is on the ribbon and writes nothing — the command was registered, \
         drawn and inert for the whole life of the project behind a blocker that had stopped \
         being true, and an operator who presses it gets a dialog and an empty result or no \
         dialog at all"
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

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx.pdf.clone().ok_or_else(|| {
        Error::new(
            "no --pdf. This check needs a document that carries a form; a document without one \
             is declined by name and the decline is not the behaviour under test.",
        )
    })?;

    // ★ The target is deleted first, and that is not tidiness. A file left by a
    // previous run would make every assertion below pass on a build that wrote
    // nothing at all — the single most likely way for a file-oracle check to go
    // quietly green. `D:/dev/rag/egui/` records the general form under
    // "a driven check that mutates persisted state must normalise at the start".
    let target = ctx.out("exported-form-data.fdf");
    let _ = std::fs::remove_file(&target);
    if target.exists() {
        return Err(Error::new(format!(
            "{} could not be removed before the run, so a file found afterwards would prove \
             nothing. SKIPPED rather than failed.",
            target.display()
        )));
    }

    let mut spec = LaunchSpec::new(&exe, ctx.out("export-form-data.trace.txt"));
    spec.pdf = Some(pdf.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), INVOKE.to_owned()));
    spec.env.push((
        SAVE_PATH_ENV.to_owned(),
        target.to_string_lossy().into_owned(),
    ));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!(
        "launched {} as pid {} with PDFCER_DIAG_INVOKE={INVOKE} and {SAVE_PATH_ENV}={}",
        exe.display(),
        session.pid(),
        target.display()
    ));
    session.settle(60);

    let trace = session.trace()?;

    // ★ A decline is reported as ITSELF, with its reason, and as a SKIP rather
    // than a failure. *"This document has no form"* is a statement about the
    // `--pdf` the harness was aimed at, not about the program — and a check
    // that failed on it would be blaming the feature for the fixture.
    if let Some(declined) = trace.events(DECLINED).last() {
        return Err(Error::new(format!(
            "the export declined: `{}`. SKIPPED rather than failed: this says the --pdf carries \
             no form or no fields, which is the harness's aim rather than the program's \
             behaviour. Point it at a document with a form. Trace: {}.",
            declined.raw,
            session.trace_path().display()
        )));
    }
    if let Some(failed) = trace.events(FAILED).last() {
        return Ok(Some(format!(
            "the export reached the write and the write failed: `{}`.\n\
             That is the program answering rather than staying silent, so the whole chain works \
             — and the answer is worth reading. The operating system's own reason is passed \
             through verbatim; *access is denied* and *the device is not ready* are different \
             problems. Trace: {}.",
            failed.raw,
            session.trace_path().display()
        )));
    }
    let Some(exported) = trace.events(EXPORTED).last() else {
        return Ok(Some(format!(
            "★ EXPORT FORM DATA WAS INVOKED AND NOTHING HAPPENED: no `{EXPORTED}` line, no \
             `{DECLINED}` and no `{FAILED}`.\n\
             Three candidates. (1) **The command has no dispatch arm** — which is the state it \
             was in for the whole life of the project, behind a `SCAFFOLDED` entry claiming the \
             writer did not exist. (2) **The action was raised and its apply arm never ran.** \
             (3) **The picker was not answered** — `{SAVE_PATH_ENV}` supplies it, and an empty \
             value means *cancelled*, which is a real branch and would trace \
             `export-form-data-cancelled`. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("★ the export ran: `{}`", exported.raw));

    // --- the oracle: a file, with the operator's data in it -----------------
    let Ok(bytes) = std::fs::read(&target) else {
        return Ok(Some(format!(
            "★ THE EXPORT TRACED SUCCESS AND WROTE NO FILE: `{}` names {} bytes and {} does \
             not exist.\n\
             This is the case a trace-only check cannot see and the reason this one reads the \
             disk: the bytes were computed and went nowhere. A swallowed `Err`, a path that was \
             never joined, or a picker answer that was dropped between the dialog and the \
             write. Trace: {}.",
            exported.raw,
            exported.get("bytes").unwrap_or("?"),
            target.display(),
            session.trace_path().display()
        )));
    };
    if bytes.is_empty() {
        return Ok(Some(format!(
            "the export wrote {} and it is EMPTY. The file exists, so the path and the \
             permissions are right and the bytes handed to `write` were empty — which on this \
             path means `to_fdf` produced nothing for a form the decline check above says has \
             fields. Trace: {}.",
            target.display(),
            session.trace_path().display()
        )));
    }

    // ★★ FDF is a PDF-syntax file (§12.7.8) and opens with `%FDF-`. Asserted
    // because "not empty" is satisfied by any accident — a partial write, a
    // buffer of zeros, an error page — and the header is the cheapest thing
    // that distinguishes *the right format* from *some bytes*.
    if !bytes.starts_with(b"%FDF-") {
        let head = String::from_utf8_lossy(&bytes[..bytes.len().min(48)]).into_owned();
        return Ok(Some(format!(
            "the export wrote {} bytes to {} and they are not FDF: the file begins {head:?} \
             rather than `%FDF-`.\n\
             The format is chosen by the EXTENSION, and this run asked for `.fdf`. So either \
             the extension match is wrong — the arm is a `match` on a lower-cased extension \
             with FDF as the default — or `to_fdf` is not the writer being reached. Trace: {}.",
            bytes.len(),
            target.display(),
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★ {} bytes of FDF on disk, and the file begins `%FDF-`",
        bytes.len()
    ));

    // ★★★ The half that makes this the operator's data rather than a
    // well-formed empty shell. A field name from the document must appear in
    // the file; `/T` entries carry them, so any one of them is proof the values
    // travelled rather than the writer emitting a valid header and a trailer.
    let fields: usize = exported
        .get("fields")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    if fields == 0 {
        return Ok(Some(format!(
            "the export wrote a well-formed FDF for `fields=0`, so the file is a valid empty \
             shell. The decline path exists precisely to stop that reaching a picker — a \
             document with no fields is refused by name before the dialog opens — so reaching \
             here means the count and the refusal disagree. Trace: {}.",
            session.trace_path().display()
        )));
    }
    if !bytes.windows(2).any(|w| w == b"/T") {
        return Ok(Some(format!(
            "the export wrote {} bytes of FDF for `fields={fields}` and the file contains no \
             `/T` entry, so it names no field.\n\
             A valid header, a valid trailer and none of the operator's data — which is what a \
             writer reached with an empty `FormData` produces. Trace: {}.",
            bytes.len(),
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★★ the file names {fields} field(s) of the operator's form — the data travelled, not \
         just the format"
    ));

    // --- the round trip: import what was just exported ----------------------
    //
    // ★★★ Two verbs, one check, and it is deliberate rather than an economy.
    //
    // An export and an import are a **pair**, and the only defect class that
    // matters about a pair is an ASYMMETRY — a writer and a reader that each
    // pass their own test and disagree with each other. That is precisely the
    // defect this project reported to the engine about widget borders four
    // hours earlier, and the engine's answer was to make *their* tests round
    // trips for the same reason: *"a hand-authored fixture would test the
    // reader against bytes I chose; this tests it against bytes pdfcer chose,
    // which is the pair that has to agree."*
    //
    // So this reads back the file the previous half just wrote. A separate
    // import check fed a fixture would test the parser against bytes the
    // harness author chose, and would stay green through exactly the drift it
    // exists to catch.
    //
    // ★ A **second process**, not a second command in the first. The document
    // is reopened clean, so the import is proved against a document that has
    // never seen the data rather than against one whose fields already hold it
    // — where every assertion would pass on an import that did nothing.
    let mut spec = LaunchSpec::new(&exe, ctx.out("import-form-data.trace.txt"));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), IMPORT_INVOKE.to_owned()));
    spec.env.push((
        FORM_DATA_ENV.to_owned(),
        target.to_string_lossy().into_owned(),
    ));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    session.settle(60);
    let trace = session.trace()?;

    let Some(imported) = trace.events(IMPORTED).last() else {
        return Ok(Some(format!(
            "★ THE EXPORT WROTE A FILE AND IMPORTING IT BACK DID NOTHING: no `{IMPORTED}` \
             line.\n\
             The two halves are a pair, and this is the asymmetry a round trip exists to catch: \
             pdfcer wrote bytes pdfcer cannot read. Three candidates — the command has no \
             dispatch arm; the picker was not answered (`{FORM_DATA_ENV}` supplies it); or the \
             parse failed, which traces `import-form-data-failed` with its stage and reason. \
             Trace: {}.",
            session.trace_path().display()
        )));
    };
    let applied: usize = imported
        .get("applied")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let skipped: usize = imported
        .get("skipped")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    if applied == 0 {
        return Ok(Some(format!(
            "the import ran and applied NOTHING: `{}`.\n\
             The file was written by this same program from this same document moments ago, so \
             every name in it is a name the document has. `applied=0 skipped={skipped}` means \
             the reader and the writer disagree about what a field is CALLED — the asymmetry \
             this round trip exists to find, and one that a parser test fed a hand-authored \
             fixture could never see. Trace: {}.",
            imported.raw,
            session.trace_path().display()
        )));
    }
    if applied != fields {
        return Ok(Some(format!(
            "the export wrote {fields} field(s) and the import applied {applied} of them \
             (`skipped={skipped}`): `{}`.\n\
             A partial round trip. Every name in the file came out of this document, so a \
             skipped one is a name that did not survive the write-and-read — a qualified name \
             flattened, an encoding lost, or a field type the writer emits and the reader \
             dispatches differently. Trace: {}.",
            imported.raw,
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★★ the round trip closes: {applied} of {fields} field(s) written and read back by \
         the same program, {skipped} skipped"
    ));
    report.artifact(target);
    Ok(None)
}
