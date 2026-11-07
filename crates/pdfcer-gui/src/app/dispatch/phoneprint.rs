//! File ▸ Print on a phone: a print-ready copy of the document as it stands,
//! handed to Android's print service, whose dialog chooses the printer, paper and
//! pages. No network is touched until the operator picks a printer there.

use pdfcer_gui_base::text::print as s;

use crate::app::actions::{record_note, record_notes};
use crate::app::state::Status;

/// The whole of `file.print` on Android.
pub(super) fn print(status: &Status) {
    let Status::Open(doc) = status else {
        return;
    };
    let name = doc.path.file_name().map_or_else(
        || "document.pdf".to_owned(),
        |n| n.to_string_lossy().into_owned(),
    );
    match doc.print_ready_copy() {
        Ok(copy) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!("phone-print bytes={}", copy.bytes.len())
            });
            crate::filedialog::print_later(copy.bytes, name);
            if !copy.report.disclosures.is_empty() {
                record_notes(doc.edit_epoch, copy.report.disclosures);
            }
        }
        Err(e) => record_note(doc.edit_epoch, s::phone_print_failed(&e.to_string())),
    }
}

/// Puts a print that showed no dialog on the status line. Called once per frame.
pub(crate) fn report_failure(status: &Status) {
    if let Some(reason) = crate::filedialog::take_print_failure() {
        let epoch = match status {
            Status::Open(doc) => doc.edit_epoch,
            _ => 0,
        };
        record_note(epoch, s::phone_print_failed(&reason));
    }
}
