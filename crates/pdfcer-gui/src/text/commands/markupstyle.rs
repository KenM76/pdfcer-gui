//! # `text::commands::markupstyle` — the labels and tooltips of **Format ▸
//! Markup**, the five controls that restyle a mark that is already on the page
//!
//! ## ★ Why this is a file of its own
//!
//! **R2** — no source file over 1,500 lines. [`super`] stood at 1,438 when
//! these five arrived, and five `CommandText`s written to this project's
//! register (every choice carrying its *why*) is comfortably more than the
//! 62 lines of headroom that left. `tools/gates/check-file-size.sh` says in its
//! own header that shaving prose to fit a threshold is the behaviour it exists
//! to refuse, so the subject moved instead.
//!
//! The seam is [`super::annotate`]'s, one step further along the same line.
//! That file holds the strings of the commands that **place** a mark — the
//! Markup and Measure tabs, *what you add on top of the page*. This one holds
//! the strings of the commands that **restyle a mark already placed**, which is
//! the contextual Format tab's half of the same subject and is governed by a
//! different engine verb (`EditSession::set_markup_style`, not
//! `add_markup`), a different operand (an `ObjId`, not a gesture), and a
//! different question in the ribbon (*what is selected?*, not *what am I about
//! to draw?*).
//!
//! ## ★★★ Every tooltip below has to read correctly in TWO states
//!
//! The same constraint that shaped the Font block in [`super`], for the same
//! mechanical reason: `egui_shell::ribbon::control::render_command` shows a
//! command's tooltip with `on_hover_text` when the control is live and
//! `on_disabled_hover_text` when it is not — **the same string** — and
//! [`crate::app::markupband`] reproduces that behaviour by hand for the custom
//! items, because the shell does none of it for an `Item::Custom`.
//!
//! The states differ from the Font group's, though, and the difference decides
//! the wording:
//!
//! | group | absent when | greyed when | so the tooltip must also say |
//! |---|---|---|---|
//! | Font | the mode cannot edit content | nothing is swept | **how to sweep** — O37's *"nothing tells you to press T"* |
//! | Markup | no markup is selected, or the mode cannot author markup | the mark is **locked**, or its geometry did not read | nothing extra |
//!
//! ⇒ The Font tooltips end *"Sweeping text with the Text tool (T) chooses what
//! it applies to"* because an operator meets those controls greyed and has no
//! way to guess the gesture. These five are **absent** in that situation rather
//! than greyed — the group is not drawn at all until a mark is selected — so an
//! operator only ever meets them with an operand already in hand, and a clause
//! telling them to select something would be describing the thing they just
//! did. The one greyed state that remains has its own sentence, drawn by
//! [`crate::app::markupband`] from [`crate::text::panels::properties`], because
//! *"this mark is locked"* is a fact about that annotation and not about the
//! command.
//!
//! ## ★★ What the fill tooltip has to say, and why it is the longest
//!
//! `canvas::markup::spec` authors every shape with `interior: None` — no fill —
//! and its reason is quoted in `panels::properties::markup`'s header: *"a
//! filled comment shape hides the drawing it is a comment about, which on a CAD
//! sheet is the whole content under it."* Acrobat's default is the same.
//!
//! So the operator's mental model is *marks have no fill*, and a control that
//! offers one has to answer two questions at once: what it does, and how to get
//! back. Its tooltip therefore names the **no fill** state explicitly, because
//! that state is the one the mark started in and the one an operator will want
//! to return to after trying a fill on a drawing.
//!
//! ★ This does **not** change what new markup is authored with. The pen is
//! `canvas::markup::pen`'s and it is untouched; this restyles one existing
//! annotation, which is a different act with a different verb.

use super::CommandText;

