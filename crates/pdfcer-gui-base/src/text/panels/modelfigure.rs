//! # `text::panels::modelfigure` — the words of the figure seen from behind
//!
//! The figure's measures and shadows, in the 3D viewer's Moving tab and on
//! the Settings window's 3D viewer page.

/// The group's heading.
#[must_use]
pub fn heading() -> &'static str {
    "Figure"
}

/// The head's size.
#[must_use]
pub fn head() -> &'static str {
    "Head"
}

/// Shoulder width.
#[must_use]
pub fn shoulders() -> &'static str {
    "Shoulders"
}

/// Arm length.
#[must_use]
pub fn arms() -> &'static str {
    "Arms"
}

/// Leg length.
#[must_use]
pub fn legs() -> &'static str {
    "Legs"
}

/// Girth.
#[must_use]
pub fn build() -> &'static str {
    "Build"
}

/// The proportions' shared tip.
#[must_use]
pub fn measure_tooltip() -> &'static str {
    "A share of the figure's own proportion: 100% is as made, 50% half, 200% twice. The figure's size follows the eye height, its eyes at yours, so longer legs make a shorter body."
}

/// A proportion's value.
#[must_use]
pub fn percent(v: f64) -> String {
    format!("{:.0}%", v * 100.0)
}

/// The shadow under the feet.
#[must_use]
pub fn blob() -> &'static str {
    "Shadow underneath"
}

/// Its tip.
#[must_use]
pub fn blob_tooltip() -> &'static str {
    "A soft dark patch on the floor under the figure's feet, so you can see where it stands."
}

/// The shadow cast by a light.
#[must_use]
pub fn cast() -> &'static str {
    "Cast a shadow"
}

/// Its tip.
#[must_use]
pub fn cast_tooltip() -> &'static str {
    "The figure's own shape laid on the floor, as from a light high to one side. It follows the figure's arms and legs as it walks."
}

/// Put the measures back as made.
#[must_use]
pub fn reset() -> &'static str {
    "As made"
}

/// Its tip.
#[must_use]
pub fn reset_tooltip() -> &'static str {
    "Every proportion back to 100%."
}
