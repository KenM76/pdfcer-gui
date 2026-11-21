//! Strings for the 3D viewer's *Face, edge, corner* measure. Consumed by
//! `pdfcer_gui::dialogs::model3d::select`.

use super::model3dreadout::Lengths;

/// The measure's button in the kind row.
pub const SELECT: &str = "Face, edge, corner";
/// The button's tip.
pub const SELECT_TIP: &str = "Pick a face, an edge or a corner of the model to see what it measures; pick a second to see \
     the distances between the two. Curved faces and edges are fitted from the model's triangles, \
     and how closely is said with them. Nothing is written to the file.";
/// What a pick stands for, in the chooser's order.
pub const ENTITIES: [&str; 3] = ["Face", "Edge", "Corner"];
/// Each choice's tip, in the same order.
pub const ENTITY_TIPS: [&str; 3] = [
    "A click picks the whole face it lands on.",
    "A click picks the model edge nearest it.",
    "A click picks the corner nearest it.",
];
/// Before the chooser.
pub const PICKS: &str = "A click picks";
/// The first pick's heading in the details.
pub const FIRST: &str = "First";
/// The second pick's heading.
pub const SECOND: &str = "Second";
/// Said when the model's file states no unit.
pub const NO_UNIT: &str = "The file states no unit, so lengths are in the model's own units.";
/// Said when the file separates no faces, so a face is the whole part.
pub const WHOLE_PART: &str =
    "This file does not separate the part into faces, so a face's figures are the whole part's.";
/// Said when the file gives the picked face only as part of a larger one, so
/// the figures are the flat patch round the click.
pub const GROWN_FACE: &str = "The file does not give this face on its own, so these are the figures of the flat patch at the click.";
/// Said of a face that is neither flat nor a cylinder.
pub const UNFITTED_FACE: &str = "Neither flat nor a cylinder.";
/// Said of an edge that is neither straight nor a circle.
pub const CURVE: &str = "A curve, neither straight nor a circle.";

/// What the next click picks, `done` already picked.
#[must_use]
pub fn next(done: usize, entity: usize) -> String {
    let what = ENTITIES
        .get(entity)
        .copied()
        .unwrap_or("Face")
        .to_lowercase();
    let article = if what.starts_with('e') { "an" } else { "a" };
    if done == 0 {
        format!("Pick {article} {what}.")
    } else {
        format!("Pick a second {what} to measure between them.")
    }
}

/// A corner's position.
#[must_use]
pub fn corner([x, y, z]: [f64; 3], l: &Lengths<'_>) -> String {
    format!(
        "Corner at X {}, Y {}, Z {}",
        l.coordinate(x),
        l.coordinate(y),
        l.coordinate(z)
    )
}

/// An edge's headline.
#[must_use]
pub fn edge_length(length: f64, l: &Lengths<'_>) -> String {
    format!("Edge length {}", l.length(length))
}

/// A straight edge, or an open one's chord.
#[must_use]
pub fn edge_chord(chord: f64, closed: bool, l: &Lengths<'_>) -> String {
    if closed {
        "Closes on itself.".to_owned()
    } else {
        format!("End to end {}", l.length(chord))
    }
}

/// A straight edge.
pub const STRAIGHT: &str = "Straight.";

/// A circular edge or face: its radius and diameter, and how closely the
/// triangles fit it.
#[must_use]
pub fn circle(radius: f64, residual: f64, l: &Lengths<'_>) -> String {
    format!(
        "Radius {}, diameter {}, fitted within {}",
        l.length(radius),
        l.length(radius * 2.0),
        l.length(residual)
    )
}

/// How far round a circular edge reaches, and its arc length.
#[must_use]
pub fn arc(sweep_degrees: f64, arc_length: f64, l: &Lengths<'_>) -> String {
    format!(
        "Arc of {sweep_degrees:.1}\u{b0}, {} long",
        l.length(arc_length)
    )
}

/// A face's headline.
#[must_use]
pub fn face_area(area: f64, l: &Lengths<'_>) -> String {
    format!("Face area {}", l.area(area))
}

/// A face's perimeter: its outline and the rims of its holes.
#[must_use]
pub fn perimeter(perimeter: f64, l: &Lengths<'_>) -> String {
    format!("Perimeter {}", l.length(perimeter))
}

/// A flat face.
#[must_use]
pub fn flat(flatness_degrees: f64) -> String {
    format!("Flat, within {flatness_degrees:.2}\u{b0}.")
}

/// A cylindrical face's length and sweep.
#[must_use]
pub fn cylinder(length: f64, sweep_degrees: f64, l: &Lengths<'_>) -> String {
    format!(
        "Cylinder {} long, {sweep_degrees:.1}\u{b0} round",
        l.length(length)
    )
}

/// The engine could not measure a pick.
#[must_use]
pub fn refused(why: &str) -> String {
    format!("Cannot measure this: {why}")
}

/// The headline of two picks: their smallest distance.
#[must_use]
pub fn minimum(length: f64, l: &Lengths<'_>) -> String {
    format!("Smallest distance {}", l.length(length))
}

/// The largest distance between two picks.
#[must_use]
pub fn maximum(length: f64, l: &Lengths<'_>) -> String {
    format!("Largest distance {}", l.length(length))
}

/// Between the centres of two picks that have one.
#[must_use]
pub fn centres(length: f64, l: &Lengths<'_>) -> String {
    format!("Centre to centre {}", l.length(length))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_area_carries_a_squared_unit_and_a_bare_one_none() {
        assert_eq!(
            face_area(16.0, &Lengths::plain("m")),
            "Face area 16.000 m\u{b2}"
        );
        assert_eq!(face_area(16.0, &Lengths::plain("")), "Face area 16.000");
    }

    #[test]
    fn the_next_pick_names_what_a_click_picks() {
        assert_eq!(next(0, 1), "Pick an edge.");
        assert_eq!(next(0, 2), "Pick a corner.");
        assert_eq!(next(1, 0), "Pick a second face to measure between them.");
    }
}
