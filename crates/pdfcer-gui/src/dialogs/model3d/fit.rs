//! The viewer's mesh-fit choice and what the model drawn under it inferred.
//!
//! The choice re-assembles the model's bytes with `pdfcer_3d::MeshFit` and
//! keeps the camera. Everything past *Exact only* is disclosed below the
//! picture, never on it: a mesh drawn in part renders exactly as a whole one
//! would (`render_model`), and the lines here say what the file does not
//! confirm.

use egui::Ui;
use pdfcer_3d::MeshFit;

use super::ModelView;
use crate::app::actions::models::{Assembled, Unassembled, refit};
use crate::text::panels::meshfit as tf;
use crate::text::panels::models as t;

/// The mesh-fit drop-down; its items add `.<index in FITS>`.
pub const REGION_MESH_FIT: &str = "model3d.mesh_fit"; // ui-text-exempt: trace region name, never displayed

/// The fits offered, in `tf::mesh_fit_names` order.
const FITS: [MeshFit; 4] = [
    MeshFit::BestFit,
    MeshFit::Unique,
    MeshFit::Partial,
    MeshFit::Continue,
];

/// The trace token of each of [`FITS`].
const TOKENS: [&str; 4] = ["best-fit", "unique", "partial", "continue"]; // ui-text-exempt: trace tokens, never displayed

/// `fit`'s index in [`FITS`]; a fit the engine adds later reads as the
/// default.
fn index_of(fit: MeshFit) -> usize {
    FITS.iter().position(|f| *f == fit).unwrap_or(0)
}

/// The `model-view-*` trace fields for what `model` drew under its fit.
pub(super) fn trace_fields(model: &Assembled) -> String {
    let d = &model.drawn;
    let (got, of) = d.partial_triangles;
    format!(
        "fit={} opened-in-part={} partial={} partial-triangles={got}/{of} left-out={}", // ui-text-exempt: trace fields, never displayed
        TOKENS[index_of(model.fit)],
        u8::from(model.opened_in_part),
        d.partial,
        d.compressed
    )
}

impl ModelView {
    /// *Mesh fit*: assemble the model again under the fit picked, keeping
    /// the camera. A fit that leaves nothing to draw is not applied, and the
    /// window says why.
    pub(super) fn mesh_fit_control(&mut self, ui: &mut Ui) {
        let names = tf::mesh_fit_names();
        let current = index_of(self.model.fit);
        let mut picked = None;
        ui.label(tf::mesh_fit())
            .on_hover_text(tf::mesh_fit_tooltip());
        let combo = egui::ComboBox::from_id_salt(REGION_MESH_FIT)
            .selected_text(names[current])
            .show_ui(ui, |ui| {
                for (i, name) in names.into_iter().enumerate() {
                    let item = ui.selectable_label(i == current, name);
                    crate::diag::ui_rect_visible(
                        // ui-text-exempt: trace region name, never displayed
                        &format!("{REGION_MESH_FIT}.{i}"),
                        item.rect,
                        ui.clip_rect(),
                    );
                    if item.clicked() && i != current {
                        picked = Some(i);
                    }
                }
            });
        crate::diag::ui_rect_visible(REGION_MESH_FIT, combo.response.rect, ui.clip_rect());
        combo.response.on_hover_text(tf::mesh_fit_tooltip());
        if let Some(i) = picked {
            self.apply_fit(i);
        }
    }

    fn apply_fit(&mut self, i: usize) {
        match refit(&self.model, FITS[i]) {
            Ok(model) => {
                self.model = model;
                self.fit_refused = None;
                if let Some(bounds) = pdfcer_3d::Bounds::of(&self.model.drawn.meshes) {
                    self.bounds = bounds;
                }
                self.rendered = None;
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed
                    format!(
                        "model-view-refit parts={} triangles={} best-fit={} {}",
                        self.model.drawn.meshes.len(),
                        self.model.triangles,
                        self.model.best_fit,
                        trace_fields(&self.model)
                    )
                });
            }
            Err(why) => {
                let why = match why {
                    Unassembled::Empty { compressed } => t::view_empty(compressed),
                    Unassembled::NotPrc => t::view_not_prc().to_owned(),
                    Unassembled::Unreadable(detail) => t::mesh_unreadable(&detail),
                };
                // ui-text-exempt: diagnostic trace, never displayed
                crate::diag::trace(|| format!("model-view-refit-refused fit={}", TOKENS[i]));
                self.fit_refused = Some(tf::mesh_fit_refused(tf::mesh_fit_names()[i], &why));
            }
        }
    }
}

/// A sentence naming a list of parts.
type PartsLine = fn(&[String]) -> String;

/// The lines below the picture saying what the model drawn inferred about
/// its compressed meshes.
pub(super) fn disclose(ui: &mut Ui, model: &Assembled, refused: Option<&str>) {
    let d = &model.drawn;
    if let Some(refused) = refused {
        ui.small(refused);
    }
    if model.opened_in_part && d.partial > 0 {
        ui.small(tf::mesh_fit_fell_back());
    }
    if model.skipped > 0 {
        ui.small(t::mesh_skipped(model.skipped));
    }
    for (why, n) in &d.skipped_why {
        ui.small(tf::mesh_left_out_why(why, *n));
    }
    if model.best_fit > 0 {
        ui.small(t::mesh_best_fit(model.best_fit));
    }
    if d.partial > 0 {
        let (got, of) = d.partial_triangles;
        ui.small(tf::mesh_partial(d.partial, got, of));
        for (why, n) in &d.partial_why {
            ui.small(tf::mesh_partial_why(why, *n));
        }
    }
    let lists: [(&[String], PartsLine); 3] = [
        (&d.left_out_parts, tf::mesh_left_out_parts),
        (&d.best_fit_parts, tf::mesh_best_fit_parts),
        (&d.partial_parts, tf::mesh_partial_parts),
    ];
    for (parts, line) in lists {
        if !parts.is_empty() {
            ui.small(line(parts));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_fit_round_trips_through_its_index() {
        for (i, fit) in FITS.into_iter().enumerate() {
            assert_eq!(index_of(fit), i);
        }
        assert_eq!(FITS[0], MeshFit::default());
        assert_eq!(tf::mesh_fit_names().len(), FITS.len());
    }
}
