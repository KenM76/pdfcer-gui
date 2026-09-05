//! # `text::paint` — the words the object-colour control uses
//!
//! `OPERATOR_REQUESTS.md` O89's vector half. A few strings, and one of them is
//! the reason the section exists at all: the sentence drawn **instead of** a
//! swatch when pdfcer cannot decode the ink.
//!
//! ## ★★★ The refusal is the important string here
//!
//! A colour control with no current value is a control that silently discards
//! what was there the moment it is touched. Over a `/Separation` stroke that
//! means one click converts a named spot ink to screen colour — permanently,
//! invisibly, and looking entirely normal while it happens. So the swatch is
//! **absent** and this sentence stands where it would have been.
//!
//! ★ It names the ink where the file names it. *"This stroke is PANTONE 300"*
//! tells a drawing office what it needs; *"pdfcer cannot show this colour"* tells
//! them only that something is wrong.

/// The section heading.
#[must_use]
pub fn heading() -> String {
    "Colour".to_owned()
}

/// The fill channel.
///
/// ★ "Fill" and "Line", not "fill" and "stroke". *Stroke* is the PDF word and
/// the drawing-office word is *line* — the same vocabulary rule
/// `text::formfield`'s header states, applied one panel along.
#[must_use]
pub fn fill_label() -> String {
    "Fill".to_owned()
}

/// The stroke channel.
#[must_use]
pub fn stroke_label() -> String {
    "Line".to_owned()
}

/// ★★★ Drawn where a swatch cannot honestly go.
///
/// Two forms, because a named ink and an unnamed undecodable space are
/// different amounts of help. Neither offers to change anything.
#[must_use]
pub fn undecoded(ink: Option<String>) -> String {
    match ink {
        Some(name) => format!(
            "{name} — a named ink. pdfcer will not overwrite it with a screen colour, because that \
             would look right here and change what prints."
        ),
        None => "Set in a colour space pdfcer does not convert, so it is left exactly as it is."
            .to_owned(),
    }
}

/// The status line after a recolour that changed everything asked.
#[must_use]
pub fn recoloured(changed: usize) -> String {
    format!("Recoloured {changed} object(s).")
}

/// ★★ The status line when some objects were refused.
///
/// The operator asked for exactly this shape: *"a selection of twelve strokes
/// where three are in a colour space pdfcer will not rewrite needs to say 'nine
/// changed', not 'done'."*
#[must_use]
pub fn recoloured_partly(changed: usize, refused: usize) -> String {
    format!(
        "Recoloured {changed} object(s). {refused} were left alone — they are painted in inks \
         pdfcer will not overwrite with a screen colour."
    )
}

// ===========================================================================
// ★★★ MORE THAN ONE OBJECT — O89 piece 2, 2026-09-05
//
// The three strings below exist because a multi-object colour control owes
// three disclosures a single-object one does not, and every one of them has to
// arrive BEFORE the press rather than in the status line after it. That is this
// project's standing ordering rule — *"a caveat below a list arrives after the
// operator has already drawn a conclusion"* — applied to a control instead of
// to a list.
// ===========================================================================

/// **What the two swatches are about to act on.**
///
/// ★★ The count is the whole safety of a multi-object colour control. A marquee
/// on a CAD sheet routinely takes hundreds of objects, and *"Fill"* over a
/// swatch says nothing about how many things pressing it changes. Word,
/// Illustrator and Inkscape all report the selection size somewhere permanent;
/// this panel has no status strip of its own, so the row says it.
///
/// ★ `not_paths` is reported separately rather than folded into the count,
/// because they are two different facts and only one is about what will change.
/// A marquee over a table catches its rules **and** its labels, and an operator
/// who recolours it needs to know the labels were not included — otherwise the
/// text staying black reads as the control half-working.
#[must_use]
pub fn subject(paths: usize, not_paths: usize) -> String {
    if not_paths == 0 {
        format!("{paths} shape(s) selected.")
    } else {
        format!(
            "{paths} shape(s) selected. {not_paths} other object(s) are also selected and are not shapes with a fill or a line — text and pictures are left alone here."
        )
    }
}

