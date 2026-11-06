//! The options menu's Measure tab, the readout it shares with the footer, and
//! keeping and deleting a saved view's measurements. `set_3d_views` replaces the
//! whole list, so both rewrite every view from what the engine reads of it, and
//! say so. The viewer re-reads the list when the document's edit epoch moves, so
//! a refusal, an undo or a redo shows here as in the file.

use egui::{Rect, RichText, Ui, Vec2};
use pdfcer_core::threed::{ThreeDMeasureKind, ThreeDMeasurement, ThreeDSavedView};
use pdfcer_gui_base::icons::{self, Icon};

use super::ModelView;
use super::measure::{KINDS, Said, picks_needed};
use crate::app::state::OpenDoc;
use crate::panels::attachments::models::has_own_poster;
use crate::text::model3dmeasure as t;
use crate::text::panels::models as tm;

/// The switch.
pub const REGION_SWITCH: &str = "model3d.measure.on"; // ui-text-exempt: trace region name, never displayed
/// The glyph over the picture.
pub const REGION_GLYPH: &str = "model3d.measure"; // ui-text-exempt: trace region name, never displayed
/// One kind; the suffix is its index in [`KINDS`].
pub const REGION_KIND_PREFIX: &str = "model3d.measure.kind."; // ui-text-exempt: trace region name, never displayed
/// *Keep on this view*.
pub const REGION_KEEP: &str = "model3d.measure.keep"; // ui-text-exempt: trace region name, never displayed
/// *Start again*.
pub const REGION_START_AGAIN: &str = "model3d.measure.again"; // ui-text-exempt: trace region name, never displayed
/// One saved measurement's delete button; the suffix is its index in the view.
pub const REGION_DELETE_PREFIX: &str = "model3d.measure.delete."; // ui-text-exempt: trace region name, never displayed

/// The full-screen readout's width and its place from the picture's corner.
const CARD_WIDTH: f32 = 320.0;
const CARD_INSET: Vec2 = Vec2::new(8.0, -8.0);
/// The value's text size, in points.
const VALUE_SIZE: f32 = 16.0;
/// The colour written with a kept measurement, since an unset one reads white.
const KEPT_COLOUR: [f64; 3] = [0.9, 0.235, 0.157];
/// Each kind's glyph, in [`KINDS`]' order.
const KIND_ICONS: [Icon; 5] = [
    Icon::MeasureLength,
    Icon::Measure,
    Icon::MeasureAngle,
    Icon::MeasureRadius,
    Icon::Comment,
];

/// `saved` as a list the engine will write: a view with no name takes its place
/// in the list, since the engine refuses an empty one.
pub(super) fn writable_views(saved: &[ThreeDSavedView]) -> Vec<ThreeDSavedView> {
    let mut views = saved.to_vec();
    for (i, view) in views.iter_mut().enumerate() {
        if view.name.trim().is_empty() {
            view.name = t::unnamed_view(i);
        }
    }
    views
}

/// Moves `old`'s measurements onto the views of `views` that share a name, and
/// appends a view holding measurements that none shares, so replacing the list
/// loses none.
pub(super) fn carry_measurements(views: &mut Vec<ThreeDSavedView>, old: &[ThreeDSavedView]) {
    for view in old.iter().filter(|v| !v.measurements.is_empty()) {
        match views.iter_mut().find(|v| v.name == view.name) {
            Some(same) => same.measurements.clone_from(&view.measurements),
            None => views.push(view.clone()),
        }
    }
}

impl ModelView {
    /// The saved view measurements belong to: the one last chosen, else the
    /// file's default, else its first.
    pub(super) fn active_view(&self) -> Option<usize> {
        let count = self.saved.views.len();
        self.saved_chosen
            .or(self.saved.default)
            .filter(|i| *i < count)
            .or((count > 0).then_some(0))
    }

    /// Read the file's saved views again when the document has changed. Called
    /// once a frame while the viewer is open.
    pub(crate) fn sync_saved(&mut self, doc: &OpenDoc) {
        let keeping = std::mem::take(&mut self.measure.keeping);
        if self.measure.epoch == Some(doc.edit_epoch) {
            return;
        }
        let first = self.measure.epoch.is_none();
        self.measure.epoch = Some(doc.edit_epoch);
        if first {
            return;
        }
        let chosen = self
            .saved_chosen
            .and_then(|i| self.saved.views.get(i))
            .map(|v| v.name.clone());
        self.saved = pdfcer_core::threed::saved_3d_views(&doc.session.view(), &self.artwork);
        self.saved_chosen =
            chosen.and_then(|name| self.saved.views.iter().position(|v| v.name == name));
        if keeping {
            self.measure.clear();
        }
        let listed = self
            .active_view()
            .map_or(0, |i| self.saved.views[i].measurements.len());
        // ui-text-exempt: diagnostic trace, never displayed
        crate::diag::trace(|| format!("model-measure-saved count={listed}"));
    }

