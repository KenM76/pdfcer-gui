//! # `app::actions::purge_passwords` — File ▸ Security ▸ Remove old passwords…
//!
//! Finds every password-field value stored in any version of the file and
//! writes a copy holding none. The open document is not changed.
//!
//! Why a copy rather than an undoable edit: this shell's Save appends an
//! incremental update (§7.5.6), which keeps every earlier version and so every
//! earlier value. Only a single-version rewrite that also drops the object
//! streams holding a purged object
//! (`pdfcer_core::edit::EditSession::to_full_bytes_decomposing_containers`) removes them, and
//! that is a save transform, like its File ▸ Security neighbours.
//!
//! The pipeline: serialize the session as Save would (so unsaved edits are
//! included), scan every version with `scan_stored_password_values`, purge a
//! fresh session built from those bytes, rewrite it as one version, and
//! re-scan the output. A copy that still holds a value is not written.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/purge_passwords.md`.

use std::path::PathBuf;

use pdfcer_core::document::Document;
use pdfcer_core::edit::{ContainerDecomposition, PasswordPurgeOutcome};
use pdfcer_core::password_history::scan_stored_password_values;

use crate::app::state::OpenDoc;
use crate::text::purge_passwords as t;

pub(super) fn purge(doc: &mut OpenDoc) {
    use crate::app::settings::SettingsExt;

    let signatures = doc.session.signature_census().signatures;
    if signatures > 0 {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!("purge-passwords-refused reason=signed signatures={signatures}")
        });
        super::record_note(doc.edit_epoch, t::refused_signed(signatures));
        return;
    }

    let options = doc.settings.save_options();
    let current = match doc.session.to_incremental_bytes(&options) {
        Ok((bytes, _report)) => bytes,
        Err(error) => return fail(doc, "serialize", &error.to_string()),
    };
    let found = scan_stored_password_values(&current, Document::from_bytes);
    let earlier = found.in_superseded().count();
    let latest = found.in_latest().count();
    if found.stored.is_empty() {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "purge-passwords-none revisions={} unreadable={}",
                found.revisions, found.unreadable_revisions
            )
        });
        let mut notes = vec![t::none_found(found.revisions)];
        if found.unreadable_revisions > 0 {
            notes.push(t::unreadable(found.unreadable_revisions));
        }
        super::record_notes(doc.edit_epoch, notes);
        return;
    }

    let base = match Document::from_bytes(current) {
        Ok(base) => base,
        Err(error) => return fail(doc, "reopen", &error.to_string()),
    };
    let mut session = doc.settings.open_session(base);
    let PasswordPurgeOutcome {
        fields_purged,
        read_only_purged,
        inherited_not_removed,
        appearance_objects_removed,
        ..
    } = match session.purge_password_values() {
        Ok(outcome) => outcome,
        Err(error) => return fail(doc, "purge", &error.to_string()),
    };
    let (
        bytes,
        _report,
        ContainerDecomposition {
            containers,
            objects_promoted,
            ..
        },
    ) = match session.to_full_bytes_decomposing_containers(&options) {
        Ok(written) => written,
        Err(error) => return fail(doc, "write", &error.to_string()),
    };
    let after = scan_stored_password_values(&bytes, Document::from_bytes);
    if !after.stored.is_empty() {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "purge-passwords-failed reason=still-present remaining={}",
                after.stored.len()
            )
        });
        super::record_note(doc.edit_epoch, t::still_present(after.stored.len()));
        return;
    }

    let suggested = suggested_path(doc);
    let crate::app::files::Picked::Path(target) =
        crate::app::files::pick_save_path(&suggested, t::save_dialog_title())
    else {
        crate::diag::trace(|| "purge-passwords-cancelled".to_owned()); // ui-text-exempt: diagnostic trace, never displayed
        return;
    };
    if let Err(error) = std::fs::write(&target, &bytes) {
        return fail(doc, "write-file", &error.to_string());
    }

    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        format!(
            "purge-passwords fields={} earlier={earlier} latest={latest} revisions={} \
             unreadable={} read_only={} inherited={} appearances={} containers={} \
             promoted={} after_revisions={} path={}",
            fields_purged.len(),
            found.revisions,
            found.unreadable_revisions,
            read_only_purged.len(),
            inherited_not_removed.len(),
            appearance_objects_removed,
            containers,
            objects_promoted,
            after.revisions,
            target.display()
        )
    });
    let mut notes = vec![t::wrote(&target.display().to_string(), earlier, latest)];
    if !read_only_purged.is_empty() {
        notes.push(t::read_only_cleared(&read_only_purged));
    }
    if !inherited_not_removed.is_empty() {
        notes.push(t::inherited_left(&inherited_not_removed));
    }
    if found.unreadable_revisions > 0 {
        notes.push(t::unreadable(found.unreadable_revisions));
    }
    super::record_notes(doc.edit_epoch, notes);
}

fn fail(doc: &OpenDoc, stage: &str, detail: &str) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        format!("purge-passwords-failed reason={stage} detail={detail}")
    });
    super::record_note(doc.edit_epoch, t::failed(detail));
}

/// `<stem>-no-passwords.pdf` beside the document.
fn suggested_path(doc: &OpenDoc) -> PathBuf {
    let stem = doc.path.file_stem().map_or_else(
        || "document".to_owned(),
        |s| s.to_string_lossy().into_owned(),
    ); // ui-text-exempt: a fallback file name
    doc.path.with_file_name(format!("{stem}-no-passwords.pdf")) // ui-text-exempt: a file name
}
