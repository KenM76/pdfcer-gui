//! Strings for the 3D viewer's light strength, its aimed sun, how many of a
//! model's lamps a frame draws, and the lamp counts said under the picture.
//! Consumed by `pdfcer_gui::dialogs::model3d`.

/// The brightness slider's label.
#[must_use]
pub fn brightness_label() -> &'static str {
    "Brightness"
}

/// Its tip.
#[must_use]
pub fn brightness_tooltip() -> &'static str {
    "How strongly the chosen lighting lights the model, its ambient light included. Each lighting choice keeps its own brightness. A part's own glow is not changed. Colours brighter than white are clipped."
}

/// A light strength `gain` as a percentage, for a slider's value.
#[must_use]
pub fn percent(gain: f64) -> String {
    format!("{:.0}%", gain * 100.0)
}

/// The third shadow caster: a sun the operator aims and colours.
#[must_use]
pub fn aimed_sun() -> &'static str {
    "A sun you aim"
}

/// The sun's bearing slider's label.
#[must_use]
pub fn sun_bearing_label() -> &'static str {
    "Sun bearing"
}

/// Its tip.
#[must_use]
pub fn sun_bearing_tooltip() -> &'static str {
    "Which side the sun shines from, measured round the Up axis: 0° is the model's front, 90° its right, 180° its back, 270° its left."
}

/// The sun's height slider's label.
#[must_use]
pub fn sun_height_label() -> &'static str {
    "Sun height"
}

/// Its tip.
#[must_use]
pub fn sun_height_tooltip() -> &'static str {
    "How high the sun stands: 0° on the horizon, 90° straight overhead. A lower sun casts longer shadows."
}

/// A slider's degrees suffix.
#[must_use]
pub fn degrees_suffix() -> &'static str {
    "°"
}

/// The switch colouring the sun by its height.
#[must_use]
pub fn sun_colour_by_height() -> &'static str {
    "Colour by height"
}

/// Its tip.
#[must_use]
pub fn sun_colour_by_height_tooltip() -> &'static str {
    "Warm orange near the horizon, as at sunrise and sunset, whitening as it rises to midday. Turn it off to set the colour temperature yourself."
}

/// The sun's colour temperature slider's label.
#[must_use]
pub fn sun_kelvin_label() -> &'static str {
    "Colour temperature"
}

/// Its tip.
#[must_use]
pub fn sun_kelvin_tooltip() -> &'static str {
    "Lower is warmer: about 2000 K at sunrise and sunset, 5500 K at midday, 6500 K pure white, higher a bluish sky light."
}

/// A slider's kelvin suffix.
#[must_use]
pub fn kelvin_suffix() -> &'static str {
    " K"
}

/// The sun's strength slider's label.
#[must_use]
pub fn sun_strength_label() -> &'static str {
    "Sun strength"
}

/// Its tip.
#[must_use]
pub fn sun_strength_tooltip() -> &'static str {
    "How bright the sun is, on top of the lighting's brightness. The sun is added to the chosen lighting and casts the shadows."
}

/// What the sun drawn is, under its controls: `kelvin` its colour
/// temperature.
#[must_use]
pub fn sun_said(kelvin: f64) -> String {
    format!(
        "The sun lights the model at {kelvin:.0} K and casts the shadows; the lighting adds the rest, unshadowed."
    )
}

/// The lamp cost drop-down's label.
#[must_use]
pub fn lamps_drawn_label() -> &'static str {
    "Lamps drawn"
}

/// Every lamp reaching the view is drawn.
#[must_use]
pub fn lamps_drawn_all() -> &'static str {
    "All in view"
}

/// Only the `n` lamps adding most at the view's centre are drawn.
#[must_use]
pub fn lamps_drawn_nearest(n: u16) -> String {
    format!("Brightest {n} at the view")
}

/// Its tip.
#[must_use]
pub fn lamps_drawn_tooltip() -> &'static str {
    "For a slow phone or tablet. Every lamp drawn costs time at every pixel, so a model with many lamps can crawl. Drawing only the brightest few at the centre of the view is faster, but whatever the others lit goes darker. Lamps outside the view are never drawn either way, and that changes nothing."
}

/// The lamp counts under the picture: `in_model` the model's lamps,
/// `in_view` those reaching the view, `lit` those drawn.
#[must_use]
pub fn lamp_counts(in_model: usize, in_view: usize, lit: usize) -> String {
    format!("Lamps: {in_model} in the model, {in_view} reaching the view, {lit} lit.")
}

/// Lamps reaching the view left out by the lamp cost choice.
#[must_use]
pub fn lamps_left_out(count: usize) -> String {
    format!(
        "{count} lamp(s) reaching the view are left out to draw faster, so the picture is darker than every lamp would light it (Lamps drawn)."
    )
}

/// Lamps lit without a shadow past the `limit` casting them.
#[must_use]
pub fn lamp_shadows_capped(casting: usize, dropped: usize, limit: usize) -> String {
    format!(
        "{casting} lamp(s) cast shadows; {dropped} more past the limit of {limit}, those adding least at the view's centre, light without one."
    )
}
