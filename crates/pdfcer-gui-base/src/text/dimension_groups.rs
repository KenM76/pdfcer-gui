//! # `text::dimension_groups` — the words the Manage-groups window shows
//!
//! ## Rule 15, and why this catalog says "the dimensions you draw"
//!
//! A **ce dimension** is one pdfcer authors. A **pdf dimension** is CAD-exported
//! page content pdfcer reads and must not silently alter. The distinction is
//! ours, not the operator's, and it has already sent one investigation down the
//! wrong path — so this catalog does what [`crate::text::scale`] does and avoids
//! the bare word entirely. On screen it is *"the dimensions you draw"*, which is
//! unambiguous without asking a drafter to learn a term that exists for our
//! benefit.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/dimension_groups.md`.

use pdfcer_core::dimension::{ArrowForm, DimStandard, ScaleState, Unit};

/// The heading over the rename and delete controls.
#[must_use]
pub const fn identity_heading() -> &'static str {
    "Rename or remove this group"
}

/// The heading over the scale phrase, the Set-scale button and the unit combo.
#[must_use]
pub const fn scale_heading() -> &'static str {
    "Scale and unit"
}

/// The paragraph under the title.
#[must_use]
pub const fn intro() -> &'static str {
    "A group is a set of the dimensions you draw that share one scale, one \
     unit, one drafting standard and one set of appearance defaults. Changing \
     any of those redraws every dimension already in the group, on every page."
}

/// The heading over the list of groups.
#[must_use]
pub const fn groups_heading() -> &'static str {
    "Groups in this document"
}

/// The column header for the radio that chooses where the next dimension goes.
#[must_use]
pub const fn draw_into_heading() -> &'static str {
    "Draw into"
}

/// The hint under the draw-into column.
#[must_use]
pub const fn draw_into_hint() -> &'static str {
    "The next dimension you draw joins the group ticked here. Dimensions \
     already placed stay where they are."
}

/// How many dimensions are in a group.
#[must_use]
pub fn member_count(n: usize) -> String {
    match n {
        0 => "no dimensions yet".to_owned(),
        1 => "1 dimension".to_owned(),
        _ => format!("{n} dimensions"),
    }
}

/// A group's scale, as a phrase.
#[must_use]
pub fn scale_phrase(scale: ScaleState, unit: Unit) -> String {
    match scale {
        ScaleState::NeverSet => pdfcer_core::dimension::NO_SCALE_DISCLOSURE.to_owned(),
        ScaleState::OneToOne => "1:1 — full size".to_owned(),
        // The ratio an operator recognises, derived the one way the engine
        // derives it. `effective_scale` is `None` only for `NeverSet`, which
        // the arm above already took, so the fallback below is unreachable —
        // and is spelled rather than unwrapped because an unreachable panic in
        // a label is still a panic in a label.
        ScaleState::Calibrated { .. } => scale.effective_scale(unit).map_or_else(
            || "calibrated".to_owned(),
            |per_point| format!("{per_point:.6} {} per point", unit_abbrev(unit)),
        ),
    }
}

/// The short unit tag used inside a phrase, where the full name would read as
/// a label rather than as part of a sentence.
#[must_use]
pub const fn unit_abbrev(unit: Unit) -> &'static str {
    match unit {
        Unit::Millimeter => "mm",
        Unit::Centimeter => "cm",
        Unit::Meter => "m",
        Unit::Kilometer => "km",
        Unit::Inch => "in",
        Unit::DecimalFeet | Unit::FeetInches => "ft",
        Unit::Yard => "yd",
        Unit::Mile => "mi",
    }
}

/// The name of a drafting standard.
#[must_use]
pub const fn standard_name(standard: DimStandard) -> &'static str {
    match standard {
        DimStandard::Ansi => "ANSI / ASME",
        DimStandard::Iso => "ISO",
    }
}

/// The heading over the drafting-standard choice.
#[must_use]
pub const fn standard_heading() -> &'static str {
    "Drafting standard"
}

/// What the drafting standard governs.
#[must_use]
pub const fn standard_hint() -> &'static str {
    "Sets the terminator form, whether the dimension line breaks for its text, \
     and how the extension lines are spaced. pdfcer draws in the style of each \
     standard; it does not certify conformance to either."
}

