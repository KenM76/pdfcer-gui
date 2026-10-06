//! # `text::handsign` — what the *Sign here* window says
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/handsign.md`.

/// The window's title bar.
#[must_use]
pub const fn window_title() -> &'static str {
    "Sign here"
}

/// The opening instruction.
#[must_use]
pub const fn intro() -> &'static str {
    "Draw your signature in the box below: hold the left mouse button down and write."
}

/// Shown faintly inside the empty drawing area.
#[must_use]
pub const fn pad_hint() -> &'static str {
    "Draw your signature here"
}

/// What the result is, and what it is not.
#[must_use]
pub const fn what_it_is() -> &'static str {
    "Your signature goes onto the page like ink on paper. It is not a digital (certificate) \
     signature."
}

/// Wipes the drawing area.
#[must_use]
pub const fn clear() -> &'static str {
    "Clear"
}

/// Takes back the last stroke drawn.
#[must_use]
pub const fn undo_stroke() -> &'static str {
    "Undo last stroke"
}

/// Puts the remembered or last-placed signature back in the drawing area.
#[must_use]
pub const fn use_last() -> &'static str {
    "Use my last signature"
}

/// The keep-a-copy option.
#[must_use]
pub const fn remember() -> &'static str {
    "Remember my signature on this computer"
}

/// Hover on [`remember`].
#[must_use]
pub const fn remember_hover() -> &'static str {
    "Keeps a copy beside pdfcer, never inside the document, so you can place it again next \
     time. Untick it and place a signature to delete the copy."
}

/// The commit button.
#[must_use]
pub const fn place() -> &'static str {
    "Place signature"
}

/// Hover on [`place`] while nothing usable is drawn.
#[must_use]
pub const fn place_needs_drawing() -> &'static str {
    "Draw your signature first."
}

/// Closes the window without signing.
#[must_use]
pub const fn cancel() -> &'static str {
    "Cancel"
}

/// The certificate route, offered only in builds that can sign with one.
#[must_use]
pub const fn digital_id() -> &'static str {
    "Use a digital ID (certificate) instead…"
}

/// Hover on [`digital_id`].
#[must_use]
pub const fn digital_id_hover() -> &'static str {
    "A certificate signature shows any later change, and names the signer as far as the \
     certificate is trusted. Use a digital ID file (.pfx or .p12), or create one in the Sign \
     window."
}

/// The tab for drawing a signature with the mouse.
#[must_use]
pub const fn tab_draw() -> &'static str {
    "Draw"
}

/// The tab for typing a name in a handwriting style.
#[must_use]
pub const fn tab_type() -> &'static str {
    "Type"
}

/// The Type tab's instruction.
#[must_use]
pub const fn type_intro() -> &'static str {
    "Type your name. It is written in a handwriting style."
}

/// Shown faintly inside the empty name field.
#[must_use]
pub const fn name_hint() -> &'static str {
    "Your name"
}

/// The handwriting-style choice.
#[must_use]
pub const fn style() -> &'static str {
    "Style"
}

/// Shown faintly in the preview before a name is typed.
#[must_use]
pub const fn preview_hint() -> &'static str {
    "Your signature appears here"
}

/// Hover on the place button while the name is blank.
#[must_use]
pub const fn place_needs_name() -> &'static str {
    "Type your name first."
}

/// The chosen style cannot write the typed name; `detail` is the reason.
#[must_use]
pub fn style_cannot_write(detail: &str) -> String {
    format!("This style cannot write that name ({detail}). Try another style.")
}

/// Hover on the greyed Type tab when the page is shown turned.
#[must_use]
pub const fn type_needs_upright_page() -> &'static str {
    "A typed signature can only be placed on a page shown upright. Turn the page back, or \
     draw your signature instead."
}

/// The signing strip's count while boxes remain.
#[must_use]
pub fn signed_of(signed: usize, total: usize) -> String {
    format!("Signed {signed} of {total}")
}

/// The signing strip's line once every box is signed.
#[must_use]
pub fn all_signed(total: usize) -> String {
    if total == 1 {
        "The signature box is signed".to_owned()
    } else {
        format!("All {total} signature boxes are signed")
    }
}

/// Hover on the signing strip's count.
#[must_use]
pub const fn strip_hint() -> &'static str {
    "Signature boxes in this document with no digital signature. Click a box on the page \
     to sign it."
}

/// The signing strip's button.
#[must_use]
pub const fn next_box() -> &'static str {
    "Next box to sign"
}

/// Hover on the signing strip's button.
#[must_use]
pub const fn next_box_hint() -> &'static str {
    "Scroll to the next signature box still to sign. Click the box to sign it."
}

/// The tab for using a picture of a signature.
#[must_use]
pub const fn tab_picture() -> &'static str {
    "Picture"
}

/// The Picture tab's instruction.
#[must_use]
pub const fn picture_intro() -> &'static str {
    "Use a picture of your signature: a scan or photo saved as PNG, JPEG, BMP or TIFF."
}

/// Opens the file picker for a signature picture.
#[must_use]
pub const fn choose_picture() -> &'static str {
    "Choose picture…"
}

/// Shown faintly in the preview before a picture is chosen.
#[must_use]
pub const fn picture_hint() -> &'static str {
    "No picture chosen"
}

/// The chosen file could not be read as a picture; `detail` is the reason.
#[must_use]
pub fn picture_unreadable(detail: &str) -> String {
    format!("That file could not be read as a picture ({detail}). Choose another.")
}

/// The make-the-paper-clear option.
#[must_use]
pub const fn clear_white() -> &'static str {
    "Make white see-through"
}

/// Hover on [`clear_white`].
#[must_use]
pub const fn clear_white_hover() -> &'static str {
    "Lets the page show through the white paper around your signature, so it does not \
     cover lines or text in the box."
}

/// Hover on [`clear_white`] when the picture cannot have it.
#[must_use]
pub const fn clear_white_unavailable() -> &'static str {
    "This picture already has see-through parts, or is in CMYK or indexed colour, so white \
     is left as it is."
}

/// Hover on the greyed Picture tab when the page is shown turned.
#[must_use]
pub const fn picture_needs_upright_page() -> &'static str {
    "A picture signature can only be placed on a page shown upright. Turn the page back, or \
     draw your signature instead."
}

/// Hover on the place button while no picture is chosen.
#[must_use]
pub const fn place_needs_picture() -> &'static str {
    "Choose a picture of your signature first."
}

/// The placement preview's instruction; `stretchable` is whether Shift
/// changes the proportions.
#[must_use]
pub const fn placement_hint(stretchable: bool) -> &'static str {
    if stretchable {
        "Where it lands in the box: drag it to move it, drag a corner to resize it. Hold \
         Shift to change its proportions."
    } else {
        "Where it lands in the box: drag it to move it, drag a corner to resize it."
    }
}

/// Puts the signature back where it lands by default.
#[must_use]
pub const fn reset_fit() -> &'static str {
    "Reset to fit"
}

/// Hover on [`reset_fit`].
#[must_use]
pub const fn reset_fit_hover() -> &'static str {
    "Put the signature back where it lands by default: as large as the box allows, at its \
     left."
}
