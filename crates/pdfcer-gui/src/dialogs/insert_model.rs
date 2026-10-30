//! The Insert 3D model window: when the placed model starts, the opening
//! view's lighting, its poster, and for a glTF file how it is converted.
//! Place hands the choices to `app::actions::model_insert`.

use egui::Ui;
use pdfcer_core::threed::{ThreeDActivation, ThreeDLighting};
use pdfcer_gui_base::editactions::ModelPlacement;

use crate::app::actions::Action;
use crate::app::actions::attachments::AttachmentAction;
use crate::app::state::Status;
use crate::text::model3d as t;

/// The window body's rect, for `ui-verify`.
// ui-text-exempt: trace region name, never displayed
pub const REGION_BODY: &str = "insert-model.body";
/// The Place button.
// ui-text-exempt: trace region name, never displayed
pub const REGION_PLACE: &str = "insert-model.place";
/// The "when the page opens" choice.
// ui-text-exempt: trace region name, never displayed
pub const REGION_PAGE_OPEN: &str = "insert-model.activation.page-open";
/// The plain-poster choice.
// ui-text-exempt: trace region name, never displayed
pub const REGION_POSTER_PLAIN: &str = "insert-model.poster.plain";
/// The first lighting scheme after Automatic, CAD lights, as a choice beside
/// the list, so a check can pick a scheme without opening the list.
// ui-text-exempt: trace region name, never displayed
pub const REGION_LIGHTING: &str = "insert-model.lighting";

const ACTIVATIONS: [ThreeDActivation; 3] = [
    ThreeDActivation::Click,
    ThreeDActivation::PageOpen,
    ThreeDActivation::PageVisible,
];

/// The window's state.
pub struct InsertModelDialog {
    page: usize,
    name: String,
    gltf: bool,
    placement: ModelPlacement,
    close_requested: bool,
    place_requested: bool,
}

impl InsertModelDialog {
    /// Draw it. Returns whether it stays open.
    pub fn show(&mut self, ctx: &egui::Context, actions: &mut Vec<Action>) -> bool {
        let (frame, ()) = crate::dialogs::host::Host::new(
            "insert-model", // ui-text-exempt: a viewport key, never displayed.
            t::insert_title(),
            egui::vec2(460.0, 360.0),
            egui::vec2(360.0, 280.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            self.body(ui);
        });
        if std::mem::take(&mut self.place_requested) {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    "insert-model-placed page={} activation={:?} lighting={:?} poster={} gltf={}",
                    self.page,
                    self.placement.activation,
                    self.placement.lighting,
                    self.placement.render_poster,
                    self.gltf
                )
            });
            actions.push(Action::Attachment(AttachmentAction::InsertModel {
                page: self.page,
                placement: self.placement.clone(),
            }));
            return false;
        }
        !frame.closed && !std::mem::take(&mut self.close_requested)
    }

    fn body(&mut self, ui: &mut Ui) {
        ui.label(t::insert_file(&self.name));
        ui.label(t::insert_where(self.page));
        ui.add_space(8.0);
        egui::Grid::new("insert-model.grid") // ui-text-exempt: a widget id, never displayed
            .num_columns(2)
            .spacing([12.0, 8.0])
            .show(ui, |ui| {
                ui.label(t::activation_label())
                    .on_hover_text(t::activation_tooltip());
                self.activation(ui);
                ui.end_row();
                ui.label(t::lighting_label())
                    .on_hover_text(t::lighting_tooltip());
                self.lighting(ui);
                ui.end_row();
                ui.label(t::poster_label())
                    .on_hover_text(t::poster_tooltip());
                self.poster(ui);
                ui.end_row();
            });
        if self.gltf {
            ui.add_space(8.0);
            ui.label(t::gltf_heading());
            ui.checkbox(&mut self.placement.gltf_linear, t::gltf_linear())
                .on_hover_text(t::gltf_linear_tooltip());
            ui.checkbox(&mut self.placement.gltf_z_up, t::gltf_z_up())
                .on_hover_text(t::gltf_z_up_tooltip());
        }
        ui.add_space(12.0);
        ui.separator();
        ui.horizontal(|ui| {
            let place = ui.button(t::place_button());
            crate::diag::ui_rect_visible(REGION_PLACE, place.rect, ui.clip_rect());
            if place.clicked() {
                self.place_requested = true;
            }
            if ui.button(t::cancel_button()).clicked() {
                self.close_requested = true;
            }
        });
    }

    fn activation(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            for a in ACTIVATIONS {
                let r = ui.radio_value(&mut self.placement.activation, a, t::activation(a));
                if a == ThreeDActivation::PageOpen {
                    crate::diag::ui_rect_visible(REGION_PAGE_OPEN, r.rect, ui.clip_rect());
                }
            }
        });
    }

    fn lighting(&mut self, ui: &mut Ui) {
        let shown = self
            .placement
            .lighting
            .map_or_else(t::lighting_auto, t::lighting);
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt("insert-model.lighting.list") // ui-text-exempt: a widget id, never displayed
                .selected_text(shown)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.placement.lighting, None, t::lighting_auto());
                    for scheme in t::lighting_schemes() {
                        ui.selectable_value(
                            &mut self.placement.lighting,
                            Some(scheme),
                            t::lighting(scheme),
                        );
                    }
                });
            let cad = Some(ThreeDLighting::Cad);
            let r = ui.selectable_label(
                self.placement.lighting == cad,
                t::lighting(ThreeDLighting::Cad),
            );
            crate::diag::ui_rect_visible(REGION_LIGHTING, r.rect, ui.clip_rect());
            if r.clicked() {
                self.placement.lighting = cad;
            }
        });
    }

    fn poster(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.radio_value(
                &mut self.placement.render_poster,
                true,
                t::poster_rendered(),
            );
            let plain = ui.radio_value(&mut self.placement.render_poster, false, t::poster_plain());
            crate::diag::ui_rect_visible(REGION_POSTER_PLAIN, plain.rect, ui.clip_rect());
        });
    }
}

/// Whether `path` names a glTF model, by its ending.
fn is_gltf(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("gltf") || e.eq_ignore_ascii_case("glb"))
}

/// The window for `source` on `page` of the open document.
pub fn open_for(
    status: &Status,
    source: std::path::PathBuf,
    page: usize,
) -> Option<InsertModelDialog> {
    let Status::Open(doc) = status else {
        return None;
    };
    if page >= doc.pages.len() {
        return None;
    }
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        format!("insert-model-opened page={page}")
    });
    Some(InsertModelDialog {
        page,
        name: crate::dialogs::sign::file_name_of(&source),
        gltf: cfg!(feature = "3d") && is_gltf(&source),
        placement: ModelPlacement {
            source,
            activation: ThreeDActivation::default(),
            lighting: None,
            render_poster: true,
            gltf_linear: true,
            gltf_z_up: true,
        },
        close_requested: false,
        place_requested: false,
    })
}