/// ★★★ **Named inks inside a selection that will still be recoloured** — drawn
/// above a live swatch, never instead of one.
///
/// The state O89 called out as the hard one: *"A mixed selection containing one
/// spot ink must not let a screen colour flatten it."* It does not.
/// `EditSession::set_object_paint` refuses each named-ink member by name and
/// reports it, so the plate is safe whatever this panel draws; what this
/// sentence adds is that the operator **knows before pressing**, rather than
/// finding out from a count afterwards.
///
/// ★ It names the inks where the file names them, for
/// [`undecoded`]'s reason: *"this stroke is spot ink PANTONE 300"* tells a
/// drawing office what it needs. An unnamed undecodable space contributes to
/// the count and not to the list, because there is nothing truthful to call it.
///
/// ★★ The list is capped at three names. A selection of two hundred strokes in
/// nine separations would otherwise put a paragraph on a 180-point panel and
/// the sentence would be scrolled past — which is the failure mode
/// `text::panels::fonts`' two-word verdicts were shortened to avoid.
#[must_use]
pub fn mixed_named_inks(inks: &[Option<String>], total: usize) -> String {
    const SHOWN: usize = 3;
    let named: Vec<&str> = inks
        .iter()
        .filter_map(|i| i.as_deref())
        .take(SHOWN)
        .collect();
    let count = inks.len();
    if named.is_empty() {
        return format!(
            "{count} of these {total} are painted in a colour space pdfcer does not convert, and will be left exactly as they are."
        );
    }
    let list = named.join(", ");
    let more = inks
        .iter()
        .filter(|i| i.is_some())
        .count()
        .saturating_sub(named.len());
    if more == 0 {
        format!(
            "{count} of these {total} are painted in named inks ({list}) and will be left exactly as they are — pdfcer will not overwrite an ink with a screen colour, because that would look right here and change what prints."
        )
    } else {
        format!(
            "{count} of these {total} are painted in named inks ({list} and {more} more) and will be left exactly as they are — pdfcer will not overwrite an ink with a screen colour, because that would look right here and change what prints."
        )
    }
}

/// ★★★ Drawn where a swatch cannot go because **every** member of the selection
/// carries an ink pdfcer will not overwrite.
///
/// The single-object refusal, widened to say how many. It is
/// [`undecoded`]'s sentence for `total == 1` — deliberately word for word, so
/// an operator who selects one spot-inked line and then selects five reads the
/// same explanation rather than wondering whether the second one is a different
/// state.
#[must_use]
pub fn undecoded_across(ink: Option<String>, total: usize) -> String {
    if total <= 1 {
        return undecoded(ink);
    }
    match ink {
        Some(name) => format!(
            "All {total} are painted in named inks ({name} among them). pdfcer will not overwrite an ink with a screen colour, because that would look right here and change what prints."
        ),
        None => format!(
            "All {total} are set in colour spaces pdfcer does not convert, so they are left exactly as they are."
        ),
    }
}

/// ★★ Drawn at the top of the colour picker when the selected **shapes**
/// disagree.
///
/// Not an error and not a refusal: the control still applies. This is the
/// indeterminate state every editor in the class shows, and the sentence says
/// what it means — there is no one colour to open on, and picking one sets all
/// of them.
///
/// ★★★ It names **shapes**, where the clicked-text twin
/// (`crate::text::panels::textobject::mixed_hint`) names **words**, and the two
/// exist separately for a defect that shipped for twenty minutes and could not
/// have been caught by a test: `panels::properties::swatch` is shared by both
/// rows, its first draft reached for the text sentence inline, and a selection
/// of paths was told *"These words are not all one colour."* Both strings
/// compile, both render, and the wrong one is grammatical. The widget takes the
/// sentence as a parameter now; see its `show`'s doc comment.
#[must_use]
pub fn mixed_hint() -> String {
    "These shapes are not all one colour. Picking one sets all of them to it.".to_owned()
}
