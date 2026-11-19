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

/// The light meter's button beside the kinds.
pub const LIGHT: &str = "Light level";
/// The light meter's tip.
pub const LIGHT_TIP: &str = "How much light arrives at a point you pick on a surface, such as a desk top: the light of \
     the lamps the picture is drawn with, shadows counted, plus the light bounced from other \
     surfaces. Nothing is written to the file.";
/// Measure at the picked point alone.
pub const LIGHT_POINT: &str = "At the point";
/// Measure the mean over a circle round the picked point.
pub const LIGHT_CIRCLE: &str = "Average over a circle";
/// The circle choice's tip.
pub const LIGHT_CIRCLE_TIP: &str = "The mean of many points spread over a circle on the picked surface, centred on the point you \
     pick. Points that fall off the surface are left out and counted.";
/// Before the circle's radius.
pub const LIGHT_RADIUS: &str = "Radius";
/// What the next click does.
pub const LIGHT_PICK: &str = "Pick a point on a surface to read the light there.";
/// A reading is being worked out.
pub const LIGHT_WORKING: &str = "Reading the light\u{2026}";
/// Above the lamps' parts of a reading.
pub const LIGHT_SHARES: &str = "Straight from each light:";
/// Above the reasons a reading is not in lux.
pub const LIGHT_RELATIVE_WHY: &str = "This is not lux. It is in the picture's own shading units, where 1.0 lights a white surface \
     to full white: good for comparing two places, not a physical figure. Because:";

/// The switch that counts the light of the model's glowing parts.
pub const LIGHT_GLOW: &str = "Count glowing parts";
/// Its tip.
pub const LIGHT_GLOW_TIP: &str = "On, a glowing part lights the surfaces it faces, shadows counted, as a lamp would. A glow that states its brightness reads in lux; one that does not makes the reading relative, not lux. Off, only the model's lamps put light on a surface.";

/// A reading's headline: the amount and its unit.
#[must_use]
pub fn light_total(total: f64, lux: bool, circle: bool) -> String {
    let what = if circle {
        "Average over the circle"
    } else {
        "At this point"
    };
    if lux {
        format!("{what}: {} lux", light_number(total + 0.0))
    } else {
        format!("{what}: {:.3}, relative, not lux", total + 0.0)
    }
}

/// Under a relative headline: why it is not lux, said where it is seen.
pub fn light_not_lux(reasons: &str) -> String {
    format!("Not in lux because {reasons}.")
}

/// A lux figure: whole above ten, two places below.
#[must_use]
pub fn light_number(lux: f64) -> String {
    if lux.abs() >= 10.0 {
        format!("{lux:.0}")
    } else {
        format!("{lux:.2}")
    }
}

/// What a reading is made of.
#[must_use]
pub fn light_parts(direct: &str, glow: Option<&str>, bounced: &str, ambient: &str) -> String {
    match glow {
        Some(glow) => format!(
            "Straight from lights {direct} · From glowing parts {glow} · Bounced {bounced} · Ambient {ambient}"
        ),
        None => format!("Straight from lights {direct} · Bounced {bounced} · Ambient {ambient}"),
    }
}

/// What a circle's reading is the mean of.
#[must_use]
pub fn light_circle(radius: f64, unit: &str, landed: usize, fell_off: usize) -> String {
    let space = if unit.is_empty() { "" } else { " " };
    if fell_off == 0 {
        format!("The mean of {landed} points in a circle of radius {radius:.3}{space}{unit}.")
    } else {
        format!(
            "The mean of {landed} points in a circle of radius {radius:.3}{space}{unit}; {fell_off} more fell off \
             the surface and are left out."
        )
    }
}

/// One light's part of a reading.
#[must_use]
pub fn light_share(name: &str, amount: &str) -> String {
    format!("{name}: {amount}")
}

/// The lights past the ones listed.
#[must_use]
pub fn light_more(count: usize) -> String {
    format!("and {count} more that give less")
}

/// The meter could not read there; `why` is the engine's reason.
#[must_use]
pub fn light_refused(why: &str) -> String {
    format!("No reading: {why}.")
}

/// The surface measure's button beside the kinds.
pub const SURFACES: &str = "Between surfaces";
/// The surface measure's tip.
pub const SURFACES_TIP: &str = "Pick two flat surfaces. Surfaces parallel within the tolerance you set give the distance \
     between them, any others the angle between them; the angle is shown either way. Nothing is \
     written to the file.";
/// Before the parallel tolerance.
pub const SURFACE_PARALLEL: &str = "Parallel within";
/// The parallel tolerance's tip.
pub const SURFACE_PARALLEL_TIP: &str = "Surfaces whose planes meet at this angle or less count as parallel and are measured for \
     distance; wider ones are measured for angle.";
/// The force-parallel switch.
pub const SURFACE_FORCE: &str = "Always measure a distance";
/// The force-parallel switch's tip.
pub const SURFACE_FORCE_TIP: &str = "Give a distance even when the surfaces are wider apart than the tolerance: from the second \
     picked point to the first surface's plane. The angle is still shown.";
