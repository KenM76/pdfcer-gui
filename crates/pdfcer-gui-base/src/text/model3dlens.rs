//! Strings for the 3D viewer's lens choice: flat, fisheye or Auto, the
//! fisheye's curve, Auto's change point, and the line under the picture
//! naming the lens drawn through. Consumed by `pdfcer_gui::dialogs::model3d`.

/// The lens drop-down's label.
#[must_use]
pub fn lens_label() -> &'static str {
    "Lens"
}

/// The lens choices, in the viewer's order: flat, fisheye, Auto.
#[must_use]
pub fn lens_names() -> [&'static str; 3] {
    ["Flat", "Fisheye", "Auto"]
}

/// The lens drop-down's tip.
#[must_use]
pub fn lens_tooltip() -> &'static str {
    "How a wide view is drawn. Flat keeps straight lines straight and stretches the edges. Fisheye bends straight lines and fits far more in, up to half a sphere top to bottom when walking. Auto stays flat until the view is taller than its change point, then turns fisheye, as a phone camera changes to its wide lens. A view written into the file is always flat: a PDF view cannot hold a fisheye."
}

/// The lens drop-down's tip while the view is orthographic and it is greyed.
#[must_use]
pub fn lens_orthographic_tooltip() -> &'static str {
    "A view without perspective has no lens. Turn Perspective on to choose one."
}

/// The fisheye curve drop-down's label.
#[must_use]
pub fn curve_label() -> &'static str {
    "Curve"
}

/// The fisheye curves, in the viewer's order: stereographic, equidistant.
#[must_use]
pub fn curve_names() -> [&'static str; 2] {
    ["Natural", "Classic"]
}

/// The curve drop-down's tip.
#[must_use]
pub fn curve_tooltip() -> &'static str {
    "Natural keeps shapes least stretched, for rooms and buildings. Classic spaces angles evenly from the middle, as most fisheye camera lenses do, and squeezes the edges more."
}

/// Auto's change-point slider's label.
#[must_use]
pub fn auto_at_label() -> &'static str {
    "Fisheye past"
}

/// The slider's unit, after its number.
#[must_use]
pub fn degrees_suffix() -> &'static str {
    "°"
}

/// The slider's tip.
#[must_use]
pub fn auto_at_tooltip() -> &'static str {
    "Auto draws a fisheye once the view is taller than this angle, and flat below it. Zoom out while walking to pass it."
}

/// The line under the picture while it is drawn through a fisheye, `fov`
/// degrees tall, with curve `curve` (one of [`curve_names`]).
#[must_use]
pub fn drawn_fisheye(curve: &str, fov: f64) -> String {
    format!("Drawn through a fisheye lens ({curve} curve), {fov:.0}° tall.")
}

/// The line under the picture while Auto draws it flat, changing at `at`
/// degrees.
#[must_use]
pub fn auto_flat(at: f64) -> String {
    format!("Drawn through a flat lens; Auto turns fisheye past {at:.0}° tall.")
}
