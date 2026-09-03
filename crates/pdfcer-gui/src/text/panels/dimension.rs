//! # `text::panels::dimension` — the words the ce-dimension properties section
//! shows
//!
//! ## What this surface is
//!
//! The **bottom tier** of `pdfcer-core`'s style cascade, made reachable. Eleven
//! properties, each with an override checkbox and a label saying which tier
//! supplied the value it is currently showing — which is
//! `docs/ui_specs/tool-options-dock-and-ce-dimension-properties.md`
//! Amendment B §B.4's second bullet, and `FEATURES.md`'s
//!
//! > ce-dimension style AND tolerance in the GUI — **one panel covering
//! > both**, showing which values are inherited and which are overridden and
//! > letting the override be set. The model and the CLI already do all of it;
//! > only the disclosure surface is missing.
//!
//! ## Rule 15, as everywhere in this feature
//!
//! A **ce dimension** is one pdfcer authors; a **pdf dimension** is CAD-exported
//! page content pdfcer reads and must not alter. This catalog says *"this
//! dimension"* only where the operator has one selected and can see which, and
//! otherwise says *"the dimensions you draw"*, exactly as
//! [`crate::text::scale`] and [`crate::text::dimension_groups`] do.
//!
//! ## ★ One vocabulary, shared with the CLI
//!
//! `pdfcer dimension-style` names these eleven `unit`, `fraction`,
//! `decimal-marker`, `standard`, `text-height`, `line-width`, `arrow-length`,
//! `arrow-form`, `color`, `tolerance` and `tolerance-places`. Amendment B §B.5
//! states the hazard of diverging: *"a panel using different words for the same
//! nine things is how an operator ends up unable to script what he just
//! clicked."* These are the same words, in the operator's English.
//!
//! ## ★ What this catalog must NOT do: build a label
//!
//! `docs/core-api/03-capabilities.md` §1.6 trap (b) is explicit, and it cost
//! `pdfcer` a shipped defect: *"A panel that previews 'nominal + tolerance' by
//! concatenation **will disagree with the bytes in the page** for every limit
//! tolerance"*, because a limit tolerance **suppresses** the nominal rather
//! than printing beside it — and `Basic` prints no text at all, the box being
//! the notation.
//!
//! So nothing here concatenates a nominal and a tolerance. The measurement is
//! rendered from the engine's own `MeasurementDisplay`, and what a tolerance
//! will *do* to the printed label is stated in a sentence
//! ([`tolerance_suppresses_nominal`], [`tolerance_is_a_box`]) rather than
//! demonstrated by a preview this module would have to derive a second time.

use pdfcer_core::dimension::{DecimalMarker, DimStandard, StyleSource, Tolerance, Unit};

/// The section's heading.
#[must_use]
pub const fn heading() -> &'static str {
    "Selected dimension"
}

/// Shown when the selected annotation is a ce dimension whose sidecar record
/// cannot be found.
///
/// ★ **Reachable, and not a bug in this panel.** A `/Line` with
/// `/IT /LineDimension` can arrive in a document from anywhere — an insert
/// from another file, a merge, an earlier pdfcer that wrote the annotation and
/// whose sidecar was dropped by a third-party tool's rewrite. The annotation is
/// then genuinely a ce dimension to look at and genuinely not one to edit,
/// because every verb takes a `DimensionId` that does not exist.
///
/// Saying so beats an empty section, which reads as the panel failing.
#[must_use]
pub const fn no_record() -> &'static str {
    "This looks like a dimension pdfcer authored, but this document carries no \
     record of it — so its group, its scale and its style cannot be read or \
     changed. It can still be moved and deleted."
}

/// The label on the group readout.
#[must_use]
pub const fn group_label() -> &'static str {
    "Group"
}

