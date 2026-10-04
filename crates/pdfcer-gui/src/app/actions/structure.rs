//! # `app::actions::structure` — File ▸ Export ▸ Export for hand editing… and
//! Compile hand edits…
//!
//! The engine's QDF round trip (`pdfcer_core::editable`): the export writes
//! the document as a PDF with every object top-level and every stream
//! decoded; the compile diffs a hand-edited export against the document and
//! writes the document plus one incremental update holding only what changed.
//! Neither changes the open document.
//!
//! Both read the document as Save would write it (unsaved edits included),
//! reparsed, because the engine takes a `Document` and the session has no
//! borrowed one.
//!
//! **The stale-base guard.** `import` diffs against whatever it is handed, so
//! compiling an export taken before a later edit would silently undo that
//! edit, and delete anything added since. Each export records a hash of the
//! bytes it was taken from, keyed by the path written; a compile of a recorded
//! path whose base no longer hashes the same is refused. A copy exported in an
//! earlier run is not recorded and is compiled as it stands, with the change
//! counts in the picker's title as the disclosure.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/structure.md`.

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

use pdfcer_core::document::Document;
use pdfcer_core::editable::{self, EditableError};

use crate::app::state::OpenDoc;
use crate::text::structure as t;

/// Every export this process wrote: path written → hash of its base bytes.
static EXPORTED: Mutex<Option<HashMap<PathBuf, u64>>> = Mutex::new(None);

pub(super) fn export(doc: &mut OpenDoc) {
    let Some((current, hash)) = current_document(doc) else {
        return;
    };
    let bytes = match editable::export(&current) {
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
    remember(&target, hash);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        format!(
            "export-structure objects={} bytes={} path={}",
            current.object_count(),
            bytes.len(),
            target.display()
        )
    });
    super::record_note(
        doc.edit_epoch,
        t::exported(&target.display().to_string(), current.object_count()),
    );
}

pub(super) fn compile(doc: &mut OpenDoc, edited_path: &Path) {
    let Some((original, hash)) = current_document(doc) else {
        return;
    };
    if original.encryption().is_some() {
        crate::diag::trace(|| "import-structure-refused reason=encrypted".to_owned()); // ui-text-exempt: diagnostic trace, never displayed
        return super::record_note(doc.edit_epoch, t::compile_encrypted().to_owned());
    }
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
    if recorded(edited_path).is_some_and(|base| base != hash) {
        crate::diag::trace(|| "import-structure-refused reason=stale-base".to_owned()); // ui-text-exempt: diagnostic trace, never displayed
        return super::record_note(doc.edit_epoch, t::stale_base().to_owned());
    }
    let (dirty, report) = editable::import(&original, &edited);
    if report.is_empty() {
        crate::diag::trace(|| "import-structure-refused reason=unchanged".to_owned()); // ui-text-exempt: diagnostic trace, never displayed
        return super::record_note(doc.edit_epoch, t::nothing_changed().to_owned());
    }
    // `EditSession::check_certification`'s refusal: an arbitrary object
    // import is at least as broad as any session edit.
    if pdfcer_core::signature::census(&original).forbids_structural_change() {
        crate::diag::trace(|| "import-structure-refused reason=certified".to_owned()); // ui-text-exempt: diagnostic trace, never displayed
        return super::record_note(doc.edit_epoch, t::certified().to_owned());
    }
    let (modified, added, removed) = (
        report.modified.len(),
        report.added.len(),
        report.removed.len(),
    );
    let title = t::compile_dialog_title(modified, added, removed);
    let crate::app::files::Picked::Path(target) =
        crate::app::files::pick_save_path(&compiled_path(doc), &title)
    else {
        crate::diag::trace(|| "import-structure-cancelled".to_owned()); // ui-text-exempt: diagnostic trace, never displayed
        return;
    };
    if crate::app::files::same_file(&target, &doc.path)
        || crate::app::files::same_file(&target, edited_path)
    {
        crate::diag::trace(|| "import-structure-refused reason=overwrite".to_owned()); // ui-text-exempt: diagnostic trace, never displayed
        return super::record_note(doc.edit_epoch, t::would_overwrite().to_owned());
    }
    // bypass-exempt: an object-level compile-back below the edit model, as
    // the engine's own `import-structure` does it. It writes a new file and
    // leaves the session untouched, so there is no undo to record; the counts
    // are disclosed in the picker title and the receipt, and the
    // certification refusal above is the one `EditSession` would apply.
    let options = {
        use crate::app::settings::SettingsExt;
        doc.settings.save_options()
    };
    let bytes = match pdfcer_core::writer::save_incremental(&original, &dirty, &options) {
        Ok((bytes, _report)) => bytes,
        Err(error) => return fail(doc, "write", &error.to_string()),
    };
    if let Err(error) = std::fs::write(&target, &bytes) {
        return fail(doc, "write-file", &error.to_string());
    }
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        format!(
            "import-structure modified={modified} added={added} removed={removed} \
             unchanged={} streams_matched={} path={}",
            report.unchanged,
            report.streams_matched_after_decode,
            target.display()
        )
    });
    let mut notes = vec![t::compiled(
        &target.display().to_string(),
        modified,
        added,
        removed,
    )];
    if report.streams_matched_after_decode > 0 {
        notes.push(t::matched_after_decode(report.streams_matched_after_decode));
    }
    super::record_notes(doc.edit_epoch, notes);
}

/// The document as Save would write it, reparsed, with a hash of the bytes.
/// `None` after recording the failure.
fn current_document(doc: &OpenDoc) -> Option<(Document, u64)> {
    use crate::app::settings::SettingsExt;
    let bytes = match doc
        .session
        .to_incremental_bytes(&doc.settings.save_options())
    {
        Ok((bytes, _report)) => bytes,
        Err(error) => {
            fail(doc, "serialize", &error.to_string());
            return None;
        }
    };
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    bytes.hash(&mut hasher);
    let hash = hasher.finish();
    match Document::from_bytes(bytes) {
        Ok(document) => Some((document, hash)),
        Err(error) => {
            fail(doc, "reopen", &error.to_string());
            None
        }
    }
}

fn remember(path: &Path, hash: u64) {
    let mut memo = EXPORTED.lock().unwrap_or_else(PoisonError::into_inner);
    memo.get_or_insert_with(HashMap::new)
        .insert(memo_key(path), hash);
}

fn recorded(path: &Path) -> Option<u64> {
    let memo = EXPORTED.lock().unwrap_or_else(PoisonError::into_inner);
    memo.as_ref()?.get(&memo_key(path)).copied()
}

/// The canonical path when it resolves, so two spellings of one file agree.
fn memo_key(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

fn fail(doc: &OpenDoc, stage: &str, detail: &str) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        format!("structure-failed reason={stage} detail={detail}")
    });
    super::record_note(doc.edit_epoch, t::failed(detail));
}

fn stem(doc: &OpenDoc) -> String {
    doc.path.file_stem().map_or_else(
        || "document".to_owned(), // ui-text-exempt: a fallback file name
        |s| s.to_string_lossy().into_owned(),
    )
}

/// `<stem>.qdf.pdf` beside the document.
fn suggested_path(doc: &OpenDoc) -> PathBuf {
    doc.path.with_file_name(format!("{}.qdf.pdf", stem(doc))) // ui-text-exempt: a file name
}

/// `<stem>-compiled.pdf` beside the document.
fn compiled_path(doc: &OpenDoc) -> PathBuf {
    doc.path
        .with_file_name(format!("{}-compiled.pdf", stem(doc))) // ui-text-exempt: a file name
}