/// `format.colour` — the mark's **stroke** colour, `/C`.
///
/// # ★ "Line colour", not "Colour" and not "Stroke"
///
/// Three names were possible and two are refused:
///
/// - **"Colour"** is what `RIBBON_IA.md` §5.8's table calls the row, and it is
///   taken: `format.font_colour` already carries it, and its own doc comment
///   records the collision from the other side — *"`format.colour` was already
///   taken, by the markup property editor in the same tab's future."* Two
///   commands sharing a label is not a style problem; it is two controls the
///   operator cannot tell apart, which is the rule
///   `no_two_commands_share_a_label` exists to hold.
/// - **"Stroke"** is the PDF word. `crate::text::paint`'s own note settles it
///   for this catalog: *"'Fill' and 'Line', not 'fill' and 'stroke'. Stroke is
///   the PDF word"* — a name the file format uses and the operator does not.
///
/// ⇒ "Line colour" pairs with [`format_fill`]'s "Fill colour" so the two read
/// as one family, which is what an operator scanning a band of five needs more
/// than either name needs to be short.
///
/// ## ★★ The tooltip names the refusal, because the refusal is common here
///
/// A `/C` that is not RGB or grey — DeviceCMYK, or a separation — has no
/// faithful sRGB, so the swatch shows the default rather than a converted
/// near-match that the next press would write back. That is
/// `panels::properties::markup::rgb_of`'s rule and
/// [`crate::app::markupband`] re-derives it; stating it in the tooltip is what
/// keeps the swatch from reading as broken on exactly the drawings this
/// program is for.
#[must_use]
pub const fn format_colour() -> CommandText {
    CommandText::new(
        "Line colour",
        "Set the colour of the selected mark's outline. A mark drawn in CMYK or a spot colour \
         shows the default swatch instead, so its ink is not converted to screen colour behind \
         your back.",
    )
}

/// `format.fill` — the mark's **interior** colour, `/IC`.
///
/// ★★★ **The one control here whose default state is "off", and the tooltip
/// has to say so.** See this module's header: pdfcer authors every shape with
/// no fill on purpose, Acrobat does the same, and an operator who tries a fill
/// on a drawing needs to know in the same sentence how to get back to the
/// mark they had. `StyleEdit::Clear` is that route and *"No fill"* is what it
/// is called on screen.
///
/// ★ It says *"shapes that have an interior"* rather than listing them,
/// because the list is the engine's and would go stale here: `/IC` is
/// meaningful for `Square`, `Circle`, `Polygon` and the cloud built on one, and
/// `MarkupStyle::interior`'s own doc says the subtypes without an interior
/// **ignore** it rather than refusing — *"a property of the shape and not an
/// error."* [`crate::app::markupband`] draws no fill control for those at all,
/// which is the honest surface for an ignored field, so this sentence only has
/// to be true of the marks the control is drawn beside.
#[must_use]
pub const fn format_fill() -> CommandText {
    CommandText::new(
        "Fill colour",
        "Fill the inside of the selected mark, for the shapes that have an interior. Marks are \
         drawn with no fill so that they do not hide the drawing underneath, and No fill puts \
         one back the way it was.",
    )
}

/// `format.line_width` — `/BS` `/W`, in points.
///
/// ★★ **The tooltip discloses that the mark's box moves.**
/// `MarkupStyle::width`'s own doc carries the warning — for every subtype
/// except `Square` and `Circle` the `/Rect` is derived from the geometry plus a
/// margin that contains the stroke and any arrowheads, so a wider pen needs a
/// bigger box — and the Properties panel discloses it in a sentence under the
/// section ([`crate::text::panels::properties::markup_note`]).
///
/// A ribbon band has no room for that sentence, so it goes in the hover, which
/// is Rule 4's shape exactly: the restyled mark renders on the canvas precisely
/// as the saved file will render it, and what pdfcer *inferred* on the
/// operator's behalf is disclosed off-canvas.
#[must_use]
pub const fn format_line_width() -> CommandText {
    CommandText::new(
        "Line width",
        "Set how thick the selected mark is drawn, in points. A thicker line needs more room, so \
         for every shape but a rectangle or an ellipse the mark's box grows with it.",
    )
}

/// `format.opacity` — `/CA`, shown as a percentage.
///
/// ★ **Per cent, not `0.0`–`1.0`.** That is the unit every application an
/// operator has used states opacity in, and `/CA`'s own range is a file-format
/// detail they should never meet. The Properties panel's twin makes the same
/// choice and its doc comment carries the argument.
///
/// ★★ The tooltip says what the setting is *for* — seeing the drawing through
/// a mark — rather than what the number means, because the number is on the
/// control. A tooltip that reads "sets the opacity" is a tooltip that has told
/// the operator nothing they could not read off the label.
#[must_use]
pub const fn format_opacity() -> CommandText {
    CommandText::new(
        "Opacity",
        "Set how solid the selected mark is, from clear to fully opaque, so the drawing \
         underneath can show through it.",
    )
}

