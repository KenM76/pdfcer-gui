//! # `find` — the registry test
//!
//! The Find bar tests that need the application: its command registry, and
//! the font set it installs (the base crate's egui has no default fonts).
#![cfg(test)]

use super::bar::OCR_COMMAND;
use crate::text::find as t;
use egui::{Context, RawInput};

/// The offer raises the command the ribbon registers, not a spelling of it.
#[test]
fn the_offer_raises_the_registered_recognise_command() {
    let mut registry = egui_shell::commands::CommandRegistry::new();
    crate::shell::commands::register(&mut registry);
    assert!(
        registry.get(OCR_COMMAND).is_some(),
        "`{OCR_COMMAND}` is not registered, so the offer's button would reach the \
         dispatcher's fall-through arm and do nothing"
    );
}

/// **Every glyph the bar draws exists in the bundled font set.**
#[test]
fn every_glyph_the_find_bar_draws_has_a_glyph() {
    let ctx = Context::default();
    let labels: Vec<String> = vec![
        t::field_label().to_owned(),
        t::previous().to_owned(),
        t::next().to_owned(),
        t::close().to_owned(),
        t::options().to_owned(),
        t::position(3, 47),
        t::no_matches().to_owned(),
        t::stale().to_owned(),
        t::match_case().to_owned(),
        t::whole_word().to_owned(),
        t::wildcards().to_owned(),
        t::word_rule().to_owned(),
        t::word_rule_alphanumeric().to_owned(),
        t::word_rule_non_space().to_owned(),
        t::word_rule_non_space_or_dash().to_owned(),
        t::toggle().to_owned(),
    ];

    let mut missing = Vec::new();
    let _ = ctx.run_ui(RawInput::default(), |ui| {
        let font = egui::FontId::proportional(14.0);
        ui.ctx().fonts_mut(|f| {
            for label in &labels {
                for c in label.chars() {
                    if !f.has_glyph(&font, c) {
                        missing.push((label.clone(), c));
                    }
                }
            }
        });
    });

    assert!(
        missing.is_empty(),
        "these labels contain codepoints the bundled fonts cannot draw, so they would \
         render as tofu boxes: {missing:?}"
    );
}
