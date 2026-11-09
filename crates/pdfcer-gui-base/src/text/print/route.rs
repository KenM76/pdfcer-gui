//! # `text::print::route` — the words for how pages travel to the printer

/// The switch on the Pages tab.
#[must_use]
pub const fn as_image() -> &'static str {
    "Print as image"
}

/// Hover on the switch.
#[must_use]
pub const fn as_image_hover() -> &'static str {
    "pdfcer sends text and lines to the printer as vector graphics, so they \
     print sharp at the printer's own resolution. Tick this to send each page \
     as a picture at the resolution above instead: slower and larger, for a \
     printer that draws a page wrongly. A page with transparency or shading \
     prints as a picture either way."
}

/// Why the switch makes no difference to this job: grey or black only.
#[must_use]
pub const fn forced_colour() -> &'static str {
    "Grey and black-only printing send every page as an image."
}

/// Why the switch makes no difference to this job: n-up or booklet.
#[must_use]
pub const fn forced_impose() -> &'static str {
    "N-up and booklet sheets are sent as images."
}

/// Why the switch makes no difference to some sheets: a poster.
#[must_use]
pub const fn forced_poster() -> &'static str {
    "Poster sheets are sent as images."
}

/// After a job some of whose pages could not go as vector graphics.
#[must_use]
pub fn fell_back(images: usize, pages: usize, reasons: &[String]) -> String {
    let why = reasons.join(", ");
    if images == pages {
        format!("Every page was sent as an image, because of {why}.")
    } else if images == 1 {
        format!("1 of {pages} pages was sent as an image, because of {why}.")
    } else {
        format!("{images} of {pages} pages were sent as images, because of {why}.")
    }
}
