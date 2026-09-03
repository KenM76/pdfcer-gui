//! # `text::scale` — the words the Set-scale dialog shows
//!
//! ## The hardest job in this catalog: explaining what a ratio is *against*
//!
//! A scale is two numbers and a unit nobody names out loud. `1:100` is
//! meaningless without saying *one what*, and the answer for a PDF is
//! **1/72 inch**, which is nobody's intuition and which the operator has never
//! had to think about in any drawing package.
//!
//! So this catalog's job is to make a basis visible without making it the
//! subject. The dialog asks for it as an ordinary control with an ordinary
//! label; the explanation lives in one hint line under the ratio, in the terms
//! a drafter already has — *"1 mm on paper is 100 mm in the world"*.
//!
//! ## Rule 15 applies throughout
//!
//! What this window calibrates is a group of **ce dimensions** — the ones pdfcer
//! authors — and never a **pdf dimension**, which is CAD-exported page content
//! pdfcer reads and must not silently alter. The copy avoids the bare word and
//! says *"the dimensions you draw"*, which is unambiguous without asking the
//! operator to learn a distinction that is ours rather than theirs.

use pdfcer_core::dimension::{FractionMode, Unit};

/// The window's title.
#[must_use]
pub const fn window_title() -> &'static str {
    "Set scale"
}

/// The paragraph under the title.
///
/// ★ **The most important sentence in this window**, and it is a disclosure
/// rather than an instruction: it says what the numbers mean *right now*.
///
/// A fresh group's scale is the tri-state's *never-set* value, so every label
/// a dimension tool has placed reads in PDF points — a measurement of the
/// **paper**, not of the thing drawn on it. An operator who has placed
/// dimensions and read numbers has been given plausible answers to a question
/// they did not ask, which is worse than no answer, and this is where they find
/// that out.
#[must_use]
pub const fn intro() -> &'static str {
    "Until a scale is set, the dimensions you draw are measured in the page's \
     own units — the size on paper, not the size of the thing drawn. Setting a \
     scale here converts every dimension in this group, including ones you have \
     already placed."
}

/// Why the ratio path is what a cold-opened dialog offers.
///
/// ★ **This string used to say the other path could not be armed at all**, and
/// it was accurate until 2026-08-17: *"needs a reference line drawn on the
/// page, which this build cannot arm yet."* That was exactly the gap the
/// operator reported — *"still missing the feature where we set the scale by
/// selecting two lines or points and defining what that distance
/// represents"* — and the gesture exists now.
///
/// Kept as a sentence rather than deleted, because a cold dialog really does
/// have only one path that can produce a scale: no line is drawn yet. What
/// changed is that it points at the button instead of apologising for an
/// absence.
#[must_use]
pub const fn ratio_only_note() -> &'static str {
    "With no line drawn yet, the scale is given as a ratio. To set it by pointing at a dimension the drawing already states, measure it first."
}

/// The ratio row's label.
#[must_use]
pub const fn ratio_label() -> &'static str {
    "Scale"
}

/// What sits between the two ratio numbers.
///
/// A catalog entry rather than a literal for the reason the settings window's
/// degree sign is one: the ui-strings gate looks for exactly this, and a
/// translator must be able to see that a separator exists — several languages
/// write a scale with something other than a colon.
#[must_use]
pub const fn ratio_separator() -> &'static str {
    " : "
}

/// What the ratio means, in a drafter's terms.
#[must_use]
pub const fn ratio_hint() -> &'static str {
    "Paper on the left, the real world on the right. 1 : 100 means one unit on \
     the page is a hundred of the same unit in the world."
}

/// The basis row's label.
#[must_use]
pub const fn basis_label() -> &'static str {
    "Paper measured in"
}

/// The display-unit row's label.
#[must_use]
pub const fn unit_label() -> &'static str {
    "Show dimensions in"
}

/// The fraction row's label.
#[must_use]
pub const fn fraction_label() -> &'static str {
    "Number style"
}

/// A unit's name, spelled out.
///
/// Full names rather than `Unit::abbrev`'s two letters. The abbreviation is
/// right on a *dimension label*, where space is scarce and the reader already
/// knows what they are looking at; it is wrong in a picker, where the reader is
/// choosing and `in` versus `ft` is two characters apart in a list they are
/// scanning once.
///
/// # Exhaustive, deliberately — and this is the one place that is safe
///
/// `pdfcer_core::dimension::Unit` is **not** `#[non_exhaustive]`, unlike most
/// of the engine's public enums, so a variant added there is a compile error
/// here rather than a silently unnamed row in a picker. That is the outcome
/// worth having: a unit with no name would render as an empty combo entry, and
/// an operator would select it without knowing what they had chosen.
///
/// Contrast `crate::text::settings::theme_preset_label`, which *does* carry a
/// catch-all, because `egui_shell::theme::Preset` IS `#[non_exhaustive]` — the
/// whole point of the shell crate is that another application may ship presets
/// pdfcer has never heard of. The two are opposite situations and get opposite
/// treatment; neither is a style preference.
#[must_use]
pub fn unit_name(unit: Unit) -> &'static str {
    match unit {
        Unit::Millimeter => "Millimetres",
        Unit::Centimeter => "Centimetres",
        Unit::Meter => "Metres",
        Unit::Inch => "Inches",
        Unit::DecimalFeet => "Feet (decimal)",
        Unit::FeetInches => "Feet and inches",
    }
}

