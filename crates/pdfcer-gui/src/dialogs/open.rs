//! **How each dialog is BUILT** — every `DialogsState::open_*` constructor, and
//! the two guards each of them applies before a window can exist.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/open.md`.

use super::{
    DialogsState, about, bates, compact, diagnostics, embed, export_dxf, export_image,
    export_tables, export_text, export_word, extract_pages, formfield, import_text, insert_image,
    insert_pages, labels, new_document, ocr, offpage, page_crop, page_size, print, protect, redact,
    scale, shortcuts, stamp_collection, textannot, unembed,
};
use crate::app::prefs::RedactionReach;
use crate::app::state::Status;

impl DialogsState {
    /// Open the print dialog for the document in `status`.
    pub(crate) fn open_print(
        &mut self,
        status: &Status,
        remembered: &crate::app::prefs::PrintPrefs,
    ) {
        let Status::Open(doc) = status else {
            return;
        };
        if self.print.is_some() {
            return;
        }
        self.print = Some(print::PrintDialog::open(doc, remembered));
    }

    /// Open the Recognise-text dialog for the document in `status`.
    pub fn open_ocr(
        &mut self,
        status: &Status,
        picked: Vec<usize>,
        prefs: &crate::app::prefs::Prefs,
    ) {
        if self.ocr.is_some() {
            return;
        }
        self.ocr = ocr::open_for(status, picked, prefs);
    }

    /// Open the Apply-redactions dialog for the document in `status`.
    pub fn open_redact(&mut self, status: &Status, reach: RedactionReach) {
        if self.redact.is_some() {
            return;
        }
        self.redact = redact::open_for(status, reach);
    }

    /// **Open the off-the-sheet census window** — `edit.offpage`.
    pub fn open_offpage(&mut self, status: &Status) {
        if self.offpage.is_some() {
            return;
        }
        self.offpage = offpage::open_for(status);
    }

    /// Open the Encrypt / Permissions window — `file.encrypt` and
    /// `file.permissions`, which differ only in the [`crate::protect::Task`].
    pub fn open_protect(&mut self, status: &Status, task: crate::protect::Task) {
        if self.protect.is_some() {
            return;
        }
        self.protect = protect::open_for(status, task);
    }

    /// **Open the Sign window** — `file.sign`.
    #[cfg(feature = "signing")]
    pub fn open_sign(&mut self, status: &Status, remembered_server: Option<&str>) {
        if self.sign.is_some() {
            return;
        }
        self.sign = super::sign::open_for(status, remembered_server);
    }

    /// **Open the *Sign here* window on the box `rect` (canvas space) of the
    /// empty signature field `field`** — a click on that box on the page.
    pub fn open_hand_sign(&mut self, status: &Status, field: &str, page: usize, rect: egui::Rect) {
        let Status::Open(doc) = status else {
            return;
        };
        let typeable = doc
            .pages
            .get(page)
            .is_some_and(pdfcer_gui_base::handsign::typed::writes_along);
        self.hand_sign = Some(super::handsign::HandSignDialog::open(
            field,
            page,
            rect,
            &self.last_signature,
            typeable,
        ));
        // ui-text-exempt: diagnostic trace, never displayed. No field name: it
        // is text from the operator's own document.
        crate::diag::trace(|| format!("hand-sign-opened page={page}"));
    }

    /// **Open the Sign window pointed at the empty signature field `field`**
    /// — a click on its box on the page (O266).
    #[cfg(feature = "signing")]
    pub fn open_sign_on(&mut self, status: &Status, remembered_server: Option<&str>, field: &str) {
        if self.sign.is_none() {
            self.sign = super::sign::open_for(status, remembered_server);
        }
        let found = self.sign.as_mut().is_some_and(|d| d.point_at(field));
        // ui-text-exempt: diagnostic trace, never displayed.
        crate::diag::trace(|| format!("sign-field-chosen found={}", u8::from(found)));
    }

    /// **Hand the signing outcome to the window that asked for it.**
    #[cfg(feature = "signing")]
    pub fn sign_outcome(&mut self, outcome: crate::sign::Outcome) {
        if let Some(dialog) = self.sign.as_mut() {
            dialog.outcome(outcome);
        }
    }

    /// **Open the Set-scale dialog on `group`.**
    pub fn open_scale(&mut self, status: &Status, group: pdfcer_core::dimension::GroupId) {
        let Status::Open(doc) = status else {
            return;
        };
        if self.scale.is_some() {
            return;
        }
        self.scale = Some(scale::ScaleDialog::open(doc, group));
    }

    /// **Open the Set-scale dialog with a reference line already measured.**
    pub fn open_scale_calibrated(
        &mut self,
        status: &Status,
        group: pdfcer_core::dimension::GroupId,
        drawn_pdf_length: f64,
    ) {
        let Status::Open(doc) = status else {
            return;
        };
        self.scale = Some(scale::ScaleDialog::calibrated(doc, group, drawn_pdf_length));
    }

    /// **Hand a measured reference line back to the Set-scale window.**
    pub fn deliver_scale_length(&mut self, drawn_pdf_length: f64) -> bool {
        let Some(dialog) = self.scale.as_mut() else {
            return false;
        };
        dialog.deliver_measured(drawn_pdf_length);
        true
    }

