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
//! ★ The line-style names are the exception, and they are here rather than in
//! `text::ribbon` or `text::panels::properties` for a reason worth stating:
//! **three surfaces show them** — the pen that authors, the Format ▸ Markup band
//! that restyles, and the Properties panel that restyles — and a name that lived
//! on one surface would be re-spelled on the other two. `canvas::markup::linestyle`
//! is the one module all three read, and this is the one place its words live.
//!
//! ⚠ The count in that first sentence has been wrong before. It read *"Three
//! tooltips and one suffix"* while the opacity tooltip and the percent suffix
//! sat forty lines below it, added on 2026-08-28 without the header being told.
//! A count in prose is a claim nothing checks;
//! [`tests::the_header_counts_what_this_module_actually_holds`] now does.
//!
//! ## Each one answers "what will this change, and when?"
//!
//! Because that is the question a swatch in a ribbon cannot answer by looking
//! like a swatch. Every tooltip here says two things: which markup the setting
//! applies to, and — the half an operator is most likely to get wrong — that it
//! applies to the **next** one rather than to anything already on the page.
//!
//! `RIBBON_IA.md` §5.5 is explicit that these are two different surfaces:
//!
//! > The `Style` group sets defaults for the next markup. Changing an
//! > *existing* markup's style happens on the contextual **Format** tab.
//!
//! The Format tab's property editors are not built yet, so an operator who
//! recolours the swatch expecting the rectangle they just drew to change will
//! be disappointed — and the tooltip is the only thing standing between them
//! and concluding the control is broken. Saying "the next one" is therefore a
//! disclosure and not a nicety.

/// Hover text for the ink swatch.
#[must_use]
pub const fn pen_colour_tooltip() -> &'static str {
    "The colour of the next shape, arrow, line or freehand mark you draw. \
     Marks already on the page keep the colour they were drawn in."
}

/// Hover text for the highlighter swatch.
///
/// A separate control and a separate sentence, because they are separate pens
/// — see `canvas::markup::pen`'s header. An operator who sets the ink to green
/// does not thereby want a green highlight, and a tooltip that said "the
/// markup colour" for both would suggest they had.
#[must_use]
pub const fn highlighter_colour_tooltip() -> &'static str {
    "The colour of the next highlight band. Kept separate from the pen above, \
     so choosing a pen colour does not change your highlighter."
}

/// Hover text for the width control.
///
/// Names the **unit** as well as the effect, because "2" on a ribbon is a
/// number without a scale — and points are what the PDF stores, so it is also
/// the number the operator would see if they opened the file in another
/// program.
#[must_use]
pub const fn pen_width_tooltip() -> &'static str {
    "How thick the next mark's line is, in points — the same unit the document \
     itself uses. A drawing's own linework is often a quarter point, so 2 sits \
     clearly above it without covering it."
}

/// Hover text for the opacity control.
///
/// # ★★★ Why this sentence names the CAD case rather than describing the slider
///
/// Because the reason to reach for it is specific and is not obvious from a
/// percentage: a comment sits on top of the thing it is about, and on a dense
/// drawing an opaque cloud hides the dimension it is drawing attention to. An
/// operator who has never used annotation transparency has no reason to guess
/// that, and a tooltip reading *"the opacity of the next mark"* would restate
/// the label.
///
/// # ★ It says the mark stays selectable, because faint is not gone
///
/// The bottom of the range is a tenth, deliberately (`canvas::markup::pen`'s
/// `MIN_OPACITY` carries the argument), and at a tenth over dark linework a
/// mark can be hard to find with the eye. Saying it is still there and still
/// listed is the disclosure that stops a faint mark reading as a failed one.
#[must_use]
pub const fn pen_opacity_tooltip() -> &'static str {
    "How much of the drawing shows through the next mark. Below 100% the mark \
     is see-through, which is what lets a cloud or a box sit over a dimension \
     without hiding it. Even the faintest mark is still selectable and still \
     listed in the Comments panel."
}

/// The opacity control's suffix.
///
/// A percent sign, because opacity is the one property in this group an
/// operator already thinks about as a percentage — every other program that
/// offers it says 40%, not 0.4. The value written into `/CA` is the fraction;
/// the conversion happens at the control and nowhere else.
#[must_use]
pub const fn opacity_suffix() -> &'static str {
    "%"
}