/// The heading over the layer switch.
#[must_use]
pub const fn layer_heading() -> &'static str {
    "Layer"
}

/// The layer switch's label.
#[must_use]
pub const fn layer_visible() -> &'static str {
    "Show this group's dimensions"
}

/// What hiding a layer actually does, said once so nobody assumes it is a view
/// toggle.
#[must_use]
pub const fn layer_hint() -> &'static str {
    "This is saved into the document, not just applied here: a reader opening \
     the file afterwards sees the group hidden too. To hide a layer only for \
     yourself, use View > Layers."
}

/// Why the default group has no layer switch.
#[must_use]
pub const fn layer_default_group() -> &'static str {
    "The default group cannot be hidden — it is where a dimension goes when no \
     other group has been chosen, so hiding it could make a drawing's \
     dimensions vanish with nothing on screen to say where they went."
}

/// The heading over the new-group controls.
#[must_use]
pub const fn new_heading() -> &'static str {
    "Add a group"
}

/// The name field's label.
#[must_use]
pub const fn new_name_label() -> &'static str {
    "Name"
}

/// The unit combo's label.
#[must_use]
pub const fn new_unit_label() -> &'static str {
    "Unit"
}

/// Why the unit is asked for at creation.
#[must_use]
pub const fn new_unit_hint() -> &'static str {
    "The unit decides how the group's numbers are written to begin with — \
     millimetres in decimals, inches in eighths. Both can be changed \
     afterwards."
}

/// The button that creates the group.
#[must_use]
pub const fn new_button() -> &'static str {
    "Add group"
}

/// Why the Add button is unavailable with an empty name.
#[must_use]
pub const fn new_needs_a_name() -> &'static str {
    "Type a name first. A group with no name is a row in this list that nothing \
     distinguishes from the one above it."
}

/// The button that opens the scale window for the selected group.
#[must_use]
pub const fn set_scale_button() -> &'static str {
    "Set scale…"
}

/// The rename field's label.
#[must_use]
pub const fn rename_label() -> &'static str {
    "Name"
}

/// The button that commits a rename.
#[must_use]
pub const fn rename_button() -> &'static str {
    "Rename"
}

/// The button that removes a group.
#[must_use]
pub const fn delete_button() -> &'static str {
    "Delete group"
}

/// Why the default group has no Delete.
#[must_use]
pub const fn delete_default_group() -> &'static str {
    "The default group cannot be removed — it is where a dimension goes when no \
     other group has been chosen."
}

/// **A populated group is not deleted; the operator is asked.**
#[must_use]
pub fn delete_needs_a_home(members: usize) -> String {
    if members == 1 {
        "This group holds 1 dimension. Choose where it should go before the \
         group can be removed."
            .to_owned()
    } else {
        format!(
            "This group holds {members} dimensions. Choose where they should go \
             before the group can be removed."
        )
    }
}

/// The label on the destination picker for a populated group's members.
#[must_use]
pub const fn delete_move_to() -> &'static str {
    "Move them to"
}

/// What moving members to another group DOES to them, said before it happens.
#[must_use]
pub const fn delete_move_changes_labels() -> &'static str {
    "They will be re-measured against the group they move to, so the numbers \
     they print may change."
}

/// Why *delete the dimensions as well* is not on offer.
#[must_use]
pub const fn delete_cannot_remove_members() -> &'static str {
    "pdfcer will not delete the dimensions with the group. Select them on the \
     page and delete them first if that is what you want."
}

/// The label on the group's unit control.
#[must_use]
pub const fn unit_label() -> &'static str {
    "Unit"
}

/// Why changing a group's unit is a bigger act than it looks.
#[must_use]
pub const fn unit_hint() -> &'static str {
    "Changing the unit re-writes every dimension in the group, the same as \
     setting the scale does. The scale itself is not changed."
}

/// The heading over the appearance defaults.
#[must_use]
pub const fn appearance_heading() -> &'static str {
    "Appearance defaults for this group"
}

