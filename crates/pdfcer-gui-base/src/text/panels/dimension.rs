//! # `text::panels::dimension` — the words the ce-dimension properties section
//! shows
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/panels/dimension.md`.

use pdfcer_core::dimension::{
    AnchorRole, ArcCondition, DecimalMarker, DimStandard, DimensionEnd, DimensionKind, StyleSource,
    Tolerance, Unit, Unsolvable,
};

/// The section's heading.
#[must_use]
pub const fn heading() -> &'static str {
    "Selected dimension"
}

/// Shown when the selected annotation is a ce dimension whose sidecar record
/// cannot be found.
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

/// **What moving a ce dimension to another group DOES**, said before it is
/// done.
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

/// The radius or diameter ce dimension's circle on/off checkbox.
#[must_use]
pub const fn display_circle() -> &'static str {
    "Draw the circle"
}

/// Hover text on [`display_circle`].
#[must_use]
pub const fn display_circle_hover() -> &'static str {
    "Off, the measurement, leader and any centre mark stay; only the circle is left out."
}

/// The heading over a ce dimension's anchored points.
#[must_use]
pub const fn anchors_heading() -> &'static str {
    "Anchored points"
}

/// Which measured point an anchor row is about, by the dependent's kind.
#[must_use]
pub fn anchor_point(kind: &DimensionKind, index: usize) -> String {
    match kind {
        DimensionKind::Linear { .. } if index == 0 => "First end".to_owned(),
        DimensionKind::Linear { .. } => "Second end".to_owned(),
        DimensionKind::Angular { .. } => "Apex".to_owned(),
        _ => format!("Corner {}", index + 1),
    }
}

/// What an anchored point follows: the role on the target, then the target.
#[must_use]
pub fn anchor_target(role: AnchorRole, target: Option<&DimensionKind>) -> String {
    let noun = match target {
        Some(DimensionKind::CentreLine { .. }) => "the centre line",
        Some(DimensionKind::CentreMark { .. }) => "the centre mark",
        Some(DimensionKind::Circular { .. }) => "the radius or diameter ce dimension",
        _ => "a ce dimension no longer in the document",
    };
    let role = match role {
        AnchorRole::MarkCentre | AnchorRole::Rim(ArcCondition::Centre) => "the centre of",
        AnchorRole::LineEnd(DimensionEnd::A) => "the first end of",
        AnchorRole::LineEnd(_) => "the second end of",
        AnchorRole::LineMidpoint => "the midpoint of",
        AnchorRole::LineAxis => "the line through",
        AnchorRole::Rim(ArcCondition::Min) => "the near edge of the circle of",
        AnchorRole::Rim(_) => "the far edge of the circle of",
        _ => "a point of",
    };
    format!("follows {role} {noun}")
}

/// The button that ends an anchor.
#[must_use]
pub const fn anchor_detach() -> &'static str {
    "Detach"
}

/// Hover text on [`anchor_detach`].
#[must_use]
pub const fn anchor_detach_hover() -> &'static str {
    "Stop following. The point stays where it is now and keeps its value."
}

/// An anchor the engine could not solve: the point kept its last position.
#[must_use]
pub fn anchor_unsolved(why: Unsolvable) -> String {
    format!("This point could not follow its anchor ({why}), so it stays where it was.")
}

/// The centre-mark switch on a radius or diameter ce dimension.
#[must_use]
pub const fn centre_mark_on() -> &'static str {
    "Centre mark"
}

/// Hover text on [`centre_mark_on`].
#[must_use]
pub const fn centre_mark_on_hover() -> &'static str {
    "A cross at the circle's centre. Its size, gap and stroke are in the style rows."
}

/// The extension-lines switch under a centre mark.
#[must_use]
pub const fn centre_mark_extension_lines() -> &'static str {
    "Extension lines"
}

/// Hover text on [`centre_mark_extension_lines`].
#[must_use]
pub const fn centre_mark_extension_lines_hover() -> &'static str {
    "Lines from the cross out past the rim."
}

/// The disclosure when adding or removing a centre mark moved the label.
#[must_use]
pub const fn centre_mark_label_moved() -> &'static str {
    "The label moved outside the rim to clear the centre mark."
}

/// What switching between them does, and does not, change.
#[must_use]
pub const fn display_hint() -> &'static str {
    "Both are read off the same fitted circle, so this changes what is printed \
     and not what was measured."
}

/// The perimeter option of a closed outline's Show as choice.
#[must_use]
pub const fn display_perimeter() -> &'static str {
    "Perimeter"
}

/// The area option of a closed outline's Show as choice.
#[must_use]
pub const fn display_area() -> &'static str {
    "Area"
}

/// What switching a closed outline between perimeter and area changes.
#[must_use]
pub const fn display_area_hint() -> &'static str {
    "Both are read off the same corners, so this changes what is printed and not what was traced."
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
#[must_use]
pub const fn set_here() -> &'static str {
    "Set on this dimension"
}

/// Which tier supplied the value currently in force.
#[must_use]
pub const fn source_name(source: StyleSource) -> &'static str {
    match source {
        StyleSource::Factory => "using pdfcer's default",
        StyleSource::Group => "using the group's setting",
        StyleSource::Dimension => "set on this dimension",
    }
}

/// Whether a group edit will move this property.
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

/// The dash-pattern row's label.
#[must_use]
pub const fn prop_dash() -> &'static str {
    "Line style"
}

/// The opacity row's label.
#[must_use]
pub const fn prop_opacity() -> &'static str {
    "Opacity"
}

/// The centre-mark size row's label.
#[must_use]
pub const fn prop_centre_mark_size() -> &'static str {
    "Centre mark size"
}

/// The line-type scale row: how large every named line type is drawn.
#[must_use]
pub const fn prop_line_type_scale() -> &'static str {
    "Line-type scale"
}

/// The heading over the radius or diameter circle's own stroke rows.
#[must_use]
pub const fn part_circle() -> &'static str {
    "Circle"
}

/// The heading over a centre line's own stroke rows.
#[must_use]
pub const fn part_centre_line() -> &'static str {
    "Centre line"
}

/// The heading over a centre mark's own stroke and extension rows.
#[must_use]
pub const fn part_centre_mark() -> &'static str {
    "Centre mark"
}

/// Under a part's heading, while every row follows the dimension's lines.
#[must_use]
pub const fn part_follows() -> &'static str {
    "Unset rows follow the dimension's own lines."
}

/// The centre mark's gap row.
#[must_use]
pub const fn prop_centre_mark_gap() -> &'static str {
    "Gap"
}

/// The centre mark's extension row.
#[must_use]
pub const fn prop_centre_mark_extension() -> &'static str {
    "Extension past the circle"
}

/// The centre mark's line-font row.
#[must_use]
pub const fn prop_centre_mark_line_font() -> &'static str {
    "Extensions in the centre-line style"
}

/// The line-font row's checkbox.
#[must_use]
pub const fn centre_mark_line_font_on() -> &'static str {
    "Dashed extensions"
}

/// A centre mark sized against the text height.
#[must_use]
pub const fn centre_mark_relative() -> &'static str {
    "Times text height"
}

/// A centre mark sized in points.
#[must_use]
pub const fn centre_mark_fixed() -> &'static str {
    "Fixed size"
}

/// The multiple's suffix.
#[must_use]
pub const fn centre_mark_relative_suffix() -> &'static str {
    " × text height"
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

/// What the reduce switch does, both ways.
#[must_use]
pub const fn precision_reduce_hint() -> &'static str {
    "On, 6/8 reads 3/4. Off keeps the denominator, as some architectural \
     drawings do so a run of dimensions reads at a glance."
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
#[must_use]
pub fn unit_name(unit: Unit) -> &'static str {
    crate::text::scale::unit_name(unit)
}

/// The name of a tolerance form.
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

/// **A limit tolerance replaces the number rather than sitting beside it.**
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
#[must_use]
pub fn tolerance_unit_note(unit: Unit) -> String {
    format!(
        "In {}, the same as the number it qualifies.",
        crate::text::dimension_groups::unit_abbrev(unit)
    )
}

/// A tolerance the engine refused, by its own name.
#[must_use]
pub fn tolerance_refused(reason: &str) -> String {
    format!("Not applied: {reason}")
}

/// The label on the tolerance-precision spinner's "follow the nominal" state.
#[must_use]
pub const fn tolerance_places_follows() -> &'static str {
    "Same decimals as the measurement"
}

/// The label-override section's heading.
#[must_use]
pub const fn label_heading() -> &'static str {
    "Override dimension"
}

/// The hint under the box.
#[must_use]
pub const fn label_hint() -> &'static str {
    "Leave it empty to show the measurement. The measurement is kept underneath either way, so \
     clearing this brings back exactly the number that was there before."
}

/// Shown while an override is in force.
#[must_use]
pub const fn label_overridden() -> &'static str {
    "This is showing your text instead of the measurement."
}

/// **The caption is on** — and this names the number it hid.
#[must_use]
pub fn label_set(measured: &str) -> String {
    format!(
        "Showing your text instead. It still measures {measured}, and clearing the box brings that back."
    )
}

/// **The caption is off** — and this names the number that came back.
#[must_use]
pub fn label_restored(printed: &str) -> String {
    format!("Back to the measurement: {printed}.")
}

/// A named drafting line type's name in the ce-dimension line-style chooser,
/// in SolidWorks' words; `None` for a type this shell does not offer.
#[must_use]
pub const fn line_type_name(line_type: pdfcer_core::dimension::LineType) -> Option<&'static str> {
    use pdfcer_core::dimension::LineType;
    Some(match line_type {
        LineType::Continuous => "Solid",
        LineType::Hidden => "Hidden",
        LineType::Phantom => "Phantom",
        LineType::Chain => "Chain",
        LineType::Center => "Center",
        LineType::Stitch => "Stitch",
        _ => return None,
    })
}

/// The closed line-style chooser for runs that are not a named type.
#[must_use]
pub const fn line_type_custom() -> &'static str {
    "Custom pattern"
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
