//! Words on the Forms ▸ Style band, which sets how the next placed field looks.

/// The fill swatch's label.
pub const fn fill_label() -> &'static str {
    "Fill"
}

/// The fill swatch's hover.
pub const fn fill_hover() -> &'static str {
    "What the next field you place is filled with. The placement dialog opens on this, and accepting the dialog sets it here."
}

/// The outline swatch's label.
pub const fn outline_label() -> &'static str {
    "Outline"
}

/// The outline swatch's hover.
pub const fn outline_hover() -> &'static str {
    "The ink the next field's outline is drawn in — and a check box's tick or a radio button's dot."
}

/// The border-width field's hover.
pub const fn width_hover() -> &'static str {
    "How thick the next field's outline is, in points. Zero draws no outline."
}

/// Hover for the border-style picker on Forms ▸ Style.
#[must_use]
pub const fn style_hover() -> &'static str {
    "How the next field's outline is drawn: solid, dashed, raised, sunken, or a line under the box only."
}
