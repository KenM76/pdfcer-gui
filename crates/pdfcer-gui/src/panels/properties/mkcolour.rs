//! # `panels::properties::mkcolour` — one `/MK` colour, wherever it is asked for
//!
//! A labelled swatch over one of a widget's two `/MK` colour keys, `/BG` and
//! `/BC`. `OPERATOR_REQUESTS.md` **O202**, whose ask covers both halves of the
//! life of a form box: *"the forms objects have no way to edit their colour
//! before or after placement."*
//!
//! ## Why this is its own module rather than two similar rows
//!
//! Before placement the answer goes into a [`crate::canvas::formfield::Draft`]
//! field; after placement it goes into a `WidgetEdit` and an undo entry. Those
//! are different destinations, but every question in front of them is the same
//! one — Table 189 gives each key four states, two of which no swatch can draw,
//! and each of those owes the operator a **mark** for the button face and a
//! **note** in the popup. Written twice, the placement dialog and the
//! properties pane would answer the CMYK question differently within a month.
//!
//! So this module owns the reading, and the two callers own only where the
//! answer goes.
//!
//! ## What it deliberately does not own
//!
//! The **sentences**. Every string arrives through [`Row`], because what *no
//! colour* means is a fact about which key this is — a background stating it
//! paints nothing at all, a border stating it is still drawn black — and that
//! is knowledge `text::panels::formfield` holds, not this file.
//!
//! ## Rule 4
//!
//! Nothing here marks the canvas, in either caller. After placement the colour
//! is applied and from that instant the page shows what the saved file will
//! show. Before placement there is no content yet to mark; the swatch is part
//! of the cursor, not part of the document.

use egui::Ui;
use pdfcer_core::forms::MkColor;

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
    /// ★★ `/BC` passes `None`, and that is a measurement rather than a
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
    /// ★ A fact about the ENGINE's radio builder, which fills a circle and says
    /// so, not a decoration chosen here. A rectangular preview over a control
    /// that comes out round is a surface mis-stating the result of the
    /// operator's own press.
    pub disc: bool,
}

/// What the operator asked the key to become.
///
/// The shell's own enum rather than `pdfcer_core::edit::MkColorEdit`, which
/// carries the same two states. `MkColorEdit` is `#[non_exhaustive]`, so
/// matching it here would need a catch-all arm — and a catch-all is exactly
/// what [`mk_value`] refuses, for the reason written there: a state added to
/// the engine's enum must break this file's build rather than fall silently
/// into a default. The conversion is one `match` in each caller, at the point
/// where the answer's destination is already known.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Pick {
    /// Write this colour, `MkColor::None` included — the empty array is a
    /// value the key can hold, not the absence of one.
    Set(MkColor),
    /// Take the key away, so the file says nothing about it.
    Remove,
}

/// Draw the row. Returns a colour **once**, on the frame the gesture ended.
///
/// `None` on every other frame, including every frame of a drag inside the
/// picker — see [`super::swatch`]'s header for why that matters and what it
/// costs when it is got wrong.
pub fn row(ui: &mut Ui, spec: &Row<'_>) -> Option<Pick> {
    // Held outside the call because `MkValue` borrows its mark, and this is the
    // one state whose mark is computed rather than a `&'static str`.
    let cmyk = match spec.colour {
        Some(MkColor::Cmyk(c, m, y, k)) => Some(t::colour_cmyk_mark(c, m, y, k)),
        _ => None,
    };
    let value = mk_value(
        spec.colour,
        cmyk.as_deref(),
        spec.unstated_note,
        spec.no_colour_note,
    );

    let mut entries = Vec::with_capacity(2);
    if let Some(label) = spec.no_colour_entry {
        entries.push(super::swatch::MkEntry {
            pick: super::swatch::MkPick::NoColour,
            label,
            available: no_colour_would_change_something(spec.colour),
            unavailable_hover: spec.no_colour_unavailable,
            trace: "none",
        });
    }
    if let Some(label) = spec.remove_entry {
        entries.push(super::swatch::MkEntry {
            pick: super::swatch::MkPick::Remove,
            label,
            available: removal_would_change_something(spec.colour),
            unavailable_hover: spec.remove_unavailable,
            trace: "remove",
        });
    }

    let picked = ui
        .horizontal(|ui| {
            ui.label(spec.label).on_hover_text(spec.hover);
            super::swatch::show_mk(
                ui,
                spec.id_salt,
                &super::swatch::MkControl {
                    value,
                    entries: &entries,
                    disc: spec.disc,
                },
                spec.region,
            )
        })
        .inner?;

    Some(match picked {
        super::swatch::MkPick::Colour([r, g, b]) => {
            Pick::Set(MkColor::Rgb(fraction(r), fraction(g), fraction(b)))
        }
        super::swatch::MkPick::NoColour => Pick::Set(MkColor::None),
        super::swatch::MkPick::Remove => Pick::Remove,
    })
}

/// Whether pressing *no colour* would change the file.
///
/// False only when the key already holds the empty array. Named rather than
/// inlined so [`the_two_entries_never_offer_the_same_state`] can walk Table
/// 189's states against both predicates at once.
const fn no_colour_would_change_something(colour: Option<MkColor>) -> bool {
    !matches!(colour, Some(MkColor::None))
}

