//! # `writeaction` — the three verbs that exist only to move a file
//! picker out of the layout pass
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/writeaction.md`.

/// The three verbs that exist only to move a native file picker out of the
/// layout pass. See the module header.
#[derive(Debug, Clone, PartialEq)]
pub enum WriteAction {
    /// **Write one page's vector geometry out as a DXF.**
    ///
    /// Raised by `pdfcer_gui::dialogs::export_dxf` and by nothing else.
    ///
    /// # Why an export is an `Action` when it changes no document
    ///
    /// The same reason `pdfcer_gui::app::actions::Action::SaveCopy` and `PageAction::ExtractPages` are:
    /// **a native file dialog must not open inside a layout pass.** It is a
    /// modal OS window that blocks the thread, so opening one from a widget's
    /// `clicked()` branch leaves egui part-way through a frame that will not
    /// finish until the operator has answered.
    ///
    /// Nothing about the document is being ordered — there is nothing to order.
    /// The funnel's *invariant* does not apply here; its **reason** does.
    ///
    /// # Why the geometry is not carried
    ///
    /// The apply phase decomposes each page from the session's view when the
    /// queue drains, so an edit raised earlier in the same frame is in the
    /// export. See `export::dxf`.
    Dxf {
        /// The 0-based pages, in order, resolved by the window when Export was
        /// pressed. More than one writes one file per page.
        pages: Vec<usize>,
        /// The engine's own options struct, edited in place by the dialog.
        ///
        /// Carried whole rather than decomposed into scale, units and two
        /// flags, for `pdfcer_gui::app::actions::Action::Dimension`'s reason one feature along: it **is**
        /// the value the writer takes, and rebuilding it in the apply arm would
        /// put a second constructor in the path.
        options: pdfcer_core::export::dxf::DxfOptions,
    },
    /// **Write one or more pages out as a picture** — PNG, JPEG or SVG.
    /// `OPERATOR_REQUESTS.md` **O120**.
    ///
    /// Raised by `pdfcer_gui::dialogs::export_image` and by nothing else.
    ///
    /// # Why it carries a whole plan, like [`Self::Dxf`] and unlike
    /// [`Self::FormData`]
    ///
    /// A dialog collected four decisions before the press — which format, which
    /// pages, what resolution, whether transparency survives — and none of them
    /// can be recovered from a save picker. `FormData` needs no plan precisely
    /// because its one decision (the format) *is* recoverable from the picker,
    /// as the extension the operator types.
    ///
    /// # Why the plan is the SHELL's type and not the engine's
    ///
    /// [`Self::Dxf`] carries `DxfOptions` because that is literally the value
    /// the writer takes. There is no engine equivalent here, and that is a fact
    /// about the feature rather than a gap: the engine offers three unrelated
    /// writers (`export::encode_png`, `export::encode_jpeg`,
    /// `svg::export_svg_view`) with three options types and three error types,
    /// and *"which of the three, over which pages"* is a question none of them
    /// asks. See `crate::imageexport` for the whole argument.
    ///
    /// # The pages are RESOLVED, not a scope and a string
    ///
    /// The window has already parsed the typed range — it needs the answer to
    /// decide whether Export is pressable — so re-parsing in the apply phase
    /// would be a second reading of the same box against a document that may
    /// have changed pages in between.
    Image {
        /// Everything the writer needs, frozen when Export was pressed.
        plan: crate::imageexport::ImagePlan,
    },
    /// **Write the words on one or more pages out as a plain text file.**
    ///
    /// Raised by `pdfcer_gui::dialogs::export_text` and by nothing else. The
    /// operator, 2026-09-04: *"also the engine can export PDFs as text. we
    /// should have export/import for that."*
    ///
    /// # Why it carries a plan, like [`Self::Image`] and unlike [`Self::FormData`]
    ///
    /// A window collected four decisions before the press — which pages, what
    /// goes between them, how lines end, and whether the file opens with a
    /// byte-order mark — and none of the four is recoverable from a save
    /// picker. `FormData` needs no plan precisely because its one decision (the
    /// format) *is* recoverable from the picker, as the extension typed.
    ///
    /// # The pages are RESOLVED, not a scope and a string
    ///
    /// [`Self::Image`]'s reason verbatim: the window has already parsed the
    /// typed range — it needs the answer to decide whether Export is pressable
    /// — so re-parsing in the apply phase would be a second reading of the same
    /// box against a document that may have changed pages in between.
    ///
    /// # The plan is the SHELL's type, and there is no engine equivalent
    ///
    /// [`Self::Dxf`] carries `DxfOptions` because that is literally the value
    /// the writer takes. There is no writer here at all: the engine offers
    /// `text_extract::extract_pages_view` and `ExtractedText::plain_text()`,
    /// and *"how do several pages become one file"* is a question neither of
    /// them asks. See `crate::exporttext` for the whole argument, and for the
    /// recorded finding that the **import** half of the operator's sentence has
    /// no engine route at all.
    Text {
        /// Everything the write needs, frozen when Export was pressed.
        plan: crate::exporttext::TextExportPlan,
    },
    /// **Write the detected tables as a workbook or CSV files.**
    Tables {
        /// The pages and format, frozen when Export was pressed.
        plan: crate::tableexport::TableExportPlan,
    },
    /// **Write the form's values out as FDF, XFDF or CSV.**
    ///
    /// # It carries nothing, and that is the difference from [`Self::Dxf`]
    ///
    /// The DXF export carries a page index and an options struct because a
    /// dialog collected both before the action was raised. This one has no
    /// dialog: the format is decided by the extension the operator types in the
    /// save picker, and the picker opens inside the apply phase for the reason
    /// `actions::export`'s header gives — **a native file dialog must not open
    /// inside a layout pass**, because it blocks the thread while egui is
    /// part-way through a frame.
    ///
    /// So this is an `Action` purely to move the picker out of the layout pass.
    /// Nothing about the document is being ordered, and nothing about it
    /// changes.
    FormData,
    /// **Write pages as a Word file.** Raised by
    /// `pdfcer_gui::dialogs::export_word`; carries its plan for
    /// [`Self::Tables`]'s reason.
    Word {
        /// The pages and the engine's choices, frozen when Export was pressed.
        plan: crate::wordexport::WordExportPlan,
    },
    /// **Write a copy holding no stored password-field value.** Carries
    /// nothing, for [`Self::FormData`]'s reason.
    PurgePasswords,
    /// **Write a copy without the metadata items named.** Raised by
    /// `pdfcer_gui::dialogs::remove_metadata`; carries the ticked ids, in
    /// `pdfcer_core::doc_metadata::MetadataItemId`'s text form.
    RemoveMetadata {
        /// The ids, frozen when Remove was pressed.
        ids: Vec<String>,
    },
    /// **Write a copy laid out for hand editing** (qpdf's QDF). Carries
    /// nothing, for [`Self::FormData`]'s reason.
    Structure,
    /// **Compile a hand-edited copy back into an incremental update** written
    /// to a file the operator picks. The open document is not changed.
    CompileStructure {
        /// The hand-edited copy, picked before the action was raised.
        edited: std::path::PathBuf,
    },
    /// **Write the already-serialised compacted copy to a file the operator
    /// picks.**
    ///
    /// Raised by `pdfcer_gui::dialogs::compact` and by nothing else.
    /// `OPERATOR_REQUESTS.md` **O48**. **`app::save::compacted` carries the
    /// argument** for why the bytes travel rather than being re-serialised here:
    /// the window quoted a measurement of them, and when a confirmation quotes a
    /// number, the thing it quoted is the operand.
    ///
    /// No path. The picker opens inside the apply phase, for
    /// [`Self::FormData`]'s reason — a native file dialog must not open
    /// inside a layout pass.
    /// **Write this document's pages out as an Acrobat stamp
    /// collection.** `OPERATOR_REQUESTS.md` **O169**.
    ///
    /// Raised by `pdfcer_gui::dialogs::stamp_collection` and by nothing else.
    ///
    /// # Why it carries a whole plan, like [`Self::Image`] and [`Self::Text`]
    ///
    /// A window collected a decision per page — whether the page is a stamp at
    /// all, and what it is called — plus the category for the set. None of that
    /// is recoverable from a save picker, and the plan is also the value the
    /// writer takes: `crate::stamps::write::build_and_write` reads exactly this
    /// struct.
    ///
    /// # Why the plan and not a name list plus a page list
    ///
    /// Because those are the two things that must not be allowed to drift
    /// apart. `pdfcer_core::stamp_file::name_stamp_pages` names `stamps[i]` to
    /// **page `i` by counting**, so a pair of lists carried separately is a
    /// pair somebody can eventually filter differently — and the result is a
    /// collection that opens, holds the right number of stamps, and names the
    /// wrong artwork every time. [`crate::stamps::Plan`] derives both lists
    /// from the same rows, so the invariant travels with the value.
    ///
    /// No path. The picker opens inside the apply phase, for
    /// [`Self::FormData`]'s reason — a native file dialog must not open inside
    /// a layout pass.
    StampCollection {
        /// Every row the window showed, with its tick and its name.
        plan: crate::stamps::Plan,
    },
    Compacted {
        /// The whole file, already written by `to_full_bytes`.
        ///
        /// Moved, never cloned: it is the document, and on a dense CAD sheet
        /// that is megabytes. The dialog closes on the frame that raises this,
        /// so nothing else is still holding it by the time the queue drains.
        bytes: Vec<u8>,
        /// What the document occupied on disk before, for the disclosure.
        before: u64,
    },
}
