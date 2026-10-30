//! # `app::actions::pagestamp` — **header & footer and printer's marks**
//!
//! The bodies of `PageAction::StampRunningText` and
//! `PageAction::AddPrinterMarks`: one engine call each, one undo step each,
//! and the receipt lines `vector_edit` puts on the status line. Printer's
//! marks change the sheet size, so their receipt says by how much, and names
//! the pages whose trim was assumed (`PrinterMarksOutcome::trim_assumed`).

use pdfcer_core::edit::{EditError, EditSession};
use pdfcer_core::printer_marks::PrinterMarks;
use pdfcer_core::running_text::RunningText;

/// Write `stamp`: one `CommandKind::StampRunningText` undo step.
///
/// # Errors
///
/// The engine's refusal, unchanged.
pub fn running_text(
    session: &mut EditSession,
    stamp: &RunningText,
) -> Result<Vec<String>, EditError> {
    let outcome = session.stamp_running_text(stamp)?;
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            "running-applied n={} kind={:?} first_label={}",
            outcome.pages.len(),
            stamp.kind,
            outcome.first_label.replace(' ', "_")
        )
    });
    Ok(vec![crate::text::running::receipt(
        stamp.kind,
        outcome.pages.len(),
        &outcome.first_label,
    )])
}

/// Draw `marks`: one `CommandKind::AddPrinterMarks` undo step.
///
/// # Errors
///
/// The engine's refusal, unchanged.
pub fn printer_marks(
    session: &mut EditSession,
    marks: &PrinterMarks,
) -> Result<Vec<String>, EditError> {
    let outcome = session.add_printer_marks(marks)?;
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            "printmarks-applied n={} grown_pt={:.2} trim_assumed={}",
            outcome.pages.len(),
            outcome.grown_by,
            outcome.trim_assumed.len()
        )
    });
    let mut notes = vec![crate::text::printmarks::receipt(
        outcome.pages.len(),
        pdfcer_gui_base::units::mm_from_points(outcome.grown_by),
    )];
    if !outcome.trim_assumed.is_empty() {
        notes.push(crate::text::printmarks::trim_assumed(
            outcome.trim_assumed.len(),
        ));
    }
    Ok(notes)
}