    /// **Open the text-annotation dialog for a just-placed annotation.**
    pub fn open_text_annot(
        &mut self,
        status: &Status,
        page: usize,
        kind: crate::canvas::textannot::TextAnnotKind,
        rect: pdfcer_core::page_tree::Rect,
    ) {
        if !matches!(status, Status::Open(_)) {
            return;
        }
        // The session's stamp memory, handed in rather than reached for: the
        // dialog resolves it against the library it scans on this opening. See
        // `crate::stamps::lastused`.
        self.text_annot = Some(textannot::TextAnnotDialog::open(
            page,
            kind,
            rect,
            self.last_stamp.as_ref(),
        ));
    }

    /// **Open the placement dialog for a form control just put on the page.**
    pub fn open_form_field(
        &mut self,
        status: &Status,
        page: usize,
        rect: pdfcer_core::page_tree::Rect,
        draft: crate::canvas::formfield::Draft,
    ) {
        if !matches!(status, Status::Open(_)) {
            return;
        }
        self.form_field = Some(formfield::FormFieldDialog::open(page, rect, draft));
    }

    /// Open the Render-diagnostics report for the document in `status`.
    pub fn open_diagnostics(&mut self, status: &Status) {
        if !matches!(status, Status::Open(_)) {
            return;
        }
        if self.diagnostics.is_some() {
            return;
        }
        self.diagnostics = Some(diagnostics::DiagnosticsDialog::open());
    }

    /// Open the About dialog.
    pub fn open_about(&mut self) {
        if self.about.is_some() {
            return;
        }
        self.about = Some(about::AboutDialog::open());
    }

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

    /// **Open the Number pages window** on `doc`, offering the rail's
    /// `picked` pages as the default range.
    pub fn open_labels(&mut self, doc: &crate::app::state::OpenDoc, picked: &[usize]) {
        if self.labels.is_some() {
            return;
        }
        self.labels = labels::LabelsDialog::open(picked, doc.pages.len(), doc.label_ranges());
    }

    /// **Open the 3D viewer** on a model decoded from `artwork`, replacing
    /// any model already shown.
    #[cfg(feature = "3d")]
    pub(crate) fn open_model_view(
        &mut self,
        artwork: pdfcer_core::threed::ThreeDArtwork,
        model: crate::app::actions::models::Assembled,
    ) {
        self.model3d = Some(super::model3d::ModelView::open(artwork, model));
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

    /// Open the Export-image window for the open document.
    pub fn open_export_image(
        &mut self,
        status: &Status,
        remembered: &crate::app::prefs::ExportImagePrefs,
    ) {
        if self.export_image.is_some() {
            return;
        }
        self.export_image = export_image::open_for(status, remembered);
    }

    /// Open the Embed-fonts window, and say so when there is nothing to open.
    pub fn open_embed_fonts(
        &mut self,
        status: &Status,
        folders: &[std::path::PathBuf],
    ) -> Option<String> {
        if self.embed.is_some() {
            return None;
        }
        let Status::Open(_) = status else {
            return None;
        };
        self.embed = embed::open_for(status, folders);
        if self.embed.is_some() {
            return None;
        }
        //
        // It read: with no folders configured, say *"pdfcer has no font folders,
        // so it cannot embed anything."* True until 2026-08-28. Since the
        // operator answered O47 with *"yes"*, pdfcer's own standard-14 faces
        // answer when nothing of theirs can — so a document with a missing
        // Helvetica and no folders at all now **opens the window** instead of
        // declining, and the only thing a decline can mean is that there was
        // nothing to do.
        //
        // ⇒ A decline message is a claim about why, and the reasons a program
        // declines change under it. This one would have kept telling operators
        // to configure a folder they no longer need, at the exact moment they
        // were most likely to believe it.
        Some(crate::text::embed::nothing_missing().to_owned())
    }

    /// Open the Add archive time-stamp window on the open document.
    #[cfg(feature = "timestamp")]
    pub fn open_archive(&mut self, status: &Status, remembered: Option<&str>) {
        if self.archive.is_none() {
            self.archive = super::archive::open_for(status, remembered);
        }
    }

    /// Open the compacted-copy window, or answer why the engine refused.
    pub fn open_compact(&mut self, status: &Status) -> Option<String> {
        if self.compact.is_some() {
            return None;
        }
        match compact::open_for(status)? {
            Ok(dialog) => {
                self.compact = Some(dialog);
                None
            }
            Err(sentence) => Some(sentence),
        }
    }

    /// Open the Remove-fonts window, and say so when there is nothing to open.
    pub fn open_unembed_fonts(&mut self, status: &Status) -> Option<String> {
        if self.unembed.is_some() {
            return None;
        }
        let Status::Open(_) = status else {
            return None;
        };
        self.unembed = unembed::open_for(status);
        if self.unembed.is_some() {
            return None;
        }
        Some(crate::text::unembed::nothing_removable().to_owned())
    }

    /// Ask what a dropped PDF is for, unless the question is already open.
    pub fn open_drop_pdf(&mut self, dialog: crate::dialogs::drop_pdf::DropPdfDialog) {
        if self.drop_pdf.is_none() {
            self.drop_pdf = Some(dialog);
        }
    }

    /// Open the Insert-image window for an already-imported picture.
    pub fn open_insert_image(
        &mut self,
        status: &Status,
        image: std::sync::Arc<pdfcer_gui_base::picture::Picture>,
        name: String,
    ) {
        if self.insert_image.is_some() {
            return;
        }
        self.insert_image = insert_image::open_for(status, image, name);
    }
}
