//! Strings for how the 3D viewer's *Face, edge, corner* measure says its
//! figures: the unit and places, the coordinate system and the projection.
//! Consumed by `pdfcer_gui::dialogs::model3d::readout` and `select`.

/// How lengths, areas and coordinates are written: model units times
/// `factor` give the shown value, to `decimals` places, followed by `unit`
/// when it is not empty, and a length or an area again in `also`'s unit.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Lengths<'a> {
    /// Shown units per model unit.
    pub factor: f64,
    /// The shown unit's word; empty when the file states none.
    pub unit: &'a str,
    /// Places after the point.
    pub decimals: usize,
    /// A second unit's units per model unit and its word, said in brackets.
    pub also: Option<(f64, &'a str)>,
}

impl Lengths<'_> {
    /// Model units to three places with `unit`, unconverted.
    #[must_use]
    pub fn plain(unit: &str) -> Lengths<'_> {
        Lengths {
            factor: 1.0,
            unit,
            decimals: 3,
            also: None,
        }
    }

    fn number(&self, value: f64) -> String {
        format!("{value:.*}", self.decimals)
    }

    /// A model length, converted and with its unit.
    #[must_use]
    pub fn length(&self, value: f64) -> String {
        let n = self.number(value * self.factor);
        let main = if self.unit.is_empty() {
            n
        } else {
            format!("{n} {}", self.unit)
        };
        match self.also {
            Some((f, u)) => format!("{main} ({} {u})", self.number(value * f)),
            None => main,
        }
    }

    /// A model area, converted and with its squared unit.
    #[must_use]
    pub fn area(&self, value: f64) -> String {
        let n = self.number(value * self.factor * self.factor);
        let main = if self.unit.is_empty() {
            n
        } else {
            format!("{n} {}\u{b2}", self.unit)
        };
        match self.also {
            Some((f, u)) => format!("{main} ({} {u}\u{b2})", self.number(value * f * f)),
            None => main,
        }
    }

    /// A coordinate or an offset along one axis, converted, no unit.
    #[must_use]
    pub fn coordinate(&self, value: f64) -> String {
        self.number(value * self.factor)
    }
}

/// A unit's word on screen from the engine's symbol (`LengthUnit::symbol`):
/// the symbol itself, with `µm` for `um`.
#[must_use]
pub fn unit_word(symbol: &str) -> &str {
    if symbol == "um" { "\u{b5}m" } else { symbol }
}

/// Before the unit chooser.
pub const UNIT: &str = "Show in";
/// The unit chooser's first choice: the file's own unit.
pub const FILE_UNIT: &str = "The file's unit";
/// The unit chooser's tip.
pub const UNIT_TIP: &str = "The unit lengths and areas are said in. The file states its own unit, so any other is an exact conversion of it.";
/// Before the second unit's chooser.
pub const SECOND_UNIT: &str = "and again in";
/// Its first choice: no second unit.
pub const NO_SECOND_UNIT: &str = "Nothing";
/// Its tip.
pub const SECOND_UNIT_TIP: &str =
    "A second unit each length and area is also said in, in brackets.";
/// Before the places chooser.
pub const DECIMALS: &str = "places";
/// The places chooser's tip.
pub const DECIMALS_TIP: &str = "How many figures after the point.";

/// Before the coordinate system's state.
pub const COORDINATES: &str = "Coordinates";
/// The model's own coordinate system.
pub const MODEL_COORDINATES: &str = "the model's";
/// A coordinate system set from two picked edges.
pub const PICKED_COORDINATES: &str = "from the picked edges";
/// The button setting the system from the picks.
pub const SET_FROM_PICKS: &str = "Set from the two edges";
/// Its tip.
pub const SET_FROM_PICKS_TIP: &str = "Puts the origin where the two straight edges come nearest, X along the first edge and Y toward the second. Corners and offsets are then said in this system, and the planes below are its planes.";
/// Its tip when the picks cannot set one.
pub const SET_FROM_PICKS_OFF: &str =
    "Pick two straight edges that are not parallel, with A click picks: Edge.";
