//! `app::dispatch::exchange` — the File ▸ Export band's commands.
//!
//! Every command here moves content **between this document and a file on
//! disk**: exports that write a derivative of the page's own content, and the
//! round-trip halves beside them that read one back in.
//!
//! | command | direction | window |
//! |---|---|---|
//! | `file.export_dxf` | out | [`crate::dialogs::export_dxf`] |
//! | `file.export_image` | out | [`crate::dialogs::export_image`] |
//! | `file.export_text` | out | [`crate::dialogs::export_text`] |
//! | `file.import_text` | **in** | picker, then [`crate::dialogs::import_text`] |
//! | `file.export_form_data` | out | a save picker, no window |
//! | `file.import_form_data` | **in** | a picker, no window |
//! | `file.stamp_collection` | out | [`crate::dialogs::stamp_collection`] |
//!
//! ## ★★ Why this is a module and not a run of arms in [`super`]
//!
//! **R2.** These commands are one subject, and [`super`] is a file about
//! several dozen. The ceiling on a file's size is not a budget to spend down
//! to; it is a signal that a file has stopped being one subject, and the seam
//! to cut on is the subject, never the line count.
//!
//! ## ★★ Why the band's SUBJECT is "exchange" and not "export"
//!
//! Because half of it imports. `RIBBON_IA.md` calls the band Export and that is
//! the right *label* — the operator meets exporting first and far more often —
//! but a module named for it would have to explain, on the day somebody adds
//! another import, why `import_form_data` lives in `export.rs`. The subject is
//! **content crossing the boundary of this document in either direction**,
//! which is what every verb here does and the only thing they all do.
//!
//! ★ `file.save_as`, `file.save_copy` and `file.save_compacted` are NOT in this
//! band and are not here. They write the document *itself*, not a derivative of
//! it, and they stay in [`super`] beside `file.save` where an operator's mental
//! model puts them.

use super::{PdfcerApp, Status};
use crate::app::actions::Action;

/// Whether `id` belongs to this module.
///
/// ★ Spelled as a `matches!` over the literals rather than a
/// `starts_with("file.export")` prefix test, which would be shorter and wrong
/// twice over: it would swallow a future `file.export_settings` that has
/// nothing to do with page content, and it would miss the imports, which do
/// not start with `export` and are the reason this module is not called that.
pub(crate) fn claims(id: &str) -> bool {
    matches!(
        id,
        "file.export_dxf"
            | "file.export_image"
            | "file.export_text"
            | "file.import_text"
            | "file.export_form_data"
            | "file.import_form_data"
            | "file.stamp_collection"
    )
}

impl PdfcerApp {
    /// Route one Export-band command.
    ///
    /// ★★ Every arm here is either *open a window* or *pick a file, then open a
    /// window* — never *do the thing*. That is the band's shape and it is worth
    /// stating once: each of these verbs has at least one decision that cannot
    /// be recovered from a picker, so none of them can be a bare command, and
    /// the two that look like exceptions (`export_form_data`,
    /// `import_form_data`) carry their own note about why the file's
    /// **extension** is the decision.
    pub(in crate::app) fn dispatch_exchange(&mut self, id: &str, actions: &mut Vec<Action>) {
        match id {
            "file.export_dxf" => self
                .dialogs
                .open_export_dxf(&self.status, &self.prefs.export.dxf),
            // **Export image — `OPERATOR_REQUESTS.md` O120.**
            //
            // ★ Gated through the registry on `doc.pages` rather than on a
            // capability, exactly as its DXF neighbour is and for that arm's
            // reason: an export reads the document and writes elsewhere, so
            // there is no mode in which it should be refused. Read mode
            // exporting a drawing is what a reading stance is FOR.
            "file.export_image" => self
                .dialogs
                .open_export_image(&self.status, &self.prefs.export.image),
            // ★★ **Export text**, on the operator's ask: *"also the engine can
            // export PDFs as text. we should have export/import for that."*
            //
            // A dialog rather than a bare picker, unlike `file.export_form_data`
            // below, because the separator, the line endings and the byte-order
            // mark all have to be settled before the bytes exist, and a save
            // picker can express none of them.
            "file.export_text" => self
                .dialogs
                .open_export_text(&self.status, &self.prefs.export.text),
            // ★★ **Import text as pages** — the other half of that ask.
            //
            // ★ A picker THEN a dialog, which is `file.insert_pages`' shape and
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
            // ★★ **Save as stamp collection — `OPERATOR_REQUESTS.md` O169.**
            //
            // In this band because it is the same act every other verb here is:
            // *content of this document, crossing its boundary to a file*. What
            // is unusual is only the destination's meaning — the bytes land in
            // the folder a second application scans at startup.
            //
            // ★ A window and not a bare picker, for `file.export_text`'s reason
            // at its strongest: the operator decides a name **per page** plus a
            // category for the set, and none of that is recoverable from a save
            // dialog. The picker still runs, in the apply phase, after the
            // window has closed.
            //
            // ★ There is no `file.import_stamp_collection` beside it, and that
            // is a recorded finding rather than an omission: a stamp collection
            // is an ordinary PDF, so its import is `file.open`. See the
            // registration in `shell::commands::catalog::file`.
            "file.stamp_collection" => self.dialogs.open_stamp_collection(&self.status),
            "file.export_form_data" => actions.push(Action::Write(
                crate::app::actions::write::WriteAction::FormData,
            )),
            // ★ The picker runs HERE, before the action, where the export's runs
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
