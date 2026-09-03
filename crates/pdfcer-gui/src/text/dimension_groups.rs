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
//! ## ★ The hardest thing this window has to explain
//!
//! Not what a group *is* — a drafter already has that idea from every CAD
//! package they have used. What is genuinely new is that **a group edit reaches
//! backwards**: setting a scale, a standard or an appearance default rewrites
//! every dimension already placed in that group, wherever it is, including on
//! pages that are not on screen.
//!
//! The operator named the fear himself, quoted in the ui-spec: *"cannot change
//! one and be surprised 40 others changed or didn't."* So every group-level
//! control in this window is accompanied by a **count of what will move**, and
//! the count is computed before the edit rather than reported after it.
//!
//! ## ★ And the number in that count is NOT the engine's return value
//!
//! `EditSession::set_group_style` returns the number of members **regenerated**,
//! which is every wired member — including the ones that override the property
//! being changed, because regenerating an overrider is byte-identical and free
//! in the diff. Showing that number would be worse than showing none: it is a
//! real number, plausibly labelled, answering a different question, and an
//! operator who reads *"40 dimensions will change"* and sees three change has
//! been misled by a fact.
//!
//! [`members_that_will_move`] therefore takes the count the *caller* computed
//! from `StyleProvenance::follows_group()`, and the caller computes it before
//! the edit is applied. See `dialogs::dimension_groups::style`.

use pdfcer_core::dimension::{ArrowForm, DimStandard, ScaleState, Unit};

/// The heading over the rename and delete controls.
///
/// ★ Not "Name". The section carries **both** verbs that act on the group's
/// identity, and a fold captioned "Name" would read as a text field — which is
/// exactly what it looks like when folded shut, with Delete hidden inside it.
/// A caption is a promise about what is under it and this one has to name the
/// destructive half.
#[must_use]
pub const fn identity_heading() -> &'static str {
    "Rename or remove this group"
}

/// The heading over the scale phrase, the Set-scale button and the unit combo.
///
/// One caption for both, because `set_group_scale` takes the scale and the
/// number format together: an operator changing the unit is calling the verb
/// that also carries the scale, and two folds would hide that from them.
#[must_use]
pub const fn scale_heading() -> &'static str {
    "Scale and unit"
}

/// The paragraph under the title.
///
/// Says what a group *carries*, because that is the fact every control below
/// depends on and the one nothing else on screen states. The second sentence is
/// the reach-backwards disclosure in its shortest honest form; the per-control
/// counts make it concrete.
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
///
/// ★ This is the control the operator asked for by name and could not find —
/// *"I still can't get to edit dimension groups when I click on it."* The
/// authoring group was fixed at the default for the whole life of the build
/// before this window, so a second group could be created from nowhere and
/// joined by nothing.
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
///
/// ★ The `NeverSet` arm renders `pdfcer_core::dimension::NO_SCALE_DISCLOSURE`
/// **verbatim**. That string lives in the engine precisely so shells cannot
/// invent their own wording for it, and
/// `docs/core-api/03-capabilities.md` §1.5 obligation 2 requires it be shown
/// rather than paraphrased. Do not "improve" it here.
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
        Unit::Inch => "in",
        Unit::DecimalFeet | Unit::FeetInches => "ft",
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
///
/// ★ Deliberately does **not** claim conformance. `pdfcer-core`'s own
/// `DimStandard` doc applies the same discipline — pdfcer draws *ISO-style*,
/// never *ISO 129-1 conformant*, because the standard is paywalled and was not
/// obtained. A window that promised conformance would be making a claim the
/// engine explicitly declines to make.
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
///
/// ★ It is not `View ▸ Layers`. That one changes what *this window* draws and
/// nothing a save would write; this one writes the group's default visibility
/// into the document's optional-content configuration, so it is what the file
/// tells the next reader — in any viewer that honours optional content.
#[must_use]
pub const fn layer_hint() -> &'static str {
    "This is saved into the document, not just applied here: a reader opening \
     the file afterwards sees the group hidden too. To hide a layer only for \
     yourself, use View > Layers."
}

/// Why the default group has no layer switch.
///
/// R9: an affordance that cannot be honoured is not drawn. The engine refuses
/// to hide the default group, so the control is absent and this sentence says
/// why — an omission with no explanation reads as a bug.
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
///
/// Greying with an explanation, not silence: this is the *temporarily*
/// unavailable case R9 reserves greying for, and the reason is one the operator
/// can act on in one keystroke.
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

/// ★ Why the default group has no Delete.
///
/// R9 again, and the same shape as the layer switch above it: the engine
/// refuses, so the control is **absent** rather than offered and declined. The
/// sentence is what stops the omission reading as a bug.
#[must_use]
pub const fn delete_default_group() -> &'static str {
    "The default group cannot be removed — it is where a dimension goes when no \
     other group has been chosen."
}

/// ★ **A populated group is not deleted; the operator is asked.**
///
/// The engine refuses by default and puts the **count** in the refusal, and its
/// reply says why in a line worth keeping: *"this group is not empty"* and
/// *"this group holds forty dimensions"* prompt different decisions, and only a
/// surface can put that question in front of an operator.
///
/// So this is the question, with the number in it. The two answers below are
/// the only two the engine offers — and the third an operator might expect,
/// *delete the dimensions too*, is deliberately absent from the engine and is
/// therefore absent here. Saying so is [`delete_cannot_remove_members`]'s job.
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

