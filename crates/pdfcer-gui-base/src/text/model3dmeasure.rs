//! Strings for the 3D viewer's measure tool. Consumed by
//! `pdfcer_gui::dialogs::model3d`.

/// The switch and the glyph over the picture.
pub const MEASURE: &str = "Measure";
/// The switch's and the glyph's tip.
pub const MEASURE_TOOLTIP: &str = "Measure from points you pick on the model. While this is on, a click picks a point; dragging \
     still turns the view.";
/// What a measurement is made of, in the order of the tool's kinds.
pub const KINDS: [&str; 5] = ["Distance", "Perpendicular", "Angle", "Radius", "Comment"];
/// Each kind's tip, in the same order.
pub const KIND_TIPS: [&str; 5] = [
    "The straight distance between two points.",
    "How far a point is from a surface, along that surface's normal.",
    "The angle at a corner, from the corner and a point along each side.",
    "The radius of the circle through three points.",
    "A note fixed to a point on the model.",
];
/// The comment's text box.
pub const COMMENT_HINT: &str = "Comment text";
/// *Keep*.
pub const KEEP: &str = "Keep on this view";
/// *Keep*'s tip.
pub const KEEP_TOOLTIP: &str = "Save this measurement on the saved view shown, in the file. 3D measurements are a PDF 2.0 \
     feature: a reader that does not support them leaves them out.";
/// *Start again*.
pub const START_AGAIN: &str = "Start again";
/// *Start again*'s tip.
pub const START_AGAIN_TOOLTIP: &str = "Forget the points picked so far.";
/// The tap landed off the model.
pub const MISSED: &str = "No part of the model there: pick a point on its surface.";
/// The points make no measurement of the kind.
pub const REFUSED: &str = "Those points make no measurement of this kind, for instance they coincide or lie in a line. \
     Pick them again.";
/// A comment was placed with nothing to say.
pub const NO_COMMENT: &str = "Type the comment first, then pick where it goes.";
/// No camera could be saved for a measurement in a file with no saved views.
pub const NOT_KEPT: &str = "This view cannot be saved, so the measurement was not kept. Leave the lens or walking mode \
     and try again.";
/// The value carries no unit.
pub const NO_UNIT: &str = "No unit is shown because the model's file does not name one pdfcer can label: this value is \
     in the model's own units.";
/// Above the list of a view's measurements.
pub const NONE_KEPT: &str = "No measurements are saved on this view.";
/// A measurement's delete button.
pub const DELETE: &str = "Delete";
/// The delete button's tip.
pub const DELETE_TOOLTIP: &str = "Remove this measurement from the file.";
/// The viewer has no saved view to name.
pub const NEW_VIEW: &str = "Kept on a new saved view of this camera, which the file opens on.";

/// A saved measurement's line: its name and its value.
#[must_use]
pub fn named_value(name: &str, value: &str) -> String {
    format!("{name}: {value}")
}

/// The word for a unit of `mm_per_unit` millimetres; empty when it has none.
#[must_use]
pub fn unit_word(mm_per_unit: f64) -> &'static str {
    let is = |mm: f64| (mm_per_unit - mm).abs() <= mm * 1e-9;
    let inch = crate::units::mm_from_points(crate::units::points_from_inches(1.0));
    match () {
        () if is(1.0) => "mm",
        () if is(10.0) => "cm",
        () if is(1000.0) => "m",
        () if is(inch) => "in",
        () if is(inch * 12.0) => "ft",
        () => "",
    }
}

/// The picks a kind still wants: `kind` in [`KINDS`]' order, `done` picked.
#[must_use]
pub fn next_pick(kind: usize, done: usize) -> &'static str {
    match (kind, done) {
        (0, 0) => "Pick the first point.",
        (0, _) => "Pick the second point.",
        (1, 0) => "Pick a point on the surface to measure from.",
        (1, _) => "Pick the point to measure to.",
        (2, 0) => "Pick the corner.",
        (2, 1) => "Pick a point along the first side.",
        (2, _) => "Pick a point along the second side.",
        (3, 0) => "Pick the first of three points on the circle.",
        (3, 1) => "Pick the second point on the circle.",
        (3, _) => "Pick the third point on the circle.",
        _ => "Type the comment, then pick where it goes.",
    }
}

/// How far a distance runs along each axis.
#[must_use]
pub fn along_axes(x: f64, y: f64, z: f64) -> String {
    format!("ΔX {x:.3}   ΔY {y:.3}   ΔZ {z:.3}")
}

/// The name a measurement is saved under, which a reader lists.
#[must_use]
pub fn saved_name(kind: usize, number: usize) -> String {
    format!("{} {number}", KINDS.get(kind).copied().unwrap_or(KINDS[0]))
}

/// The heading above the list of `view`'s measurements.
#[must_use]
pub fn saved_on(view: &str) -> String {
    format!("Saved on the view \u{201c}{view}\u{201d}:")
}

/// The status line after a measurement was kept.
#[must_use]
pub fn kept_on(view: &str) -> String {
    format!("The measurement is saved on the view \u{201c}{view}\u{201d}.")
}

/// The status line after a measurement was deleted.
#[must_use]
pub fn deleted_from(view: &str) -> String {
    format!("The measurement is gone from the view \u{201c}{view}\u{201d}.")
}

/// A view with no name, by its place in the list.
#[must_use]
pub fn unnamed_view(index: usize) -> String {
    format!("View {}", index + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_unit_is_named_only_when_it_is_one_we_know() {
        assert_eq!(unit_word(1.0), "mm");
        // ORACLE, NOT A CONVERSION: the value a model file states for an inch.
        assert_eq!(unit_word(25.4), "in");
        assert_eq!(unit_word(1000.0), "m");
        assert_eq!(unit_word(7.0), "");
    }
}
