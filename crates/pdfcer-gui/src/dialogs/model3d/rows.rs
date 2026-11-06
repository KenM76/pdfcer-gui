//! The viewer window's view rows: *File's view* and its note, the *Up* and
//! *Front looks along* drop-downs, and *Save views in the file*.

use egui::Ui;
use pdfcer_core::threed::ThreeDSavedView;
use pdfcer_gui_base::icons::Icon;

use super::axes::{self, Axis};
use super::orbit::Orbit;
use super::{
    ModelView, REGION_FILE_VIEW, REGION_FRONT, REGION_SAVE_VIEWS, REGION_UP, REGION_VIEWS_LIGHTING,
    phone,
};
use crate::text::panels::models as t;

impl ModelView {
    /// *File's view*: back to the camera the file opens on, offered only
    /// when its opening view carries one.
    pub(super) fn file_view_control(&mut self, ui: &mut Ui) {
        let Some(saved) = self.opening.as_ref().and_then(Orbit::saved) else {
            return;
        };
        let button = ui
            .button(t::view_file())
            .on_hover_text(t::view_file_tooltip());
        crate::diag::ui_rect_visible(REGION_FILE_VIEW, button.rect, ui.clip_rect());
        if button.clicked() {
            self.orbiting();
            self.orbit = saved;
            self.aimed = self.opening.clone();
            self.saved_chosen = None;
        }
    }

    /// While the file's view fixes the framing, how its scale was read: the
    /// standard gives the orthographic scale no unit.
    pub(super) fn file_view_note(&self, ui: &mut Ui) {
        if !self.orbit.framed || self.orbit.perspective {
            return;
        }
        let aspect = self.rendered.map_or(1.0, |r| {
            f64::from(r.size[0].max(1)) / f64::from(r.size[1].max(1))
        });
        if let Some(aim) = self.aimed.as_ref().and_then(|v| v.aim(aspect)) {
            ui.small(t::view_file_note(&aim.source()));
        }
    }

    /// *Up* and *Front looks along*. A change re-applies the named view last
    /// chosen in the new axes.
    pub(super) fn axes_control(&mut self, ui: &mut Ui) {
        let was = self.upright;
        let up = axis_choice(
            ui,
            (t::view_up(), t::view_up_tooltip()),
            REGION_UP,
            was.up,
            axes::ALL.into_iter(),
        );
        let front = axis_choice(
            ui,
            (t::view_front(), t::view_front_tooltip()),
            REGION_FRONT,
            was.front,
            was.fronts(),
        );
        let chosen = match (up, front) {
            (Some(up), _) => was.with_up(up),
            (None, Some(front)) => was.with_front(front),
            (None, None) => return,
        };
        if chosen == was {
            return;
        }
        self.upright = chosen;
        self.orbiting();
        self.orbit = Orbit::named(self.named, self.orbit.perspective, chosen);
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "model-view-axes up={} front={} view={}",
                chosen.up.token(),
                chosen.front.token(),
                self.named
            )
        });
    }

    /// *Save views in the file*: the named views about the chosen axes, and
    /// this view when it is none of them, written into the model's file so a
    /// PDF reader lists them and opens the model on this one.
    pub(super) fn views_control(&mut self, ui: &mut Ui) {
        let button = phone::glyph_button(
            ui,
            Icon::Save,
            t::view_save_views(),
            t::view_save_views_tooltip(),
        );
        crate::diag::ui_rect_visible(REGION_SAVE_VIEWS, button.rect, ui.clip_rect());
        self.lighting_combo(ui, REGION_VIEWS_LIGHTING);
        if !button.clicked() {
            return;
        }
        let views = self.file_views();
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            match &views {
                Some((list, default)) => {
                    format!(
                        "model-view-views count={} default={default} lighting={:?}",
                        list.len(),
                        self.view_lighting
                    )
                }
                None => "model-view-views count=0 default=none".to_owned(),
            }
        });
        match views {
            Some(views) => self.views = Some(views),
            None => self.failed = Some(t::views_not_built().to_owned()),
        }
    }

    /// Each named view fitted to the model as the page frames it, then this
    /// view when it is not the named one last chosen; with the index of
    /// this view. `None` when a view cannot be formed.
    fn file_views(&self) -> Option<(Vec<ThreeDSavedView>, usize)> {
        let [w, h] = self.rendered.map_or([4, 3], |r| r.size);
        let aspect = self
            .model
            .aspect
            .unwrap_or_else(|| f64::from(w.max(1)) / f64::from(h.max(1)));
        let perspective = self.orbit.perspective;
        let mut views = Vec::new();
        for (i, name) in t::view_names().into_iter().enumerate() {
            let orbit = Orbit::named(i, perspective, self.upright);
            let camera = orbit.camera(&self.bounds, self.reach, aspect, None).ok()?;
            views.push(self.lit(ThreeDSavedView::from_camera(name, &camera, aspect)?));
        }
        if self.nav.walk.is_none()
            && self.orbit == Orbit::named(self.named, perspective, self.upright)
        {
            return Some((views, self.named));
        }
        let camera = self.flat_camera(aspect).ok()?;
        views.push(self.lit(ThreeDSavedView::from_camera(
            t::view_custom(),
            &camera,
            aspect,
        )?));
        let custom = views.len() - 1;
        Some((views, custom))
    }

    /// `view` with the chosen lighting.
    fn lit(&self, view: ThreeDSavedView) -> ThreeDSavedView {
        match self.view_lighting {
            Some(scheme) => view.with_lighting(scheme),
            None => view,
        }
    }
}

/// A label and an axis drop-down, the button published as `region` and each
/// open item as `region.<index>`. Returns the axis picked this frame.
fn axis_choice(
    ui: &mut Ui,
    (label, tip): (&str, &str),
    region: &str,
    current: Axis,
    options: impl Iterator<Item = Axis>,
) -> Option<Axis> {
    let names = t::axis_names();
    let mut picked = None;
    ui.label(label).on_hover_text(tip);
    let combo = egui::ComboBox::from_id_salt(region)
        .selected_text(names.get(current.index()).copied().unwrap_or_default())
        .show_ui(ui, |ui| {
            for axis in options {
                let name = names.get(axis.index()).copied().unwrap_or_default();
                let item = ui.selectable_label(axis == current, name);
                crate::diag::ui_rect_visible(
                    // ui-text-exempt: trace region name, never displayed
                    &format!("{region}.{}", axis.index()),
                    item.rect,
                    ui.clip_rect(),
                );
                if item.clicked() {
                    picked = Some(axis);
                }
            }
        });
    crate::diag::ui_rect_visible(region, combo.response.rect, ui.clip_rect());
    combo.response.on_hover_text(tip);
    picked
}