/// ★ **What moving a ce dimension to another group DOES**, said before it is
/// done.
///
/// The one disclosure this control cannot ship without, and the engine spent a
/// section of its reply making sure a shell would not miss it:
/// `set_dimension_group` is **not a field assignment**. A ce dimension's label
/// is derived from its group — the scale it is measured at, the unit and
/// precision it is formatted with, the standard it is drawn to — so the verb
/// re-measures and regenerates the appearance, and **the number on the page
/// changes**.
///
/// The engine's own measured example, from the test that pins it: the same
/// geometry reads `70.6 mm` in a 1:1 millimetre group and `2.00 m` in a metre
/// group at 1 cm per point. Both are correct. An operator who expected a
/// filing change and got a different number would be right to file a bug.
///
/// Worded as *"may change"* rather than *"will change"* because moving between
/// two groups at the same scale and unit changes nothing, and that is the
/// commoner case — an operator tidying which group a dimension belongs to.
#[must_use]
pub const fn group_move_changes_the_number() -> &'static str {
    "Moving a dimension re-measures it against the group it joins, so the \
     number it prints may change."
}

/// The label on the measured-value readout.
#[must_use]
pub const fn measured_label() -> &'static str {
    "Measured"
}

/// The label on the radius / diameter choice.
#[must_use]
pub const fn display_label() -> &'static str {
    "Show as"
}

/// The radius option.
#[must_use]
pub const fn display_radius() -> &'static str {
    "Radius"
}

/// The diameter option.
#[must_use]
pub const fn display_diameter() -> &'static str {
    "Diameter"
}

/// What switching between them does, and does not, change.
///
/// ★ The point of the sentence is the second half. Both numbers come from
/// **one fitted circle**, so this is a change of notation and not a
/// re-measurement — which is why it was worth building rather than telling the
/// operator to delete and re-draw. The ui-spec called that a *"real, named
/// usability gap"*.
#[must_use]
pub const fn display_hint() -> &'static str {
    "Both are read off the same fitted circle, so this changes what is printed \
     and not what was measured."
}

/// The heading over the eleven overrides.
#[must_use]
pub const fn overrides_heading() -> &'static str {
    "This dimension's own settings"
}

/// What an override is, in one sentence.
#[must_use]
pub const fn overrides_hint() -> &'static str {
    "Every setting here is inherited from the group until you tick it. \
     Clearing a tick restores what the group says — it does not freeze the \
     value that was showing."
}

/// The label on each override checkbox.
///
/// ★ **"Set on this dimension", not "Enabled".** The `Option` is the checkbox
/// and `None` does not mean *off*: the property has a value, supplied by a tier
/// above. "Enabled" would say the property is absent when unticked, which is
/// false for all eleven of them.
#[must_use]
pub const fn set_here() -> &'static str {
    "Set on this dimension"
}

/// Which tier supplied the value currently in force.
///
/// ★ **`Factory` is rendered as *"pdfcer's default"* and NOT as *"inherited"***
/// — the three tiers are three different answers to *"where did this come
/// from?"*, and collapsing two of them would hide the one fact the operator
/// asked for: whether a group edit is going to move this. It is, for both
/// `Factory` and `Group`, and [`follows_group_note`] says so beneath.
#[must_use]
pub const fn source_name(source: StyleSource) -> &'static str {
    match source {
        StyleSource::Factory => "using pdfcer's default",
        StyleSource::Group => "using the group's setting",
        StyleSource::Dimension => "set on this dimension",
    }
}

/// Whether a group edit will move this property.
///
/// ★ The predicate is `StyleSource::follows_group()`, which is **true for
/// `Factory` as well as `Group`** — a property nobody has set anywhere still
/// moves the moment the group speaks. This sentence is the operator-facing
/// half of that, and it is why `Factory` and `Group` are named separately above
/// and answered together here.
#[must_use]
pub const fn follows_group_note(follows: bool) -> &'static str {
    if follows {
        "Changing the group changes this."
    } else {
        "Changing the group leaves this alone."
    }
}

/// The name of each of the eleven properties. See the module header on why
/// these are the CLI's words.
#[must_use]
pub const fn prop_unit() -> &'static str {
    "Unit"
}

/// See [`prop_unit`].
#[must_use]
pub const fn prop_fraction() -> &'static str {
    "Precision"
}

