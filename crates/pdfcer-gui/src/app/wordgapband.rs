//! # `app::wordgapband` — the Edit ▸ Content word-gap Auto switch and slider
//!
//! Two controls, drawn for one custom-item kind
//! ([`crate::shell::manifest::WORD_GAP`]), reporting `Settings::word_gap_auto`
//! when the switch flips and `Settings::word_gap_ratio` when the operator lets
//! go of the slider. Under Auto the slider is the fallback for a page too
//! sparse to measure; the threshold the current page used is on hover.
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

/// The word-gap settings the band shows and reports.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct WordGap {
    /// `Settings::word_gap_ratio`: the fixed threshold, and Auto's fallback.
    pub ratio: f32,
    /// `Settings::word_gap_auto`: each page measures its own threshold.
    pub auto: bool,
}

/// Draw the Auto switch and the slider, if `kind` is theirs; `Some` on a
/// commit. `page` is the threshold the current page was extracted with and
/// whether it was measured (`PageText::word_gap_ratio`, `word_gap_measured`),
/// when that page's text is already built; it is disclosed on hover.
pub(super) fn draw(
    ui: &mut egui::Ui,
    kind: &str,
    registry: &egui_shell::commands::CommandRegistry,
    at: WordGap,
    page: Option<(f32, bool)>,
) -> Option<WordGap> {
    if kind != crate::shell::manifest::WORD_GAP || registry.get("edit.word_gap").is_none() {
        return None;
    }
    let used = page.map(|(ratio, measured)| t::word_gap_on_page(ratio, measured));
    crate::diag::trace_changed("word-gap-band", || {
        let page = page.map_or_else(
            || "unbuilt".to_owned(),
            |(ratio, measured)| format!("{ratio:.3} measured={measured}"),
        );
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            "word-gap-band auto={} ratio={:.3} page={page}",
            at.auto, at.ratio
        )
    });
    let hover = |note: &str| match &used {
        Some(used) => format!(
            "{note}

{used}"
        ),
        None => note.to_owned(),
    };
    let mut auto = at.auto;
    let switch = ui
        .checkbox(&mut auto, t::word_gap_auto_label())
        .on_hover_text(hover(t::word_gap_auto_note()));
    crate::diag::ui_rect(&format!("{}.auto", band_item_name()), switch.rect);
    if switch.changed() {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!("word-gap auto={auto}")
        });
        return Some(WordGap { auto, ..at });
    }
    let held = egui::Id::new(crate::shell::manifest::WORD_GAP);
    let mut ratio = ui.data(|d| d.get_temp::<f32>(held)).unwrap_or(at.ratio);
    let response = ui.add_sized(
        egui::Vec2::new(TRAVEL_PT, ui.spacing().interact_size.y),
        egui::Slider::new(&mut ratio, MIN_WORD_GAP_RATIO..=MAX_WORD_GAP_RATIO)
            .logarithmic(true)
            .text(t::word_gap_slider_label())
            .max_decimals(2),
    );
    crate::diag::ui_rect(&band_item_name(), response.rect);
    let response = response.on_hover_text(hover(t::word_gap_note()));
    if response.dragged() {
        ui.data_mut(|d| d.insert_temp(held, ratio));
        return None;
    }
    ui.data_mut(|d| d.remove::<f32>(held));
    if !(response.changed() || response.drag_stopped()) || (ratio - at.ratio).abs() < f32::EPSILON {
        return None;
    }
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("word-gap set={ratio:.3}")
    });
    Some(WordGap { ratio, ..at })
}

/// The slider's region name; the switch's is this plus `.auto`.
fn band_item_name() -> String {
    egui_shell::ribbon::report::band_item(crate::shell::manifest::WORD_GAP)
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
            let _ = draw(
                ui,
                kind,
                registry,
                WordGap {
                    ratio: 0.2,
                    auto: false,
                },
                None,
            );
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

    /// A click on the track (press and release, no movement) commits the
    /// value under the pointer.
    #[test]
    fn a_click_on_the_track_commits() {
        let ctx = egui::Context::default();
        let registry = registered();
        let at = WordGap {
            ratio: 0.2,
            auto: false,
        };
        let mut committed = None;
        let mut frame = |events: Vec<egui::Event>| {
            let input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(800.0, 200.0),
                )),
                events,
                ..Default::default()
            };
            let _ = ctx.run_ui(input, |ui| {
                ui.horizontal(|ui| {
                    if let Some(set) =
                        draw(ui, crate::shell::manifest::WORD_GAP, &registry, at, None)
                    {
                        committed = Some(set);
                    }
                });
            });
        };
        frame(vec![]);
        frame(vec![]);
        // The switch is about 50 pt wide; the slider's track follows it.
        let point = egui::pos2(150.0, 10.0);
        let button = |pressed| egui::Event::PointerButton {
            pos: point,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        };
        frame(vec![egui::Event::PointerMoved(point), button(true)]);
        frame(vec![button(false)]);
        frame(vec![]);
        let set = committed.expect("a click on the track commits");
        assert!((set.ratio - at.ratio).abs() > f32::EPSILON, "{set:?}");
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
