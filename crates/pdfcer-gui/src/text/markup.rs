//! # `text::markup` — the words the Markup ▸ Style group shows
//!
//! Five tooltips, two suffixes, **ten colour names** and the **five names a
//! line style goes by**, which is the whole operator-visible surface of
//! `canvas::markup::swatch` and of `canvas::markup::linestyle`. Most of the
//! controls are colour chips and numbers: none of those can carry a label
//! without doubling the width of a ribbon group, so **the tooltip is the only
//! place they say what they are** — which makes these strings load-bearing
//! rather than supplementary.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/text/markup.md`.

/// Hover text for the ink swatch.
#[must_use]
pub const fn pen_colour_tooltip() -> &'static str {
    "The colour of the next shape, arrow, line or freehand mark you draw. \
     Marks already on the page keep the colour they were drawn in."
}

/// Hover text for the highlighter swatch.
#[must_use]
pub const fn highlighter_colour_tooltip() -> &'static str {
    "The colour of the next highlight band. Kept separate from the pen above, \
     so choosing a pen colour does not change your highlighter."
}

/// Hover text for the width control.
#[must_use]
pub const fn pen_width_tooltip() -> &'static str {
    "How thick the next mark's line is, in points — the same unit the document \
     itself uses. A drawing's own linework is often a quarter point, so 2 sits \
     clearly above it without covering it."
}

/// Hover text for the opacity control.
#[must_use]
pub const fn pen_opacity_tooltip() -> &'static str {
    "How much of the drawing shows through the next mark. Below 100% the mark \
     is see-through, which is what lets a cloud or a box sit over a dimension \
     without hiding it. Even the faintest mark is still selectable and still \
     listed in the Comments panel."
}

/// The opacity control's suffix.
#[must_use]
pub const fn opacity_suffix() -> &'static str {
    "%"
}

/// Hover text for the pen's line-style chooser.
#[must_use]
pub const fn pen_dash_tooltip() -> &'static str {
    "Whether the next shape, arrow, line or freehand mark is drawn with a solid \
     line or a dashed one. Highlights, underlines and strikeouts have no outline \
     to dash, so it does not change those."
}

// ---------------------------------------------------------------------------
// The line styles — the five things a border can be called
// ---------------------------------------------------------------------------
//
// FOUR ENTRIES AND A FIFTH STATE, and the fifth is not an entry.
//
// `canvas::markup::linestyle::LineStyle` has four variants and every one of
// them is offered. `DashReading::Foreign` is a fifth thing the closed chooser
// can say and is deliberately NOT in the list: it means *the file states a dash
// this shell does not offer*, and there is no press that produces it.
//
// THE NAMES ARE WHAT A DRAUGHTSMAN SAYS, NOT WHAT THE FILE STORES. Not
// "[8 3 1 3]", not "/S /D" — the run lengths are in `LineStyle::pattern` where a
// number is useful, and a combo entry reading `[8 3 1 3]` would make an operator
// open all four to find out which is which.

/// The chooser's first entry — no dash at all.
#[must_use]
pub const fn line_style_solid() -> &'static str {
    "Solid"
}

/// Table 166's own default dash, `[3]`.
#[must_use]
pub const fn line_style_dashed() -> &'static str {
    "Dashed"
}

/// `[8 4]`.
#[must_use]
pub const fn line_style_long_dash() -> &'static str {
    "Long dash"
}

/// `[8 3 1 3]`.
#[must_use]
pub const fn line_style_dash_dot() -> &'static str {
    "Dash-dot"
}

/// What the closed chooser says for a dash the file states and this shell does
/// not offer.
#[must_use]
pub const fn line_style_foreign() -> &'static str {
    "Dashed (the file's own pattern)"
}

