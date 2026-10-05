//! # `app::actions::structure` — File ▸ Export ▸ Export for hand editing… and
//! Compile hand edits…
//!
//! The engine's QDF round trip (`pdfcer_core::editable`). The export writes
//! the open session, unsaved edits included, as a PDF with every object
//! top-level and every stream decoded, and records the session's content
//! fingerprint in its header. The compile applies a hand-edited export to the
//! session as one undo entry (`EditSession::import_editable`); Save writes it
//! as an incremental update holding only what changed.
//!
//! **The stale-base guard.** The compile diffs against the session as it is
//! now, so a copy exported before a later edit would undo that edit. A copy
//! whose recorded fingerprint differs from the session's is refused. A copy
//! with no record (its marker line deleted, or written by another tool) is
//! applied and the receipt says it could not be checked.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/structure.md`.

use std::path::{Path, PathBuf};

use pdfcer_core::document::Document;
use pdfcer_core::edit::EditError;
use pdfcer_core::editable::{self, EditableError, ExportBase, ImportReport};

use crate::app::state::OpenDoc;
use crate::text::structure as t;

pub(super) fn export(doc: &mut OpenDoc) {
    let bytes = match editable::export(&*doc.session) {
        Ok(bytes) => bytes,
        Err(EditableError::Encrypted) => {
            crate::diag::trace(|| "export-structure-refused reason=encrypted".to_owned()); // ui-text-exempt: diagnostic trace, never displayed
            return super::record_note(doc.edit_epoch, t::encrypted().to_owned());
        }
        Err(error) => return fail(doc, "export", &error.to_string()),
    };
    let crate::app::files::Picked::Path(target) =
        crate::app::files::pick_save_path(&suggested_path(doc), t::export_dialog_title())
    else {
        crate::diag::trace(|| "export-structure-cancelled".to_owned()); // ui-text-exempt: diagnostic trace, never displayed
        return;
    };
    if crate::app::files::same_file(&target, &doc.path) {
        crate::diag::trace(|| "export-structure-refused reason=overwrite".to_owned()); // ui-text-exempt: diagnostic trace, never displayed
        return super::record_note(doc.edit_epoch, t::export_would_overwrite().to_owned());
    }
    if let Err(error) = std::fs::write(&target, &bytes) {
        return fail(doc, "write-file", &error.to_string());
    }
    let objects = editable::EditableSource::object_ids(&*doc.session).len();
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        format!(
            "export-structure objects={objects} bytes={} path={}",
            bytes.len(),
            target.display()
        )
    });
    super::record_note(
        doc.edit_epoch,
        t::exported(&target.display().to_string(), objects),
    );
}

pub(super) fn compile(doc: &mut OpenDoc, edited_path: &Path) {
    let edited = match std::fs::read(edited_path)
        .map_err(|e| e.to_string())
        .and_then(|bytes| Document::from_bytes(bytes).map_err(|e| e.to_string()))
    {
        Ok(edited) => edited,
        Err(detail) => {
            crate::diag::trace(|| {
                format!("import-structure-failed reason=unreadable detail={detail}")
            }); // ui-text-exempt: diagnostic trace, never displayed
            return super::record_note(doc.edit_epoch, t::unreadable(&detail));
        }
    };
    let current = editable::fingerprint(&*doc.session);
    if editable::recorded_base(&edited).is_some_and(|base| base != current) {
        crate::diag::trace(|| "import-structure-refused reason=stale-base".to_owned()); // ui-text-exempt: diagnostic trace, never displayed
        return super::record_note(doc.edit_epoch, t::stale_base().to_owned());
    }
    let mut outcome: Option<Result<ImportReport, EditError>> = None;
    super::apply::vector_edit(doc, "compile-hand-edits", 0, 0, |session| {
        let result = session.import_editable(&edited);
        outcome = Some(result.clone());
        result.map(|report| receipt(&report))
    });
    match outcome {
        Some(Ok(report)) => trace_applied(&report),
        Some(Err(error)) => refused(doc, &error),
        None => {}
    }
}

/// The worded refusal: `import_editable` documents the encryption and
/// certification refusals; anything else is reported in the engine's words.
fn refused(doc: &OpenDoc, error: &EditError) {
    let (reason, said) = match error {
        EditError::DocumentEncrypted => ("encrypted", t::compile_encrypted().to_owned()), // ui-text-exempt: a trace token
        EditError::CertificationForbidsChange { .. } => ("certified", t::certified().to_owned()), // ui-text-exempt: a trace token
        other => ("engine", t::failed(&other.to_string())), // ui-text-exempt: a trace token
    };
    crate::diag::trace(|| format!("import-structure-refused reason={reason}")); // ui-text-exempt: diagnostic trace, never displayed
    super::record_note(doc.edit_epoch, said);
}

/// The receipt for an applied compile: the counts, then what the operator
/// cannot see in them.
fn receipt(report: &ImportReport) -> Vec<String> {
    if report.is_empty() {
        return vec![t::nothing_changed().to_owned()];
    }
    let mut notes = vec![t::compiled(
        report.modified.len(),
        report.added.len(),
        report.removed.len(),
    )];
    if report.streams_matched_after_decode > 0 {
        notes.push(t::matched_after_decode(report.streams_matched_after_decode));
    }
    if report.base == ExportBase::Unrecorded {
        notes.push(t::unrecorded_base().to_owned());
    }
    notes
}

fn trace_applied(report: &ImportReport) {
    if report.is_empty() {
        crate::diag::trace(|| "import-structure-refused reason=unchanged".to_owned()); // ui-text-exempt: diagnostic trace, never displayed
        return;
    }
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        format!(
            "import-structure modified={} added={} removed={} unchanged={} \
             streams_matched={} base={}",
            report.modified.len(),
            report.added.len(),
            report.removed.len(),
            report.unchanged,
            report.streams_matched_after_decode,
            base_token(report.base)
        )
    });
}

fn base_token(base: ExportBase) -> &'static str {
    match base {
        ExportBase::Matches => "matches", // ui-text-exempt: a trace token
        ExportBase::Differs => "differs", // ui-text-exempt: a trace token
        _ => "unrecorded",                // ui-text-exempt: a trace token
    }
}

fn fail(doc: &OpenDoc, stage: &str, detail: &str) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        format!("structure-failed reason={stage} detail={detail}")
    });
    super::record_note(doc.edit_epoch, t::failed(detail));
}

/// `<stem>.qdf.pdf` beside the document.
fn suggested_path(doc: &OpenDoc) -> PathBuf {
    let stem = doc.path.file_stem().map_or_else(
        || "document".to_owned(), // ui-text-exempt: a fallback file name
        |s| s.to_string_lossy().into_owned(),
    );
    doc.path.with_file_name(format!("{stem}.qdf.pdf")) // ui-text-exempt: a file name
}
