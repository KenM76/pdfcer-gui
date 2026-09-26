//! # `panels::properties::disclose` — what pdfcer last worked out, at a width
//! it fits in
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/properties/disclose.md`.

use crate::app::state::OpenDoc;

/// The region this block publishes when it has something to say.
///
/// Published only on the frames it draws, so its **absence** is the evidence
/// that there is no disclosure — which is the distinction a driven check about
/// a refusal is actually asking about.
pub const REGION: &str = "properties.disclosures"; // ui-text-exempt: trace region name, never displayed

/// Draw the disclosure block, and say whether it drew.
pub(super) fn section(ui: &mut egui::Ui, doc: &OpenDoc) -> bool {
    let Some(disclosure) = crate::app::actions::disclosure::last_edit_disclosure(doc.edit_epoch)
    else {
        return false;
    };
    if disclosure.notes.is_empty() {
        return false;
    }
    ui.label(crate::text::tool::disclosures_heading());
    // `ui_rect_visible`, not `ui_rect`. This panel is one `ScrollArea`, and a
    // rect published for a scrolled-out region is a coordinate the harness will
    // click — landing on whatever is really drawn there, and reporting the
    // resulting failure against this block.
    crate::diag::ui_rect_visible(REGION, ui.min_rect(), ui.clip_rect());
    for note in &disclosure.notes {
        // VERBATIM, and wrapped rather than elided — `ui-spec` §6, and the
        // whole reason this block is in a panel rather than in the status row:
        // a disclosure shortened to fit is a disclosure edited by the program
        // doing the disclosing.
        ui.label(egui::RichText::new(note).small());
    }
    ui.separator();
    true
}

#[cfg(test)]
mod tests {
    /// The region names the Properties panel, and no other.
    #[test]
    fn the_region_moved_with_the_block() {
        assert_eq!(super::REGION, "properties.disclosures");
        assert!(!super::REGION.starts_with("tool."));
    }
}
