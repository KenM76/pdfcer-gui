//! # `text::settings::extract` — what changing it makes you GET OUT
//!
//! One of three copy modules under [`crate::text::settings`], split on
//! 2026-08-17 at rule R2's 1,500-line ceiling.
//!
//! ## ★ The split is by BLAST RADIUS, which is the window's own taxonomy
//!
//! Not by dialog group, and not alphabetically. Every setting in this window
//! carries a `*_radius` line stating *which way costs what*, and that line is
//! one of exactly three things:
//!
//! | module | radius | settings |
//! |---|---|---|
//! | [`super::look`] | changes what you SEE; the file is untouched | theme, CMYK intent, CMYK JPEG polarity, mask resampling, minification |
//! | [`super::extract`] | changes what you GET OUT — copy, search, redaction-by-pattern, new dimensions | word gap, unmappable codes, replacement text, parallel tolerance |
//! | [`super::bytes`] | changes what pdfcer WRITES | separations, missing appearance state, index line endings, trailing newline |
//!
//! That taxonomy is load-bearing rather than a filing convenience: it is the
//! distinction the window exists to make legible, and a test in
//! [`super`] asserts that exactly the byte-changing settings say they change
//! the file — in both directions, so a preview setting cannot quietly claim a
//! consequence it does not have.
//!
//! One setting is filed by its radius rather than by its group and it is worth
//! naming: **CMYK JPEG polarity** appears above under *look*, and its radius
//! line also says *"and the saved file if pdfcer re-compresses the image"*. It
//! is the only setting whose radius spans two of the three. It sits with the
//! others in its dialog group, where an operator looks for it.

// ===========================================================================
// Copying and extracting text — word gap
// ===========================================================================

/// Word gap: what it is.
#[must_use]
pub const fn word_gap_title() -> &'static str {
    "Where extracted text gets its spaces"
}

/// Word gap: what the standard leaves open.
#[must_use]
pub const fn word_gap_silence() -> &'static str {
    "A PDF does not have to store spaces between words at all — it can just leave \
     a gap. Nothing in the standard says how wide a gap means a space, so every \
     program guesses, and they guess differently."
}

/// Word gap: what changing it costs.
#[must_use]
pub const fn word_gap_radius() -> &'static str {
    "Affects copied and extracted text, and therefore which text a redaction \
     pattern matches. Does not change the file."
}

/// The slider's own label.
#[must_use]
pub const fn word_gap_slider_label() -> &'static str {
    "Gap width"
}

/// How to use it, and that the default is a guess.
#[must_use]
pub const fn word_gap_note() -> &'static str {
    "Measured as a fraction of the text size. Raise it if words are being split \
     apart; lower it if separate words are running together. pdfcer's default of \
     0.2 is a considered guess rather than a rule from anywhere — no standard or \
     reference program defines this."
}

// ===========================================================================
// Copying and extracting text — unmappable codes
// ===========================================================================

/// Unmappable codes: what it is.
#[must_use]
pub const fn unmappable_title() -> &'static str {
    "Text pdfcer cannot read"
}

/// Unmappable codes: what the standard leaves open.
///
/// *"The standard's own sentence about what to do here is incomplete"* is not
/// a figure of speech: §9.10.2's failure clause is grammatically broken — it
/// says a reader *"may choose a character code of their choosing"* where a
/// Unicode value is what is produced — and specifies no sentinel anywhere.
#[must_use]
pub const fn unmappable_silence() -> &'static str {
    "Some documents draw text without recording which characters it is. The \
     standard's own sentence about what to do here is incomplete, and defines no \
     answer."
}

/// Unmappable codes: what changing it costs.
///
/// ★ Names **redaction**, which the source's radius line did not. R35 is
/// explicit that a redaction built under one value is not equivalent under
/// another: the sentinel changes character offsets, which changes which runs a
/// pattern matches. An operator who redacts by pattern needs to know that
/// changing this setting invalidates the reasoning behind a redaction they
/// have already reviewed.
#[must_use]
pub const fn unmappable_radius() -> &'static str {
    "Affects copied and extracted text — including which text a redaction by \
     pattern finds. Does not change the file."
}

