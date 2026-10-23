//! # `dialogs::compact_target` — the compact window's starting point and size target
//!
//! Contract: [`rows`] draws *Start from*, the engine's optimize presets, and
//! *Fit under*, a byte budget, and says whether either changed. The preset is
//! the base the window's own switches are laid over (see
//! `dialogs::recompress::settings_for`); the budget makes the copy go through
//! `EditSession::optimize_to_budget`, whose tries [`done_lines`] reports.
//! A budget dragged is acted on when the drag ends, because each change
//! restarts a search that writes the whole file up to six times.

use egui::Ui;
use pdfcer_core::optimize::Preset;

use crate::dialogs::recompress::Budget;
use crate::text::compact_target as t;

/// The starting-point list; each entry is `<this>.<Preset::as_str>`.
pub const REGION_PRESET: &str = "compact.preset";
/// The size-target switch.
pub const REGION_TARGET: &str = "compact.target";
/// The size box beside it.
pub const REGION_TARGET_SIZE: &str = "compact.target.size";
/// The line saying whether the copy fits.
pub const REGION_TARGET_SAID: &str = "compact.target.said";

/// Where a newly opened window starts: which command opened it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Start {
    /// *Save a compacted copy*: Lossless, no target; nothing on a page changes.
    Plain,
    /// *Reduce file size*: the Standard preset.
    Reduce,
    /// *Fit to a size*: Lossless with the target on at its opening size.
    Fit,
}

impl Start {
    /// The opening preset and budget, for a plain copy of `plain` bytes.
    #[must_use]
    pub fn choices(self, plain: u64) -> (Preset, Option<u64>) {
        match self {
            Self::Plain => (Preset::Lossless, None),
            Self::Reduce => (Preset::Standard, None),
            Self::Fit => (Preset::Lossless, Some(opening_budget(plain))),
        }
    }
}

const MIB: f64 = 1024.0 * 1024.0;
/// The smallest target offered, in MB.
const FLOOR_MB: f64 = 0.1;

/// The target a newly switched-on size box opens at: half of `plain`, the
/// plain compacted copy, so the first search has something to do.
#[must_use]
pub fn opening_budget(plain: u64) -> u64 {
    let mb = (plain as f64 / MIB / 2.0 * 10.0).round() / 10.0;
    to_bytes(mb)
}

fn to_bytes(mb: f64) -> u64 {
    (mb.max(FLOOR_MB) * MIB).round() as u64
}

/// The two rows; whether either changed in a way that needs the work redone.
pub fn rows(ui: &mut Ui, preset: &mut Preset, budget: &mut Option<u64>, plain: u64) -> bool {
    let picked = preset_row(ui, preset);
    let target = target_row(ui, budget, plain);
    picked || target
}

fn preset_row(ui: &mut Ui, preset: &mut Preset) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(t::preset_label());
        let list = egui::ComboBox::from_id_salt("compact-preset") // ui-text-exempt: an id, never displayed
            .selected_text(t::preset(*preset))
            .show_ui(ui, |ui| {
                for choice in Preset::ALL {
                    let item = ui
                        .selectable_label(*preset == choice, t::preset(choice))
                        .on_hover_text(t::preset_tooltip(choice));
                    let region = format!("{REGION_PRESET}.{}", choice.as_str());
                    crate::diag::ui_rect_visible(&region, item.rect, ui.clip_rect());
                    if item.clicked() && *preset != choice {
                        *preset = choice;
                        changed = true;
                    }
                }
            });
        crate::diag::ui_rect_visible(REGION_PRESET, list.response.rect, ui.clip_rect());
        list.response.on_hover_text(t::preset_tooltip(*preset));
    });
    changed
}

fn target_row(ui: &mut Ui, budget: &mut Option<u64>, plain: u64) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        let mut on = budget.is_some();
        let switch = ui
            .checkbox(&mut on, t::target_toggle())
            .on_hover_text(t::target_tooltip());
        crate::diag::ui_rect_visible(REGION_TARGET, switch.rect, ui.clip_rect());
        if switch.changed() {
            *budget = on.then(|| opening_budget(plain));
            changed = true;
        }
        if let Some(max) = budget {
            let mut mb = (*max as f64 / MIB * 10.0).round() / 10.0;
            let size = ui.add(
                egui::DragValue::new(&mut mb)
                    .range(FLOOR_MB..=1_000_000.0)
                    .speed(0.1)
                    .max_decimals(1)
                    .update_while_editing(false),
            );
            ui.label(t::target_unit());
            crate::diag::ui_rect_visible(REGION_TARGET_SIZE, size.rect, ui.clip_rect());
            if size.changed() {
                *max = to_bytes(mb);
                changed |= !size.dragged();
            }
            changed |= size.drag_stopped();
        }
    });
    changed
}

/// Whether the copy fits, then each try.
pub fn done_lines(ui: &mut Ui, budget: &Budget) {
    let said = ui.label(t::target_result(
        budget.max,
        budget.met,
        budget.chosen.as_ref(),
    ));
    crate::diag::ui_rect_visible(REGION_TARGET_SAID, said.rect, ui.clip_rect());
    for attempt in &budget.attempts {
        ui.small(t::target_attempt(attempt));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_opening_target_is_half_the_copy_to_a_tenth_of_a_megabyte() {
        assert_eq!(opening_budget(10 * 1024 * 1024), to_bytes(5.0));
        assert_eq!(opening_budget(3 * 1024 * 1024), to_bytes(1.5));
    }

    #[test]
    fn a_tiny_copy_opens_at_the_floor() {
        assert_eq!(opening_budget(1000), to_bytes(FLOOR_MB));
    }
}