// ---------------------------------------------------------------------------
// The palette grid — the name of each colour Acrobat marks up in
// ---------------------------------------------------------------------------
//
// THESE WORDS ARE THE ONLY LABEL A COLOUR CELL HAS.
//
// A cell in `canvas::markup::palette::ACROBAT` is a filled square about twelve
// points on a side. It cannot carry text, so the tooltip is the whole of its
// accessible name — the same argument this module's header makes about the two
// swatches, one size down and one step more acute, because there are ten of
// them and they differ only by hue.
//
// THEY ARE PLAIN COLOUR WORDS, NOT ACROBAT ROLES, AND THAT IS A DECISION.
//
// The tempting alternative was "Underline blue", "Sticky-note violet" — naming
// each cell after the Acrobat tool whose default it is. Rejected: one grid is
// offered from every swatch, so a cell reading "Underline blue" under the
// HIGHLIGHTER swatch would be describing a tool the operator is not using and
// a setting they are not making. The Acrobat role is recorded at each palette
// constant's own doc comment, where the reader who wants it is; the operator
// gets the word they would say out loud.
//
// NO HEX, NO RGB TRIPLE. A tooltip reading "Blue (#1373E8)" tells an operator
// choosing a pen colour nothing they can act on, and pushes the useful word off
// the front of a narrow tip. The numbers are in the code and in the palette
// module's table, which is where a number is useful.

/// The palette cell at [`crate::canvas::markup::palette::MARKUP_RED`].
#[must_use]
pub const fn colour_red() -> &'static str {
    "Red"
}

/// The palette cell at [`crate::canvas::markup::palette::HIGHLIGHTER_ORANGE`].
#[must_use]
pub const fn colour_orange() -> &'static str {
    "Orange"
}

/// The palette cell at [`crate::canvas::markup::palette::CLASSIC_YELLOW`].
#[must_use]
pub const fn colour_yellow() -> &'static str {
    "Yellow"
}

/// The palette cell at [`crate::canvas::markup::palette::FREETEXT_GREEN`].
#[must_use]
pub const fn colour_green() -> &'static str {
    "Green"
}

/// The palette cell at [`crate::canvas::markup::palette::UNDERLINE_BLUE`].
#[must_use]
pub const fn colour_blue() -> &'static str {
    "Blue"
}

/// The palette cell at [`crate::canvas::markup::palette::NOTE_PURPLE`].
#[must_use]
pub const fn colour_violet() -> &'static str {
    "Violet"
}

/// The palette cell at [`crate::canvas::markup::palette::CARET_MAGENTA`].
#[must_use]
pub const fn colour_magenta() -> &'static str {
    "Magenta"
}

/// The palette cell at [`crate::canvas::markup::palette::STRIKEOUT_PINK`].
#[must_use]
pub const fn colour_pink() -> &'static str {
    "Pink"
}

/// The palette cell at [`crate::canvas::markup::palette::BLACK`].
#[must_use]
pub const fn colour_black() -> &'static str {
    "Black"
}

/// The palette cell at [`crate::canvas::markup::palette::WHITE`].
#[must_use]
pub const fn colour_white() -> &'static str {
    "White — invisible on a white page"
}

/// The heading over the palette grid.
#[must_use]
pub const fn palette_heading() -> &'static str {
    "Acrobat's markup colours"
}

/// The route out of the grid to the full colour picker.
#[must_use]
pub const fn more_colours() -> &'static str {
    "More colours…"
}

/// Hover text for the More-colours button.
#[must_use]
pub const fn more_colours_tooltip() -> &'static str {
    "Open the full colour picker to choose a colour that is not in the grid. \
     Anything you pick there is used exactly as chosen."
}

/// The width control's suffix.
#[must_use]
pub const fn width_suffix() -> &'static str {
    " pt"
}

mod edits;

pub use edits::{
    AnnotDeleteRefusal, NodeEditRefusal, ShapeWord, annot_delete_locked, appearance_distorted,
    deleted_collateral, deletion_would_take, ink_redrawn_straight, measure_stale, note_removed,
    note_replaced, popup_left_behind, rich_text_dropped, stroke_width_unchanged,
};

#[cfg(test)]
mod tests {
    use super::*;

