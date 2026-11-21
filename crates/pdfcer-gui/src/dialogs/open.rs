//! **How each dialog is BUILT** — every `DialogsState::open_*` constructor, and
//! the two guards each of them applies before a window can exist.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/open.md`.

use super::{
    DialogsState, about, compact, diagnostics, embed, export_image, formfield, insert_image,
    netprint, ocr, offpage, print, protect, redact, scale, textannot, unembed, xfa_preview,
};
use crate::app::prefs::RedactionReach;
use crate::app::state::Status;

impl DialogsState {
    /// Open the print dialog for the document in `status`.
    #[cfg(not(target_os = "android"))]
    pub(crate) fn open_print(&mut self, status: &Status, prefs: &crate::app::prefs::Prefs) {
        let Status::Open(doc) = status else {
            return;
        };
        if self.print.is_some() {
            return;
        }
        let presets = pdfcer_gui_base::prefs::printpresets::default_folder();
        self.print = Some(
            print::PrintDialog::open(doc, &prefs.print)
                .with_presets(presets, prefs.print_presets.search_list()),
        );
    }

    /// **Open Send to network printer** on the document in `status`: a print-ready
    /// copy of it, or the reason there is none.
    pub(crate) fn open_netprint(&mut self, status: &Status) {
        let Status::Open(doc) = status else {
            return;
        };
        if self.netprint.is_some() {
            return;
        }
        let bytes = doc
            .print_ready_copy()
            .map(|copy| {
                if !copy.report.disclosures.is_empty() {
                    crate::app::actions::record_notes(doc.edit_epoch, copy.report.disclosures);
                }
                copy.bytes
            })
            .map_err(|e| e.to_string());
        let name = doc
            .path
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let pages = u32::try_from(doc.pages.len()).unwrap_or(u32::MAX);
        self.netprint = Some(netprint::NetPrintDialog::open(
            &name,
            bytes,
            pages,
            self.netprint_form.as_ref(),
        ));
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

    /// Open Security ▸ Remove metadata for the document in `status`.
    pub fn open_remove_metadata(&mut self, status: &Status) {
        if self.remove_metadata.is_none() {
            self.remove_metadata = super::remove_metadata::open_for(status);
        }
    }

    /// Open File ▸ Straighten scans or Enlarge scans for the document in
    /// `status`. One window runs either; while one is open the other waits.
    pub fn open_deskew(
        &mut self,
        status: &Status,
        picked: Vec<usize>,
        min_confidence: u8,
        kind: super::deskew::ScanKind,
    ) {
        if self.deskew.is_none() {
            self.deskew = super::deskew::open_for(status, picked, min_confidence, kind);
        }
    }

    /// Open Format ▸ Make pictures smaller on the pictures `ids` of
    /// document `doc`, replacing a window open on an earlier selection.
    pub fn open_shrink_images(
        &mut self,
        doc: u64,
        page: usize,
        ids: Vec<pdfcer_core::object::ObjId>,
    ) {
        self.shrink_images = Some(super::shrink_images::ShrinkImagesDialog::new(
            doc, page, ids,
        ));
    }

    /// Open File ▸ Remove OCR text. `false` when the document holds no pdfcer
    /// OCR layer, so the caller sends the unfiltered removal whose refusal
    /// says so.
    pub fn open_remove_ocr(&mut self, status: &Status, picked: Vec<usize>) -> bool {
        if self.remove_ocr.is_some() {
            return true;
        }
        self.remove_ocr = super::remove_ocr::open_for(status, picked);
        self.remove_ocr.is_some()
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
            crate::app::settings::SettingsExt::render_options(&doc.settings),
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

    /// **Open the Set-scale dialog with a reference line already measured**,
    /// `measured` user-space units long on page `page`.
    pub fn open_scale_calibrated(
        &mut self,
        status: &Status,
        group: pdfcer_core::dimension::GroupId,
        measured: f64,
        page: usize,
    ) {
        let Status::Open(doc) = status else {
            return;
        };
        let Some(reference) = scale::reference_on(doc, measured, page) else {
            return;
        };
        self.scale = Some(scale::ScaleDialog::calibrated(doc, group, reference));
    }

    /// **Hand a measured reference line back to the Set-scale window.**
    /// `false` when no window is open, or the page is not in the document.
    pub fn deliver_scale_length(&mut self, status: &Status, measured: f64, page: usize) -> bool {
        let (Some(dialog), Status::Open(doc)) = (self.scale.as_mut(), status) else {
            return false;
        };
        let Some(reference) = scale::reference_on(doc, measured, page) else {
            return false;
        };
        dialog.deliver_measured(reference);
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

    /// **Open the attach-file dialog** for a marker clicked onto `page`,
    /// once `file` has been picked.
    pub fn open_attach_annot(
        &mut self,
        status: &Status,
        page: usize,
        rect: pdfcer_core::page_tree::Rect,
        file: std::path::PathBuf,
    ) {
        if !matches!(status, Status::Open(_)) {
            return;
        }
        self.text_annot = Some(
            textannot::TextAnnotDialog::open(
                page,
                crate::canvas::textannot::TextAnnotKind::Attachment,
                rect,
                None,
            )
            .with_file(file),
        );
    }

    /// **Open the replace-text dialog** over text selected on `page`, whose
    /// line boxes are `struck`.
    pub fn open_replace_text(
        &mut self,
        status: &Status,
        page: usize,
        struck: Vec<pdfcer_core::annot_author::Quad>,
    ) {
        if !matches!(status, Status::Open(_)) {
            return;
        }
        self.text_annot = textannot::TextAnnotDialog::replacing(page, struck);
    }

    /// **Open the attach-sound dialog** for an icon clicked onto `page`, once
    /// the recording has been picked.
    pub fn open_sound_annot(
        &mut self,
        status: &Status,
        page: usize,
        rect: pdfcer_core::page_tree::Rect,
        file: std::path::PathBuf,
    ) {
        if !matches!(status, Status::Open(_)) {
            return;
        }
        self.text_annot = Some(
            textannot::TextAnnotDialog::open(
                page,
                crate::canvas::textannot::TextAnnotKind::Sound,
                rect,
                None,
            )
            .with_sound(file),
        );
    }

    /// **Open the media-clip dialog** for a region dragged on `page`, once
    /// the clip has been picked.
    pub fn open_screen_annot(
        &mut self,
        status: &Status,
        page: usize,
        rect: pdfcer_core::page_tree::Rect,
        file: std::path::PathBuf,
    ) {
        if !matches!(status, Status::Open(_)) {
            return;
        }
        self.text_annot = Some(
            textannot::TextAnnotDialog::open(
                page,
                crate::canvas::textannot::TextAnnotKind::Screen,
                rect,
                None,
            )
            .with_screen(file),
        );
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

    /// Open the Form-preview window for the dynamic form in `status`.
    pub fn open_xfa_preview(&mut self, ctx: &egui::Context, status: &Status) {
        let Status::Open(doc) = status else {
            return;
        };
        if self.xfa_preview.is_none() {
            self.xfa_preview = Some(xfa_preview::XfaPreviewDialog::open(
                ctx,
                doc,
                crate::app::settings::SettingsExt::render_options(&doc.settings),
            ));
        }
    }

    /// Open the About dialog.
    pub fn open_about(&mut self) {
        if self.about.is_some() {
            return;
        }
        self.about = Some(about::AboutDialog::open());
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

    /// Open the Graphics card switches window. Needs no document.
    pub fn open_card_switches(&mut self) {
        self.card_switches.get_or_insert_with(Default::default);
    }

    /// Open the Manage OCR models window. Needs no document.
    pub fn open_ocr_models(&mut self) {
        self.ocr_models.get_or_insert_with(Default::default);
    }

    /// Open the Add archive time-stamp window on the open document.
    #[cfg(feature = "timestamp")]
    pub fn open_archive(&mut self, status: &Status, remembered: Option<&str>) {
        if self.archive.is_none() {
            self.archive = super::archive::open_for(status, remembered);
        }
    }

    /// Open the Add validation evidence window on the open document.
    /// `false` when the document carries no signature, which the caller
    /// leaves to `app::actions::evidence` to refuse in its own words.
    #[cfg(feature = "signing")]
    pub fn open_evidence(&mut self, status: &Status) -> bool {
        if self.evidence.is_none() {
            self.evidence = super::evidence::open_for(status);
        }
        self.evidence.is_some()
    }

    /// Open the compacted-copy window, or answer why the engine refused.
    pub fn open_compact(
        &mut self,
        status: &Status,
        start: super::compact_target::Start,
    ) -> Option<String> {
        if self.compact.is_some() {
            return None;
        }
        match compact::open_for(status, start)? {
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

    /// Open Security ▸ Trust ▸ Web sites… over `trust`, unless it is already up.
    pub fn open_trust_sites(&mut self, trust: &pdfcer_gui_base::prefs::trust::TrustPrefs) {
        if self.trust_sites.is_none() {
            self.trust_sites = Some(crate::dialogs::trust_sites::TrustSitesDialog::new(trust));
        }
    }

    /// Raise the Security Warning for `uri`, unless one is already up.
    pub fn open_weblink(&mut self, uri: String) {
        if self.weblink.is_none() {
            self.weblink = Some(crate::dialogs::weblink::WebLinkDialog::new(uri));
        }
    }

    /// Raise the Launch File prompt for `file`, unless one is already up.
    pub fn open_launch(&mut self, file: std::path::PathBuf) {
        if self.launch.is_none() {
            self.launch = Some(crate::dialogs::launch::LaunchDialog::new(file));
        }
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

    /// Open Insert 3D model on `source`, to place it on `page`.
    pub fn open_insert_model(&mut self, status: &Status, source: std::path::PathBuf, page: usize) {
        if self.insert_model.is_none() {
            self.insert_model = super::insert_model::open_for(status, source, page);
        }
    }
}