/// Hover text for the pen's line-style chooser.
///
/// # ★★★ Why this sentence is about the DRAWING and not about the dash
///
/// "Choose a dash pattern" tells an operator what the widget obviously is. What
/// they cannot see from the control is *when* it applies — this is the pen, so
/// it governs the **next** mark and not the one they are looking at — and that
/// is the half every tooltip in this module leads with, for the reason its
/// header gives.
///
/// ★ It also names the one subtype family the setting does nothing for.
/// `MarkupOptions::dash` is *"ignored by the text-markup family"*: a highlight
/// is a colour wash and an underline is its own line, and neither draws a
/// `/BS` border for a dash to be in. The chooser is on the Style group beside
/// the pen colour, which serves the highlighter too, so an operator who set it
/// and then drew a highlight would otherwise be owed an explanation nobody
/// gave them.
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
// ★★ FOUR ENTRIES AND A FIFTH STATE, and the fifth is not an entry.
//
// `canvas::markup::linestyle::LineStyle` has four variants and every one of
// them is offered. `DashReading::Foreign` is a fifth thing the closed chooser
// can say and is deliberately NOT in the list: it means *the file states a dash
// this shell does not offer*, and there is no press that produces it.
//
// ★ THE NAMES ARE WHAT A DRAUGHTSMAN SAYS, NOT WHAT THE FILE STORES. Not
// "[8 3 1 3]", not "/S /D" — the run lengths are in `LineStyle::pattern` where a
// number is useful, and a combo entry reading `[8 3 1 3]` would make an operator
// open all four to find out which is which.

/// The chooser's first entry — no dash at all.
///
/// ★ *Solid*, not *None*. "None" is the word this shell uses for the **absence
/// of a property** — `markup_fill_none`, the arrowhead chooser's first position
/// — and a solid line is not an absence, it is a line. Table 166 agrees: `/S`
/// is a named border style, not a missing one.
#[must_use]
pub const fn line_style_solid() -> &'static str {
    "Solid"
}

/// Table 166's own default dash, `[3]`.
///
/// The plain word, because it is the plain case: an operator who wants "a dashed
/// line" and does not care which dash should find the entry they would have
/// named, and it should be the one the standard itself would have given them.
#[must_use]
pub const fn line_style_dashed() -> &'static str {
    "Dashed"
}

/// `[8 4]`.
///
/// ★ Named by its **appearance**, not by what it is conventionally used for. The
/// tempting name was "Hidden" — the draughting convention this pattern echoes —
/// and it was rejected for `text::markup`'s standing reason about the palette
/// cells: a mark drawn in it is not thereby hidden, and a name that describes a
/// convention rather than the thing on screen makes a claim about the operator's
/// drawing that the annotation does not make.
#[must_use]
pub const fn line_style_long_dash() -> &'static str {
    "Long dash"
}

/// `[8 3 1 3]`.
///
/// Named for what it draws, for [`line_style_long_dash`]'s reason — the centre-
/// line convention it echoes is in `LineStyle`'s doc comment, where a reader who
/// wants the rationale is.
#[must_use]
pub const fn line_style_dash_dot() -> &'static str {
    "Dash-dot"
}

/// What the closed chooser says for a dash the file states and this shell does
/// not offer.
///
/// # ★★★ It names the FILE, and that is the whole job of this string
///
/// The engine preserves a foreign dash through a restyle that does not mention
/// one, so this state is not a defect and is not going to be corrected by
/// anything the operator does — it is simply what their producer wrote.
/// Showing *Dashed* for it would be the quiet lie the colour swatch's CMYK arm
/// was rewritten to stop telling: a control claiming a value that is not the
/// file's, which the operator would discover by pressing something else and
/// watching the pattern change.
///
/// ★ The parenthetical is what keeps it from reading as an error. *"Dashed (the
/// file's own pattern)"* says **this is fine and it is theirs**; a bare
/// *"Unknown dash"* would read as damage and would send an operator looking for
/// a repair that is not needed.
#[must_use]
pub const fn line_style_foreign() -> &'static str {
    "Dashed (the file's own pattern)"
}