/// Whether pressing *remove* would change the file.
///
/// False only when the key is already absent — including when it is absent and
/// the operator is looking at a *no colour* entry that is live, which is the
/// pair everyone reads backwards.
const fn removal_would_change_something(colour: Option<MkColor>) -> bool {
    colour.is_some()
}

/// What the swatch should show for one `/MK` colour key.
///
/// Separate from [`row`] because this is the whole of Table 189's four-state
/// reading and the only part a test can reach without a live `Ui`. `cmyk` is
/// threaded in rather than computed here so the caller owns the `String` the
/// returned value borrows.
fn mk_value<'a>(
    colour: Option<MkColor>,
    cmyk: Option<&'a str>,
    unstated_note: &'a str,
    no_colour_note: &'a str,
) -> super::swatch::MkValue<'a> {
    match colour {
        // DeviceGray widens exactly — one component repeated three times is the
        // same colour, not an approximation — so it is SHOWN. DeviceCMYK does
        // not, and is not. O202 decision 4.
        Some(MkColor::Gray(g)) => super::swatch::MkValue::Shown([component(g); 3]),
        Some(MkColor::Rgb(r, g, b)) => {
            super::swatch::MkValue::Shown([component(r), component(g), component(b)])
        }
        Some(MkColor::Cmyk(..)) => super::swatch::MkValue::Unshowable {
            mark: cmyk.unwrap_or_default(),
            note: t::colour_cmyk_note(),
        },
        Some(MkColor::None) => super::swatch::MkValue::Unshowable {
            mark: t::colour_mark_no_colour(),
            note: no_colour_note,
        },
        None => super::swatch::MkValue::Unshowable {
            mark: t::colour_mark_unstated(),
            note: unstated_note,
        },
        // NO catch-all arm, for `border_style_label`'s reason: `MkColor` is not
        // `#[non_exhaustive]`, so a fifth colour space added to `pdfcer-core`
        // fails to build in this file — which is where the decision about how
        // to show it belongs.
    }
}

/// A `/MK` component as a screen byte.
///
/// Clamped, because the engine reports the file's own numbers **unclamped** —
/// *"an out-of-range component is a malformed file, not a value to silently
/// correct"* — and a byte is what a swatch needs. The clamp happens on the way
/// to the SCREEN and never on the way to the file: nothing here writes a
/// clamped value back.
fn component(v: f32) -> u8 {
    (v.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// A screen byte as a `/MK` component.
fn fraction(v: u8) -> f32 {
    f32::from(v) / 255.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::panels::properties::swatch::MkValue;

    /// **The four states of a `/MK` colour key stay four.**
    ///
    /// The two that matter and would not be noticed if they collapsed:
    /// DeviceGray is drawn and DeviceCMYK is not, and *the file is silent*
    /// carries a different face from *the file states no colour*. The second is
    /// the whole reason the engine models the key as `Option<MkColor>` with an
    /// `MkColor::None` inside, and a surface showing one glyph for both would
    /// throw that distinction away where the operator reads it.
    #[test]
    fn a_mk_key_reaches_the_swatch_in_four_distinguishable_states() {
        assert_eq!(
            mk_value(Some(MkColor::Gray(1.0)), None, "unstated", "none"),
            MkValue::Shown([255, 255, 255])
        );
        assert_eq!(
            mk_value(Some(MkColor::Rgb(1.0, 0.0, 0.0)), None, "unstated", "none"),
            MkValue::Shown([255, 0, 0])
        );

        // A separation is never converted, so it is never shown.
        let mark = t::colour_cmyk_mark(0.1, 0.2, 0.3, 0.4);
        assert_eq!(
            mk_value(
                Some(MkColor::Cmyk(0.1, 0.2, 0.3, 0.4)),
                Some(&mark),
                "unstated",
                "none"
            ),
            MkValue::Unshowable {
                mark: "0.10 0.20 0.30 0.40",
                note: t::colour_cmyk_note(),
            }
        );

        let stated = mk_value(Some(MkColor::None), None, "unstated", "none");
        let absent = mk_value(None, None, "unstated", "none");
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

    /// **The clamp is on the way to the screen and nowhere else.**
    ///
    /// The engine reports a `/MK` component as the file states it, unclamped,
    /// because an out-of-range component is a malformed file rather than a
    /// value to silently correct. A swatch needs a byte, so it clamps — and the
    /// thing to prove is that nothing clamped comes back the other way, which
    /// is what `fraction`'s domain being `u8` gives for free and what this pins.
    #[test]
    fn an_out_of_range_component_is_clamped_for_the_screen_only() {
        assert_eq!(component(2.5), 255);
        assert_eq!(component(-1.0), 0);
        for byte in [0_u8, 1, 127, 254, 255] {
            assert_eq!(component(fraction(byte)), byte, "round trip at {byte}");
        }
    }

    /// Table 189's four states against both entries at once.
    ///
    /// The pair this guards is the one that reads backwards: an **absent** key
    /// offers *no colour* and not *remove*, and an **empty** key offers
    /// *remove* and not *no colour*. Swap the two predicates and every state
    /// still lights exactly one entry, so nothing short of the full table
    /// catches it.
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
