//! # `canvas::markup::palette` — **Acrobat's own markup colours**, measured
//! rather than chosen
//!
//! The operator's ask, verbatim, on 2026-09-06:
//!
//! > *"Also make sure you've used the same default colours and style look for
//! > these things as Adobe."*
//!
//! This module is the answer's data half: the ten colours Adobe Acrobat itself
//! authors comments in, and the grid the Style swatch offers them from.
//! [`super::pen`] is the answer's behaviour half.
//!
//! ## ★★★ WHERE THESE NUMBERS COME FROM — the whole point of this header
//!
//! A colour written into `/C` reaches the operator's saved file. Under this
//! project's standing **claim-bearing copy** rule — *verify the source, don't
//! invent* — a document colour presented as *"what Adobe uses"* is a claim, and
//! a plausible-looking hex triple sourced from memory or from a blog would be
//! exactly the invention that rule forbids.
//!
//! So every value below was **read out of Acrobat's own defaults store** on the
//! operator's machine on **2026-09-06**:
//!
//! ```text
//! HKEY_CURRENT_USER\Software\Adobe\Adobe Acrobat\DC\Annots\cAnnots\<subtype>\cstrokeColor
//!     t0 = RGB            the colour space
//!     d1, d2, d3          the three /DeviceRGB components, 0.0 – 1.0
//! ```
//!
//! That key is where Acrobat stores the *tool default properties* each markup
//! tool draws with — the values its own **Properties ▸ Make Properties Default**
//! writes — so it is the same number Acrobat would put in `/C`, not a
//! description of one.
//!
//! ### The measurement, verbatim
//!
//! | Acrobat key | `d1, d2, d3` | as bytes | this module |
//! |---|---|---|---|
//! | `cSquare`, `cCircle`, `cLine`, `cLine:LineArrow`, `cPolyLine`, `cPolygon`, `cPolygon:PolygonCloud`, `cInk`, `cSquiggly`, `cStamp` | `0.858826, 0.203918, 0.145096` | `219, 52, 37` | [`MARKUP_RED`] |
//! | `cHighlight`, `cInk:InkHighlight`, `cSound` | `1.000000, 0.384308, 0.000000` | `255, 98, 0` | [`HIGHLIGHTER_ORANGE`] |
//! | `cUnderline` | `0.074509, 0.450974, 0.909805` | `19, 115, 232` | [`UNDERLINE_BLUE`] |
//! | `cStrikeOut`, `cFreeText\cstrokeColor` | `0.972549, 0.392151, 0.392151` | `248, 100, 100` | [`STRIKEOUT_PINK`] |
//! | `cText` (sticky note), `cFileAttachment`, `cHighlight:HighlightNote` | `0.588242, 0.262741, 0.988235` | `150, 67, 252` | [`NOTE_PURPLE`] |
//! | `cCaret` | `0.752945, 0.215683, 0.768631` | `192, 55, 196` | [`CARET_MAGENTA`] |
//! | `cFreeText\crichDefaults\ctextColor` | `0.023529, 0.541183, 0.109802` | `6, 138, 28` | [`FREETEXT_GREEN`] |
//! | every `ctextColor` | `0.000000, 0.000000, 0.000000` | `0, 0, 0` | [`BLACK`] |
//! | `cFreeText\cfillColor` | `1.000000, 1.000000, 1.000000` | `255, 255, 255` | [`WHITE`] |
//!
//! ### ★★ Why this is Acrobat's FACTORY default and not Ken's last click
//!
//! `HKCU` is a per-user store, so the honest first question is whether these are
//! the operator's own past choices rather than Adobe's shipped values. Three
//! pieces of evidence say factory, and they are recorded because the reader who
//! doubts this table deserves the reasoning rather than an assurance:
//!
//! 1. **Unrelated subtypes share exact values.** `cHighlight`, `cInk:InkHighlight`
//!    and `cSound` all hold `1.0, 0.384308, 0.0` to six places. Nobody sets a
//!    *Sound annotation's* colour by hand, and certainly not to bit-for-bit
//!    agreement with the highlighter.
//! 2. **Ten shape subtypes agree to six decimal places.** A user who recoloured
//!    a rectangle would move `cSquare` and leave `cProjection` behind.
//! 3. **Every component lands on an exact 1/255 boundary.** `0.858826 × 255 =
//!    219.0006`; `0.384308 × 255 = 98.0`; `0.074509 × 255 = 19.0`. These are
//!    byte values a designer picked and a float store round-tripped, not values
//!    a colour wheel produced.
//!
//! ### ★★★ THE SURPRISE, and it is the reason a measurement beat a memory
//!
//! **Acrobat's highlighter is ORANGE, not yellow.** `1.0, 0.384308, 0.0` is
//! `#FF6200`. Everything anyone "knows" about PDF highlighting says yellow, this
//! shell has shipped `(1.0, 1.0, 0.0)` since the pen existed, and the reasoning
//! written into [`super::pen::Pen::default`] said *"yellow … because that is what
//! every PDF reader draws it in"*. That sentence was written from memory and it
//! was **wrong about the program the operator actually compares against**.
//!
//! Yellow is still in the grid — as [`CLASSIC_YELLOW`], sourced honestly as *this
//! shell's own shipped highlighter*, one click away — because an operator who
//! wants yellow must not have to leave the palette to get it. What changed is
//! which of the two is the **default**, and the answer to *"is it the same as
//! Adobe"* is now measured rather than assumed.
//!
//! ## Why the stored form is BYTES and not Acrobat's own fractions
//!
//! Acrobat's registry holds `0.858826`; this module holds `219`. `219 / 255 =
//! 0.858823…`, which differs from Acrobat's stored number in the sixth decimal
//! place and is invisible at any output resolution that exists.
//!
//! The byte form wins because it makes **one** value the answer to two questions.
//! A colour reaches `/C` down two paths — as a shipped default, and as a cell the
//! operator clicked in the grid — and the grid can only offer a `Color32`, which
//! is bytes. Storing fractions would make the shipped orange and the clicked
//! orange different numbers in the file for no reason an operator could ever see,
//! and *"why is my second highlight a different colour from my first"* is a bug
//! report nobody could reproduce.
//!
//! ⇒ The fractions are preserved in the table above, which is the record. The
//! bytes are the code, which is the behaviour.
//!
//! ## What is deliberately NOT here
//!
//! * **A line width.** Acrobat's `cAnnots` tree carries no width, thickness or
//!   border key at all — searched for `width`, `thick` and `border` across the
//!   whole `DC` tree, and the only hits were print and multimedia settings. So
//!   there is no measured Adobe number to match, and [`super::pen::Pen::default`]
//!   keeps this shell's own 2 pt with the argument written out there rather than
//!   adopting a number nobody sourced.
//! * **A dimension colour.** `cLine:LineDimension` exists in the same tree and is
//!   deliberately not read: a **ce dimension** is not markup, it has its own style
//!   verb and its own pen (rule 15), and folding Acrobat's dimension default into
//!   the markup palette would be the exact conflation that rule forbids.
//! * **A redaction colour.** `cRedact` is likewise present and likewise not
//!   markup; `crate::text::redact` owns that surface and its own vocabulary.