/// A number style's name, including the *"use the unit's own default"* state.
///
/// ★ The `None` entry is a **real choice**, not an absence, and its wording
/// says so. It is what an operator who never opens this control gets, and the
/// field stores `Option<FractionMode>` rather than re-deriving from the unit
/// precisely so that an explicit choice survives a unit change — somebody who
/// asked for eighths does not want them silently reverted by switching from
/// inches to feet.
///
/// The fraction entries say `1/8` rather than "eighths" because that is how a
/// drawing writes it, and matching the drawing's own notation is the whole
/// reason this control exists — the operator's ask was *"also want to be able
/// to choose the units and display type - rounding, fraction, etc."*, made
/// after a drawing dimensioned in inches always read `55.63"` and could never
/// read `55 5/8"`.
#[must_use]
pub fn fraction_name(fraction: Option<FractionMode>) -> &'static str {
    match fraction {
        None => "Whatever suits the unit",
        Some(FractionMode::Decimal { places: 0 }) => "Whole numbers",
        Some(FractionMode::Decimal { places: 1 }) => "One decimal place",
        Some(FractionMode::Decimal { places: 2 }) => "Two decimal places",
        Some(FractionMode::Decimal { places: 3 }) => "Three decimal places",
        Some(FractionMode::Decimal { .. }) => "Decimal",
        Some(FractionMode::Fraction {
            denominator: 8,
            reduce: false,
        }) => "Eighths (1/8)",
        Some(FractionMode::Fraction {
            denominator: 16,
            reduce: false,
        }) => "Sixteenths (1/16)",
        Some(FractionMode::Fraction {
            denominator: 32,
            reduce: false,
        }) => "Thirty-seconds (1/32)",
        Some(FractionMode::Fraction { .. }) => "Fractions",
    }
}

/// The live preview of what the entry back-calculates to.
///
/// ★ Takes the engine's own `ratio_label` rather than formatting the scale
/// here. `ScalePreview::ratio_label` is documented as the `/R`-style label —
/// `1:100`, or `25 ft = 42.3 pt` — and it is DISPLAY-ONLY, which is exactly
/// what this line is. Formatting a scale in the GUI would be a second
/// implementation of a string the engine already produces, and the two would
/// eventually disagree about rounding on a number the operator is checking.
#[must_use]
pub fn preview(ratio_label: &str, unit: Unit) -> String {
    format!(
        "Dimensions will read in {} at {ratio_label}.",
        unit_name(unit)
    )
}

/// The entry does not describe a scale.
///
/// Reachable by typing a zero into either side of the ratio. Worded as what is
/// wrong rather than as "invalid", because the operator can see the fields and
/// the useful half is which one pdfcer could not use.
#[must_use]
pub const fn degenerate() -> &'static str {
    "Both sides of the scale have to be more than zero."
}

/// A length the parser could not read.
#[must_use]
pub fn parse_failed(detail: &str) -> String {
    format!("pdfcer could not read that length: {detail}")
}

/// The commit control.
///
/// **"Set scale", not "OK".** The button says what it does, because what it
/// does is larger than it looks: it re-propagates every member's appearance,
/// so dimensions already on the page change. An "OK" would be a button whose
/// blast radius the operator has to have read the intro to know.
#[must_use]
pub const fn accept() -> &'static str {
    "Set scale"
}

/// Why the commit control is greyed.
#[must_use]
pub const fn accept_disabled_tooltip() -> &'static str {
    "Enter a scale with both sides above zero first."
}

/// The abort control.
#[must_use]
pub const fn cancel() -> &'static str {
    "Cancel"
}

/// What Cancel promises.
#[must_use]
pub const fn cancel_tooltip() -> &'static str {
    "Close without changing the scale. Nothing you have typed here has taken \
     effect yet."
}

// ===========================================================================
// Calibrating by picking two points on the drawing
// ===========================================================================
//
// ★ The operator asked for this by name on 2026-08-17: "set the scale by
// selecting two lines or points and defining what that distance represents."
// It is the workflow every drafting tool calls calibration, and it is the one
// a drafter reaches for, because it needs no arithmetic from them: point at
// something the drawing already dimensions, type what it says, done.

/// The button that starts the two-point pick.
///
/// ★ Not "Calibrate". That is a word from our side of the fence — it names
/// the operation rather than the action — and an operator scanning this
/// window is looking for a way to avoid computing a ratio. This says what the
/// click does.
#[must_use]
pub const fn calibrate_button() -> &'static str {
    "Measure it on the drawing..."
}

/// What the button will do, on hover.
#[must_use]
pub const fn calibrate_tooltip() -> &'static str {
    "Close this window and click two points on the page. pdfcer measures the distance between them, then asks what that distance is on the real thing."
}

