//! # `app::actions::historymove` — a moved document's undo history, taken in
//!
//! The window a document moves to opens the file pristine and replays the
//! sender's exported history through `EditSession::import_history`, so the
//! edits arrive as undo steps. A refusal leaves the session untouched.

use pdfcer_core::edit::{HistoryError, HistoryImport};

use crate::app::state::OpenDoc;

/// Replay `bytes` onto `doc`'s pristine session, as one document change, and
/// count its steps as unsaved.
///
/// # Errors
///
/// The engine's refusal; `None` in its place means the session was shared.
pub(crate) fn import_history(
    doc: &mut OpenDoc,
    bytes: &[u8],
) -> Result<HistoryImport, Option<HistoryError>> {
    let session = super::funnel::session_to_ask(doc).ok_or(None)?;
    let report = session.import_history(bytes).map_err(Some)?;
    let steps = u64::try_from(report.undo_steps).unwrap_or(u64::MAX);
    doc.edit_epoch = doc
        .edit_epoch
        .wrapping_add(1)
        .max(doc.saved_epoch.saturating_add(steps));
    doc.page_epochs.bump_all();
    doc.last_edit_at = Some(std::time::Instant::now());
    super::pages::resync(doc);
    doc.refresh_content_generation();
    Ok(report)
}
