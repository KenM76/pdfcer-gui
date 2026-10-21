//! # `text::colourpick` — the words of the colour picker
//!
//! Tab names, component labels and hovers for the Grey / RGB / CMYK picker.

use crate::colourpick::Space;

/// A tab's name.
#[must_use]
pub const fn space(space: Space) -> &'static str {
    match space {
        Space::Gray => "Grey",
        Space::Rgb => "RGB",
        Space::Cmyk => "CMYK",
    }
}

/// Hover on a tab.
#[must_use]
pub const fn space_tooltip(space: Space) -> &'static str {
    match space {
        Space::Gray => "Store the colour as a shade of grey, from white to black.",
        Space::Rgb => "Store the colour as red, green and blue, as a screen shows it.",
        Space::Cmyk => {
            "Store the colour as cyan, magenta, yellow and black ink, as a press prints it. \
             Switching here starts from a plain conversion of the current colour."
        }
    }
}

/// Labels for red, green and blue.
#[must_use]
pub const fn rgb_labels() -> [&'static str; 3] {
    ["R", "G", "B"]
}

/// Labels for cyan, magenta, yellow and black.
#[must_use]
pub const fn cmyk_labels() -> [&'static str; 4] {
    ["Cyan", "Magenta", "Yellow", "Black"]
}

/// The grey row's label: the amount of black.
#[must_use]
pub const fn gray_label() -> &'static str {
    "Black"
}

/// The opacity row's label.
#[must_use]
pub const fn opacity() -> &'static str {
    "Opacity"
}

/// The suffix on every percentage.
#[must_use]
pub const fn percent() -> &'static str {
    " %"
}

/// Under the CMYK swatch.
#[must_use]
pub const fn cmyk_on_screen() -> &'static str {
    "The swatch shows the ink as pdfcer draws it; the numbers are what is saved."
}

/// Under the CMYK numbers when they were seeded from a colour CMYK cannot draw.
#[must_use]
pub const fn outside_cmyk() -> &'static str {
    "That colour is outside CMYK; the nearest CMYK colour is used."
}
