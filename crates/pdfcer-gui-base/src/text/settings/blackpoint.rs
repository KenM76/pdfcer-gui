//! # `text::settings::blackpoint` — black point compensation in Settings ▸ Colour

/// Title for the black point compensation setting.
pub const fn bpc_title() -> &'static str {
    "Dark colours in a picture that do not match the colours around it"
}

/// What the standard leaves open here.
pub const fn bpc_silence() -> &'static str {
    "When a colour is converted between colour profiles, the darkest black of one may be darker than the other can show. Black point compensation scales the dark range to fit, so a picture and the drawing beside it land on the same colours. A file may switch it on or off for its own content; where it does not, the standard leaves the choice to the reader."
}

/// The radius line.
pub const fn bpc_radius() -> &'static str {
    "Affects what you see and what you print. Does not change the file."
}

/// Label for the switch.
pub const fn bpc_label() -> &'static str {
    "Compensate the black point (pdfcer's default)"
}

/// Note for the switch.
pub const fn bpc_note() -> &'static str {
    "Off, dark colours converted between profiles can come out a few levels lighter or darker than matching ink drawn beside them. A file's own choice always wins."
}

/// Title for the calibrated-CMYK setting.
pub const fn calibrated_cmyk_title() -> &'static str {
    "CMYK colours tied to a colour profile, on a page drawn in ink"
}

/// What the standard leaves open here.
pub const fn calibrated_cmyk_silence() -> &'static str {
    "A CMYK colour can carry its own colour profile. When the page is drawn in ink, converting it through the page's output profile can change its ink numbers; many print shops keep them as written instead. The standard allows either."
}

/// The radius line.
pub const fn calibrated_cmyk_radius() -> &'static str {
    "Affects what you see and what you print on pages drawn in ink. Does not change the file."
}

/// Label for the switch.
pub const fn calibrated_cmyk_label() -> &'static str {
    "Keep the CMYK numbers as written"
}

/// Note for the switch.
pub const fn calibrated_cmyk_note() -> &'static str {
    "On, the colour is used as the ink it names. Off (pdfcer's default), it is converted through the output profile, so it matches its colour rather than its numbers."
}