    /// The options menu's tab.
    pub(super) fn measure_tab(&mut self, ui: &mut Ui) {
        let mut on = self.measure.on;
        let switch = ui
            .checkbox(&mut on, t::MEASURE)
            .on_hover_text(t::MEASURE_TOOLTIP);
        crate::diag::ui_rect_visible(REGION_SWITCH, switch.rect, ui.clip_rect());
        if on != self.measure.on {
            self.measure.switch(on);
        }
        ui.horizontal_wrapped(|ui| self.kind_row(ui));
        if self.measure.kind() == ThreeDMeasureKind::Comment {
            // escape-disposition: keeps-draft — the comment lives in `Measure`, so
            // the key leaves the box and the words stay for the next pick.
            ui.add(
                egui::TextEdit::singleline(&mut self.measure.comment).hint_text(t::COMMENT_HINT),
            );
        }
        ui.separator();
        self.saved_measurements(ui);
    }

    fn kind_row(&mut self, ui: &mut Ui) {
        for (i, name) in t::KINDS.into_iter().enumerate() {
            let chosen = self.measure.kind == i;
            let image = if chosen {
                icons::selected_image(ui, KIND_ICONS[i])
            } else {
                icons::image(ui, KIND_ICONS[i])
            };
            let kind = ui
                .add(
                    egui::Button::image_and_text(image.fit_to_exact_size(Vec2::splat(18.0)), name)
                        .selected(chosen),
                )
                .on_hover_text(t::KIND_TIPS[i]);
            // ui-text-exempt: trace region name, never displayed
            let region = format!("{REGION_KIND_PREFIX}{i}");
            crate::diag::ui_rect_visible(&region, kind.rect, ui.clip_rect());
            if kind.clicked() && !chosen {
                self.measure.kind = i;
                self.measure.clear();
            }
        }
    }

    /// The readout over the picture where filling the screen leaves no footer
    /// to say it in.
    pub(super) fn measure_card(&mut self, ctx: &egui::Context, rect: Rect) {
        if !self.measure.on || !self.filling_screen(ctx) {
            return;
        }
        egui::Area::new(egui::Id::new("model3d-measure-card")) // ui-text-exempt: an egui id, never displayed
            .order(egui::Order::Foreground)
            .pivot(egui::Align2::LEFT_BOTTOM)
            .fixed_pos(rect.left_bottom() + CARD_INSET)
            .show(ctx, |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    ui.set_max_width(CARD_WIDTH);
                    self.measure_readout(ui);
                });
            });
    }

    /// While measuring: the next pick, the value made, and keeping it.
    pub(super) fn measure_readout(&mut self, ui: &mut Ui) {
        if !self.measure.on {
            return;
        }
        match &self.measure.result {
            Some(m) => {
                ui.label(RichText::new(m.value_text().unwrap_or_default()).size(VALUE_SIZE));
                if let Some(deltas) = deltas_of(m) {
                    ui.small(deltas);
                }
                self.unit_note(ui, m.kind);
            }
            None => {
                let done = self
                    .measure
                    .picked
                    .len()
                    .min(picks_needed(self.measure.kind()));
                ui.small(t::next_pick(self.measure.kind, done));
                if let Some(text) = self.measure.preview.as_ref().and_then(|m| m.value_text()) {
                    ui.label(RichText::new(text).size(VALUE_SIZE));
                }
            }
        }
        if let Some(said) = self.measure.said {
            ui.small(match said {
                Said::Missed => t::MISSED,
                Said::Refused => t::REFUSED,
                Said::NoComment => t::NO_COMMENT,
                Said::NotKept => t::NOT_KEPT,
            });
        }
        ui.horizontal_wrapped(|ui| self.readout_buttons(ui));
    }

    fn unit_note(&self, ui: &mut Ui, kind: ThreeDMeasureKind) {
        let plain = matches!(
            kind,
            ThreeDMeasureKind::Angular | ThreeDMeasureKind::Comment
        );
        if !plain && super::measure::units_word(self.model.drawn.units).is_empty() {
            ui.small(t::NO_UNIT);
        }
    }

    fn readout_buttons(&mut self, ui: &mut Ui) {
        if self.measure.result.is_some() && has_own_poster(&self.artwork) {
            let keep = ui.button(t::KEEP).on_hover_text(t::KEEP_TOOLTIP);
            crate::diag::ui_rect_visible(REGION_KEEP, keep.rect, ui.clip_rect());
            if keep.clicked() {
                self.keep_measurement();
            }
        }
        if self.measure.result.is_some() || !self.measure.picked.is_empty() {
            let again = ui
                .button(t::START_AGAIN)
                .on_hover_text(t::START_AGAIN_TOOLTIP);
            crate::diag::ui_rect_visible(REGION_START_AGAIN, again.rect, ui.clip_rect());
            if again.clicked() {
                self.measure.clear();
            }
        }
    }

    /// The active view's measurements, each deletable where the file can be
    /// written.
    fn saved_measurements(&mut self, ui: &mut Ui) {
        let Some(index) = self.active_view() else {
            ui.small(t::NONE_KEPT);
            return;
        };
        let view = &self.saved.views[index];
        if view.measurements.is_empty() {
            ui.small(t::NONE_KEPT);
            return;
        }
        ui.small(t::saved_on(&view.name));
        let writable = has_own_poster(&self.artwork);
        let mut delete = None;
        for (i, m) in view.measurements.iter().enumerate() {
            ui.horizontal_wrapped(|ui| {
                if writable {
                    let button = ui.small_button(t::DELETE).on_hover_text(t::DELETE_TOOLTIP);
                    // ui-text-exempt: trace region name, never displayed
                    let region = format!("{REGION_DELETE_PREFIX}{i}");
                    crate::diag::ui_rect_visible(&region, button.rect, ui.clip_rect());
                    if button.clicked() {
                        delete = Some(i);
                    }
                }
                ui.label(listed_line(m));
            });
        }
        if let Some(i) = delete {
            self.delete_measurement(index, i);
        }
    }

    fn keep_measurement(&mut self) {
        let Some(mut m) = self.measure.result.clone() else {
            return;
        };
        m.color.get_or_insert(KEPT_COLOUR);
        let mut views = writable_views(&self.saved.views);
        let mut notes = Vec::new();
        let target = match self.active_view() {
            Some(i) => {
                notes.push(t::kept_on(&views[i].name));
                notes.push(t::REWRITES_VIEWS.to_owned());
                i
            }
            None => {
                let Some(view) = self.camera_view() else {
                    self.measure.said = Some(Said::NotKept);
                    return;
                };
                views.push(view);
                notes.push(t::NEW_VIEW.to_owned());
                0
            }
        };
        let Some(view) = views.get_mut(target) else {
            return;
        };
        m.name = Some(t::saved_name(
            self.measure.kind,
            view.measurements.len() + 1,
        ));
        view.measurements.push(m);
        let total = view.measurements.len();
        let name = view.name.clone();
        // ui-text-exempt: diagnostic trace, never displayed
        crate::diag::trace(|| format!("model-measure-keep view={name} count={total}"));
        let default = self.saved.default.unwrap_or(0).min(views.len() - 1);
        self.views = Some((views, default, notes));
        self.measure.keeping = true;
    }

    /// A saved view of this camera, named as *Save views* names one.
    fn camera_view(&self) -> Option<ThreeDSavedView> {
        let aspect = self.file_aspect();
        let camera = self.flat_camera(aspect).ok()?;
        ThreeDSavedView::from_camera(tm::view_custom(), &camera, aspect).map(|v| self.lit(v))
    }

    fn delete_measurement(&mut self, view: usize, measurement: usize) {
        let mut views = writable_views(&self.saved.views);
        let Some(target) = views.get_mut(view) else {
            return;
        };
        if measurement >= target.measurements.len() {
            return;
        }
        target.measurements.remove(measurement);
        let notes = vec![t::deleted_from(&target.name), t::REWRITES_VIEWS.to_owned()];
        // ui-text-exempt: diagnostic trace, never displayed
        crate::diag::trace(|| format!("model-measure-delete index={measurement}"));
        let default = self.saved.default.unwrap_or(0).min(views.len() - 1);
        self.views = Some((views, default, notes));
    }
}

