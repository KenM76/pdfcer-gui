//! # `dialogs::settings::text` — what comes out when you copy
//!
//! Three settings, all of which reach further than the group heading suggests.
//! *"Copying and extracting text"* is where an operator will look for them, and
//! extraction is also what **search**, **selection** and **redaction by
//! pattern** are built on — so two of the three carry a consequence the source
//! did not disclose.
//!
//! ## R35, and why two radius lines here name redaction
//!
//! `pdfcer-core` is explicit: *a redaction built under one value is not
//! equivalent under another.* Changing the unmappable sentinel changes
//! character offsets, which changes which runs a pattern matches; the same is
//! true of whether a document's own replacement text is trusted.
//!
//! The old window said only *"Affects copied and extracted text"* for both,
//! which is true and is not the half that matters. An operator who has reviewed
//! a redaction and then changes one of these settings has invalidated the
//! reasoning behind the review, and nothing told them.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/settings/text.md`.

use egui::Ui;
use pdfcer_core::settings::{
    ActualTextPrecedence, MAX_WORD_GAP_RATIO, MIN_WORD_GAP_RATIO, UnmappableCode,
};

use super::{Draft, widgets};
use crate::app::prefs::Prefs;
use crate::text::settings as t;

/// How wide a gap between glyphs means a space.
pub fn word_gap(ui: &mut Ui, draft: &mut Draft) {
    widgets::header(
        ui,
        t::word_gap_title(),
        t::word_gap_silence(),
        t::word_gap_radius(),
    );
    ui.add(
        egui::Slider::new(
            &mut draft.working.word_gap_ratio,
            MIN_WORD_GAP_RATIO..=MAX_WORD_GAP_RATIO,
        )
        .logarithmic(true)
        .text(t::word_gap_slider_label()),
    );
    ui.label(egui::RichText::new(t::word_gap_note()).small().weak());
}

/// What stands in for text pdfcer cannot decode.
pub fn unmappable(ui: &mut Ui, draft: &mut Draft) {
    widgets::header(
        ui,
        t::unmappable_title(),
        t::unmappable_silence(),
        t::unmappable_radius(),
    );
    widgets::option(
        ui,
        &mut draft.working.unmappable_code,
        UnmappableCode::ReplacementChar,
        t::unmappable_replacement_label(),
        Some(t::unmappable_replacement_note()),
    );
    widgets::option(
        ui,
        &mut draft.working.unmappable_code,
        UnmappableCode::QuestionMark,
        t::unmappable_question_label(),
        Some(t::unmappable_question_note()),
    );
    widgets::option(
        ui,
        &mut draft.working.unmappable_code,
        UnmappableCode::Omit,
        t::unmappable_omit_label(),
        Some(t::unmappable_omit_note()),
    );
}

/// How far a document's own replacement text is trusted over the glyphs drawn.
pub fn actual_text(ui: &mut Ui, draft: &mut Draft) {
    widgets::header(
        ui,
        t::actual_text_title(),
        t::actual_text_silence(),
        t::actual_text_radius(),
    );
    widgets::option(
        ui,
        &mut draft.working.actual_text,
        ActualTextPrecedence::Always,
        t::actual_text_always_label(),
        Some(t::actual_text_always_note()),
    );
    widgets::option(
        ui,
        &mut draft.working.actual_text,
        ActualTextPrecedence::TaggedOnly,
        t::actual_text_tagged_label(),
        Some(t::actual_text_tagged_note()),
    );
    widgets::option(
        ui,
        &mut draft.working.actual_text,
        ActualTextPrecedence::Glyphs,
        t::actual_text_glyphs_label(),
        Some(t::actual_text_glyphs_note()),
    );
    widgets::disclosure(ui, t::actual_text_bound());
}

/// **Whether a blank at either end of a search is ignored** —
/// `OPERATOR_REQUESTS.md` **O180**, 2026-09-12.
pub fn find_trim(ui: &mut Ui, prefs: &mut Prefs) {
    widgets::header(
        ui,
        t::find_trim_title(),
        t::find_trim_silence(),
        t::find_trim_radius(),
    );
    widgets::toggle(
        ui,
        &mut prefs.find_trim_query,
        t::find_trim_label(),
        Some(t::find_trim_note()),
    );
}
