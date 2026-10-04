//! The words for the offer to make a refused text edit another way, in the
//! Properties panel.

use pdfcer_core::text_edit::Workaround;

/// The offer's heading.
#[must_use]
pub const fn heading() -> &'static str {
    "This edit can be made another way"
}

/// What the workaround does, in the operator's terms, and whether the result
/// matches the original exactly.
#[must_use]
pub const fn offer(workaround: Workaround) -> &'static str {
    match workaround {
        Workaround::JoinTextObjects => {
            "The text is drawn in separate pieces. pdfcer can join the pieces into one and \
             make the edit exactly."
        }
        Workaround::RewriteQuoteOperator => {
            "This line is drawn with an older kind of text instruction. pdfcer can rewrite it \
             as the plain instruction it stands for and make the edit exactly."
        }
        Workaround::Retype => {
            "pdfcer can remove this text and set your words in its place, at the same \
             position, size and colour, in the same font or the nearest one that has the \
             letters. The spacing between letters may differ a little from the original."
        }
        _ => "pdfcer can make this edit another way, and the status line will say how.",
    }
}

/// What the operator typed, so he can see which edit the offer is for.
#[must_use]
pub fn typed(replacement: &str) -> String {
    format!("Your text: \u{201c}{replacement}\u{201d}")
}

/// The button that makes the edit the other way.
#[must_use]
pub const fn apply_button() -> &'static str {
    "Make the edit this way"
}

/// The button's hover text.
#[must_use]
pub const fn apply_hover() -> &'static str {
    "Makes the edit you typed using the way described above. Undo takes it back."
}

/// The button that types a refused letter in a face from the font folders.
#[must_use]
pub fn installed_letter_button(character: char) -> String {
    format!("Type \u{201c}{character}\u{201d} in an installed font")
}

/// The button's hover text.
#[must_use]
pub const fn installed_letter_hover() -> &'static str {
    "pdfcer picks a font from your font folders that has this letter, embeds the letters \
     it needs, and makes the edit. The status line names the font. Undo takes it back."
}

/// What the press changes, above the button.
#[must_use]
pub const fn installed_letter_note() -> &'static str {
    "Or keep this font and set only the letters it lacks in an installed font."
}

/// The other way was tried and could not be applied; `why` is the engine's
/// reason.
#[must_use]
pub fn failed(why: &str) -> String {
    format!("pdfcer tried the other way and it could not be done either: {why}")
}