// ---------------------------------------------------------------------------
// The palette grid — the name of each colour Acrobat marks up in
// ---------------------------------------------------------------------------
//
// ★★★ THESE WORDS ARE THE ONLY LABEL A COLOUR CELL HAS.
//
// A cell in `canvas::markup::palette::ACROBAT` is a filled square about twelve
// points on a side. It cannot carry text, so the tooltip is the whole of its
// accessible name — the same argument this module's header makes about the two
// swatches, one size down and one step more acute, because there are ten of
// them and they differ only by hue.
//
// ★★ THEY ARE PLAIN COLOUR WORDS, NOT ACROBAT ROLES, AND THAT IS A DECISION.
//
// The tempting alternative was "Underline blue", "Sticky-note violet" — naming
// each cell after the Acrobat tool whose default it is. Rejected: one grid is
// offered from every swatch, so a cell reading "Underline blue" under the
// HIGHLIGHTER swatch would be describing a tool the operator is not using and
// a setting they are not making. The Acrobat role is recorded at each palette
// constant's own doc comment, where the reader who wants it is; the operator
// gets the word they would say out loud.
//
// ★ NO HEX, NO RGB TRIPLE. A tooltip reading "Blue (#1373E8)" tells an operator
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
///
/// **Violet, not purple**, and the difference is worth the thought it took.
/// `#9643FC` sits on the blue side of purple, and the two neighbouring cells are
/// Blue and Magenta — so an operator scanning for "the purple one" between a
/// blue and a magenta gets no help from a word that could mean either. Violet
/// names the position in the spectrum, which is how the cell is found.
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
///
/// Acrobat's strikeout colour, which is a light desaturated red. "Light red"
/// would be the accurate description and is the wrong label: it puts two cells
/// called Red and Light red side by side in a grid, which is a distinction the
/// eye has to make twice. Pink is the word for it.
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
///
/// ★ The one cell whose tooltip earns a second clause. A white mark on a
/// black-on-white CAD sheet is invisible everywhere except over the drawing's
/// own linework, so an operator who picks it by accident sees a tool that has
/// stopped working. Saying so at the moment of choosing is cheaper than the
/// support question.
#[must_use]
pub const fn colour_white() -> &'static str {
    "White — invisible on a white page"
}

/// The heading over the palette grid.
///
/// It names **Adobe**, deliberately and once. The operator's ask was for
/// Acrobat's colours specifically, and a grid captioned "Colours" would look
/// like ten colours somebody liked. This is the one place the provenance of the
/// values is visible from inside the program.
#[must_use]
pub const fn palette_heading() -> &'static str {
    "Acrobat's markup colours"
}

/// The route out of the grid to the full colour picker.
///
/// ★ The trailing ellipsis is the platform convention for *"this opens
/// something"* and is load-bearing here: every other cell in the popup applies
/// immediately, and this one does not.
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
///
/// A separate entry rather than a literal in the widget call, for the reason
/// the settings window's degree sign is: `check-ui-strings.sh` looks for
/// exactly this, and a translator localising the ribbon must be able to see
/// that a unit abbreviation exists.
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

    /// ★ Every tooltip says the setting applies to the NEXT mark.
    ///
    /// The disclosure this module exists for. `RIBBON_IA.md` §5.5 puts
    /// "restyle what is already there" on the contextual Format tab, whose
    /// property editors are not built — so an operator who recolours the swatch
    /// expecting the rectangle they just drew to change has no other way to
    /// learn otherwise, and would reasonably report the control as broken.
    ///
    /// A test rather than a convention, because the natural edit when a tooltip
    /// reads long is to cut its second sentence.
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

    /// ★★ **The ten palette names are ten different words.**
    ///
    /// A cell's name is its whole accessible label — see this module's palette
    /// section — so two cells reading "Purple" would be two controls an operator
    /// cannot tell apart by any means the program offers, hover included.
    ///
    /// It also asserts each is non-empty, which is the failure a `const fn`
    /// returning `""` produces: a cell with no tooltip at all, silently, on a
    /// control that has nothing else to say what it is.
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

    /// ★ **The palette heading names Adobe, and the white cell warns.**
    ///
    /// Two disclosures that a shortening edit would take out first, and both are
    /// the kind this project does not leave to convention:
    ///
    /// * the heading is the only place in the running program where the
    ///   provenance of these ten values is visible — the operator asked for
    ///   *Adobe's* colours and is entitled to see the claim being made;
    /// * white is invisible on a white page, and an operator who picks it sees a
    ///   tool that has stopped working rather than a colour they chose.
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

    /// ★★★ **The header's count of what this module holds is checked.**
    ///
    /// It read *"Three tooltips and one suffix"* for four months after a fourth
    /// tooltip and a second suffix were added. Nothing was broken by it and
    /// nobody could have noticed, which is exactly the class of statement that
    /// rots — a count in prose is a claim with no reader that verifies it.
    ///
    /// Falsified by changing the header to say "five tooltips": the assertion
    /// fired. Restored.
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

    /// ★★ **The five line-style names are distinct, and none of them is a
    /// pattern.**
    ///
    /// The first half is the ordinary anti-collision assertion: a combo whose
    /// two entries read the same is a control an operator cannot use.
    ///
    /// ★ The second half is the one worth having. These names are the whole
    /// reason `LineStyle::pattern`'s run lengths never reach an operator, and
    /// the cheap way to add a fifth style is to name it after its array. This
    /// asserts no name contains a digit — which is what a `[8 4]` or an
    /// `8, 4` creeping into the list would trip.
    ///
    /// Falsified by renaming *Long dash* to `"Dashed 8 4"`, which turned the
    /// digit assertion red.
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
    ///
    /// They are two controls sitting side by side with no labels, so identical
    /// or near-identical hover text would make them indistinguishable — which
    /// is the state the operator is already in before they hover.
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
