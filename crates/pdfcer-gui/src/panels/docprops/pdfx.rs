//! # `panels::docprops::pdfx` — the PDF/X level the file claims
//!
//! A claim only (`pdfcer_core::prepress::pdfx_claim`): the engine reads where
//! each part of ISO 15930 puts the claim and checks nothing, so the row says
//! "claims" and never "is". Disagreeing carriers, an unregistered level and a
//! missing `/GTS_PDFX` output intent are each said off-canvas, here.

use crate::app::state::OpenDoc;
use crate::text::panels::docprops as t;
use pdfcer_core::prepress::{PdfxClaim, pdfx_claim};

/// The PDF/X lines, published only when the file claims a level.
pub const REGION: &str = "docprops.pdfx"; // ui-text-exempt: trace region name, never displayed

/// The claim, read once per document revision: it parses the XMP packet.
fn claim(ui: &egui::Ui, doc: &OpenDoc) -> Option<PdfxClaim> {
    let id = egui::Id::new(("docprops-pdfx", doc.path.as_path(), doc.edit_epoch));
    if let Some(cached) = ui.ctx().data(|d| d.get_temp::<Option<PdfxClaim>>(id)) {
        return cached;
    }
    let read = pdfx_claim(&doc.session.view());
    ui.ctx().data_mut(|d| d.insert_temp(id, read.clone()));
    read
}

/// Draws the claim and what it lacks, and traces it every frame.
pub(super) fn lines(ui: &mut egui::Ui, doc: &OpenDoc) {
    let Some(claim) = claim(ui, doc) else {
        crate::diag::trace(|| "pdfx-claim none".to_owned()); // ui-text-exempt: trace line, never displayed
        return;
    };
    crate::diag::trace(|| {
        // ui-text-exempt: trace line, never displayed
        format!(
            "pdfx-claim level={} recognised={} agree={} intents={}",
            claim.level.replace(char::is_whitespace, "_"),
            u8::from(claim.is_recognised()),
            u8::from(claim.carriers_agree()),
            claim.output_intents.len()
        )
    });
    let block = ui
        .vertical(|ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(t::pdfx_label());
                ui.label(t::pdfx_level(&claim.level));
            });
            ui.weak(t::pdfx_claim_only());
            if !claim.is_recognised() {
                ui.weak(t::pdfx_unregistered());
            }
            if let (false, Some(xmp), Some(info)) = (
                claim.carriers_agree(),
                &claim.xmp_version,
                &claim.info_version,
            ) {
                ui.weak(t::pdfx_disagree(xmp, info));
            }
            if claim.has_output_intent() {
                let named: Vec<&str> = claim
                    .output_intents
                    .iter()
                    .map(String::as_str)
                    .filter(|s| !s.is_empty())
                    .collect();
                if !named.is_empty() {
                    ui.weak(t::pdfx_intents(&named));
                }
            } else {
                ui.weak(t::pdfx_no_intent());
            }
        })
        .response
        .rect;
    crate::diag::ui_rect_visible(REGION, block, ui.clip_rect());
}
