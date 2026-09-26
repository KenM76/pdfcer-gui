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
//!
//! Design and rationale: `docs/modules/pdfcer-gui/text/scale.md`.

use pdfcer_core::dimension::{FractionMode, Unit};

/// The window's title.
#[must_use]
pub const fn window_title() -> &'static str {
    "Set scale"
}

/// The paragraph under the title.
#[must_use]
pub const fn intro() -> &'static str {
    "Until a scale is set, the dimensions you draw are measured in the page's \
     own units — the size on paper, not the size of the thing drawn. Setting a \
     scale here converts every dimension in this group, including ones you have \
     already placed."
}

/// Why the ratio path is what a cold-opened dialog offers.
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

/// **The group picker's label** -- `OPERATOR_REQUESTS.md` O193.
#[must_use]
pub const fn group_label() -> &'static str {
    "Set the scale of"
}

/// **The current-scale line** -- `OPERATOR_REQUESTS.md` O192.
#[must_use]
pub fn current_scale(phrase: &str) -> String {
    format!("Currently: {phrase}")
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
#[must_use]
pub fn unit_name(unit: Unit) -> &'static str {
    match unit {
        Unit::Millimeter => "Millimetres",
        Unit::Centimeter => "Centimetres",
        Unit::Meter => "Metres",
        Unit::Kilometer => "Kilometres",
        Unit::Inch => "Inches",
        Unit::DecimalFeet => "Feet (decimal)",
        Unit::FeetInches => "Feet and inches",
        Unit::Yard => "Yards",
        Unit::Mile => "Miles",
    }
}

/// A number style's name, including the *"use the unit's own default"* state.
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
#[must_use]
pub fn preview(ratio_label: &str, unit: Unit) -> String {
    format!(
        "Dimensions will read in {} at {ratio_label}.",
        unit_name(unit)
    )
}

/// The entry does not describe a scale.
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

/// The button that starts the two-point pick.
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
#[must_use]
pub const fn calibrate_note() -> &'static str {
    "Easier than a ratio if the drawing already states a dimension: point at it, type what it says, and pdfcer works the scale out."
}

/// What pdfcer measured, once the two points are picked.
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

    /// The intro discloses that dimensions are currently measuring the paper.
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
    #[test]
    fn every_unit_is_named_distinctly() {
        let units = Unit::all();
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
    #[test]
    fn the_commit_button_names_what_it_does() {
        let label = accept();
        assert!(!label.eq_ignore_ascii_case("ok"));
        assert!(label.to_lowercase().contains("scale"));
    }
}
