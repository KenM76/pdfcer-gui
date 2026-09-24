//! # `app::actions::ocrlayers` — OCR text layers pdfcer wrote: the re-run policy
//! and File ▸ Remove OCR text
//!
//! The engine marks every layer it writes (`pdfcer_core::ocr::marker`), so a
//! re-run can replace the old one and a removal can find it. Layers written by
//! other software carry no marker and are never touched.
//!
//! Contract:
//! - [`options`] is what [`super::Action::ApplyOcr`] writes with:
//!   `ExistingLayers::Replace`, so a re-run is one copy of every word, and the
//!   recogniser's key in the marker.
//! - [`recognised_disclosures`] adds one sentence totalling `layers_replaced`
//!   ahead of the engine's per-page lines.
//! - [`remove_all`] is [`super::Action::RemoveOcrLayers`]: every marked layer
//!   off, as ONE undo entry (`CommandKind::RemoveOcrLayer`), through the funnel.
//! - `LayerPresent`, `LayerNotFound` and "none found" reach the status bar as
//!   [`crate::text::ocr::OcrLayerRefusal`] sentences.

use pdfcer_core::edit::{CommandKind, EditSession};
use pdfcer_core::ocr::layer::{ExistingLayers, OcrLayerError, OcrLayerOptions, OcrLayerReport};

use crate::app::state::OpenDoc;
use crate::app::status::decline;
use crate::text::ocr::{self as t, OcrLayerRefusal};

/// The options a recognition by `engine` is applied with.
pub(super) fn options(engine: pdfcer_gui_base::ocr::EngineId) -> OcrLayerOptions {
    OcrLayerOptions::new()
        .with_existing(ExistingLayers::Replace)
        .with_engine(engine.key())
}

/// Every page's engine disclosures, preceded by one run-wide total of the
/// earlier layers the write replaced (none when it replaced nothing).
pub(super) fn recognised_disclosures(reports: &[OcrLayerReport]) -> Vec<String> {
    let layers: usize = reports.iter().map(|r| r.layers_replaced).sum();
    let pages = reports.iter().filter(|r| r.layers_replaced > 0).count();
    let mut out = Vec::new();
    if layers > 0 {
        out.push(t::layers_replaced(layers, pages));
    }
    out.extend(reports.iter().flat_map(OcrLayerReport::disclosures));
    out
}

/// Word the refusals this shell can name; the rest take the funnel's generic
/// sentence and keep the engine's words on the trace.
pub(super) fn word_refusal(error: &OcrLayerError) {
    let why = match error {
        OcrLayerError::LayerPresent { .. } => OcrLayerRefusal::AlreadyPresent,
        OcrLayerError::LayerNotFound { .. } => OcrLayerRefusal::LayerGone,
        _ => return,
    };
    decline::record_ocr_layer(why);
}

/// Why a removal run stopped with nothing removed.
enum RemoveError {
    PageTree(pdfcer_core::page_tree::PageTreeError),
    Layer(OcrLayerError),
    NoneFound,
}

impl std::fmt::Display for RemoveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PageTree(e) => write!(f, "{e}"),
            Self::Layer(e) => write!(f, "{e}"),
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            Self::NoneFound => f.write_str("no pdfcer OCR layer in the document"),
        }
    }
}

/// File ▸ Remove OCR text: remove every layer pdfcer wrote, as one undo step.
pub(super) fn remove_all(doc: &mut OpenDoc) {
    super::apply::vector_edit(doc, "remove-ocr-layers", 0, 0, |session| {
        remove_in(session).inspect_err(|e| match e {
            RemoveError::NoneFound => decline::record_ocr_layer(OcrLayerRefusal::NoneFound),
            RemoveError::Layer(e) => word_refusal(e),
            RemoveError::PageTree(_) => {}
        })
    });
}

fn remove_in(session: &mut EditSession) -> Result<Vec<String>, RemoveError> {
    let layers = session.find_ocr_layers().map_err(RemoveError::PageTree)?;
    if layers.is_empty() {
        return Err(RemoveError::NoneFound);
    }
    let mut pages: Vec<usize> = Vec::new();
    let mut removed = 0usize;
    let mut failure = None;
    for layer in &layers {
        match session.remove_ocr_layer(layer) {
            Ok(()) => {
                removed += 1;
                if !pages.contains(&layer.page_index) {
                    pages.push(layer.page_index);
                }
            }
            // Nothing committed yet: an ordinary refusal.
            Err(e) if removed == 0 => return Err(RemoveError::Layer(e)),
            Err(e) => {
                failure = Some(e);
                break;
            }
        }
    }
    // Committed commands must reach the epoch bump, so a partial run is an
    // `Ok` that says what stayed. The fold is checked; a `false` leaves the
    // removals applied as separate undo steps.
    if !session.coalesce_last(removed, CommandKind::RemoveOcrLayer) {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!("remove-ocr-layers-unfolded n={removed}")
        });
    }
    Ok(vec![match failure {
        None => t::layers_removed(removed, pages.len()),
        Some(e) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!(
                    "remove-ocr-layers-partial removed={removed} of={} why={e}",
                    layers.len()
                )
            });
            t::layers_removed_partly(removed, layers.len())
        }
    }])
}
