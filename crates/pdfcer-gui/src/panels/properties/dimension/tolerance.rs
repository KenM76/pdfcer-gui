//! # `panels::properties::dimension::tolerance` — the seven forms, and the two
//! things a panel must not do with them
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/properties/dimension/tolerance.md`.

use egui::Ui;
use pdfcer_core::dimension::{Tolerance, Unit};

use crate::text::panels::dimension as t;

/// The drag speed for a tolerance magnitude.
const SPEED: f64 = 0.005;

/// Every form, in the order the combo offers them.
const FORMS: [Tolerance; 7] = [
    Tolerance::None,
    Tolerance::Symmetric { magnitude: 0.1 },
    Tolerance::Deviation {
        plus: 0.1,
        minus: 0.1,
    },
    Tolerance::Limit {
        upper: 0.1,
        lower: -0.1,
    },
    Tolerance::Basic,
    Tolerance::Min,
    Tolerance::Max,
];

/// Draw the tolerance editor, mutating `value` in place.
pub fn show(ui: &mut Ui, value: &mut Tolerance, unit: Unit) -> bool {
    egui::ComboBox::from_id_salt("dimension-tolerance-form")
        .selected_text(t::tolerance_name(*value))
        .show_ui(ui, |ui| {
            for form in FORMS {
                // Compared by DISCRIMINANT, not by value: the combo picks a
                // shape, and `Symmetric { magnitude: 0.1 }` in the list must
                // read as selected when the operator is editing
                // `Symmetric { magnitude: 0.37 }`. `selectable_value` compares
                // with `PartialEq`, which would say no.
                let selected = same_form(*value, form);
                if ui
                    .selectable_label(selected, t::tolerance_name(form))
                    .clicked()
                    && !selected
                {
                    *value = reshape(*value, form);
                }
            }
        });

    match value {
        Tolerance::None | Tolerance::Min | Tolerance::Max => {}
        Tolerance::Basic => {
            ui.weak(t::tolerance_is_a_box());
        }
        Tolerance::Symmetric { magnitude } => {
            ui.horizontal(|ui| {
                ui.label(t::tolerance_magnitude());
                ui.add(egui::DragValue::new(magnitude).speed(SPEED));
            });
            ui.weak(t::tolerance_unit_note(unit));
        }
        Tolerance::Deviation { plus, minus } => {
            ui.horizontal(|ui| {
                ui.label(t::tolerance_plus());
                ui.add(egui::DragValue::new(plus).speed(SPEED));
                ui.label(t::tolerance_minus());
                ui.add(egui::DragValue::new(minus).speed(SPEED));
            });
            ui.weak(t::tolerance_unit_note(unit));
        }
        Tolerance::Limit { upper, lower } => {
            ui.horizontal(|ui| {
                ui.label(t::tolerance_upper());
                ui.add(egui::DragValue::new(upper).speed(SPEED));
                ui.label(t::tolerance_lower());
                ui.add(egui::DragValue::new(lower).speed(SPEED));
            });
            ui.weak(t::tolerance_unit_note(unit));
            // The disclosure that matters most in this panel, and it is shown
            // whenever the form is chosen rather than on hover: an operator who
            // sets a limit expecting it beside the measurement will find a
            // drawing that says something else, and will find it after
            // plotting.
            ui.weak(t::tolerance_suppresses_nominal());
        }
    }

    // The refusal is the ENGINE's, rendered verbatim. Nothing here decides
    // what is invalid, and nothing here corrects it.
    match value.validate() {
        Ok(_) => true,
        Err(error) => {
            ui.colored_label(
                ui.visuals().error_fg_color,
                t::tolerance_refused(&error.to_string()),
            );
            false
        }
    }
}

/// Whether two tolerances are the same **form**, ignoring their values.
#[must_use]
fn same_form(a: Tolerance, b: Tolerance) -> bool {
    core::mem::discriminant(&a) == core::mem::discriminant(&b)
}

/// Change a tolerance's **form**, carrying across whatever value survives the
/// change.
#[must_use]
fn reshape(from: Tolerance, to: Tolerance) -> Tolerance {
    match (from, to) {
        (Tolerance::Symmetric { magnitude }, Tolerance::Deviation { .. }) => Tolerance::Deviation {
            plus: magnitude,
            minus: magnitude,
        },
        (Tolerance::Symmetric { magnitude }, Tolerance::Limit { .. }) => Tolerance::Limit {
            upper: magnitude,
            lower: -magnitude,
        },
        (Tolerance::Deviation { plus, minus }, Tolerance::Symmetric { .. }) => {
            Tolerance::Symmetric {
                magnitude: plus.abs().max(minus.abs()),
            }
        }
        (Tolerance::Deviation { plus, minus }, Tolerance::Limit { .. }) => Tolerance::Limit {
            upper: plus,
            lower: minus,
        },
        (Tolerance::Limit { upper, lower }, Tolerance::Deviation { .. }) => Tolerance::Deviation {
            plus: upper,
            minus: lower,
        },
        (Tolerance::Limit { upper, lower }, Tolerance::Symmetric { .. }) => Tolerance::Symmetric {
            magnitude: upper.abs().max(lower.abs()),
        },
        // Every other pair has nothing to carry: the target form holds no
        // value, or the source held none.
        (_, to) => to,
    }
}

#[cfg(test)]
#[allow(
    clippy::float_cmp,
    reason = "exact carry-across is the property under test"
)] // ui-text-exempt: lint justification, never displayed
mod tests {
    use super::*;

    /// Every form in the combo is a distinct shape, and all seven are offered.
    #[test]
    fn the_combo_offers_each_form_exactly_once() {
        let mut shapes: Vec<std::mem::Discriminant<Tolerance>> =
            FORMS.iter().map(std::mem::discriminant).collect();
        let before = shapes.len();
        shapes.sort_by_key(|d| format!("{d:?}"));
        shapes.dedup();
        assert_eq!(shapes.len(), before, "a form is listed twice");
        assert_eq!(
            before, 7,
            "the engine has seven forms; the combo must offer all of them"
        );
    }

    /// Switching form carries the operator's number across.
    #[test]
    fn a_typed_magnitude_survives_a_change_of_form() {
        let sym = Tolerance::Symmetric { magnitude: 0.35 };
        assert_eq!(
            reshape(
                sym,
                Tolerance::Deviation {
                    plus: 0.0,
                    minus: 0.0
                }
            ),
            Tolerance::Deviation {
                plus: 0.35,
                minus: 0.35
            }
        );
        assert_eq!(
            reshape(
                sym,
                Tolerance::Limit {
                    upper: 0.0,
                    lower: 0.0
                }
            ),
            Tolerance::Limit {
                upper: 0.35,
                lower: -0.35
            }
        );
    }

    /// Collapsing a deviation to a symmetric takes the LARGER magnitude.
    #[test]
    fn collapsing_a_deviation_never_tightens_it() {
        let dev = Tolerance::Deviation {
            plus: 0.5,
            minus: -0.1,
        };
        assert_eq!(
            reshape(dev, Tolerance::Symmetric { magnitude: 0.0 }),
            Tolerance::Symmetric { magnitude: 0.5 }
        );
    }

    /// A form holding no value takes nothing across, and taking nothing across
    /// is not an error.
    #[test]
    fn the_valueless_forms_are_plain_replacements() {
        for target in [
            Tolerance::None,
            Tolerance::Basic,
            Tolerance::Min,
            Tolerance::Max,
        ] {
            assert_eq!(
                reshape(Tolerance::Symmetric { magnitude: 9.0 }, target),
                target
            );
        }
    }

    /// The combo's selected predicate follows the shape, not the value.
    #[test]
    fn the_combo_stays_selected_while_the_number_changes() {
        assert!(same_form(
            Tolerance::Symmetric { magnitude: 0.1 },
            Tolerance::Symmetric { magnitude: 0.37 }
        ));
        assert!(!same_form(
            Tolerance::Symmetric { magnitude: 0.1 },
            Tolerance::Basic
        ));
    }

    /// An inverted limit pair is refused by the engine and this module reports
    /// it rather than swapping.
    #[test]
    fn an_inverted_limit_is_refused_rather_than_corrected() {
        let bad = Tolerance::Limit {
            upper: 0.0,
            lower: 1.0,
        };
        assert!(bad.validate().is_err(), "the engine must refuse this");
        let good = Tolerance::Limit {
            upper: 1.0,
            lower: 0.0,
        };
        assert!(good.validate().is_ok());
    }
}