/// The default.
#[must_use]
pub const fn unmappable_replacement_label() -> &'static str {
    "Insert the replacement character (pdfcer's default)"
}

/// Why, with the guess admitted.
#[must_use]
pub const fn unmappable_replacement_note() -> &'static str {
    "Puts a visible marker where the text could not be read, so the gap is \
     obvious and countable rather than silently swallowed. It is the only choice \
     that both keeps the length right and looks wrong, which is pdfcer's own \
     reasoning — nothing defines a marker for this."
}

/// The plainer sentinel.
#[must_use]
pub const fn unmappable_question_label() -> &'static str {
    "Insert a question mark"
}

/// What it costs.
#[must_use]
pub const fn unmappable_question_note() -> &'static str {
    "Plainer in software that cannot display the replacement character, at the \
     cost of being indistinguishable from a question mark the document really \
     contains."
}

/// The dangerous one.
#[must_use]
pub const fn unmappable_omit_label() -> &'static str {
    "Leave it out"
}

/// ★ **The disappearing-run consequence, which the old note omitted.**
///
/// The source warned that extracted text reads as complete when characters are
/// missing. True, and the smaller half. The larger one is documented in
/// `pdfcer-core` and was shown nowhere: the layout pass drops a run with no
/// characters, so a run whose codes are *all* unmappable **vanishes entirely,
/// glyph records included** — a page of `Identity-H` text with no `/ToUnicode`
/// yields *zero runs* rather than runs of sentinels.
///
/// That is the more surprising failure and the one that breaks anything
/// needing per-glyph positions.
#[must_use]
pub const fn unmappable_omit_note() -> &'static str {
    "Cleanest-looking output, and the most dangerous: text you extract will read \
     as complete when characters are missing from it. Where a whole run is \
     unreadable it disappears altogether rather than becoming markers, so that \
     text cannot be found, selected or redacted by pattern at all."
}

// ===========================================================================
// Copying and extracting text — /ActualText
// ===========================================================================

/// Replacement text: what it is.
#[must_use]
pub const fn actual_text_title() -> &'static str {
    "When a document supplies replacement text"
}

/// Replacement text: what the standard leaves open.
///
/// Three ISO 32000-1 statements disagree and none dislodges the others. The
/// only sentence addressing precedence is a *may*, and it sits in an
/// informative note — which is why neither reading can be eliminated and why
/// this is a setting rather than a decision.
#[must_use]
pub const fn actual_text_silence() -> &'static str {
    "A document can attach replacement text to a piece of content — used for \
     things like ligatures, where the drawn shape and the real characters differ. \
     The standard does not say how far to trust it over the shapes themselves."
}

/// Replacement text: what changing it costs.
#[must_use]
pub const fn actual_text_radius() -> &'static str {
    "Affects copied and extracted text — including which text a redaction by \
     pattern finds. Does not change the file."
}

/// The default.
#[must_use]
pub const fn actual_text_always_label() -> &'static str {
    "Always use it (pdfcer's default)"
}

/// Why, with the guess admitted.
#[must_use]
pub const fn actual_text_always_note() -> &'static str {
    "The author said what this text really is, so pdfcer believes them, and \
     extraction marks which text came from this source so it stays traceable. \
     The standard says both \"use it\" and \"you may use it\" in different \
     places, so this is pdfcer reading the stronger of the two."
}

/// The middle option.
#[must_use]
pub const fn actual_text_tagged_label() -> &'static str {
    "Only in properly tagged documents"
}

/// What it does.
#[must_use]
pub const fn actual_text_tagged_note() -> &'static str {
    "Trusts the replacement text only where the document is structured well \
     enough for it to be reliable, and falls back to the drawn characters \
     elsewhere."
}

/// The refusal.
#[must_use]
pub const fn actual_text_glyphs_label() -> &'static str {
    "Ignore it"
}