/// Why the button is worth pressing, under it.
///
/// Names the case it saves the operator from — working the ratio out by hand —
/// because a button whose advantage is unstated reads as a longer route to the
/// same place.
#[must_use]
pub const fn calibrate_note() -> &'static str {
    "Easier than a ratio if the drawing already states a dimension: point at it, type what it says, and pdfcer works the scale out."
}

/// What pdfcer measured, once the two points are picked.
///
/// ★ The number is shown, and that is a disclosure rather than decoration.
/// It is the half of the equation pdfcer contributed, and an operator checking
/// their work needs to see that pdfcer measured the line they meant to pick —
/// a snap that landed on the wrong endpoint is visible here and nowhere else
/// once the dialog is up.
///
/// Points, because that is the unit the measurement is in and inventing a
/// friendlier one would mean picking a scale, which is the thing not yet
/// known.
#[must_use]
pub fn calibrated_note(measured_pt: f64) -> String {
    format!(
        "You picked a line {measured_pt:.2} points long on the page. What is that distance on the real thing?"
    )
}

/// The real-length field's label.
#[must_use]
pub const fn real_length_label() -> &'static str {
    "That distance is"
}

/// The real-length field's placeholder.
///
/// An example rather than a unit name, because the grammar accepts several
/// shapes and the fastest way to say so is to show one that is not obvious.
#[must_use]
pub const fn real_length_hint() -> &'static str {
    "e.g. 4'-7 1/2\" or 2500mm"
}

/// What the field accepts, under it.
///
/// The grammar lives in `pdfcer_core::dimension::parse_length` and is shared
/// with the CLI, so this describes it rather than defining it — two
/// descriptions of one grammar is how the GUI and the CLI come to disagree
/// about what `55 5/8"` means.
#[must_use]
pub const fn real_length_hint_long() -> &'static str {
    "Feet and inches, fractions, or a plain number with a unit. Leave the unit off and pdfcer uses the one selected below."
}

/// A length pdfcer could not read.
///
/// Quotes the engine's own message rather than replacing it: the parser knows
/// which character it stopped at and this module does not.
#[must_use]
pub fn length_parse_error(engine_message: &str) -> String {
    format!("pdfcer could not read that length: {engine_message}")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★ The intro discloses that dimensions are currently measuring the paper.
    ///
    /// The sentence this whole window exists to deliver. An operator who has
    /// placed dimensions and read numbers has been given plausible answers to
    /// a question they did not ask, and this is where they find out. A test
    /// rather than a convention, because the natural edit when an intro reads
    /// long is to cut its first clause.
    #[test]
    fn the_intro_says_the_numbers_currently_measure_the_paper() {
        let intro = intro();
        assert!(intro.contains("paper"), "{intro:?}");
        assert!(
            intro.contains("already placed"),
            "the intro does not say existing dimensions change too: {intro:?}"
        );
    }

    /// Every unit has a name, and no two share one.
    ///
    /// A picker with two identically-labelled rows is a picker whose choice is
    /// a coin toss.
    #[test]
    fn every_unit_is_named_distinctly() {
        let units = [
            Unit::Millimeter,
            Unit::Centimeter,
            Unit::Meter,
            Unit::Inch,
            Unit::DecimalFeet,
            Unit::FeetInches,
        ];
        for i in 0..units.len() {
            assert!(!unit_name(units[i]).is_empty());
            for j in (i + 1)..units.len() {
                assert_ne!(
                    unit_name(units[i]),
                    unit_name(units[j]),
                    "{:?} and {:?} share a label",
                    units[i],
                    units[j]
                );
            }
        }
    }

    /// The default number style reads as a choice, not as an absence.
    ///
    /// "Whatever suits the unit" is a thing an operator can decide they want.
    /// An empty string, or "Default", is a row that looks like a missing value
    /// — and this one is the row most operators will leave selected.
    #[test]
    fn the_default_number_style_is_worded_as_a_choice() {
        let name = fraction_name(None);
        assert!(!name.is_empty());
        assert!(
            !name.eq_ignore_ascii_case("default") && !name.eq_ignore_ascii_case("none"),
            "the default number style reads as an absence: {name:?}"
        );
    }

    /// The fraction entries write the fraction the way a drawing does.
    ///
    /// The operator's ask was to be able to read `55 5/8"` rather than
    /// `55.63"`. A picker offering "eighths" without showing `1/8` makes them
    /// translate in their head at the moment they are trying to match a
    /// drawing.
    #[test]
    fn the_fraction_styles_show_the_fraction() {
        for (denominator, needle) in [(8_u32, "1/8"), (16, "1/16"), (32, "1/32")] {
            let name = fraction_name(Some(FractionMode::Fraction {
                denominator,
                reduce: false,
            }));
            assert!(name.contains(needle), "{name:?} does not show {needle}");
        }
    }

    /// The commit button names its act rather than saying OK.
    ///
    /// It re-propagates every member's appearance, so dimensions already on the
    /// page change. "OK" would be a button whose blast radius the operator has
    /// to have read the intro to know.
    #[test]
    fn the_commit_button_names_what_it_does() {
        let label = accept();
        assert!(!label.eq_ignore_ascii_case("ok"));
        assert!(label.to_lowercase().contains("scale"));
    }
}