/// See [`prop_unit`].
#[must_use]
pub const fn prop_decimal_marker() -> &'static str {
    "Decimal marker"
}

/// See [`prop_unit`].
#[must_use]
pub const fn prop_standard() -> &'static str {
    "Drafting standard"
}

/// See [`prop_unit`].
#[must_use]
pub const fn prop_text_height() -> &'static str {
    "Text height"
}

/// See [`prop_unit`].
#[must_use]
pub const fn prop_line_width() -> &'static str {
    "Line width"
}

/// See [`prop_unit`].
#[must_use]
pub const fn prop_arrow_length() -> &'static str {
    "Arrow length"
}

/// See [`prop_unit`].
#[must_use]
pub const fn prop_arrow_form() -> &'static str {
    "Arrow form"
}

/// See [`prop_unit`].
#[must_use]
pub const fn prop_color() -> &'static str {
    "Colour"
}

/// See [`prop_unit`].
#[must_use]
pub const fn prop_tolerance() -> &'static str {
    "Tolerance"
}

/// See [`prop_unit`].
#[must_use]
pub const fn prop_tolerance_places() -> &'static str {
    "Tolerance decimals"
}

/// The two precision modes.
#[must_use]
pub const fn precision_decimal() -> &'static str {
    "Decimals"
}

/// See [`precision_decimal`].
#[must_use]
pub const fn precision_fraction() -> &'static str {
    "Fractions"
}

/// The label on the decimal-places spinner.
#[must_use]
pub const fn precision_places() -> &'static str {
    "Places"
}

/// The label on the fraction-denominator combo.
#[must_use]
pub const fn precision_denominator() -> &'static str {
    "Nearest"
}

/// One entry in the denominator combo.
#[must_use]
pub fn precision_denominator_entry(denominator: u32) -> String {
    format!("1/{denominator}")
}

/// The reduce checkbox.
#[must_use]
pub const fn precision_reduce() -> &'static str {
    "Reduce the fraction"
}

/// ★ Why *not* reducing is the drafting convention rather than an oversight.
///
/// `pdfcer-core`'s own `FractionMode` doc calls the unreduced form *"the
/// architectural convention (`6/8"` not `3/4"`)"*, which is the opposite of
/// what a reader's arithmetic instinct expects — so it is stated rather than
/// left to be discovered by an operator who ticks the box to "fix" it.
#[must_use]
pub const fn precision_reduce_hint() -> &'static str {
    "Architectural drawings usually keep the denominator — 6/8 rather than \
     3/4 — so the run of dimensions along an elevation reads at a glance."
}

/// A decimal marker, as it is written.
#[must_use]
pub const fn decimal_marker_name(marker: DecimalMarker) -> &'static str {
    match marker {
        DecimalMarker::Point => "Point — 1.5",
        DecimalMarker::Comma => "Comma — 1,5",
    }
}

/// A drafting standard.
#[must_use]
pub const fn standard_name(standard: DimStandard) -> &'static str {
    match standard {
        DimStandard::Ansi => "ANSI / ASME",
        DimStandard::Iso => "ISO",
    }
}

/// A unit, for the per-dimension override combo.
///
/// Delegates to [`crate::text::scale::unit_name`] rather than repeating six
/// arms, because a unit is a unit wherever it is offered and two spellings of
/// "Feet and inches" in one application is exactly the drift the ui_text
/// catalog exists to prevent.
#[must_use]
pub fn unit_name(unit: Unit) -> &'static str {
    crate::text::scale::unit_name(unit)
}

/// The name of a tolerance form.
///
/// ★ **`Basic` is named *"Basic — boxed, no ± text"*.** Its caption is the
/// empty string and the **box** is the notation, drawn by the appearance-stream
/// baker. An operator who picked "Basic" and saw no numbers appear would
/// reasonably conclude the control had failed, so the name says what to expect
/// before it is chosen.
#[must_use]
pub const fn tolerance_name(tolerance: Tolerance) -> &'static str {
    match tolerance {
        Tolerance::None => "None",
        Tolerance::Basic => "Basic — boxed, no ± text",
        Tolerance::Symmetric { .. } => "Symmetric — ± one value",
        Tolerance::Deviation { .. } => "Deviation — separate + and −",
        Tolerance::Limit { .. } => "Limit — upper over lower",
        Tolerance::Min => "Minimum",
        Tolerance::Max => "Maximum",
    }
}