use egui::Color32;

use crate::text::markup as t;

/// **Acrobat's markup red** — `#DB3425`.
///
/// The single most-used value in Acrobat's defaults store: ten different
/// subtypes hold it, covering every shape tool this shell offers. It is what a
/// rectangle, an ellipse, an arrow, a polyline, a polygon, a revision cloud and
/// a freehand mark are drawn in when Acrobat is opened for the first time.
///
/// Not the same red this shell shipped. `Pen::default`'s old ink was
/// `(0.85, 0.16, 0.16)` = `#D92929`, chosen by eye to read as "comment red".
/// Acrobat's is fractionally lighter and distinctly warmer.
pub const MARKUP_RED: [u8; 3] = [219, 52, 37];

/// **Acrobat's highlighter** — `#FF6200`, an orange.
///
/// See the module header: this is the measurement that contradicted what
/// everybody, including this file's previous author, believed.
pub const HIGHLIGHTER_ORANGE: [u8; 3] = [255, 98, 0];

/// **Acrobat's underline** — `#1373E8`.
///
/// Its own key, its own colour, and nothing else in the store shares it. This is
/// the clearest single refutation of the *"one pen for all linework"* argument
/// [`super::pen`] used to make: Adobe gives underline a colour that is not the
/// shape pen's and not the strikeout's.
pub const UNDERLINE_BLUE: [u8; 3] = [19, 115, 232];