/// The button going back to the model's system.
pub const MODEL_BUTTON: &str = "Use the model's";
/// Its tip.
pub const MODEL_BUTTON_TIP: &str = "Say corners and offsets in the model's own coordinates again.";

/// Before the projection chooser.
pub const PROJECTED: &str = "Projected on";
/// The projection choices, in `readout::PROJECTIONS`' order.
pub const PROJECTIONS: [&str; 5] = ["Nothing", "The screen", "XY plane", "YZ plane", "ZX plane"];
/// Each choice's tip, in the same order.
pub const PROJECTION_TIPS: [&str; 5] = [
    "Distances and areas as they are in the model.",
    "Distances and a face's area as they look from here, flattened onto the screen. They change as the view turns.",
    "Distances and a face's area flattened onto the XY plane of the coordinates above.",
    "Distances and a face's area flattened onto the YZ plane of the coordinates above.",
    "Distances and a face's area flattened onto the ZX plane of the coordinates above.",
];

/// A distance projected, and the part of it the projection removed.
#[must_use]
pub fn projected_distance(plane: &str, length: &str, out: &str) -> String {
    format!("On {plane}: {length}, {out} square to it")
}

/// A face's projected area when the whole face lies on one side.
#[must_use]
pub fn projected_area(plane: &str, area: &str) -> String {
    format!("Area on {plane}: {area}")
}

/// A face's projected area when it turns past edge-on.
#[must_use]
pub fn projected_area_sides(plane: &str, toward: &str, away: &str) -> String {
    format!(
        "Area on {plane}: {toward} facing it, {away} facing away. The face turns past edge-on, so neither is its shadow."
    )
}

/// Under a projected area whose face the file gives only inside a larger one.
pub const LARGER_FACE_PROJECTED: &str = "The file gives this face only as part of a larger one, so the projected area is the larger one's.";

/// An offset along the three axes of the chosen system.
#[must_use]
pub fn deltas(l: &Lengths<'_>, [x, y, z]: [f64; 3]) -> String {
    format!(
        "\u{394}X {}   \u{394}Y {}   \u{394}Z {}",
        l.coordinate(x),
        l.coordinate(y),
        l.coordinate(z)
    )
}

/// The projection could not be made.
#[must_use]
pub fn not_projected(why: &str) -> String {
    format!("Cannot project this: {why}")
}

/// The plane's name in a sentence, from its choice.
#[must_use]
pub fn plane_name(choice: usize) -> &'static str {
    match choice {
        1 => "the screen",
        2 => "XY",
        3 => "YZ",
        _ => "ZX",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_conversion_scales_lengths_once_and_areas_twice() {
        // ORACLE, NOT A CONVERSION: a test's inch factor, written out by hand.
        let l = Lengths {
            factor: 1.0 / 25.4,
            unit: "in",
            decimals: 2,
            also: None,
        };
        assert_eq!(l.length(50.8), "2.00 in");
        assert_eq!(l.area(645.16), "1.00 in\u{b2}");
        // ORACLE, NOT A CONVERSION: one inch, in millimetres.
        assert_eq!(l.coordinate(25.4), "1.00");
        assert_eq!(Lengths::plain("").length(1.0), "1.000");
        // ORACLE, NOT A CONVERSION: a test's inch factor, written out by hand.
        let both = Lengths {
            also: Some((1.0 / 25.4, "in")),
            ..Lengths::plain("mm")
        };
        // ORACLE, NOT A CONVERSION: a test's inch factor, written out by hand.
        assert_eq!(both.length(25.4), "25.400 mm (1.000 in)");
        assert_eq!(both.area(645.16), "645.160 mm\u{b2} (1.000 in\u{b2})");
    }

    #[test]
    fn micrometres_are_written_with_a_mu() {
        assert_eq!(unit_word("um"), "\u{b5}m");
        assert_eq!(unit_word("ft"), "ft");
    }
}