/// The symmetric magnitude's label.
#[must_use]
pub const fn tolerance_magnitude() -> &'static str {
    "±"
}

/// The deviation's upper field.
#[must_use]
pub const fn tolerance_plus() -> &'static str {
    "+"
}

/// The deviation's lower field.
#[must_use]
pub const fn tolerance_minus() -> &'static str {
    "−"
}

/// The limit's upper field.
#[must_use]
pub const fn tolerance_upper() -> &'static str {
    "Upper"
}

/// The limit's lower field.
#[must_use]
pub const fn tolerance_lower() -> &'static str {
    "Lower"
}

/// ★ **A limit tolerance replaces the number rather than sitting beside it.**
///
/// The single most important sentence in this catalog, and it is here because
/// the engine's appearance-stream baker branches on
/// `Tolerance::suppresses_nominal()` and prints `50.20/49.90` with **the
/// nominal gone**. An operator who set a limit expecting `50.00 50.20/49.90`
/// would find a drawing that says something else, and would find it after
/// plotting.
#[must_use]
pub const fn tolerance_suppresses_nominal() -> &'static str {
    "A limit tolerance replaces the measured number on the drawing — the sheet \
     will show the upper and lower values only."
}

/// What a Basic tolerance draws.
#[must_use]
pub const fn tolerance_is_a_box() -> &'static str {
    "Basic prints no ± text at all: the box around the number is the notation."
}

/// That tolerance values are in the displayed unit.
///
/// `pdfcer-core`'s `tolerance.rs` states it as a property of the model — the
/// values are *"in the displayed unit, never PDF points"* — and the caption it
/// emits carries **no unit suffix** in any branch, because a tolerance is read
/// in the nominal's unit and `"50.00 mm ±0.10 mm"` is not how a drawing is
/// written. The field therefore has no suffix either, and this says which unit
/// it means.
#[must_use]
pub fn tolerance_unit_note(unit: Unit) -> String {
    format!(
        "In {}, the same as the number it qualifies.",
        crate::text::dimension_groups::unit_abbrev(unit)
    )
}

/// A tolerance the engine refused, by its own name.
///
/// ★ **Never paraphrased, never clamped.** `ToleranceError`'s `Display` is
/// written for an operator — *"a symmetric tolerance's magnitude must not be
/// negative (write ±0.1, not ±-0.1)"* — and `pdfcer-core`'s own comment on the
/// refusal is the argument for showing it verbatim: *"a corrected value the
/// operator never saw is exactly the sneaky case."*
#[must_use]
pub fn tolerance_refused(reason: &str) -> String {
    format!("Not applied: {reason}")
}

/// The label on the tolerance-precision spinner's "follow the nominal" state.
///
/// `None` on `tolerance_places` means *use the same number of decimals as the
/// measurement itself*, which the reference tool hides inside a −3 sentinel in
/// the digit count. Naming it is the whole reason pdfcer spells it as an absent
/// value.
#[must_use]
pub const fn tolerance_places_follows() -> &'static str {
    "Same decimals as the measurement"
}

/// The label-override section's heading.
///
/// ★ *"What it says"*, not *"Label"* or *"Text"*. The operator is choosing
/// between the measured number and their own words, and the heading that makes
/// that obvious is the one phrased as the question they are answering.
#[must_use]
pub const fn label_heading() -> &'static str {
    "What it says"
}

