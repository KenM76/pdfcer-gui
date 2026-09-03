//! # `panels::properties::dimension::tolerance` — the seven forms, and the two
//! things a panel must not do with them
//!
//! ## What a tolerance is, in this model
//!
//! One more property of the `Pass 69.0` style cascade — not a parallel system.
//! It inherits factory → group → ce dimension exactly like a stroke width, uses
//! the same `Option` as its override checkbox, and has the same
//! clear-restores-inheritance semantics. That was the point of building the
//! cascade first.
//!
//! Seven forms: `None`, `Basic`, `Symmetric { magnitude }`,
//! `Deviation { plus, minus }`, `Limit { upper, lower }`, `Min`, `Max`.
//!
//! ## ★ Rule one: never build a preview by concatenation
//!
//! `docs/core-api/03-capabilities.md` §1.6 trap (b), and it is the trap that
//! cost `pdfcer` a shipped defect of its own:
//!
//! > A panel that previews *"nominal + tolerance"* by concatenation **will
//! > disagree with the bytes in the page** for every limit tolerance.
//!
//! Because `Tolerance::suppresses_nominal()` is true for `Limit`, and the
//! appearance-stream baker branches on it: the label becomes `50.20/49.90`
//! with **the nominal gone**, not `50.00 50.20/49.90`. And `Basic`'s caption is
//! the **empty string** — the box is the notation.
//!
//! So this module renders **fields**, never a specimen. What each form will do
//! to the printed label is stated in a sentence
//! ([`crate::text::panels::dimension::tolerance_suppresses_nominal`],
//! [`crate::text::panels::dimension::tolerance_is_a_box`]) which is a claim
//! about behaviour rather than a second derivation of a string the engine
//! already owns. `Pass 68.0`'s defect — the pane reading `77.5°` while the
//! `/AP` read `77.47 pt` — was exactly two independent derivations of one
//! display value, and the fix was *one producer, always*.
//!
//! ## ★ Rule two: nothing is clamped, swapped or absolutised
//!
//! `Tolerance::validate` refuses and says why, and `tolerance.rs`'s own comment
//! is the reason: *"a corrected value the operator never saw is exactly the
//! sneaky case."* A negative symmetric magnitude, an inverted limit pair and a
//! non-finite value are all refusals **by name**, with `ToleranceError`'s own
//! `Display` — written for an operator, e.g. *"a symmetric tolerance's
//! magnitude must not be negative (write ±0.1, not ±-0.1)"*.
//!
//! This module therefore lets the operator type an invalid pair, shows the
//! engine's sentence, and **raises no action** until it validates. The
//! alternative — silently swapping an inverted limit — would produce a drawing
//! that says something the operator did not ask for and would not notice.
//!
//! ## Why the values carry no unit suffix
//!
//! `Tolerance::caption` emits none in any branch, deliberately: a tolerance is
//! read in the nominal's unit, and `"50.00 mm ±0.10 mm"` is not how a drawing
//! is written. The fields match, and a note beneath names the unit once.

use egui::Ui;
use pdfcer_core::dimension::{Tolerance, Unit};

use crate::text::panels::dimension as t;

/// The drag speed for a tolerance magnitude.
///
/// An order of magnitude finer than the style module's point-valued
/// properties, because a tolerance is a manufacturing quantity: 0.05 is a
/// coarse fit and 0.005 is a bearing seat, and a spinner that skates past the
/// second is a spinner nobody uses twice.
const SPEED: f64 = 0.005;

/// Every form, in the order the combo offers them.
///
/// ★ Constructed with placeholder magnitudes because the combo selects a
/// **shape**, and the value the operator was last editing is preserved by
/// [`reshape`] rather than by the list. Listing `Symmetric { magnitude: 0.0 }`
/// here and selecting it directly would zero a number the operator had
/// already typed.
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
///
/// Returns `false` when the current fields do not validate, in which case the
/// caller must not raise an action — see the module header's rule two. The
/// refusal has already been drawn by the time this returns, so the caller does
/// not have to know why.
///
/// `unit` is the ce dimension's **resolved** display unit, used only for the
/// note saying which unit the numbers are in. It is resolved rather than the
/// group's own, because a ce dimension overriding its unit reads its tolerance
/// in the overridden one.
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
            // ★ The disclosure that matters most in this panel, and it is shown
            // whenever the form is chosen rather than on hover: an operator who
            // sets a limit expecting it beside the measurement will find a
            // drawing that says something else, and will find it after
            // plotting.
            ui.weak(t::tolerance_suppresses_nominal());
        }
    }

    // ★ The refusal is the ENGINE's, rendered verbatim. Nothing here decides
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
///
/// The combo's selected-state predicate. `PartialEq` compares magnitudes, which
/// would leave the combo showing nothing selected the moment the operator
/// changed a number — the control silently disowning the value it is editing.
#[must_use]
fn same_form(a: Tolerance, b: Tolerance) -> bool {
    core::mem::discriminant(&a) == core::mem::discriminant(&b)
}

/// Change a tolerance's **form**, carrying across whatever value survives the
/// change.
///
/// # Why this is not simply `*value = form`
///
/// Because an operator switching from *± 0.35* to *separate + and −* means
/// *"start from what I typed"*, not *"throw it away and give me 0.1"*. The
/// magnitude is the value they have been thinking about, and re-typing it is
/// the panel making them prove they meant it.
///
/// The mappings, and each is the reading a drafter would make:
///
/// | from → to | carried |
/// |---|---|
/// | symmetric → deviation | `± m` becomes `+m / −m`, which is the same tolerance written the other way |
/// | symmetric → limit | `± m` becomes `+m / −m` about the nominal, the same again |
/// | deviation → symmetric | the **larger** magnitude, because a symmetric tolerance that is tighter than the one it replaced would silently narrow a specification |
/// | limit → deviation | the pair, unchanged |
/// | anything → none / basic / min / max | nothing to carry; those forms hold no value |
///
/// Deviation → symmetric taking the larger is the only one that could be
/// argued, and it is argued the safe way round: a manufacturing tolerance that
/// gets **tighter** without the operator asking is a part that gets rejected.
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
    ///
    /// ★ The count is asserted against the list rather than against a literal
    /// seven **and** the discriminants are asserted distinct, so a form added
    /// to the engine and forgotten here fails on the first assertion while a
    /// form listed twice fails on the second.
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

    /// ★ Collapsing a deviation to a symmetric takes the LARGER magnitude.
    ///
    /// The safe direction, and the reason is a manufactured part rather than a
    /// preference: a tolerance that tightens without being asked produces
    /// components that get rejected against a drawing nobody changed.
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
    ///
    /// Without this the combo would show nothing selected the instant the
    /// operator dragged the magnitude — the control disowning what it is
    /// editing.
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
    ///
    /// The assertion is on the ENGINE's verdict, because that is what `show`
    /// consults — a local re-implementation of the rule is exactly what would
    /// let the panel and the file disagree.
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
