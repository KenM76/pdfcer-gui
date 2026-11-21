//! Page-and-export `DialogsState::open_*` constructors, split from `open`.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/open.md`.

use super::{
    DialogsState, bates, bates_batch, export_dxf, export_tables, export_text, export_word,
    extract_pages, import_text, insert_blank, insert_pages, labels, new_document, page_crop,
    page_size, print, printmarks, running, shortcuts, split_pages, stamp_collection,
};
use crate::app::state::Status;

impl DialogsState {
    /// Open the sized-New dialog.
    pub fn open_insert_pages(&mut self, path: std::path::PathBuf, current_page: usize) {
        if self.insert_pages.is_some() {
            return;
        }
        let count = match pdfcer_core::document::Document::load(&path) {
            Ok(doc) => pdfcer_core::page_tree::pages(&doc).map_or(0, |p| p.len()),
            Err(error) => {
                let detail = error.to_string();
                crate::diag::trace(|| {
                    format!(
                        // ui-text-exempt: diagnostic trace, never displayed in the UI
                        "insert-picked-unreadable path={path:?} reason={detail}"
                    )
                });
                0
            }
        };
        if count == 0 {
            // Nothing to ask about. The refusal is the status-bar sentence the
            // insert path already owns, so the operator meets one voice rather
            // than a dialog and then a note.
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!("insert-declined path={path:?} reason=no-pages")
            });
            return;
        }
        self.insert_pages = Some(insert_pages::InsertPagesDialog::open(
            path,
            count,
            current_page,
        ));
    }

    /// **The dispatch target for `pages.insert_blank`**, opened on the page
    /// the operator is looking at.
    pub fn open_insert_blank(&mut self, doc: &crate::app::state::OpenDoc) {
        if self.insert_blank.is_some() {
            return;
        }
        let current = doc.view.page_index;
        let Some(page) = doc.pages.get(current) else {
            return;
        };
        self.insert_blank = Some(insert_blank::InsertBlankDialog::open(
            current,
            insert_blank::displayed_size(page),
        ));
    }

    /// **The dispatch target for `file.import_text`.**
    pub fn open_import_text(&mut self, path: std::path::PathBuf, current_page: usize) {
        if self.import_text.is_some() {
            return;
        }
        self.import_text = Some(import_text::ImportTextDialog::open(path, current_page));
    }

    pub fn open_new_document(&mut self) {
        if self.new_document.is_some() {
            return;
        }
        self.new_document = Some(new_document::NewDocumentDialog::open());
    }

    /// **Open the crop window** over `pages`, the operand sheets.
    pub fn open_page_crop(&mut self, doc: &crate::app::state::OpenDoc, pages: &[usize]) {
        if self.page_crop.is_some() {
            return;
        }
        self.page_crop = page_crop::PageCropDialog::open(doc, pages);
    }

    /// **Open the Bates window** on a document of `of` pages, offering the
    /// rail's `picked` pages as an alternative to all of them.
    pub fn open_bates(&mut self, picked: &[usize], of: usize) {
        if self.bates.is_some() {
            return;
        }
        self.bates = bates::BatesDialog::open(picked, of, self.bates_form.as_ref());
    }

    /// **Open the Header & footer window** on a document of `of` pages,
    /// offering the rail's `picked` pages as an alternative to all of them.
    pub fn open_running(&mut self, picked: &[usize], of: usize) {
        if self.running.is_some() {
            return;
        }
        self.running = running::RunningDialog::open(picked, of, self.running_form.as_ref());
    }

    /// **Open the Printer's marks window** on a document of `of` pages whose
    /// page-information line starts with `label`.
    pub fn open_printmarks(&mut self, picked: &[usize], of: usize, label: &str) {
        if self.printmarks.is_some() {
            return;
        }
        self.printmarks =
            printmarks::MarksDialog::open(picked, of, label, self.printmarks_form.as_ref());
    }

    /// **Open the Print window on a batch of files**, replacing any Print
    /// window already open. Opens nothing when no file could be opened.
    pub(crate) fn open_batch_print(
        &mut self,
        batch: print::batch::Batch,
        prefs: &crate::app::prefs::Prefs,
    ) {
        if let Some(dialog) = print::PrintDialog::open_batch(batch, &prefs.print) {
            let presets = pdfcer_gui_base::prefs::printpresets::default_folder();
            self.print = Some(dialog.with_presets(presets, prefs.print_presets.search_list()));
        }
    }

    /// **Open the Bates-number-files window** on `sources`, continuing the
    /// numbering the last Bates window left.
    pub fn open_bates_batch(
        &mut self,
        sources: Vec<std::path::PathBuf>,
        settings: pdfcer_core::settings::Settings,
    ) {
        if self.bates_batch.is_some() {
            return;
        }
        self.bates_batch = Some(bates_batch::BatesBatchDialog::open(
            sources,
            self.bates_form.as_ref(),
            settings,
        ));
    }

    /// **Open the Number pages window** on `doc`, offering the rail's
    /// `picked` pages as the default range.
    pub fn open_labels(&mut self, doc: &crate::app::state::OpenDoc, picked: &[usize]) {
        if self.labels.is_some() {
            return;
        }
        self.labels = labels::LabelsDialog::open(picked, doc.pages.len(), doc.label_ranges());
    }

    /// **Open the 3D viewer** on a model decoded from `artwork` in the
    /// document `owner` names (its path and serial), replacing any model
    /// already shown.
    #[cfg(feature = "3d")]
    pub(crate) fn open_model_view(
        &mut self,
        owner: (std::path::PathBuf, u64),
        artwork: pdfcer_core::threed::ThreeDArtwork,
        model: crate::app::actions::models::Assembled,
        eye_mm: Option<u32>,
        walk: bool,
        settings: &std::collections::BTreeMap<String, String>,
    ) {
        let mut view = super::model3d::ModelView::open(owner, artwork, model, eye_mm, walk);
        view.recall(settings);
        view.start_inside();
        self.model3d = Some(view);
    }

    /// Raise the 3D prompt for `artwork`, unless one is already up.
    #[cfg(feature = "3d")]
    pub(crate) fn open_model_trust(&mut self, artwork: pdfcer_core::threed::ThreeDArtwork) {
        if self.model_trust.is_none() {
            self.model_trust = Some(super::model_trust::ModelTrustDialog::new(artwork));
        }
    }

    /// **Open the sheet-size window** over `pages`, the operand sheets.
    pub fn open_page_size(&mut self, doc: &crate::app::state::OpenDoc, pages: &[usize]) {
        if self.page_size.is_some() {
            return;
        }
        self.page_size = page_size::PageSizeDialog::open(doc, pages);
    }

    /// Open the keyboard reference.
    pub fn open_shortcuts(&mut self) {
        if self.shortcuts.is_some() {
            return;
        }
        self.shortcuts = Some(shortcuts::ShortcutsDialog::open());
    }

    /// Open the Save-as-stamp-collection window for the open document.
    pub fn open_stamp_collection(&mut self, status: &Status) {
        if self.stamp_collection.is_some() {
            return;
        }
        self.stamp_collection = stamp_collection::open_for(status);
    }

    /// Open the Export-DXF window for the page on screen.
    pub fn open_export_dxf(
        &mut self,
        status: &Status,
        remembered: &crate::app::prefs::ExportDxfPrefs,
    ) {
        if self.export_dxf.is_some() {
            return;
        }
        self.export_dxf = export_dxf::open_for(status, remembered);
    }

    /// **The dispatch target for the `file.export_text` command**, with
    /// [`Self::open_export_dxf`]'s two guards and for its reasons, and its
    /// `remembered` argument for its reason too — O196.
    pub fn open_export_text(
        &mut self,
        status: &Status,
        remembered: &crate::app::prefs::ExportTextPrefs,
    ) {
        if self.export_text.is_some() {
            return;
        }
        self.export_text = export_text::open_for(status, remembered);
    }

    /// The dispatch target for `file.export_tables`, guarded as
    /// [`Self::open_export_text`] is.
    pub fn open_export_tables(
        &mut self,
        status: &Status,
        remembered: &crate::app::prefs::ExportTablePrefs,
    ) {
        if self.export_tables.is_some() {
            return;
        }
        self.export_tables = export_tables::open_for(status, remembered);
    }

    /// The dispatch target for `file.export_word`, guarded as
    /// [`Self::open_export_text`] is.
    pub fn open_export_word(&mut self, status: &Status) {
        if self.export_word.is_some() {
            return;
        }
        self.export_word = export_word::open_for(status);
    }

    /// The dispatch target for `pages.extract`, starting from `pages`.
    pub fn open_extract_pages(&mut self, doc: &crate::app::state::OpenDoc, pages: &[usize]) {
        if self.extract_pages.is_some() {
            return;
        }
        self.extract_pages = Some(extract_pages::ExtractPagesDialog::open(doc, pages));
    }

    /// The dispatch target for `pages.split`, with the rail's picked pages.
    pub fn open_split_pages(&mut self, doc: &crate::app::state::OpenDoc, picked: &[usize]) {
        if self.split_pages.is_none() {
            self.split_pages = Some(split_pages::SplitPagesDialog::open(doc, picked));
        }
    }
}