/// The hint under the box.
///
/// ★★★ Both halves matter and neither is optional.
///
/// *"Leave empty to show the measurement"* is the **only** discoverable route
/// back: there is no Clear button, because clearing the box IS the restore —
/// the engine's `None` — and a control whose reset is invisible is a control an
/// operator gets stuck in.
///
/// *"The measurement is kept underneath"* is the fact that decides whether they
/// trust the feature at all. On a drawing, text that replaces a measured value
/// is indistinguishable from text that *changed* it, and those are a note and a
/// lie respectively. The engine keeps the measurement and restores it with no
/// re-measurement; this is where an operator learns that.
#[must_use]
pub const fn label_hint() -> &'static str {
    "Leave it empty to show the measurement. The measurement is kept underneath either way, so \
     clearing this brings back exactly the number that was there before."
}

/// Shown while an override is in force.
///
/// ★★ It says the measurement is still there and does **not** say what it is,
/// which is honest rather than coy: `DimensionRecord` does not carry the
/// measured value — it is computed from the geometry and the group's scale —
/// so this shell cannot print it here without re-measuring, and a number
/// derived a second way is exactly the divergence this project keeps meeting.
///
/// ⇒ Filed as a gap. The interim sentence is the half that is true and useful.
#[must_use]
pub const fn label_overridden() -> &'static str {
    "This is showing your text instead of the measurement."
}

/// **The caption is on** — and this names the number it hid.
///
/// ★★★ The measured value goes in the receipt because this is the one moment
/// the shell has it. `DimensionRecord` does not carry the measurement — it is
/// computed from the geometry and the group's scale — and only
/// `DimensionLabelChange` hands it back. So an operator who overrides a ce
/// dimension and wants to know what it actually measured has exactly one
/// chance to read it, here.
#[must_use]
pub fn label_set(measured: &str) -> String {
    format!(
        "Showing your text instead. It still measures {measured}, and clearing the box brings that back."
    )
}

/// **The caption is off** — and this names the number that came back.
///
/// ★★ It names the value deliberately, because the reassurance this whole
/// feature rests on is that clearing an override restores the ORIGINAL
/// measurement rather than re-measuring. A receipt carrying the number is what
/// lets an operator confirm that rather than take it on trust.
#[must_use]
pub fn label_restored(printed: &str) -> String {
    format!("Back to the measurement: {printed}.")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every tolerance form is named, and `Basic` warns about its own
    /// emptiness in the name itself.
    #[test]
    fn every_tolerance_form_is_named_and_basic_says_it_prints_nothing() {
        for t in [
            Tolerance::None,
            Tolerance::Basic,
            Tolerance::Symmetric { magnitude: 0.1 },
            Tolerance::Deviation {
                plus: 0.1,
                minus: 0.1,
            },
            Tolerance::Limit {
                upper: 1.0,
                lower: 0.0,
            },
            Tolerance::Min,
            Tolerance::Max,
        ] {
            assert!(!tolerance_name(t).is_empty(), "{t:?} has no name");
        }
        assert!(
            tolerance_name(Tolerance::Basic).contains("no ±"),
            "an operator choosing Basic must know before they choose that no \
             numbers will appear"
        );
    }

    /// The three tiers are three different sentences.
    ///
    /// ★ The property under test is that `Factory` and `Group` do **not**
    /// collapse into one "inherited". They differ in what a `--clear` on the
    /// group would do, and an operator reading "inherited" cannot tell which
    /// tier to go and edit.
    #[test]
    fn the_three_style_sources_read_differently() {
        let names = [
            source_name(StyleSource::Factory),
            source_name(StyleSource::Group),
            source_name(StyleSource::Dimension),
        ];
        let unique: std::collections::BTreeSet<&str> = names.iter().copied().collect();
        assert_eq!(
            unique.len(),
            3,
            "{names:?} must be three distinct sentences"
        );
    }

    /// The follows-group note answers the question `follows_group()` asks,
    /// in both directions.
    #[test]
    fn the_follows_group_note_takes_a_side() {
        assert_ne!(follows_group_note(true), follows_group_note(false));
        assert!(follows_group_note(true).contains("changes this"));
        assert!(follows_group_note(false).contains("leaves this alone"));
    }
}