/// **Acrobat's strikeout** — `#F86464`, a light red/pink.
///
/// Shared with `cFreeText`'s *border* colour, which is a detail worth keeping:
/// Acrobat draws a text box's frame in this and its text in [`MARKUP_RED`], so
/// the two are one design pair rather than two coincidences.
pub const STRIKEOUT_PINK: [u8; 3] = [248, 100, 100];

/// **Acrobat's sticky note** — `#9643FC`, a violet.
///
/// Shared with the file-attachment marker and with a highlight-carrying-a-note.
/// ★ Emphatically **not** the yellow sticky of folk memory — that is the icon
/// Acrobat *used* to draw, and the current comment UI marks a note in this
/// violet. Another value that a memory would have got wrong.
pub const NOTE_PURPLE: [u8; 3] = [150, 67, 252];

/// **Acrobat's caret** — `#C037C4`, a magenta.
///
/// This shell authors no `/Caret`, so nothing defaults to it. It is in the grid
/// because it is a colour **Acrobat itself marks up in** and a picker offering
/// no magenta at all sends the operator to the full picker for a hue Adobe
/// already chose.
pub const CARET_MAGENTA: [u8; 3] = [192, 55, 196];

/// **Acrobat's text-box lettering** — `#068A1C`, a dark green.
///
/// From `cFreeText\crichDefaults\ctextColor`: the colour Acrobat types rich
/// `/FreeText` content in. Same argument as [`CARET_MAGENTA`] — nothing here
/// defaults to it, and it is the green Adobe picked rather than a green this
/// module picked.
pub const FREETEXT_GREEN: [u8; 3] = [6, 138, 28];

/// **This shell's own highlighter yellow** — `#FFFF00`.
///
/// ★ The one entry in the grid whose source is **not** Acrobat, and it is
/// labelled as such rather than smuggled in. It was `Pen::default`'s
/// `highlighter` from the day the pen existed until 2026-09-06, so every
/// highlight this shell has ever authored is this colour and an operator
/// re-marking an old drawing needs it reachable in one click.
///
/// Pure `#FFFF00` is also the value a highlighter yellow *is* in every program
/// that offers one, which is why it needs no further defence — only an honest
/// label saying it came from here and not from Adobe.
pub const CLASSIC_YELLOW: [u8; 3] = [255, 255, 0];

/// **Black** — `#000000`, from every `ctextColor` in Acrobat's store.
pub const BLACK: [u8; 3] = [0, 0, 0];

/// **White** — `#FFFFFF`, from `cFreeText\cfillColor`.
///
/// ★ Worth having in a *markup* palette specifically because this shell's
/// drawings are black-on-white CAD sheets: a white mark is the one that
/// disappears, and an operator who picks it by accident needs to be able to see
/// that they did. The grid draws every cell with a border for exactly that
/// reason — see [`Swatch`].
pub const WHITE: [u8; 3] = [255, 255, 255];

/// One cell of the palette grid: a colour and the word for it.
///
/// # ★ The name is not decoration — it is the only label the cell has
///
/// A colour cell is a filled square about twelve points on a side. It cannot
/// carry text, so the tooltip is the entire accessible name of the control, in
/// exactly the way [`crate::text::markup`]'s header says the Style group's
/// swatches are. `check-ui-strings.sh` is what keeps those words in the text
/// module; the `&'static str` here is produced by a `const fn` in that module,
/// so this array can be a `const` and the words can still live where they
/// belong.
///
/// # Why the name is a plain colour word and not Acrobat's role
///
/// The tempting alternative was *"Underline blue"*, *"Sticky-note violet"* —
/// naming each cell after the Acrobat tool it is the default for. It is
/// rejected: the grid is offered for **both** swatches and for every future
/// slot, so a cell called "Underline blue" appearing under the highlighter
/// swatch would be describing a tool the operator is not using. The role is
/// recorded at each constant's own doc comment, where a reader of the code
/// wants it, and the operator gets the word they would use out loud.
pub struct Swatch {
    /// The colour, as sRGB bytes. See the module header on why bytes.
    pub rgb: [u8; 3],
    /// The operator-visible name, from [`crate::text::markup`].
    pub name: &'static str,
}

