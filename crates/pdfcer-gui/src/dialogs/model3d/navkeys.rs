//! The options' Controls tab: the ready layouts as cards, the swap of the
//! turning and strafing sides, a chip per key slot that takes the next key
//! pressed, and the mouse's choices. Every change is kept and traced as the
//! other switches are. Design: `DESIGNS.md` § O411 / O412.

use egui::{Event, Key, Response, RichText, Sense, Ui};

use super::ModelView;
use super::bindings::{Act, Drag, KEYS, Preset, Run, SENSITIVITY, SLOTS};
use crate::text::panels::modelkeys as n;

/// A ready layout's card; the suffix is its word.
pub const REGION_PRESET: &str = "model3d.controls.preset."; // ui-text-exempt: trace region name, never displayed
/// The swap switch.
pub const REGION_SWAP: &str = "model3d.controls.swap"; // ui-text-exempt: trace region name, never displayed
/// A key chip: the suffix is the action's word, `_`, and the slot.
pub const REGION_CHIP: &str = "model3d.controls.key."; // ui-text-exempt: trace region name, never displayed

impl ModelView {
    /// The Controls tab.
    pub(super) fn controls_tab(&mut self, ui: &mut Ui) {
        if self.nav.walk.is_none() {
            ui.label(RichText::new(n::walk_only()).small());
        }
        self.preset_cards(ui);
        let mut swap = self.nav.prefs.bind.swap_sides;
        let r = ui
            .checkbox(&mut swap, n::swap())
            .on_hover_text(n::swap_tooltip());
        crate::diag::ui_rect_visible(REGION_SWAP, r.rect, ui.clip_rect());
        if r.changed() {
            self.nav.prefs.bind.swap_sides = swap;
            self.switched(ui.ctx());
        }
        ui.separator();
        ui.label(n::keys());
        self.take_key(ui);
        self.key_grid(ui);
        ui.separator();
        ui.label(n::mouse());
        self.mouse_rows(ui);
    }

    /// The three cards, the one in use outlined, and which is in use.
    fn preset_cards(&mut self, ui: &mut Ui) {
        ui.label(n::presets());
        let now = self.nav.prefs.bind.matches();
        let names = n::preset_names();
        for ((p, name), note) in Preset::ALL.into_iter().zip(names).zip(n::preset_notes()) {
            let card = card(ui, now == Some(p), name, note);
            // ui-text-exempt: trace region name, never displayed
            let region = format!("{REGION_PRESET}{}", p.word());
            crate::diag::ui_rect_visible(&region, card.rect, ui.clip_rect());
            if card.clicked() {
                self.nav.prefs.bind.apply(p);
                if p == Preset::Avatar {
                    self.nav.prefs.behind = true;
                    self.nav.arm = None;
                }
                self.switched(ui.ctx());
            }
        }
        let name = now.map_or(n::custom(), |p| names[p as usize]);
        ui.label(n::in_use(name));
    }

    /// Each action's row: its name and a chip per slot.
    fn key_grid(&mut self, ui: &mut Ui) {
        egui::Grid::new("model3d-controls-keys") // ui-text-exempt: an egui id, never displayed
            .num_columns(1 + SLOTS)
            .striped(true)
            .show(ui, |ui| {
                for (act, name) in Act::ALL.into_iter().zip(n::acts()) {
                    ui.label(name);
                    for slot in 0..SLOTS {
                        self.chip(ui, act, slot);
                    }
                    ui.end_row();
                }
            });
    }

    /// One key slot's chip: its key, or the wait for one while chosen.
    fn chip(&mut self, ui: &mut Ui, act: Act, slot: usize) {
        let waiting = self.nav.capturing == Some((act, slot));
        let key = self.nav.prefs.bind.keys[act as usize][slot];
        let text = if waiting {
            n::press_key()
        } else {
            key.map_or(n::no_key(), n::key)
        };
        let r = ui
            .add(
                egui::Button::new(text)
                    .selected(waiting)
                    .min_size(egui::vec2(72.0, 0.0)),
            )
            .on_hover_text(n::chip_tooltip());
        // ui-text-exempt: trace region name, never displayed
        let region = format!("{REGION_CHIP}{}_{slot}", act.word());
        crate::diag::ui_rect_visible(&region, r.rect, ui.clip_rect());
        if r.clicked() {
            self.nav.capturing = (!waiting).then_some((act, slot));
        }
    }

