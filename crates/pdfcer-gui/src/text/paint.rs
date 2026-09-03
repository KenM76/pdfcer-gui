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
