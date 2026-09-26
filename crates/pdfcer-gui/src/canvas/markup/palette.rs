//! # `canvas::markup::palette` — **Acrobat's own markup colours**, measured
//! rather than chosen
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/markup/palette.md`.

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

#[cfg(test)]
mod tests {
    use super::*;

    /// **Every colour a markup kind defaults to is IN the grid.**
    #[test]
    fn every_shipped_default_is_one_click_away_in_the_grid() {
        use super::super::pen::{Pen, PenSlot};
        let pen = Pen::default();
        for slot in PenSlot::ALL {
            let wanted = pen.colour_of(*slot);
            assert!(
                ACROBAT.iter().any(|s| close(s.rgb_components(), wanted)),
                "{slot:?} ships at {wanted:?}, which no cell in the palette offers — \
                 an operator who changes it has no way back"
            );
        }
    }

    /// Two cells with the same colour would be two ways to say one thing, and
    /// the operator would have no way to tell which they had picked.
    #[test]
    fn no_two_cells_are_the_same_colour() {
        for (i, a) in ACROBAT.iter().enumerate() {
            for (j, b) in ACROBAT.iter().enumerate().skip(i + 1) {
                assert_ne!(a.rgb, b.rgb, "cells {i} and {j} are the same colour");
            }
        }
    }

    /// …and no two carry the same word, for the same reason one step further
    /// out: the word is the cell's only label.
    #[test]
    fn no_two_cells_are_named_the_same() {
        for (i, a) in ACROBAT.iter().enumerate() {
            for (j, b) in ACROBAT.iter().enumerate().skip(i + 1) {
                assert_ne!(a.name, b.name, "cells {i} and {j} share a name");
            }
        }
    }

    /// **The measured Acrobat fractions round-trip to these bytes.**
    #[test]
    fn each_constant_is_the_registry_value_it_claims_to_be() {
        /// One row of the measurement: the bytes this module stores, the
        /// fractions Acrobat's registry holds, and the key they were read from.
        type Reading = ([u8; 3], (f64, f64, f64), &'static str);
        let measured: [Reading; 7] = [
            (MARKUP_RED, (0.858_826, 0.203_918, 0.145_096), "cSquare"),
            (HIGHLIGHTER_ORANGE, (1.0, 0.384_308, 0.0), "cHighlight"),
            (
                UNDERLINE_BLUE,
                (0.074_509, 0.450_974, 0.909_805),
                "cUnderline",
            ),
            (
                STRIKEOUT_PINK,
                (0.972_549, 0.392_151, 0.392_151),
                "cStrikeOut",
            ),
            (NOTE_PURPLE, (0.588_242, 0.262_741, 0.988_235), "cText"),
            (CARET_MAGENTA, (0.752_945, 0.215_683, 0.768_631), "cCaret"),
            (
                FREETEXT_GREEN,
                (0.023_529, 0.541_183, 0.109_802),
                "cFreeText/crichDefaults",
            ),
        ];
        // Half a byte, in component units: the largest error a correct rounding
        // can produce. One byte out fails.
        let tolerance = 0.5 / 255.0;
        for (bytes, fractions, key) in measured {
            let (r, g, b) = components(bytes);
            assert!(
                (r - fractions.0).abs() < tolerance
                    && (g - fractions.1).abs() < tolerance
                    && (b - fractions.2).abs() < tolerance,
                "{key}: the registry holds {fractions:?} and this module stores \
                 {bytes:?}, which is {:?} — the header's table is wrong",
                (r, g, b)
            );
        }
    }

    /// The grid divides evenly into rows, so the last row is not a ragged
    /// remainder.
    #[test]
    fn the_grid_is_rectangular() {
        assert_eq!(
            ACROBAT.len() % COLUMNS,
            0,
            "a partial last row leaves a gap the operator reads as a missing colour"
        );
    }

    /// Component-wise equality at PDF precision.
    fn close(a: (f64, f64, f64), b: (f64, f64, f64)) -> bool {
        (a.0 - b.0).abs() < 1e-9 && (a.1 - b.1).abs() < 1e-9 && (a.2 - b.2).abs() < 1e-9
    }
}