    /// Every tooltip says the setting applies to the NEXT mark.
    #[test]
    fn every_style_tooltip_says_it_applies_to_the_next_mark() {
        for tip in [
            pen_colour_tooltip(),
            highlighter_colour_tooltip(),
            pen_width_tooltip(),
        ] {
            assert!(
                tip.contains("next"),
                "a Style tooltip no longer says it applies to the next mark: {tip:?}"
            );
        }
    }

    /// **The ten palette names are ten different words.**
    #[test]
    fn every_palette_cell_has_its_own_word() {
        let names = [
            colour_red(),
            colour_orange(),
            colour_yellow(),
            colour_green(),
            colour_blue(),
            colour_violet(),
            colour_magenta(),
            colour_pink(),
            colour_black(),
            colour_white(),
        ];
        for (i, name) in names.iter().enumerate() {
            assert!(!name.trim().is_empty(), "cell {i} has no name at all");
            for (j, other) in names.iter().enumerate().skip(i + 1) {
                assert_ne!(name, other, "cells {i} and {j} are both named {name:?}");
            }
        }
    }

    /// **The palette heading names Adobe, and the white cell warns.**
    #[test]
    fn the_palette_says_where_its_colours_came_from() {
        assert!(
            palette_heading().contains("Acrobat"),
            "the heading must name the program these values were measured from: {:?}",
            palette_heading()
        );
        assert!(
            colour_white().to_lowercase().contains("invisible"),
            "the white cell must warn that it disappears on a white page: {:?}",
            colour_white()
        );
        assert!(
            more_colours().ends_with('…'),
            "the ellipsis is the convention for 'this opens something', and it is \
             the only cell in the popup that does: {:?}",
            more_colours()
        );
    }

    /// **The header's count of what this module holds is checked.**
    #[test]
    fn the_header_counts_what_this_module_actually_holds() {
        let header = include_str!("markup.rs");
        let first_line = header
            .lines()
            .find(|l| l.contains("tooltips"))
            .expect("the header's opening sentence names a count of tooltips");
        // Five: pen colour, highlighter colour, width, opacity, line style.
        // Counted here rather than derived, because the point is to compare the
        // prose against a number a human had to think about.
        let tooltips = [
            pen_colour_tooltip(),
            highlighter_colour_tooltip(),
            pen_width_tooltip(),
            pen_opacity_tooltip(),
            pen_dash_tooltip(),
        ];
        assert_eq!(tooltips.len(), 5);
        assert!(
            first_line.contains("Five tooltips"),
            "this module holds {} tooltips and its header says: {first_line:?}",
            tooltips.len()
        );
        let suffixes = [opacity_suffix(), width_suffix()];
        assert_eq!(suffixes.len(), 2);
        assert!(first_line.contains("two suffixes"), "{first_line:?}");
    }

    /// **The five line-style names are distinct, and none of them is a
    /// pattern.**
    #[test]
    fn the_line_style_names_are_words_rather_than_arrays() {
        let names = [
            line_style_solid(),
            line_style_dashed(),
            line_style_long_dash(),
            line_style_dash_dot(),
            line_style_foreign(),
        ];
        for name in names {
            assert!(!name.trim().is_empty());
            assert!(
                !name.chars().any(|c| c.is_ascii_digit()),
                "a line style is named for what it draws, not for its run lengths: {name:?}"
            );
        }
        let mut sorted = names.to_vec();
        let total = sorted.len();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), total, "two line styles share a name");
        assert!(
            line_style_foreign().contains("file"),
            "the foreign reading must name the FILE, or it reads as one of ours: {:?}",
            line_style_foreign()
        );
    }

    /// The two colour tooltips are different sentences about different pens.
    #[test]
    fn the_two_swatches_are_told_apart_by_their_words() {
        assert_ne!(pen_colour_tooltip(), highlighter_colour_tooltip());
        assert!(highlighter_colour_tooltip().contains("highlight"));
    }

    /// The width suffix names a unit and is not empty.
    #[test]
    fn the_width_carries_its_unit() {
        assert!(width_suffix().contains("pt"));
        assert!(pen_width_tooltip().contains("points"));
    }
}
