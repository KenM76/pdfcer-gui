//! # `dialogs::model3d::saved` — the views saved in the file
//!
//! Every `/VA` entry `pdfcer_core::threed::saved_3d_views` reads, in a
//! *Saved views* menu beside *File's view*, the file's default marked.
//! Choosing one looks from it and follows its framing while the camera stays
//! framed. A view that saves no camera is greyed with its reason. Writing the
//! list back is *Save views in the file* (`set_3d_views`).

use egui::Ui;
use pdfcer_core::threed::SavedViews;

use super::ModelView;
use super::orbit::Orbit;
use crate::text::panels::savedviews as t;

/// The menu's published region.
pub const REGION_MENU: &str = "model3d.saved_views"; // ui-text-exempt: trace region name, never displayed
/// One item's region; the suffix is its index in the file's `/VA`.
pub const REGION_ITEM_PREFIX: &str = "model3d.saved_view."; // ui-text-exempt: trace region name, never displayed

/// The listing line, traced once as the viewer opens.
pub(super) fn trace_listed(saved: &SavedViews) {
    let count = saved.views.len();
    let cameras = saved
        .views
        .iter()
        .filter(|v| Orbit::saved(v).is_some())
        .count();
    let default = saved.default;
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        format!("model-view-saved-views count={count} cameras={cameras} default={default:?}")
    });
}

impl ModelView {
    /// *Saved views*: offered only when the file saves at least one.
    pub(super) fn saved_views_control(&mut self, ui: &mut Ui) {
        if self.saved.views.is_empty() {
            return;
        }
        let menu = ui
            .menu_button(t::view_saved(self.saved.views.len()), |ui| {
                for i in 0..self.saved.views.len() {
                    self.saved_item(ui, i);
                }
            })
            .response
            .on_hover_text(t::view_saved_tooltip());
        crate::diag::ui_rect_visible(REGION_MENU, menu.rect, ui.clip_rect());
    }

    fn saved_item(&mut self, ui: &mut Ui, i: usize) {
        let view = &self.saved.views[i];
        let orbit = Orbit::saved(view);
        let default = self.saved.default == Some(i);
        let label = t::view_saved_item(&view.name, i, default);
        let item = ui
            .add_enabled(
                orbit.is_some(),
                egui::Button::selectable(self.saved_chosen == Some(i), label),
            )
            .on_disabled_hover_text(t::view_saved_no_camera());
        crate::diag::ui_rect_visible(
            // ui-text-exempt: trace region name, never displayed
            &format!("{REGION_ITEM_PREFIX}{i}"),
            item.rect,
            ui.clip_rect(),
        );
        let Some(orbit) = orbit.filter(|_| item.clicked()) else {
            return;
        };
        self.orbit = orbit;
        self.aimed = Some(view.clone());
        self.saved_chosen = Some(i);
        let (perspective, framed) = (orbit.perspective, orbit.framed);
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "model-view-saved-view index={i} default={} perspective={} framed={}",
                u8::from(default),
                u8::from(perspective),
                u8::from(framed)
            )
        });
        ui.close();
    }
}
