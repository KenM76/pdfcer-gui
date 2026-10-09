//! # `app::actions::remove_metadata` — the write behind Security ▸ Remove metadata…
//!
//! Writes a copy of the document without the items the window ticked. The
//! open document is not changed.
//!
//! Why a copy: this shell's Save appends an incremental update (§7.5.6), which
//! keeps every removed value in the revision before it. Only a one-version
//! rewrite that also unpacks the object streams holding an edited object
//! (`EditSession::to_full_bytes_decomposing_containers`) takes them out, and
//! the Save options keep `/Producer` as found (`SaveOptions::identity`), so a
//! removed one is not stamped back.
//!
//! The pipeline, as `super::purge_passwords`: serialize the session as Save
//! would (so unsaved edits are included), remove from a fresh session built
//! from those bytes, rewrite it, and list the output again. A copy that still
//! lists a removed item is not written.

use std::path::PathBuf;

use pdfcer_core::doc_metadata::{MetadataItemId, MetadataRemoval, MetadataRemoveOptions};
use pdfcer_core::document::Document;
use pdfcer_core::edit::EditSession;

use crate::app::state::OpenDoc;
use crate::text::metadata as t;

/// The id whose removal writes a fresh value rather than none, so the copy
/// still lists it (`MetadataItemId`'s documented text form).
const DOCUMENT_ID: &str = "document-id"; // ui-text-exempt: an engine id, never displayed

/// The one disclosure `remove_metadata` always adds, which this write
/// discharges by being the rewrite it asks for. Matched on the verb it names
/// because the engine keeps the sentence private (engine request G172).
const REWRITE_DISCLOSURE_NAMES: &str = "to_full_bytes_decomposing_containers"; // ui-text-exempt: an engine symbol, never displayed

pub(super) fn remove(doc: &mut OpenDoc, ids: &[String]) {
    use crate::app::settings::SettingsExt;

    let signatures = doc.session.signature_census().signatures;
    if signatures > 0 {
        trace_failed("signed", &signatures.to_string());
        super::record_note(doc.edit_epoch, t::refused_signed(signatures));
        return;
    }
    let options = doc.settings.save_options();
    let current = match doc.session.to_incremental_bytes(&options) {
        Ok((bytes, _report)) => bytes,
        Err(error) => return fail(doc, "serialize", &error.to_string()),
    };
    let base = match Document::from_bytes(current) {
        Ok(base) => base,
        Err(error) => return fail(doc, "reopen", &error.to_string()),
    };
    let mut session = doc.settings.open_session(base);
    let wanted: Vec<MetadataItemId> = ids.iter().map(MetadataItemId::new).collect();
    let removal = match session.remove_metadata(&wanted, &MetadataRemoveOptions::default()) {
        Ok(removal) => removal,
        Err(error) => return fail(doc, "remove", &error.to_string()),
    };
    let bytes = match write(&session, &options) {
        Ok(bytes) => bytes,
        Err(detail) => return fail(doc, "write", &detail),
    };
    let left = still_listed(doc, &bytes, &removal);
    if !left.is_empty() {
        trace_failed("still-present", &left.join(","));
        super::record_note(doc.edit_epoch, t::still_present(&left));
        return;
    }
    let crate::app::files::Picked::Path(target) =
        crate::app::files::pick_save_path(&suggested_path(doc), t::save_dialog_title())
    else {
        crate::diag::trace(|| "remove-metadata-cancelled".to_owned()); // ui-text-exempt: diagnostic trace, never displayed
        return;
    };
    if let Err(error) = std::fs::write(&target, &bytes) {
        return fail(doc, "write-file", &error.to_string());
    }
    receipt(doc, &target, &removal);
}

/// The one-version rewrite.
fn write(
    session: &EditSession,
    options: &pdfcer_core::writer::SaveOptions,
) -> Result<Vec<u8>, String> {
    session
        .to_full_bytes_decomposing_containers(options)
        .map(|(bytes, _report, _decomposition)| bytes)
        .map_err(|e| e.to_string())
}

/// The removed ids the written copy still lists, the file identifier aside.
/// An unreadable copy lists everything removed: it is not evidence of
/// absence.
fn still_listed(doc: &OpenDoc, bytes: &[u8], removal: &MetadataRemoval) -> Vec<String> {
    use crate::app::settings::SettingsExt;
    let removed = removal
        .removed
        .iter()
        .filter(|id| id.as_str() != DOCUMENT_ID);
    let Ok(copy) = Document::from_bytes(bytes.to_vec()) else {
        return removed.map(|id| id.as_str().to_owned()).collect();
    };
    let listed: std::collections::HashSet<MetadataItemId> = doc
        .settings
        .open_session(copy)
        .metadata_inventory()
        .items
        .into_iter()
        .map(|i| i.id)
        .collect();
    removed
        .filter(|id| listed.contains(*id))
        .map(|id| id.as_str().to_owned())
        .collect()
}

/// The trace and the off-canvas notes for a written copy.
fn receipt(doc: &OpenDoc, target: &std::path::Path, removal: &MetadataRemoval) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        format!(
            "remove-metadata-wrote removed={} not_found={} not_removed={} freed={} path={}",
            removal.removed.len(),
            removal.not_found.len(),
            removal.not_removed.len(),
            removal.objects_freed,
            target.display()
        )
    });
    let mut notes = vec![t::wrote(
        &target.display().to_string(),
        removal.removed.len(),
    )];
    if !removal.not_removed.is_empty() {
        let items: Vec<(String, String)> = removal
            .not_removed
            .iter()
            .map(|n| (n.id.as_str().to_owned(), n.reason.clone()))
            .collect();
        notes.push(t::not_removed(&items));
    }
    if !removal.not_found.is_empty() {
        notes.push(t::gone_before(removal.not_found.len()));
    }
    notes.extend(
        removal
            .disclosures
            .iter()
            .filter(|d| !d.contains(REWRITE_DISCLOSURE_NAMES))
            .cloned(),
    );
    super::record_notes(doc.edit_epoch, notes);
}

fn trace_failed(stage: &str, detail: &str) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        format!("remove-metadata-failed reason={stage} detail={detail}")
    });
}

fn fail(doc: &OpenDoc, stage: &str, detail: &str) {
    trace_failed(stage, detail);
    super::record_note(doc.edit_epoch, t::failed(detail));
}

/// `<stem>-no-metadata.pdf` beside the document.
fn suggested_path(doc: &OpenDoc) -> PathBuf {
    let stem = doc.path.file_stem().map_or_else(
        || "document".to_owned(),
        |s| s.to_string_lossy().into_owned(),
    ); // ui-text-exempt: a fallback file name
    doc.path.with_file_name(format!("{stem}-no-metadata.pdf")) // ui-text-exempt: a file name
}
