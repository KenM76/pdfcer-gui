//! The Find bar's Replace toggle and Replace row.
//!
//! Contract: drawn only where the mode may edit content (the caller passes
//! `offered`); the buttons are live only while the bar is on a current hit,
//! and pressing one pushes `Action::Find(FindRequest::Replace { .. })` for the
//! app to carry out. Enter in the field replaces the current hit, as in every
//! editor's find-and-replace.

use egui::{Align, Layout, Vec2};

use super::{FIELD_WIDTH_PTS, ROW_HEIGHT_PTS};
use crate::appaction::Action;
use crate::find::{FindRequest, FindState, Readout};
use crate::text::replace as t;

/// The toggle that shows the row.
const REGION_TOGGLE: &str = "find-replace-toggle"; // ui-text-exempt: trace region name, never displayed

/// The whole Replace row.
const REGION_ROW: &str = "find-replace-row"; // ui-text-exempt: trace region name, never displayed

/// The replacement field.
const REGION_FIELD: &str = "find-replace-field"; // ui-text-exempt: trace region name, never displayed

/// The Replace button.
const REGION_ONE: &str = "find-replace-one"; // ui-text-exempt: trace region name, never displayed

/// The Replace all button.
const REGION_ALL: &str = "find-replace-all"; // ui-text-exempt: trace region name, never displayed

/// The replacement field's id.
const FIELD_ID: &str = "pdfcer-replace-field"; // ui-text-exempt: widget id, never displayed

/// The toggle, for the first row's right-to-left group.
pub(super) fn toggle(ui: &mut egui::Ui, state: &mut FindState) {
    let response = ui
        .selectable_label(state.replace_open(), t::toggle())
        .on_hover_text(t::toggle_tooltip());
    if response.clicked() {
        state.toggle_replace();
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!("find-replace-toggled open={}", state.replace_open())
        });
    }
    crate::diag::ui_rect(REGION_TOGGLE, response.rect);
}

/// The second row: the replacement field and the two buttons.
pub(super) fn row(ui: &mut egui::Ui, state: &mut FindState, epoch: u64, actions: &mut Vec<Action>) {
    let live = matches!(state.readout(epoch), Readout::At { .. });
    let size = Vec2::new(super::BAR_WIDTH_PTS, ROW_HEIGHT_PTS);
    let rect = ui
        .allocate_ui_with_layout(size, Layout::left_to_right(Align::Center), |ui| {
            ui.set_min_size(size);
            ui.set_max_size(size);
            ui.label(t::field_label());
            let enter = ui.input(|i| i.key_pressed(egui::Key::Enter));
            let field = ui
                .add_sized(
                    Vec2::new(FIELD_WIDTH_PTS, ROW_HEIGHT_PTS),
                    // escape-disposition: not-content — the replacement text. Escape
                    // only leaves the field, so Undo and the canvas keys work again.
                    egui::TextEdit::singleline(state.replacement_mut())
                        .id(egui::Id::new(FIELD_ID))
                        .hint_text(t::field_label()),
                )
                .on_hover_text(t::field_tooltip());
            crate::diag::ui_rect(REGION_FIELD, field.rect);
            if field.lost_focus() && enter && live {
                field.request_focus();
                actions.push(Action::Find(FindRequest::Replace { all: false }));
            }
            button(
                ui,
                live,
                (t::one(), t::one_tooltip()),
                REGION_ONE,
                false,
                actions,
            );
            button(
                ui,
                live,
                (t::all(), t::all_tooltip()),
                REGION_ALL,
                true,
                actions,
            );
        })
        .response
        .rect;
    crate::diag::ui_rect(REGION_ROW, rect);
}

fn button(
    ui: &mut egui::Ui,
    live: bool,
    (label, tooltip): (&str, &str),
    region: &str,
    all: bool,
    actions: &mut Vec<Action>,
) {
    let response = ui
        .add_enabled(live, egui::Button::new(label))
        .on_hover_text(tooltip)
        .on_disabled_hover_text(t::unavailable_tooltip());
    crate::diag::ui_rect(region, response.rect);
    if response.clicked() {
        actions.push(Action::Find(FindRequest::Replace { all }));
    }
}