/// `format.arrowheads` — `/LE`, the pair of line endings. `/Line` only.
///
/// # ★★★ Why the control offers four POSITIONS and not nine pairs
///
/// `/LE` is two independent endings (§12.5.6.7, Table 176) and pdfcer's author
/// side offers three shapes each, which is nine combinations — a menu nobody
/// would read on a ribbon band. [`crate::app::markupband`] therefore offers the
/// four **positions** an operator actually means (none, at the start, at the
/// end, at both) and **preserves the shape** the mark already carries, so a
/// closed arrowhead stays closed and an open one stays open.
///
/// ⇒ The tooltip says "arrowheads" and not "line endings", for
/// [`format_colour`]'s reason: `/LE` is the file format's word and *arrowhead*
/// is the operator's. It also names the `/Line`-only restriction, because the
/// control is **absent** for every other subtype and an operator who saw it on
/// an arrow and then not on a cloud is owed the rule rather than left to infer
/// one.
#[must_use]
pub const fn format_arrowheads() -> CommandText {
    CommandText::new(
        "Arrowheads",
        "Choose which ends of the selected line carry an arrowhead. Only a line or an arrow has \
         ends to put one on, so this is not offered for the other marks.",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The five hold the catalog's two copy rules — a label is a name and
    /// takes no trailing period, a tooltip is prose and ends in one.
    ///
    /// ★ Asserted **here** rather than by adding five rows to
    /// `super::tests::all()`. That list is a hand-maintained enumeration of the
    /// whole catalog and is edited by whoever adds a command anywhere; five
    /// concurrent tracks were writing in this tree on the day these landed, and
    /// a shared list is the file two of them collide in. The rules it asserts
    /// are asserted here over exactly the strings this module owns, which is
    /// weaker in one way — it cannot see a label these five share with a
    /// command in another file — and stronger in another: it fails in the file
    /// whose author can fix it.
    ///
    /// ⚠ Recorded rather than silently chosen, because the whole-catalog check
    /// is the better one and should get these five when the tree is quiet.
    #[test]
    fn the_five_markup_style_strings_hold_the_catalog_conventions() {
        let all = [
            format_colour(),
            format_fill(),
            format_line_width(),
            format_opacity(),
            format_arrowheads(),
        ];
        for t in all {
            assert!(!t.label.trim().is_empty(), "empty label: {t:?}");
            assert!(!t.tooltip.trim().is_empty(), "empty tooltip: {t:?}");
            assert!(
                t.tooltip.ends_with('.'),
                "a tooltip is prose and ends in a full stop: {:?}",
                t.tooltip
            );
            assert!(
                !t.label.ends_with('.'),
                "a label is a name and takes no trailing period: {:?}",
                t.label
            );
        }
        let mut labels: Vec<&str> = all.iter().map(|t| t.label).collect();
        let total = labels.len();
        labels.sort_unstable();
        labels.dedup();
        assert_eq!(labels.len(), total, "two of these five share a label");
    }

    /// ★ **None of the five reuses a label the Font group already carries.**
    ///
    /// The collision that nearly happened: `RIBBON_IA.md` §5.8 calls this
    /// group's first row *Colour*, and `format.font_colour` is already called
    /// "Colour" — on the **same tab**, one group over. Two adjacent controls
    /// with one label is the defect `no_two_commands_share_a_label` was written
    /// for after `edit_text_tool_button` and `add_text_tool_button` both
    /// returned `"Aa"`.
    ///
    /// Only the Font group is checked and that is deliberate: it is the one
    /// that shares a tab with this one, so it is the one where a duplicate is
    /// on screen at the same moment.
    #[test]
    fn no_markup_style_label_collides_with_the_font_group_on_the_same_tab() {
        let font = [
            super::super::format_font().label,
            super::super::format_font_size().label,
            super::super::format_font_colour().label,
            super::super::format_bold().label,
            super::super::format_italic().label,
        ];
        for t in [
            format_colour(),
            format_fill(),
            format_line_width(),
            format_opacity(),
            format_arrowheads(),
        ] {
            assert!(
                !font.contains(&t.label),
                "`{}` is already a Format ▸ Font label, and the two groups are on one tab",
                t.label
            );
        }
    }
}
