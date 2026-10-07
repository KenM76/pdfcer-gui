//! # `panels::objects::disagree` — where the object list and the picture differ
//!
//! The decomposition counts three ways the list beneath can disagree with the
//! page the operator sees: paths listed but drawn fully transparent, gradients
//! drawn but never listed, and paths whose named colour is not the one shown.
//! Each non-zero count is one line under the panel's summary. The canvas is
//! never marked; this is the off-canvas half of "render normally, report
//! separately".

use crate::text::panels::objects as t;
use pdfcer_core::vector::DecomposeDiagnostics;

/// The block of disagreement lines, published only when one is drawn.
pub const REGION: &str = "panel.objects.disagree"; // ui-text-exempt: trace region name, never displayed

/// Draws the non-zero lines for `page` and traces all three counts every
/// frame, zero included, so a check can tell "nothing to say" from "never
/// asked".
pub(super) fn lines(ui: &mut egui::Ui, page: usize, d: &DecomposeDiagnostics) {
    let (invisible, shadings, undecoded) = (
        d.paths_invisible_by_alpha,
        d.shadings_unmodelled,
        d.paths_with_undecoded_colour,
    );
    crate::diag::trace(|| {
        format!(
            "objects-disagree page={page} invisible={invisible} shadings={shadings} undecoded={undecoded}"
        )
    });
    let said: Vec<String> = [
        (invisible, t::objects_dock_invisible as fn(usize) -> String),
        (shadings, t::objects_dock_shadings),
        (undecoded, t::objects_dock_undecoded_colour),
    ]
    .into_iter()
    .filter(|(n, _)| *n > 0)
    .map(|(n, say)| say(n))
    .collect();
    if said.is_empty() {
        return;
    }
    let block = ui
        .vertical(|ui| {
            for line in said {
                ui.label(egui::RichText::new(line).small());
            }
        })
        .response
        .rect;
    crate::diag::ui_rect_visible(REGION, block, ui.clip_rect());
}