/// The first of the two surfaces, in the tab.
pub const SURFACE_FIRST: &str = "First surface";
/// The second of the two.
pub const SURFACE_SECOND: &str = "Second surface";

/// What the next click picks, `done` surfaces picked.
#[must_use]
pub fn surface_next(done: usize) -> &'static str {
    if done == 0 {
        "Pick the first surface."
    } else {
        "Pick the second surface."
    }
}

/// `value` followed by `unit` when there is one.
fn with_unit(value: f64, unit: &str) -> String {
    if unit.is_empty() {
        format!("{value:.3}")
    } else {
        format!("{value:.3} {unit}")
    }
}

/// The headline of a distance between parallel surfaces.
#[must_use]
pub fn surface_distance(distance: f64, unit: &str) -> String {
    format!("Distance {}", with_unit(distance, unit))
}

/// The headline of an angle between surfaces.
#[must_use]
pub fn surface_angle(degrees: f64) -> String {
    format!("Angle {degrees:.2}\u{b0}")
}

/// How the two planes lie against the tolerance: `forced` is `None` for a
/// pair measured for angle.
#[must_use]
pub fn surface_relation(angle: f64, tolerance: f64, forced: Option<bool>) -> String {
    match forced {
        Some(false) => {
            format!("Parallel within {tolerance:.2}\u{b0}: the planes are {angle:.2}\u{b0} apart.")
        }
        Some(true) => format!(
            "Not parallel: the planes are {angle:.2}\u{b0} apart, wider than {tolerance:.2}\u{b0}. The \
             distance is from the second point to the first surface's plane."
        ),
        None => format!(
            "Not parallel within {tolerance:.2}\u{b0}, so the angle is measured: {angle:.2}\u{b0}."
        ),
    }
}

/// What the two fitted planes rest on: triangles counted, how many surfaces
/// are only part of their face, and whether the file marks faces at all.
#[must_use]
pub fn surface_fit(triangles: [usize; 2], partial: usize, faces: bool) -> String {
    let [a, b] = triangles;
    let head = format!("Planes fitted from {a} and {b} triangles.");
    match (faces, partial) {
        (false, _) => format!(
            "{head} The file marks no faces, so each surface is the flat part of its part round the pick."
        ),
        (true, 0) => format!("{head} Each is a whole face."),
        (true, 1) => {
            format!("{head} One is only the flat part round the pick of a curved or stepped face.")
        }
        (true, _) => format!(
            "{head} Both are only the flat part round the pick of a curved or stepped face."
        ),
    }
}

/// One surface in the tab: its name, what it was fitted from, its area and
/// how flat the fit is.
#[must_use]
pub fn surface_detail(
    which: &str,
    triangles: usize,
    area: f64,
    unit: &str,
    flatness: f64,
) -> String {
    let squared = if unit.is_empty() {
        String::new()
    } else {
        format!(" {unit}\u{b2}")
    };
    format!("{which}: {triangles} triangles, area {area:.3}{squared}, flat to {flatness:.2}\u{b0}.")
}

/// The straight distance between the two picked points.
#[must_use]
pub fn surface_points(distance: f64, unit: &str) -> String {
    format!("Between the picked points: {}", with_unit(distance, unit))
}

/// The angle between the surfaces' outward sides.
#[must_use]
pub fn surface_sides(degrees: f64) -> String {
    format!(
        "Their outward sides are {degrees:.2}\u{b0} apart: 180\u{b0} for the two sides of a plate, \
         0\u{b0} for two steps facing the same way."
    )
}

/// The engine refused the measurement; `why` is its reason.
#[must_use]
pub fn surface_refused(why: &str) -> String {
    format!("No measurement: {why}.")
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

    #[test]
    fn a_surface_measure_says_which_relation_and_what_the_planes_rest_on() {
        assert_eq!(surface_distance(1.0, "mm"), "Distance 1.000 mm");
        assert_eq!(surface_distance(1.0, ""), "Distance 1.000");
        assert!(surface_relation(0.1, 0.5, Some(false)).starts_with("Parallel within"));
        assert!(surface_relation(90.0, 0.5, Some(true)).starts_with("Not parallel:"));
        assert!(surface_relation(90.0, 0.5, None).contains("angle is measured"));
        assert!(surface_fit([2, 2], 0, true).ends_with("Each is a whole face."));
        assert!(surface_fit([2, 2], 1, true).contains("One is only"));
        assert!(surface_fit([2, 2], 2, true).contains("Both are only"));
        assert!(surface_fit([2, 2], 2, false).contains("marks no faces"));
        assert_eq!(surface_refused("no area"), "No measurement: no area.");
    }

    #[test]
    fn a_light_headline_names_the_area_and_says_when_it_is_not_lux() {
        assert_eq!(
            light_total(312.4, true, true),
            "Average over the circle: 312 lux"
        );
        assert!(light_total(0.25, false, false).starts_with("At this point: 0.250"));
        assert!(light_total(-0.0, false, false).ends_with("0.000, relative, not lux"));
        assert_eq!(light_not_lux("a glow"), "Not in lux because a glow.");
    }
}