impl Swatch {
    /// The cell's colour as egui sees it.
    ///
    /// **DOCUMENT COLOUR.** A palette cell is a preview of a value that is one
    /// click from `/C` and therefore from the saved file. A restyle that moved
    /// it would be the application claiming an annotation had changed colour,
    /// which is the case `check-theme-colors.sh`'s escape hatch exists for and
    /// is stated at [`super::pen::color32_of`] in the same words.
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
///
/// Divided by `255.0`, not `256.0`, for the reason [`super::pen::rgb_of`] gives
/// at length: the component range is inclusive of both ends, so `255` must map
/// to exactly `1.0` or white would be written as `0.996` and would not be white.
#[must_use]
pub fn components([r, g, b]: [u8; 3]) -> (f64, f64, f64) {
    (
        f64::from(r) / 255.0,
        f64::from(g) / 255.0,
        f64::from(b) / 255.0,
    )
}

/// **How many cells per row.**
///
/// Five, giving a 5 × 2 grid for [`ACROBAT`]'s ten entries.
///
/// # Why not one long row, and why not a square
///
/// A single row of ten cells is about 150 points wide, which is wider than the
/// ribbon group that opens it and puts the last cell a long way from the button
/// the operator pressed. A 3 × 4 would need twelve entries and there are ten
/// sourced ones — padding it to twelve would mean inventing two colours, which
/// is the whole thing this module refuses to do.
///
/// Two rows of five is also the shape Acrobat's own in-place colour strip has,
/// which is the tie-breaker the operator himself named: *make it work the way
/// the other program does.*
pub const COLUMNS: usize = 5;

/// **The palette**, in the order it is drawn: hues left to right, neutrals last.
///
/// # ★ The order is a spectrum, deliberately
///
/// Red, orange, yellow, green, blue, violet, magenta, pink, then black and
/// white. Not the order the constants are declared in, and not Acrobat's
/// registry order (which is alphabetical by subtype and is meaningless to a
/// human eye). A colour grid is scanned visually, and a scan finds *"the blue
/// one"* in a spectrum and hunts for it in a list.
///
/// The two neutrals go last, together, on the end of the second row, because
/// they are the two an operator picks for a *reason* rather than by hue —
/// black to match a drawing's own linework, white to sit on top of it.
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

    /// ★★★ **Every colour a markup kind defaults to is IN the grid.**
    ///
    /// The property that makes the palette a palette rather than a decoration:
    /// an operator who changes the highlighter to red and wants it back must be
    /// able to click the value it shipped with. If a default is not in the grid
    /// there is no way back to it except by remembering three numbers and typing
    /// them into the full picker — which is the state a "restore defaults" bug
    /// report describes.
    ///
    /// ⚠ Note which cell is load-bearing here, because it is not the obvious
    /// one: [`CLASSIC_YELLOW`] is **not** a default any more (the highlighter
    /// ships at [`HIGHLIGHTER_ORANGE`]), so removing the yellow row would not
    /// fire this. The cells this test actually protects are the five a slot
    /// ships at.
    ///
    /// Falsified by changing [`ACROBAT`]'s orange cell to `[254, 98, 0]` — one
    /// byte off the highlighter's default: the assertion fired naming
    /// `Highlighter`. Restored.
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

    /// ★ **The measured Acrobat fractions round-trip to these bytes.**
    ///
    /// This is the test that keeps the module header honest. The header claims
    /// each byte triple is the registry's float triple; a typo in either would
    /// make the claim false and nothing else would notice, because a wrong-but-
    /// plausible red still looks like a red.
    ///
    /// The tolerance is a **half a byte** — the largest error a correct
    /// conversion can have — so a value that is one byte out fails.
    ///
    /// Falsified by changing [`UNDERLINE_BLUE`]'s green from 115 to 116: the
    /// assertion fired on the `cUnderline` row. Restored.
    #[test]
    fn each_constant_is_the_registry_value_it_claims_to_be() {
        /// One row of the measurement: the bytes this module stores, the
        /// fractions Acrobat's registry holds, and the key they were read from.
        ///
        /// A named type because the tuple is three unrelated things and clippy
        /// is right that an inline `[([u8; 3], (f64, f64, f64), &str); 7]` is
        /// unreadable — but the *shape* is the point of the test, so it is named
        /// rather than simplified away.
        type Reading = ([u8; 3], (f64, f64, f64), &'static str);
        // NOT A THEME COLOUR: these are the /DeviceRGB fractions read out of
        // Acrobat's own defaults store on 2026-09-06 — the measurement this
        // module's table transcribes, asserted against the bytes it stores.
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
