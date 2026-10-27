//! # `dialogs::ocr_output` — Recognise text's output choice
//!
//! Searchable image (the default: invisible words over the untouched scan) or
//! editable text and images (`pdfcer_core::ocr::editable`: recognised words
//! erased from the scan and written as visible text). The engine's defaults
//! for the editable output's confidence floor and image codec are used; its
//! report's sentences reach the operator through `OcrLayerReport::disclosures`.

use pdfcer_core::ocr::editable::{EditableTextOptions, OcrOutput};

use crate::text::ocroutput as t;

const REGION_SEARCHABLE: &str = "ocr-output-searchable"; // ui-text-exempt: trace region name
const REGION_EDITABLE: &str = "ocr-output-editable"; // ui-text-exempt: trace region name

/// The operator's output choice for one Recognise-text transaction.
#[derive(Debug, Clone, Copy, Default)]
pub(super) struct Output {
    editable: bool,
}

impl Output {
    /// What the run writes.
    #[must_use]
    pub(super) fn chosen(self) -> OcrOutput {
        if self.editable {
            OcrOutput::EditableText(EditableTextOptions::new())
        } else {
            OcrOutput::SearchableImage
        }
    }

    /// The trace token for `ocr-started`.
    #[must_use]
    pub(super) const fn token(self) -> &'static str {
        if self.editable {
            "editable" // ui-text-exempt: trace token
        } else {
            "searchable" // ui-text-exempt: trace token
        }
    }

    /// Draw the two choices.
    pub(super) fn show(&mut self, ui: &mut egui::Ui) {
        ui.add_space(8.0);
        ui.label(t::heading());
        let searchable = ui
            .radio_value(&mut self.editable, false, t::searchable())
            .on_hover_text(t::searchable_tooltip());
        crate::diag::ui_rect(REGION_SEARCHABLE, searchable.rect);
        let editable = ui
            .radio_value(&mut self.editable, true, t::editable())
            .on_hover_text(t::editable_tooltip());
        crate::diag::ui_rect(REGION_EDITABLE, editable.rect);
    }
}
