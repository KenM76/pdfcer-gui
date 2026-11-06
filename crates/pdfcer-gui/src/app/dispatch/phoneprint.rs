//! File ▸ Print on a phone: the document as it stands, edits included, handed
//! to Android's print service, whose dialog chooses the printer, paper and
//! pages. No network is touched until the operator picks a printer there.

use pdfcer_gui_base::text::print as s;

use crate::app::actions::record_note;
use crate::app::settings::SettingsExt;
use crate::app::state::{OpenDoc, Status};

/// The whole of `file.print` on Android.
pub(super) fn print(status: &Status) {
    let Status::Open(doc) = status else {
        return;
    };
    let name = doc.path.file_name().map_or_else(
        || "document.pdf".to_owned(),
        |n| n.to_string_lossy().into_owned(),
    );
    match doc
        .session
        .to_incremental_bytes(&doc.settings.save_options())
    {
        Ok((bytes, _report)) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!("phone-print bytes={}", bytes.len())
            });
            let omits = preview_may_omit(doc);
            let brief = if omits {
                s::phone_print_preview_brief()
            } else {
                ""
            };
            crate::filedialog::print_later(bytes, name, brief);
            if omits {
                record_note(doc.edit_epoch, s::phone_print_preview_note().to_owned());
            }
        }
        Err(e) => record_note(doc.edit_epoch, s::phone_print_failed(&e.to_string())),
    }
}

/// Whether a page carries an annotation Android's preview may leave out. A link
/// draws nothing, so it does not count.
fn preview_may_omit(doc: &OpenDoc) -> bool {
    let graph = doc.session.graph();
    doc.pages.iter().any(|page| {
        pdfcer_core::annot::page_annotations(&graph, page.id)
            .iter()
            .any(|a| a.subtype.as_slice() != b"Link")
    })
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
