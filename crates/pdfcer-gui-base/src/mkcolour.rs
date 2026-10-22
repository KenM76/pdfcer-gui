//! # `mkcolour` — one `/MK` colour, wherever it is asked for
//!
//! A labelled swatch over one of a widget's two `/MK` colour keys, `/BG` and
//! `/BC`. `OPERATOR_REQUESTS.md` **O202**, whose ask covers both halves of the
//! life of a form box: *"the forms objects have no way to edit their colour
//! before or after placement."*
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/mkcolour.md`.

use egui::Ui;
use pdfcer_core::forms::MkColor;

use crate::colourpick::Ink;

use crate::text::panels::formfield as t;

/// One `/MK` colour row's inputs.
///
/// A struct because there are nine and seven of them are strings or bools,
/// which is precisely the shape a positional argument list gets silently wrong.
pub struct Row<'a> {
    /// The row's label.
    pub label: &'a str,
    /// What the label says on hover.
    pub hover: &'a str,
    /// The swatch's `egui` id salt. Must be unique within the surface.
    pub id_salt: &'a str,
    /// The trace region the swatch publishes.
    pub region: &'a str,
    /// What the key currently says: `None` for absent, `Some(MkColor::None)`
    /// for Table 189's empty array, otherwise the colour.
    pub colour: Option<MkColor>,
    /// The popup note when the key is absent.
    pub unstated_note: &'a str,
    /// The popup note when the key is the empty array.
    pub no_colour_note: &'a str,
    /// The *no colour* entry's label, or `None` when this key has no such
    /// state worth offering.
    ///
    /// `/BC` passes `None`, and that is a measurement rather than a
    /// simplification. `WidgetChrome::stroke` resolves the empty array and the
    /// absent key to the **same black**, deliberately — a border's thickness
    /// lives in `/BS` `/W`, and letting an empty `/BC` mean *omit the stroke*
    /// would give two unrelated keys one meaning. An entry writing it would
    /// change a byte, rebuild an appearance stream, cost an undo entry and
    /// alter no pixel, which is R9's case for rendering nothing.
    ///
    /// O202's decision 1 assumed the opposite. It was written against the
    /// engine as it stood before `Pass 308.0`.
    pub no_colour_entry: Option<&'a str>,
    /// Why that entry is greyed, when it is offered and would change nothing.
    pub no_colour_unavailable: &'a str,
    /// The *remove* entry's label, or `None` to offer no way back to silence.
    ///
    /// Distinct from [`Self::no_colour_entry`] and the two words must stay
    /// apart in front of the operator: on a push button *no colour* gives no
    /// plate and *remove* gives the builder's own grey one back.
    pub remove_entry: Option<&'a str>,
    /// Why that entry is greyed, which is when the key is already absent.
    pub remove_unavailable: &'a str,
    /// Draw the swatch as a disc rather than a bar.
    ///
    /// A fact about the ENGINE's radio builder, which fills a circle and says
    /// so, not a decoration chosen here. A rectangular preview over a control
    /// that comes out round is a surface mis-stating the result of the
    /// operator's own press.
    pub disc: bool,
}

/// What the operator asked the key to become.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Pick {
    /// Write this colour, `MkColor::None` included — the empty array is a
    /// value the key can hold, not the absence of one.
    Set(MkColor),
    /// Take the key away, so the file says nothing about it.
    Remove,
}

/// Draw the row. Returns a colour **once**, on the frame the gesture ended.
pub fn row(ui: &mut Ui, spec: &Row<'_>) -> Option<Pick> {
    let value = mk_value(spec.colour, spec.unstated_note, spec.no_colour_note);

    let mut entries = Vec::with_capacity(2);
    if let Some(label) = spec.no_colour_entry {
        entries.push(crate::swatch::MkEntry {
            pick: crate::swatch::MkPick::NoColour,
            label,
            available: no_colour_would_change_something(spec.colour),
            unavailable_hover: spec.no_colour_unavailable,
            trace: "none",
        });
    }
    if let Some(label) = spec.remove_entry {
        entries.push(crate::swatch::MkEntry {
            pick: crate::swatch::MkPick::Remove,
            label,
            available: removal_would_change_something(spec.colour),
            unavailable_hover: spec.remove_unavailable,
            trace: "remove",
        });
    }

    let picked = ui
        .horizontal(|ui| {
            ui.label(spec.label).on_hover_text(spec.hover);
            crate::swatch::show_mk(
                ui,
                spec.id_salt,
                &crate::swatch::MkControl {
                    value,
                    entries: &entries,
                    disc: spec.disc,
                },
                spec.region,
            )
        })
        .inner?;

    Some(match picked {
        crate::swatch::MkPick::Colour(ink) => Pick::Set(match ink {
            Ink::Gray(g) => MkColor::Gray(g),
            Ink::Rgb([r, g, b]) => MkColor::Rgb(r, g, b),
            Ink::Cmyk([c, m, y, k]) => MkColor::Cmyk(c, m, y, k),
        }),
        crate::swatch::MkPick::NoColour => Pick::Set(MkColor::None),
        crate::swatch::MkPick::Remove => Pick::Remove,
    })
}

/// Whether pressing *no colour* would change the file.
const fn no_colour_would_change_something(colour: Option<MkColor>) -> bool {
    !matches!(colour, Some(MkColor::None))
}

/// Whether pressing *remove* would change the file.
const fn removal_would_change_something(colour: Option<MkColor>) -> bool {
    colour.is_some()
}

/// What the swatch should show for one `/MK` colour key.
fn mk_value<'a>(
    colour: Option<MkColor>,
    unstated_note: &'a str,
    no_colour_note: &'a str,
) -> crate::swatch::MkValue<'a> {
    // Every colour is shown in its own space, so the picker opens on the
    // file's numbers and an untouched space is written back unchanged.
    match colour {
        Some(MkColor::Gray(g)) => crate::swatch::MkValue::Shown(Ink::Gray(g)),
        Some(MkColor::Rgb(r, g, b)) => crate::swatch::MkValue::Shown(Ink::Rgb([r, g, b])),
        Some(MkColor::Cmyk(c, m, y, k)) => crate::swatch::MkValue::Shown(Ink::Cmyk([c, m, y, k])),
        Some(MkColor::None) => crate::swatch::MkValue::Unshowable {
            mark: t::colour_mark_no_colour(),
            note: no_colour_note,
        },
        None => crate::swatch::MkValue::Unshowable {
            mark: t::colour_mark_unstated(),
            note: unstated_note,
        },
        // NO catch-all arm, for `border_style_label`'s reason: `MkColor` is not
        // `#[non_exhaustive]`, so a fifth colour space added to `pdfcer-core`
        // fails to build in this file — which is where the decision about how
        // to show it belongs.
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::swatch::MkValue;

    /// **The four states of a `/MK` colour key stay four.**
    #[test]
    fn a_mk_key_reaches_the_swatch_in_four_distinguishable_states() {
        assert_eq!(
            mk_value(Some(MkColor::Gray(1.0)), "unstated", "none"),
            MkValue::Shown(Ink::Gray(1.0))
        );
        assert_eq!(
            mk_value(Some(MkColor::Rgb(1.0, 0.0, 0.0)), "unstated", "none"),
            MkValue::Shown(Ink::Rgb([1.0, 0.0, 0.0]))
        );
        // CMYK is shown in its own space, never converted.
        assert_eq!(
            mk_value(Some(MkColor::Cmyk(0.1, 0.2, 0.3, 0.4)), "unstated", "none"),
            MkValue::Shown(Ink::Cmyk([0.1, 0.2, 0.3, 0.4]))
        );

        let stated = mk_value(Some(MkColor::None), "unstated", "none");
        let absent = mk_value(None, "unstated", "none");
        assert_ne!(
            stated, absent,
            "`no colour` and `the file is silent` must not read the same"
        );
        assert_eq!(
            stated,
            MkValue::Unshowable {
                mark: t::colour_mark_no_colour(),
                note: "none",
            }
        );
        assert_eq!(
            absent,
            MkValue::Unshowable {
                mark: t::colour_mark_unstated(),
                note: "unstated",
            }
        );
    }

    /// Table 189's four states against both entries at once.
    #[test]
    fn the_two_entries_never_offer_the_same_state() {
        let table = [
            (None, true, false),
            (Some(MkColor::None), false, true),
            (Some(MkColor::Gray(0.5)), true, true),
            (Some(MkColor::Rgb(0.1, 0.2, 0.3)), true, true),
            (Some(MkColor::Cmyk(0.1, 0.2, 0.3, 0.4)), true, true),
        ];
        for (colour, no_colour, remove) in table {
            assert_eq!(
                no_colour_would_change_something(colour),
                no_colour,
                "the no-colour entry is wrong for {colour:?}"
            );
            assert_eq!(
                removal_would_change_something(colour),
                remove,
                "the remove entry is wrong for {colour:?}"
            );
        }
    }
}
