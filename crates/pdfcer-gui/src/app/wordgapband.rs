//! # `app::wordgapband` — the Edit ▸ Content word-gap slider
//!
//! One control, drawn for one custom-item kind
//! ([`crate::shell::manifest::WORD_GAP`]), reporting a new
//! `Settings::word_gap_ratio` when the operator lets go of it.
//!
//! Contract: drawn only while `edit.word_gap` is registered (R8), so a build
//! without the capability loses the slider with the button. The position
//! under the pointer lives in egui memory until release, because every commit
//! drops each open document's extracted text and a drag would otherwise
//! re-extract once per frame. A click or keyboard step commits at once.

use pdfcer_core::settings::{MAX_WORD_GAP_RATIO, MIN_WORD_GAP_RATIO};

use crate::text::settings as t;

/// How wide the slider is drawn, in points.
const TRAVEL_PT: f32 = 120.0;

/// Draw the slider, if `kind` is its kind; `Some(ratio)` on a commit.
pub(super) fn draw(
    ui: &mut egui::Ui,
    kind: &str,
    registry: &egui_shell::commands::CommandRegistry,
    at: f32,
) -> Option<f32> {
    if kind != crate::shell::manifest::WORD_GAP || registry.get("edit.word_gap").is_none() {
        return None;
    }
    let held = egui::Id::new(crate::shell::manifest::WORD_GAP);
    let mut ratio = ui.data(|d| d.get_temp::<f32>(held)).unwrap_or(at);
    let response = ui.add_sized(
        egui::Vec2::new(TRAVEL_PT, ui.spacing().interact_size.y),
        egui::Slider::new(&mut ratio, MIN_WORD_GAP_RATIO..=MAX_WORD_GAP_RATIO)
            .logarithmic(true)
            .text(t::word_gap_slider_label())
            .max_decimals(2),
    );
    crate::diag::ui_rect(
        &egui_shell::ribbon::report::band_item(crate::shell::manifest::WORD_GAP),
        response.rect,
    );
    let response = response.on_hover_text(t::word_gap_note());
    if response.dragged() {
        ui.data_mut(|d| d.insert_temp(held, ratio));
        return None;
    }
    ui.data_mut(|d| d.remove::<f32>(held));
    if !(response.changed() || response.drag_stopped()) || (ratio - at).abs() < f32::EPSILON {
        return None;
    }
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("word-gap set={ratio:.3}")
    });
    Some(ratio)
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_shell::commands::CommandRegistry;

    fn registered() -> CommandRegistry {
        let mut registry = CommandRegistry::new();
        crate::shell::commands::register(&mut registry);
        registry
    }

    /// How wide the laid-out region is after `draw` is asked for one kind.
    fn laid_out(kind: &'static str, registry: &CommandRegistry) -> f32 {
        let ctx = egui::Context::default();
        let mut width = f32::NAN;
        let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
            let before = ui.min_rect().width();
            let _ = draw(ui, kind, registry, 0.2);
            width = ui.min_rect().width() - before;
        });
        width
    }

    #[test]
    fn the_slider_draws_for_its_kind_while_its_command_is_registered() {
        assert!(laid_out(crate::shell::manifest::WORD_GAP, &registered()) >= TRAVEL_PT);
    }

    #[test]
    fn a_build_without_the_command_draws_no_slider() {
        assert_eq!(
            laid_out(crate::shell::manifest::WORD_GAP, &CommandRegistry::new()),
            0.0
        );
    }

    #[test]
    fn a_foreign_kind_draws_nothing() {
        for kind in [
            crate::shell::manifest::OCR_BLEND,
            crate::shell::manifest::FONT_SIZE,
            crate::shell::manifest::RECENT_FILES,
        ] {
            assert_eq!(laid_out(kind, &registered()), 0.0, "{kind}");
        }
    }

    /// The edit tab holds the slider, so the ribbon asks for it at all.
    #[test]
    fn the_edit_tab_holds_the_slider() {
        let shell = crate::shell::manifest::built_in();
        let held = shell
            .all_tabs()
            .flat_map(egui_shell::manifest::Tab::groups)
            .flat_map(egui_shell::manifest::Group::items)
            .any(|item| {
                matches!(item, egui_shell::manifest::Item::Custom { kind, .. }
                    if kind == crate::shell::manifest::WORD_GAP)
            });
        assert!(held);
    }
}
