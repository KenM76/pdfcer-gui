//! # `panels::properties::dimension::overrides` — eleven properties, eleven
//! checkboxes, and the tier each value came from
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/properties/dimension/overrides.md`.

use egui::Ui;
use pdfcer_core::dimension::{
    ArrowForm, DecimalMarker, DimStandard, FractionMode, Group, StyleOverrides, StyleSource, Unit,
    resolve_style,
};
use pdfcer_core::vector::Rgb;

use crate::text::panels::dimension as t;

/// The region the override block publishes.
pub const REGION: &str = "dimension-properties.overrides"; // ui-text-exempt: trace region name, never displayed
/// The region the text-height control publishes, so a driven check has one
/// numeric control it can move.
pub const REGION_TEXT_HEIGHT: &str = "dimension-properties.text_height"; // ui-text-exempt: trace region name, never displayed

/// The drag speed for the point-valued properties. See
/// `dialogs::dimension_groups::style`'s constant of the same name.
const POINT_SPEED: f64 = 0.05;
/// The legal ranges, matching the group tier's so that a value set at one tier
/// cannot be un-representable at the other.
const TEXT_HEIGHT_RANGE: std::ops::RangeInclusive<f64> = 1.0..=48.0;
/// See [`TEXT_HEIGHT_RANGE`].
const LINE_WIDTH_RANGE: std::ops::RangeInclusive<f64> = 0.05..=6.0;
/// See [`TEXT_HEIGHT_RANGE`].
const ARROW_LENGTH_RANGE: std::ops::RangeInclusive<f64> = 1.0..=30.0;
/// The decimal-places range, matching the engine's own `/D = 10^places`.
const PLACES_RANGE: std::ops::RangeInclusive<u32> = 0..=6;
/// The fraction denominators a drawing actually uses.
const DENOMINATORS: [u32; 6] = [2, 4, 8, 16, 32, 64];

/// Draw all eleven rows against `overrides`, mutating it in place.
pub fn show(ui: &mut Ui, group: &Group, overrides: &mut StyleOverrides) -> bool {
    crate::diag::ui_rect(REGION, ui.max_rect());
    let provenance = pdfcer_core::dimension::style_provenance(group, overrides);
    let resolved = resolve_style(group, overrides);
    let mut valid = true;

    // --- unit -----------------------------------------------------------
    row(
        ui,
        t::prop_unit(),
        provenance.unit,
        &mut overrides.unit,
        || resolved.format.unit,
    )
    .edit(ui, |ui, value| {
        egui::ComboBox::from_id_salt("dimension-override-unit")
            .selected_text(t::unit_name(*value))
            .show_ui(ui, |ui| {
                for unit in Unit::all().iter().copied() {
                    ui.selectable_value(value, unit, t::unit_name(unit));
                }
            });
    });

    // --- precision ------------------------------------------------------
    row(
        ui,
        t::prop_fraction(),
        provenance.fraction,
        &mut overrides.fraction,
        || resolved.format.fraction,
    )
    .edit(ui, fraction_editor);

    // --- decimal marker -------------------------------------------------
    row(
        ui,
        t::prop_decimal_marker(),
        provenance.decimal_marker,
        &mut overrides.decimal_marker,
        || resolved.format.decimal_marker,
    )
    .edit(ui, |ui, value| {
        egui::ComboBox::from_id_salt("dimension-override-marker")
            .selected_text(t::decimal_marker_name(*value))
            .show_ui(ui, |ui| {
                for marker in [DecimalMarker::Point, DecimalMarker::Comma] {
                    ui.selectable_value(value, marker, t::decimal_marker_name(marker));
                }
            });
    });

    // --- drafting standard ----------------------------------------------
    row(
        ui,
        t::prop_standard(),
        provenance.standard,
        &mut overrides.standard,
        || resolved.standard,
    )
    .edit(ui, |ui, value| {
        egui::ComboBox::from_id_salt("dimension-override-standard")
            .selected_text(t::standard_name(*value))
            .show_ui(ui, |ui| {
                for standard in [DimStandard::Ansi, DimStandard::Iso] {
                    ui.selectable_value(value, standard, t::standard_name(standard));
                }
            });
    });

    // --- text height ----------------------------------------------------
    row(
        ui,
        t::prop_text_height(),
        provenance.text_height,
        &mut overrides.text_height,
        || resolved.text_height,
    )
    .edit(ui, |ui, value| {
        let r = ui.add(
            egui::DragValue::new(value)
                .speed(POINT_SPEED)
                .range(TEXT_HEIGHT_RANGE),
        );
        crate::diag::ui_rect(REGION_TEXT_HEIGHT, r.rect);
    });

    // --- line width -----------------------------------------------------
    row(
        ui,
        t::prop_line_width(),
        provenance.line_width,
        &mut overrides.line_width,
        || resolved.line_width,
    )
    .edit(ui, |ui, value| {
        ui.add(
            egui::DragValue::new(value)
                .speed(POINT_SPEED)
                .range(LINE_WIDTH_RANGE),
        );
    });

    // --- arrow length ---------------------------------------------------
    row(
        ui,
        t::prop_arrow_length(),
        provenance.arrow_length,
        &mut overrides.arrow_length,
        || resolved.arrow_length,
    )
    .edit(ui, |ui, value| {
        ui.add(
            egui::DragValue::new(value)
                .speed(POINT_SPEED)
                .range(ARROW_LENGTH_RANGE),
        );
    });

    // --- arrow form -----------------------------------------------------
    row(
        ui,
        t::prop_arrow_form(),
        provenance.arrow_form,
        &mut overrides.arrow_form,
        || resolved.arrow_form,
    )
    .edit(ui, |ui, value| {
        egui::ComboBox::from_id_salt("dimension-override-arrow-form")
            .selected_text(crate::text::dimension_groups::arrow_form_name(*value))
            .show_ui(ui, |ui| {
                for form in ArrowForm::ALL {
                    ui.selectable_value(
                        value,
                        form,
                        crate::text::dimension_groups::arrow_form_name(form),
                    );
                }
            });
    });

    // --- colour ---------------------------------------------------------
    row(
        ui,
        t::prop_color(),
        provenance.color,
        &mut overrides.color,
        || resolved.color,
    )
    .edit(ui, |ui, value| {
        let mut screen = color32_of(*value);
        if ui.color_edit_button_srgba(&mut screen).changed() {
            *value = rgb_of(screen);
        }
    });

    // --- tolerance ------------------------------------------------------
    let unit = resolved.format.unit;
    row(
        ui,
        t::prop_tolerance(),
        provenance.tolerance,
        &mut overrides.tolerance,
        || resolved.tolerance,
    )
    .edit(ui, |ui, value| {
        valid &= super::tolerance::show(ui, value, unit);
    });

    // --- tolerance precision --------------------------------------------
    //
    // The one row whose cleared state is NOT simply "inherit": the resolved
    // value is itself an `Option<u32>`, whose `None` means *use the same number
    // of decimals as the measurement*. So this control offers a number when
    // ticked, and the caption underneath names what the inherited state
    // resolves to — which may be a count, or may be "follow the nominal".
    row(
        ui,
        t::prop_tolerance_places(),
        provenance.tolerance_places,
        &mut overrides.tolerance_places,
        // Seeded from the resolved value when there is one, and from zero when
        // the resolved state is "follow the nominal" — there is no number to
        // carry across in that case, and zero is the honest starting point for
        // a control the operator has just asked to take over.
        || resolved.tolerance_places.unwrap_or(0),
    )
    .edit(ui, |ui, value| {
        ui.add(egui::DragValue::new(value).range(PLACES_RANGE));
    });
    if overrides.tolerance_places.is_none() && resolved.tolerance_places.is_none() {
        ui.weak(t::tolerance_places_follows());
    }

    valid
}

/// One row's checkbox and disclosure, with the editor still to be drawn.
struct Row<'a, T> {
    /// The slot the checkbox governs, `Some` once it is ticked.
    slot: &'a mut Option<T>,
}

impl<T: Copy> Row<'_, T> {
    /// Draw the editor, if this row is overridden.
    fn edit(self, ui: &mut Ui, editor: impl FnOnce(&mut Ui, &mut T)) {
        if let Some(value) = self.slot.as_mut() {
            ui.indent("dimension-override-editor", |ui| editor(ui, value));
        }
    }
}

/// Draw one row's label, checkbox and provenance sentence.
fn row<'a, T: Copy>(
    ui: &mut Ui,
    label: &str,
    source: StyleSource,
    slot: &'a mut Option<T>,
    seed: impl FnOnce() -> T,
) -> Row<'a, T> {
    ui.horizontal(|ui| {
        ui.label(label);
        let mut set = slot.is_some();
        let response = ui.checkbox(&mut set, t::set_here());
        if response.changed() {
            *slot = if set { Some(seed()) } else { None };
        }
        // The predicate is the ENGINE's. `follows_group()` is true for
        // `Factory` as well as `Group`, and a hand-rolled `== Group` here would
        // tell an operator that a never-set property will survive a group edit.
        // It will not.
        response.on_hover_text(t::follows_group_note(source.follows_group()));
        if slot.is_none() {
            ui.weak(t::source_name(source));
        }
    });
    Row { slot }
}

/// The precision editor — a mode, then the control that mode implies.
fn fraction_editor(ui: &mut Ui, value: &mut FractionMode) {
    let is_decimal = matches!(value, FractionMode::Decimal { .. });
    ui.horizontal(|ui| {
        if ui
            .selectable_label(is_decimal, t::precision_decimal())
            .clicked()
            && !is_decimal
        {
            *value = FractionMode::Decimal { places: 2 };
        }
        if ui
            .selectable_label(!is_decimal, t::precision_fraction())
            .clicked()
            && is_decimal
        {
            // 8ths, and `reduce: false` — the architectural convention the
            // engine's own doc names. See `precision_reduce_hint`.
            *value = FractionMode::Fraction {
                denominator: 8,
                reduce: false,
            };
        }
    });
    match value {
        FractionMode::Decimal { places } => {
            ui.horizontal(|ui| {
                ui.label(t::precision_places());
                ui.add(egui::DragValue::new(places).range(PLACES_RANGE));
            });
        }
        FractionMode::Fraction {
            denominator,
            reduce,
        } => {
            ui.horizontal(|ui| {
                ui.label(t::precision_denominator());
                egui::ComboBox::from_id_salt("dimension-override-denominator")
                    .selected_text(t::precision_denominator_entry(*denominator))
                    .show_ui(ui, |ui| {
                        for d in DENOMINATORS {
                            ui.selectable_value(denominator, d, t::precision_denominator_entry(d));
                        }
                    });
            });
            ui.checkbox(reduce, t::precision_reduce());
            ui.weak(t::precision_reduce_hint());
        }
    }
}

/// A screen colour from PDF components. See
/// `dialogs::dimension_groups::style`'s twin for why the divisor is 255.
fn color32_of(rgb: Rgb) -> egui::Color32 {
    let to_byte = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    egui::Color32::from_rgb(to_byte(rgb.r), to_byte(rgb.g), to_byte(rgb.b)) // DOCUMENT COLOUR: this ce dimension's own `/C`, chosen by the operator — not a theme colour
}

/// PDF components from a screen colour — the inverse of [`color32_of`].
fn rgb_of(c: egui::Color32) -> Rgb {
    Rgb {
        r: f32::from(c.r()) / 255.0,
        g: f32::from(c.g()) / 255.0,
        b: f32::from(c.b()) / 255.0,
    }
}

/// Every property this panel draws, paired with the provenance field that
/// discloses it.
#[cfg(test)]
const DRAWN: [&str; 11] = [
    "unit",
    "fraction",
    "decimal-marker",
    "standard",
    "text-height",
    "line-width",
    "arrow-length",
    "arrow-form",
    "color",
    "tolerance",
    "tolerance-places",
];

#[cfg(test)]
mod tests {
    use super::*;
    use pdfcer_core::dimension::GroupId;

    /// **Every property the cascade has, this panel draws.**
    #[test]
    fn no_property_of_the_cascade_is_left_without_a_row() {
        let group = Group::new(GroupId(0), "Plan", Unit::Millimeter);
        let provenance =
            pdfcer_core::dimension::style_provenance(&group, &StyleOverrides::default());
        let engine: Vec<&'static str> = provenance.each().iter().map(|(name, _)| *name).collect();
        let drawn: Vec<&'static str> = DRAWN.to_vec();
        assert_eq!(
            engine, drawn,
            "the cascade's properties and this panel's rows have diverged — a \
             property in the engine's list and not in `DRAWN` has no editor"
        );
    }

    /// Ticking a box seeds the value that was already in force.
    #[test]
    fn ticking_an_override_starts_from_the_value_that_was_showing() {
        let mut group = Group::new(GroupId(0), "Plan", Unit::Millimeter);
        group.style.text_height = Some(3.0);
        let overrides = StyleOverrides::default();
        let resolved = resolve_style(&group, &overrides);
        assert!(
            (resolved.text_height - 3.0).abs() < f64::EPSILON,
            "the group's value is what is in force"
        );
        // What `row`'s seed closure would produce.
        assert!(
            (resolved.text_height - 3.0).abs() < f64::EPSILON,
            "and it is what the checkbox must seed"
        );
    }

    /// The four concrete-field properties never report `Factory`.
    #[test]
    fn the_four_concrete_properties_never_claim_a_factory_source() {
        let group = Group::new(GroupId(0), "Plan", Unit::Millimeter);
        let p = pdfcer_core::dimension::style_provenance(&group, &StyleOverrides::default());
        for (name, source) in [
            ("unit", p.unit),
            ("fraction", p.fraction),
            ("decimal-marker", p.decimal_marker),
            ("standard", p.standard),
        ] {
            assert_ne!(
                source,
                StyleSource::Factory,
                "{name} has a concrete field on Group and must never report Factory"
            );
        }
    }
}
