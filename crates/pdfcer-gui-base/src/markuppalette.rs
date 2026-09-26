//! # `markuppalette` — **Acrobat's own markup colours**, measured
//! rather than chosen
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/markuppalette.md`.

use egui::Color32;

use crate::text::markup as t;

/// **Acrobat's markup red** — `#DB3425`.
pub const MARKUP_RED: [u8; 3] = [219, 52, 37];

/// **Acrobat's highlighter** — `#FF6200`, an orange.
///
/// See the module header: this is the measurement that contradicted what
/// everybody, including this file's previous author, believed.
pub const HIGHLIGHTER_ORANGE: [u8; 3] = [255, 98, 0];

/// **Acrobat's underline** — `#1373E8`.
pub const UNDERLINE_BLUE: [u8; 3] = [19, 115, 232];

/// **Acrobat's strikeout** — `#F86464`, a light red/pink.
pub const STRIKEOUT_PINK: [u8; 3] = [248, 100, 100];

/// **Acrobat's sticky note** — `#9643FC`, a violet.
pub const NOTE_PURPLE: [u8; 3] = [150, 67, 252];

/// **Acrobat's caret** — `#C037C4`, a magenta.
pub const CARET_MAGENTA: [u8; 3] = [192, 55, 196];

/// **Acrobat's text-box lettering** — `#068A1C`, a dark green.
pub const FREETEXT_GREEN: [u8; 3] = [6, 138, 28];

/// **This shell's own highlighter yellow** — `#FFFF00`.
pub const CLASSIC_YELLOW: [u8; 3] = [255, 255, 0];

/// **Black** — `#000000`, from every `ctextColor` in Acrobat's store.
pub const BLACK: [u8; 3] = [0, 0, 0];

/// **White** — `#FFFFFF`, from `cFreeText\cfillColor`.
pub const WHITE: [u8; 3] = [255, 255, 255];

/// One cell of the palette grid: a colour and the word for it.
pub struct Swatch {
    /// The colour, as sRGB bytes. See the module header on why bytes.
    pub rgb: [u8; 3],
    /// The operator-visible name, from [`crate::text::markup`].
    pub name: &'static str,
}

impl Swatch {
    /// The cell's colour as egui sees it.
    #[must_use]
    pub const fn color32(&self) -> Color32 {
        // DOCUMENT COLOUR: a palette cell, one click from the annotation's `/C`.
        Color32::from_rgb(self.rgb[0], self.rgb[1], self.rgb[2])
    }

    /// The cell's colour as PDF `/DeviceRGB` components.
    #[must_use]
    pub fn rgb_components(&self) -> (f64, f64, f64) {
        components(self.rgb)
    }
}

/// PDF `/DeviceRGB` components from sRGB bytes.
#[must_use]
pub fn components([r, g, b]: [u8; 3]) -> (f64, f64, f64) {
    (
        f64::from(r) / 255.0,
        f64::from(g) / 255.0,
        f64::from(b) / 255.0,
    )
}

/// **How many cells per row.**
pub const COLUMNS: usize = 5;

/// **The palette**, in the order it is drawn: hues left to right, neutrals last.
pub const ACROBAT: [Swatch; 10] = [
    Swatch {
        rgb: MARKUP_RED,
        name: t::colour_red(),
    },
    Swatch {
        rgb: HIGHLIGHTER_ORANGE,
        name: t::colour_orange(),
    },
    Swatch {
        rgb: CLASSIC_YELLOW,
        name: t::colour_yellow(),
    },
    Swatch {
        rgb: FREETEXT_GREEN,
        name: t::colour_green(),
    },
    Swatch {
        rgb: UNDERLINE_BLUE,
        name: t::colour_blue(),
    },
    Swatch {
        rgb: NOTE_PURPLE,
        name: t::colour_violet(),
    },
    Swatch {
        rgb: CARET_MAGENTA,
        name: t::colour_magenta(),
    },
    Swatch {
        rgb: STRIKEOUT_PINK,
        name: t::colour_pink(),
    },
    Swatch {
        rgb: BLACK,
        name: t::colour_black(),
    },
    Swatch {
        rgb: WHITE,
        name: t::colour_white(),
    },
];
