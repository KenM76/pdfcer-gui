//! # `app::dispatch::snapshotsave` — the snapshot box, saved as a one-page PDF
//!
//! `view.snapshot_save_pdf`, from the box's own menu: has the engine cut the
//! box's region out of its page, asks where, and writes it. The document is
//! not changed. A refusal comes before the save dialog, so a dialog is never
//! answered for nothing.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/dispatch/snapshotsave.md`.

use std::path::{Path, PathBuf};

use crate::app::PdfcerApp;
use crate::app::files::Picked;
use crate::app::state::Status;
use crate::text::snapshot as t;

/// Cut the box, ask where, write it, and say what was written or why not.
pub fn save(app: &PdfcerApp) {
    let Status::Open(doc) = &app.status else {
        return;
    };
    let epoch = doc.edit_epoch;
    let (bytes, report) = match crate::clipboard::snapshot::snapshot_pdf(doc) {
        Ok(cut) => cut,
        Err(crate::clipboard::place::Refusal::Render(why)) => {
            refused("engine", epoch, t::save_refused(&why));
            return;
        }
        Err(_) => {
            refused("no-box", epoch, t::save_no_box().to_owned());
            return;
        }
    };
    let Picked::Path(target) =
        crate::app::files::pick_save_path(&suggested_path(&doc.path), t::save_title())
    else {
        // ui-text-exempt: diagnostic trace, never displayed
        crate::diag::trace(|| "snapshot-save-cancelled".to_owned());
        return;
    };
    if target == doc.path {
        refused("over-document", epoch, t::save_over_document().to_owned());
        return;
    }
    if let Err(error) = std::fs::write(&target, &bytes) {
        refused(
            "write",
            epoch,
            t::save_failed(&target.display().to_string(), &error.to_string()),
        );
        return;
    }
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        format!(
            "snapshot-save-pdf w={:.1} h={:.1} bytes={} glyphs_removed={} residual={} path={}",
            report.rect.urx - report.rect.llx,
            report.rect.ury - report.rect.lly,
            bytes.len(),
            report.glyphs_removed,
            u8::from(report.has_residuals()),
            target.display()
        )
    });
    crate::app::actions::record_note(epoch, t::saved_pdf(&target.display().to_string(), &report));
}

/// Trace the refusal under `reason` and put `said` on the status row.
fn refused(reason: &str, epoch: u64, said: String) {
    // ui-text-exempt: diagnostic trace, never displayed
    crate::diag::trace(|| format!("snapshot-save-refused reason={reason}"));
    crate::app::actions::record_note(epoch, said);
}

/// `<stem>-snapshot.pdf` beside the document. Not `set_extension`: a stem
/// holding a dot would lose everything after it.
fn suggested_path(document: &Path) -> PathBuf {
    let stem = document.file_stem().map_or_else(
        || "snapshot".to_owned(),
        |s| s.to_string_lossy().into_owned(),
    ); // ui-text-exempt: a fallback file name
    document.with_file_name(format!("{stem}-snapshot.pdf")) // ui-text-exempt: a file name
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_suggestion_sits_beside_the_document_and_keeps_a_dotted_stem() {
        let got = suggested_path(Path::new("C:/plans/site.rev2.pdf"));
        assert_eq!(got, PathBuf::from("C:/plans/site.rev2-snapshot.pdf"));
    }
}
