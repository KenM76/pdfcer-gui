//! `app::dispatch::exchange` — the File ▸ Export band's commands.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/dispatch/exchange.md`.

use super::{PdfcerApp, Status};
use crate::app::actions::Action;

/// Whether `id` belongs to this module.
pub(crate) fn claims(id: &str) -> bool {
    matches!(
        id,
        "file.export_dxf"
            | "file.export_image"
            | "file.export_text"
            | "file.export_tables"
            | "file.export_word"
            | "file.import_text"
            | "file.export_form_data"
            | "file.import_form_data"
            | "file.stamp_collection"
    )
}

impl PdfcerApp {
    /// Route one Export-band command.
    pub(in crate::app) fn dispatch_exchange(&mut self, id: &str, actions: &mut Vec<Action>) {
        match id {
            "file.export_dxf" => self
                .dialogs
                .open_export_dxf(&self.status, &self.prefs.export.dxf),
            // **Export image — `OPERATOR_REQUESTS.md` O120.**
            //
            // Gated through the registry on `doc.pages` rather than on a
            // capability, exactly as its DXF neighbour is and for that arm's
            // reason: an export reads the document and writes elsewhere, so
            // there is no mode in which it should be refused. Read mode
            // exporting a drawing is what a reading stance is FOR.
            "file.export_image" => self
                .dialogs
                .open_export_image(&self.status, &self.prefs.export.image),
            // **Export text**, on the operator's ask: *"also the engine can
            // export PDFs as text. we should have export/import for that."*
            //
            // A dialog rather than a bare picker, unlike `file.export_form_data`
            // below, because the separator, the line endings and the byte-order
            // mark all have to be settled before the bytes exist, and a save
            // picker can express none of them.
            "file.export_text" => self
                .dialogs
                .open_export_text(&self.status, &self.prefs.export.text),
            "file.export_tables" => self
                .dialogs
                .open_export_tables(&self.status, &self.prefs.export.tables),
            // **Import text as pages** — the other half of that ask.
            //
            // A picker THEN a dialog, which is `file.insert_pages`' shape and
            // not `file.export_text`'s. The difference is which end the
            // decisions are at: an export decides how to write the file it is
            // about to create, so the window comes first and the save picker
            // last; an import decides what to do with a file that already
            // exists, so the picker names the subject and the window configures
            // the output.
            //
            // ⚠ Unlike `insert_pages` directly above, **nothing is read from
            // the file here** — `dialogs::open::open_import_text` carries the
            // argument, and the short form is that nothing this window asks
            // depends on the file's contents.
            "file.import_text" => {
                if let Status::Open(doc) = &self.status {
                    let current = doc.view.page_index;
                    if let crate::app::files::Picked::Path(path) =
                        crate::app::files::pick_text_source()
                    {
                        self.dialogs.open_import_text(path, current);
                    }
                }
            }
            // **Save as stamp collection — `OPERATOR_REQUESTS.md` O169.**
            //
            // In this band because it is the same act every other verb here is:
            // *content of this document, crossing its boundary to a file*. What
            // is unusual is only the destination's meaning — the bytes land in
            // the folder a second application scans at startup.
            //
            // A window and not a bare picker, for `file.export_text`'s reason
            // at its strongest: the operator decides a name **per page** plus a
            // category for the set, and none of that is recoverable from a save
            // dialog. The picker still runs, in the apply phase, after the
            // window has closed.
            //
            // There is no `file.import_stamp_collection` beside it, and that
            // is a recorded finding rather than an omission: a stamp collection
            // is an ordinary PDF, so its import is `file.open`. See the
            // registration in `shell::commands::catalog::file`.
            "file.stamp_collection" => self.dialogs.open_stamp_collection(&self.status),
            "file.export_form_data" => actions.push(Action::Write(
                crate::app::actions::write::WriteAction::FormData,
            )),
            // No window: the whole document, the engine's defaults, and the
            // receipt says what was inferred.
            "file.export_word" => {
                actions.push(Action::Write(crate::app::actions::write::WriteAction::Word))
            }
            // The picker runs HERE, before the action, where the export's runs
            // inside the apply phase. Both are right for their case: an export
            // computes the bytes before it can honestly ask where they go, and
            // an import has nothing to compute until it knows which file.
            //
            // `dispatch_command` is not a layout pass — it runs between frames,
            // from the drained token queue — so a modal here blocks nothing
            // egui is part-way through.
            "file.import_form_data" => {
                if let crate::app::files::Picked::Path(path) =
                    crate::app::files::pick_form_data_source()
                {
                    actions.push(Action::Field(
                        crate::app::actions::forms::FieldAction::Import { path },
                    ));
                }
            }
            _ => (),
        }
    }
}