/// ★ What moving members to another group DOES to them, said before it happens.
///
/// Not a warning — a fact, and the one an operator would otherwise discover by
/// reading a drawing. A ce dimension's label is derived from its group's scale,
/// unit and number format, so members arriving in a different group are
/// **re-measured** and print different numbers. The engine's own measured
/// example: `70.6 mm` in a 1:1 millimetre group becomes `2.00 m` in a metre
/// group at 1 cm per point. Same geometry, different group, correctly different
/// label.
#[must_use]
pub const fn delete_move_changes_labels() -> &'static str {
    "They will be re-measured against the group they move to, so the numbers \
     they print may change."
}

/// Why *delete the dimensions as well* is not on offer.
///
/// ★ Stated because it is the answer an operator may be reaching for, and its
/// absence is a decision on the engine's side with a reason worth passing on
/// rather than a gap. Deleting a ce dimension also removes its annotation from
/// the page, so doing it inside the group verb would be a second implementation
/// of that removal — and looping the existing one would make undoing a group
/// deletion take one press per member and be able to stop halfway.
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

/// ★ Why changing a group's unit is a bigger act than it looks.
///
/// It goes through `set_group_scale`, because a unit lives inside the group's
/// `NumberFormat` and there is no narrower verb — the engine's reply called
/// that *"a discoverability problem, not a missing capability"*, and this
/// sentence is the discoverability half.
///
/// The consequence is real and is the reason the sentence exists: every member
/// is re-formatted and its appearance regenerated, exactly as a recalibration
/// does. An operator who expects a unit change to be cosmetic is expecting the
/// wrong thing.
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
///
/// ★ **The checkbox IS the `Option`.** `GroupStyle`'s seven fields are each an
/// `Option`: clear means *this group has not spoken, use the factory value*,
/// ticked means *this group says this*. Rendering the tick as "set by this
/// group" rather than as "enabled" is what keeps the two states legible —
/// "enabled" would imply the property is off when unticked, and it is not, it
/// is inherited.
#[must_use]
pub const fn set_by_group() -> &'static str {
    "Set by this group"
}

/// The caption on a property the group has not set.
#[must_use]
pub fn using_factory(value: &str) -> String {
    format!("using pdfcer's default, {value}")
}

/// ★ **How many members a group edit will visibly move.**
///
/// `moving` is computed by the caller from `StyleProvenance::follows_group()`
/// over the group's members, **before** the edit. It is deliberately not the
/// engine's returned count — see this module's header for why that number
/// answers a different question.
///
/// `total` is stated beside it so the difference is visible rather than
/// implied: *"3 of 40"* tells an operator that thirty-seven members have their
/// own value, which is the fact that stops the change being a surprise in
/// either direction.
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
///
/// ★ **One vocabulary, shared with the CLI.** `pdfcer group-style` uses
/// `text-height`, `line-width`, `arrow-length`, `arrow-form`, `color`,
/// `tolerance` and `tolerance-places` for these same seven, and the ui-spec's
/// Amendment B §B.5 names the hazard of diverging: *"a panel using different
/// words for the same nine things is how an operator ends up unable to script
/// what he just clicked."* These are the same words, capitalised for a label
/// and spelled in the operator's own English where the flag is hyphenated.
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
///
/// In the catalog rather than formatted at the call site because it is
/// operator-visible text — and because the unit belongs beside the number in
/// exactly one place. `10` and `10 pt` are different claims.
#[must_use]
pub fn points_value(v: f64) -> String {
    format!("{v}{}", points_suffix())
}

/// The unit suffix on the three point-valued properties.
///
/// Points, and said so, because a text height of `10` is meaningless without it
/// and because these are the one place in this window where the number is in
/// **paper** units rather than in the group's own unit — a dimension's text is
/// 10 pt tall whatever the drawing is scaled at.
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
    ///
    /// ★ Asserted against the constant rather than against a literal, which is
    /// the difference between a test that pins the *relation* and one that pins
    /// two copies of a magnitude. `NO_SURFACE.md` records the day a test in
    /// this crate asserted a literal triple against a function returning the
    /// literal triple and could therefore never fail; this is the shape that
    /// does fail if somebody paraphrases.
    #[test]
    fn the_no_scale_disclosure_is_the_engines_own_words() {
        assert_eq!(
            scale_phrase(ScaleState::NeverSet, Unit::Millimeter),
            pdfcer_core::dimension::NO_SCALE_DISCLOSURE,
            "the engine owns this wording so shells cannot invent their own"
        );
    }

    /// The moving-count sentence distinguishes all five cases it has to.
    ///
    /// The one that matters is `(0, n)`: an operator pressing a control that
    /// will change nothing on screen, because every member overrides the
    /// property. Reporting "0" as a bare number would read as a failure; the
    /// sentence says which of the two zeroes it is.
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
        for unit in Unit::all() {
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