/// What it loses.
///
/// The second sentence is not in the source: `Glyphs` **loses genuinely
/// unrecoverable text**, because a ligature whose only Unicode identity was
/// its replacement text extracts as whatever can be made of the glyph.
#[must_use]
pub const fn actual_text_glyphs_note() -> &'static str {
    "Always uses the drawn characters. Useful when a document's replacement text \
     is wrong, which does happen — but where the replacement text was the only \
     record of what the characters are, that text becomes unreadable rather than \
     merely different."
}

/// ★ **A bound that is not a setting, disclosed because it is a fact.**
///
/// New in this port; the old window disclosed it nowhere despite `pdfcer-core`
/// documenting it and calling it *"a fact to disclose, not a direction to
/// pick"*.
///
/// There is **no length correspondence** between replacement text and the
/// content it replaces — the standard's own example maps two shown characters
/// to one — so character-level mapping back to glyph positions is *impossible*
/// across such a run. That bounds search highlighting, selection and
/// redaction-by-text to **sequence** granularity **whichever of the three
/// options is chosen**, which is exactly why it belongs under the group rather
/// than inside one option's note: an operator who reads it as an argument for
/// picking *Ignore it* has been misled.
#[must_use]
pub const fn actual_text_bound() -> &'static str {
    "Whichever you choose: where a document supplies replacement text, pdfcer can \
     locate it only as a whole piece, not character by character. Highlighting a \
     search hit, selecting part of it, or redacting inside it therefore covers \
     the whole piece. That is a limit of what the file records, not a choice \
     made here."
}

// ===========================================================================
// Measuring and dimensioning — parallel tolerance
// ===========================================================================

/// Parallel tolerance: what it is.
#[must_use]
pub const fn parallel_title() -> &'static str {
    "When two lines count as parallel"
}

/// Parallel tolerance: what nobody defines.
///
/// Not a spec silence — the PDF standard has no view on dimensioning at all —
/// but the same shape of silence, and worth saying because the operator would
/// otherwise reasonably assume CAD practice had settled it. A search of the
/// SolidWorks dimension corpus for a threshold found none.
#[must_use]
pub const fn parallel_silence() -> &'static str {
    "Dimensioning between two lines has to decide whether they are parallel — \
     giving a distance — or at an angle. Nothing defines how close to parallel is \
     close enough, and CAD programs do not document a threshold either."
}

/// Parallel tolerance: what changing it costs.
///
/// A **third** radius category, which neither the preview settings nor the
/// byte-changing ones cover: it affects *new authoring only*. Dimensions
/// already placed do not move, and nothing in the file changes.
#[must_use]
pub const fn parallel_radius() -> &'static str {
    "Affects new dimensions you draw between two lines. Does not change \
     dimensions you have already placed, and does not change the file."
}

/// The slider's own label.
#[must_use]
pub const fn parallel_slider_label() -> &'static str {
    "Within"
}

/// The degree sign, as a catalog entry.
///
/// # Why one character has a function
///
/// `check-ui-strings.sh` requires every operator-visible string to come from
/// this catalog, and a bare `"°"` in a `Slider::suffix` call is exactly what
/// that gate looks for. The old shell solved it with a private constant in the
/// panel, which satisfies the gate's letter; a catalog entry satisfies its
/// reason, since a translator localising this window would otherwise never see
/// that the suffix exists.
#[must_use]
pub const fn degree_suffix() -> &'static str {
    "\u{b0}"
}

/// How to choose a tolerance, and the escape hatch.
///
/// The last sentence is deliberate and was in the source for a stated reason:
/// an operator reading this control needs to know that a wrong global value is
/// a one-click per-dimension fix, not something they must come back here to
/// adjust. Without it, the natural response to one bad classification is to
/// change a global default on the strength of a single drawing.
#[must_use]
pub const fn parallel_note() -> &'static str {
    "Exported CAD geometry is usually exact, so a small value keeps a rounding \
     artefact from being read as a taper. Zero means exactly parallel only. \
     Whatever you set here, you can still tick \"treat as parallel\" on any \
     single dimension without changing this."
}
