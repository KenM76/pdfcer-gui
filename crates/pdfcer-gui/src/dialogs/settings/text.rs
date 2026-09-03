//! # `dialogs::settings::text` — what comes out when you copy
//!
//! Three settings, all of which reach further than the group heading suggests.
//! *"Copying and extracting text"* is where an operator will look for them, and
//! extraction is also what **search**, **selection** and **redaction by
//! pattern** are built on — so two of the three carry a consequence the source
//! did not disclose.
//!
//! ## ★ R35, and why two radius lines here name redaction
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

use egui::Ui;
use pdfcer_core::settings::{
    ActualTextPrecedence, MAX_WORD_GAP_RATIO, MIN_WORD_GAP_RATIO, UnmappableCode,
};

use super::{Draft, widgets};
use crate::text::settings as t;

/// How wide a gap between glyphs means a space.
///
/// # A slider, not a text box
///
/// Free text invites a number that then has to be silently clamped, and a
/// silent clamp on a setting is an edit the operator did not make.
///
/// # ★ The range MUST be the store's own accepted range
///
/// `MIN_WORD_GAP_RATIO` and `MAX_WORD_GAP_RATIO` are `pub` in `pdfcer-core`
/// **specifically so a front end can bound its control by the same numbers the
/// parser clamps to**, and using them rather than a "usable band" is
/// load-bearing rather than tidy.
///
/// The first attempt at this control used `0.05..=1.0`, on the reasoning that
/// nothing outside it is useful. That range cannot represent a legal value an
/// operator may already have hand-edited into their file — so **merely opening
/// this window would drag a hand-edited `2.0` down to `1.0`, and Save would
/// write the changed value back**. A silent, unrequested edit to a
/// configuration, invisible because the operator never touched the slider. The
/// file is explicitly meant to be hand-editable; a window that quietly narrows
/// what the file may say is a window that punishes using it.
///
/// # Logarithmic, unlike the tolerance slider in [`super::measuring`]
///
/// The useful resolution is all at the low end, where `0.15` against `0.25`
/// decides whether words run together. Everything above `1.0` behaves much the
/// same, so a linear slider would spend four fifths of its travel on
/// indistinguishable values and make the range that matters unhittable.
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
///
/// # ★ The consequence the old note omitted
///
/// The source's warning for *Leave it out* was that extracted text reads as
/// complete when characters are missing. True, and the smaller half.
///
/// The larger half is documented in `pdfcer-core` and was shown nowhere: the
/// layout pass drops a run with no characters, so a run whose codes are **all**
/// unmappable **disappears entirely, glyph records included**. A page of
/// `Identity-H` text with no `/ToUnicode` yields *zero runs* rather than runs of
/// sentinels — so that text cannot be found, selected, or redacted by pattern
/// at all. That is the surprising failure and the one that breaks anything
/// needing per-glyph positions.
///
/// # What the setting cannot switch off
///
/// The rung-4 counter keeps counting whatever is chosen — it is the headline
/// honesty metric and this setting must not be able to silence it — and three
/// internal paths pin the sentinel to `ReplacementChar` regardless of the
/// operator's choice, each saying so at the call: the text-editing slot table
/// (a zero-length span is a glyph the operator can see and cannot address), the
/// redaction audit record (must not report a removal as nothing), and the
/// vector-object text preview (must not make an undecodable run look empty).
///
/// The setting's scope is **extraction output**, which is why the window does
/// not offer it as a global rendering choice.
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
///
/// # Three statements in the standard, and none dislodges the others
///
/// §14.9.4 says `/ActualText` *"shall be used as a replacement"* — the only
/// **shall**. §14.8.2.4.2's note 2 says a reader *"may choose to use"* it — a
/// **may**, inside an *informative* note. §9.10.1 says it *"may be used"*.
///
/// The only sentence addressing precedence is the `may`, and it sits in a note,
/// so under the standing normative-versus-informative rule it cannot be cited
/// alone as authority — and neither reading can be eliminated. That is why this
/// is a setting, and why the default is `Always`: it follows the one `shall`.
///
/// # ★ The bound that is not a setting, disclosed under the group
///
/// **No length correspondence exists** between replacement text and the content
/// it replaces — the standard's own example maps two shown characters to one —
/// so character-level mapping back to glyph positions is *impossible* across
/// such a run. That bounds search highlighting, selection and redaction-by-text
/// to **sequence** granularity **whichever of the three options is chosen**.
///
/// `pdfcer-core` calls it *"a fact to disclose, not a direction to pick"*, and
/// the old window disclosed it nowhere. It is rendered as a
/// [`widgets::disclosure`] under the whole group rather than as a note on one
/// option, because an operator who read it as an argument for *Ignore it* has
/// been misled by the placement.
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
