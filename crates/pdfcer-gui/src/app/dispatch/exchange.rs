//! `app::dispatch::exchange` — the File ▸ Export band's commands, and the two
//! lines of `dispatch.rs` they cost
//!
//! Seven commands that move content **between this document and a file on
//! disk**:
//! three exports that write a derivative of the page's own content, and the
//! three round-trip halves beside them.
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
//! ## ★★★ Why this is a module and not six arms in [`super`]
//!
//! **R2, and [`super::security`]' precedent, taken deliberately.** That module's
//! header states the rule this one is applying:
//!
//! > *"The ceiling is not a budget to spend down to; it is a signal that a file
//! > has stopped being one subject."*
//!
//! `file.import_text` was wired on 2026-09-07 — the other half of the
//! operator's *"we should have export/import for that"*, which had been half a
//! feature since 2026-09-04 because `pdfcer-core` could not create a page. Its
//! arm took `dispatch.rs` to **1,521 lines**, and it took `action.rs` and
//! `apply.rs` over the ceiling in the same commit.
//!
//! ⚠ **The line count was the symptom and not the defect.** What had actually
//! happened is that one feature's argument was written **three times, in three
//! files nobody owns** — the action enum, the apply match and the dispatcher —
//! none of which is where somebody looks to understand importing text. R2's own
//! wording is *"when a file approaches the limit, that is the signal to find
//! the seam, not to raise the limit"*, and the seam was that these six commands
//! are one subject sitting in a file about forty subjects.
//!
//! ⇒ `RESUME.md` had warned about this in advance and by name — *"one added
//! line in either fails the build. Split before adding, not after."* It was
//! read, and the split was still done after. The cost was one red gate and
//! twenty minutes; recording that is worth more than the twenty minutes.
//!
//! ## ★★ Why the band's SUBJECT is "exchange" and not "export"
//!
//! Because half of it imports. `RIBBON_IA.md` calls the band Export and that is
//! the right *label* — the operator meets exporting first and far more often —
//! but a module named for it would have to explain, on the day somebody adds a
//! seventh command, why `import_form_data` lives in `export.rs`. The subject is
//! **content crossing the boundary of this document in either direction**,
//! which is what all six do and the only thing all six do.
//!
//! ★ `file.save_as`, `file.save_copy` and `file.save_compacted` are NOT in this
//! band and are not here. They write the document *itself*, not a derivative of
//! it, and they stay in [`super`] beside `file.save` where an operator's mental
//! model puts them.

use super::{PdfcerApp, Status};
use crate::app::actions::Action;

/// Whether `id` belongs to this module.
///
/// ★ Spelled as a `matches!` over the six literals rather than a
/// `starts_with("file.export")` prefix test, which would be shorter and wrong
/// twice over: it would swallow a future `file.export_settings` that has
/// nothing to do with page content, and it would miss both imports, which do
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
    /// stating once: each of these six has at least one decision that cannot be
    /// recovered from a picker, so none of them can be a bare command, and the
    /// two that look like exceptions (`export_form_data`, `import_form_data`)
    /// carry their own note about why the file's **extension** is the decision.
    pub(in crate::app) fn dispatch_exchange(&mut self, id: &str, actions: &mut Vec<Action>) {
        match id {
            "file.export_dxf" => self.dialogs.open_export_dxf(&self.status),
            // ★★★ **Export image — `OPERATOR_REQUESTS.md` O120, wired
            // 2026-09-04.** The operator asked the ENGINE side for it on
            // 2026-09-03; the engine shipped all of it the same day and sent a
            // note marked *"informational, no reply needed"*, which nothing
            // here was required to read. There was no row on this side until a
            // session happened to read the request channel looking for
            // something else.
            //
            // ★ Gated through the registry on `doc.pages` rather than on a
            // capability, exactly as its DXF neighbour is and for that arm's
            // reason: an export reads the document and writes elsewhere, so
            // there is no mode in which it should be refused. Read mode
            // exporting a drawing is what a reading stance is FOR.
            "file.export_image" => self.dialogs.open_export_image(&self.status),
            // ★★★ **Export text — wired 2026-09-04**, on the operator's ask:
            // *"also the engine can export PDFs as text. we should have
            // export/import for that."* A dialog rather than a bare picker,
            // unlike `file.export_form_data` below, because four decisions have
            // to be made before the bytes exist and none is recoverable from a
            // save picker.
            //
            // ★★ **`file.import_text` IS beside it now — 2026-09-07.** This
            // comment used to read *"there is no `file.import_text` beside it,
            // and that is a recorded finding rather than an omission … the
            // engine offers none of them. R9: an absence is honest; a control
            // that declines when pressed is a promise the program cannot
            // keep."* That was correct, it was filed rather than shrugged at,
            // and `pdfcer-core` `Pass 252.0` answered it on 2026-09-06 with
            // `place_text` and the `blank_document` primitive underneath —
            // **nothing in the crate could create a page before, only copy
            // one**, which is why the absence lasted two days rather than one
            // afternoon.
            "file.export_text" => self.dialogs.open_export_text(&self.status),
            // ★★★ **Import text as pages — wired 2026-09-07.** The other half.
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
            // ★★★ **Save as stamp collection — O169, wired 2026-09-10.**
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