/// A saved measurement's line in the list: its name and its value.
fn listed_line(m: &ThreeDMeasurement) -> String {
    let value = m.value_text().unwrap_or_default();
    match m.name.as_deref().filter(|n| !n.is_empty()) {
        Some(name) if !value.is_empty() => t::named_value(name, &value),
        Some(name) => name.to_owned(),
        None => value,
    }
}

/// How far a distance runs along each axis.
fn deltas_of(m: &ThreeDMeasurement) -> Option<String> {
    let linear = m.kind == KINDS[0];
    let (a1, a2) = (m.a1.filter(|_| linear)?, m.a2?);
    Some(t::along_axes(a2[0] - a1[0], a2[1] - a1[1], a2[2] - a1[2]))
}

#[cfg(test)]
mod tests {
    use super::*;

    const AXES: [f64; 12] = [1., 0., 0., 0., 1., 0., 0., 0., 1., 0., 0., 5.];

    fn view(name: &str, measured: bool) -> ThreeDSavedView {
        let v = ThreeDSavedView::new(name, AXES);
        if measured {
            let m = ThreeDMeasurement::linear([0.0; 3], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0], "mm")
                .unwrap();
            v.with_measurement(m)
        } else {
            v
        }
    }

    #[test]
    fn replacing_the_list_loses_no_measurement() {
        let old = [
            view("Front", true),
            view("Plain", false),
            view("Mine", true),
        ];
        let mut views = vec![view("Front", false), view("Top", false)];
        carry_measurements(&mut views, &old);
        let names: Vec<_> = views.iter().map(|v| v.name.as_str()).collect();
        assert_eq!(names, ["Front", "Top", "Mine"]);
        assert_eq!(views[0].measurements.len(), 1);
        assert_eq!(views[1].measurements.len(), 0);
        assert_eq!(views[2].measurements.len(), 1);
    }

    #[test]
    fn a_view_with_no_name_gets_one_the_engine_will_write() {
        let views = writable_views(&[view("", false), view("Front", false)]);
        assert_eq!(views[0].name, t::unnamed_view(0));
        assert_eq!(views[1].name, "Front");
    }
}