/// What the appearance defaults are defaults *for*.
#[must_use]
pub const fn appearance_hint() -> &'static str {
    "These apply to every dimension in the group that has not been given its \
     own value. Clear one and pdfcer's own default takes over again."
}

/// The label of the checkbox that turns a group default on.
#[must_use]
pub const fn set_by_group() -> &'static str {
    "Set by this group"
}

/// The caption on a property the group has not set.
#[must_use]
pub fn using_factory(value: &str) -> String {
    format!("using pdfcer's default, {value}")
}

/// **How many members a group edit will visibly move.**
#[must_use]
pub fn members_that_will_move(moving: usize, total: usize) -> String {
    match (moving, total) {
        (0, 0) => "nothing to redraw — this group has no dimensions yet".to_owned(),
        (0, _) => format!(
            "no change on screen — all {total} dimensions in this group have \
             their own value for this"
        ),
        (1, 1) => "will redraw the 1 dimension in this group".to_owned(),
        (m, t) if m == t => format!("will redraw all {t} dimensions in this group"),
        (m, t) => format!("will redraw {m} of the {t} dimensions in this group"),
    }
}

/// The names of the seven group-level appearance properties.
#[must_use]
pub const fn prop_text_height() -> &'static str {
    "Text height"
}

/// See [`prop_text_height`].
#[must_use]
pub const fn prop_line_width() -> &'static str {
    "Line width"
}

/// See [`prop_text_height`].
#[must_use]
pub const fn prop_arrow_length() -> &'static str {
    "Arrow length"
}

/// See [`prop_text_height`].
#[must_use]
pub const fn prop_arrow_form() -> &'static str {
    "Arrow form"
}

/// See [`prop_text_height`].
#[must_use]
pub const fn prop_color() -> &'static str {
    "Colour"
}

/// A point-valued property's inherited value, for the caption that stands in
/// for the editor when the group has not set it.
#[must_use]
pub fn points_value(v: f64) -> String {
    format!("{v}{}", points_suffix())
}

/// The unit suffix on the three point-valued properties.
#[must_use]
pub const fn points_suffix() -> &'static str {
    " pt"
}

/// The name of an arrowhead form.
#[must_use]
pub const fn arrow_form_name(form: ArrowForm) -> &'static str {
    match form {
        ArrowForm::Filled => "Filled triangle",
        ArrowForm::Open => "Open V",
        ArrowForm::Slash => "Slash",
        ArrowForm::Dot => "Dot",
        ArrowForm::None => "None",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The never-set scale phrase is the engine's own string, unaltered.
    #[test]
    fn the_no_scale_disclosure_is_the_engines_own_words() {
        assert_eq!(
            scale_phrase(ScaleState::NeverSet, Unit::Millimeter),
            pdfcer_core::dimension::NO_SCALE_DISCLOSURE,
            "the engine owns this wording so shells cannot invent their own"
        );
    }

    /// The moving-count sentence distinguishes all five cases it has to.
    #[test]
    fn the_moving_count_says_which_kind_of_zero_it_is() {
        assert!(members_that_will_move(0, 0).contains("no dimensions yet"));
        assert!(members_that_will_move(0, 40).contains("their own value"));
        assert!(members_that_will_move(3, 40).contains("3 of the 40"));
        assert!(members_that_will_move(40, 40).contains("all 40"));
        assert!(members_that_will_move(1, 1).contains("the 1 dimension"));
    }

    /// Every unit and every arrow form has a name; none falls through to a
    /// debug rendering.
    #[test]
    fn every_engine_variant_is_named_in_english() {
        for unit in Unit::all().iter().copied() {
            let abbrev = unit_abbrev(unit);
            assert!(!abbrev.is_empty(), "{unit:?} has no abbreviation");
            assert!(
                !abbrev.contains(char::is_uppercase),
                "{unit:?}'s abbreviation reads as a symbol, not a proper noun"
            );
        }
        for form in [
            ArrowForm::Filled,
            ArrowForm::Open,
            ArrowForm::Slash,
            ArrowForm::Dot,
            ArrowForm::None,
        ] {
            assert!(!arrow_form_name(form).is_empty(), "{form:?} has no name");
        }
    }
}