    /// While a chip waits, the frame's first key press: Escape stops the
    /// wait, Delete clears the slot, a key a binding may name takes it. The
    /// press is used up, so it neither walks nor closes anything.
    fn take_key(&mut self, ui: &mut Ui) {
        let Some((act, slot)) = self.nav.capturing else {
            return;
        };
        let pressed = ui.input_mut(|i| {
            let key = i.events.iter().find_map(|e| match e {
                Event::Key {
                    key,
                    pressed: true,
                    repeat: false,
                    ..
                } => Some(*key),
                _ => None,
            })?;
            i.consume_key(i.modifiers, key);
            Some(key)
        });
        let chosen = match pressed {
            None => return,
            Some(Key::Escape) => {
                self.nav.capturing = None;
                return;
            }
            Some(Key::Delete) => None,
            Some(key) if KEYS.contains(&key) => Some(key),
            Some(_) => return,
        };
        self.nav.capturing = None;
        self.nav.prefs.bind.bind(act, slot, chosen);
        self.switched(ui.ctx());
    }

    /// The buttons' drags, both buttons, free look, inversion, sensitivity
    /// and the run modifier.
    fn mouse_rows(&mut self, ui: &mut Ui) {
        let mut b = self.nav.prefs.bind;
        let mut changed = false;
        let drags = n::drags();
        let mut drag = |ui: &mut Ui, label: &str, into: &mut Drag| {
            let r = egui::ComboBox::from_label(label)
                .selected_text(drags[*into as usize])
                .show_ui(ui, |ui| {
                    for (d, text) in Drag::ALL.into_iter().zip(drags) {
                        changed |= ui.selectable_value(into, d, text).changed();
                    }
                });
            r.response
        };
        drag(ui, n::left_drag(), &mut b.left_drag);
        drag(ui, n::right_drag(), &mut b.right_drag);
        changed |= ui
            .checkbox(&mut b.both_forward, n::both_forward())
            .changed();
        changed |= ui
            .checkbox(&mut b.free_look, n::free_look())
            .on_hover_text(n::free_look_tooltip())
            .changed();
        changed |= ui.checkbox(&mut b.invert_y, n::invert_y()).changed();
        let slider = ui.add(
            egui::Slider::new(&mut b.sensitivity, SENSITIVITY)
                .logarithmic(true)
                .text(n::sensitivity()),
        );
        let runs = n::runs();
        egui::ComboBox::from_label(n::run())
            .selected_text(runs[b.run as usize])
            .show_ui(ui, |ui| {
                for (r, text) in Run::ALL.into_iter().zip(runs) {
                    changed |= ui.selectable_value(&mut b.run, r, text).changed();
                }
            });
        self.nav.prefs.bind = b;
        if changed || slider.drag_stopped() || slider.lost_focus() {
            self.switched(ui.ctx());
        }
    }
}

/// A ready layout's card: its name over its note, outlined while in use.
fn card(ui: &mut Ui, on: bool, name: &str, note: &str) -> Response {
    let stroke = if on {
        let (accent, _) = egui_shell::theme::Theme::accent_pair(ui.ctx());
        egui::Stroke::new(2.0, accent)
    } else {
        ui.visuals().widgets.noninteractive.bg_stroke
    };
    let shown = egui::Frame::group(ui.style())
        .stroke(stroke)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(name);
            ui.label(RichText::new(note).small());
        });
    let r = ui
        .interact(
            shown.response.rect,
            ui.id().with(("controls-preset", name)),
            Sense::click(),
        )
        .on_hover_text(n::preset_tooltip());
    if r.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    r
}
