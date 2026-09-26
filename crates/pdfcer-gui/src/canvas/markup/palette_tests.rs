//! Tests for `crate::canvas::markup::palette`, kept in the gui because they reach gui modules.

use crate::canvas::markup::palette::*;

/// **Every colour a markup kind defaults to is IN the grid.**
#[test]
fn every_shipped_default_is_one_click_away_in_the_grid() {
    use crate::canvas::markup::pen::{Pen, PenSlot};
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
