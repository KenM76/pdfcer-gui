//! The two windows the snapshot region opens: Redact outside the region and
//! Crop to region. Both close with their document.

use pdfcer_core::page_tree::Rect;
use pdfcer_gui_base::snapshotbox::SnapshotBox;

use super::{DialogsState, page_crop, redact_outside};
use crate::app::actions::Action;
use crate::app::state::OpenDoc;

impl DialogsState {
    /// Open Redact outside the region over `region`.
    pub fn open_redact_outside(&mut self, _doc: &OpenDoc, region: SnapshotBox) {
        if self.redact_outside.is_none() {
            self.redact_outside = Some(redact_outside::RedactOutsideDialog::open(region));
        }
    }

    /// Open the crop window on `page`, its margins seeded from `rect`.
    pub fn open_crop_to_region(&mut self, doc: &OpenDoc, page: usize, rect: Rect) {
        if self.page_crop.is_none() {
            self.page_crop = page_crop::PageCropDialog::open_at(doc, page, rect);
        }
    }

    /// Draw Redact outside the region, with the redaction panel's look.
    pub(super) fn show_region(
        &mut self,
        ctx: &egui::Context,
        doc: &OpenDoc,
        actions: &mut Vec<Action>,
        appearance: &pdfcer_core::annot_author::RedactAppearance,
    ) {
        if self
            .redact_outside
            .as_mut()
            .map(|d| d.show(ctx, doc, actions, appearance.clone()))
            == Some(false)
        {
            self.redact_outside = None;
        }
    }
}
