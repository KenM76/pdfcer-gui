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
     printer that draws a page wrongly."
}

/// The switch keeping gradients vector.
#[must_use]
pub const fn vector_gradients() -> &'static str {
    "Gradients as vector graphics"
}

/// Hover on it.
#[must_use]
pub const fn vector_gradients_hover() -> &'static str {
    "Send smooth colour gradients as fine vector triangles, which print sharp \
     and match the screen to within a shade. Untick it if this printer draws \
     gradients wrongly: a page with a gradient is then sent as a picture."
}

/// The switch keeping transparency vector.
#[must_use]
pub const fn vector_transparency() -> &'static str {
    "Transparency as vector graphics"
}

/// Hover on it.
#[must_use]
pub const fn vector_transparency_hover() -> &'static str {
    "Send see-through, blended and overprinted artwork as vector graphics, \
     already combined with what lies under it. Where it sits over a photo, a \
     gradient or a soft edge, that small area goes as a picture inside the \
     page. Untick it to send any page with transparency as a picture."
}

/// The switch shrinking those picture areas.
#[must_use]
pub const fn tight_patches() -> &'static str {
    "Smaller picture areas"
}

/// Hover on it.
#[must_use]
pub const fn tight_patches_hover() -> &'static str {
    "Cut those picture areas down to where a soft edge actually shows. Text \
     taken out of a picture area is drawn by the printer driver, which does \
     not smooth it, so at the resolution above it can look rougher."
}

/// After a job whose vector pages held picture areas.
#[must_use]
pub fn patched(areas: usize) -> String {
    if areas == 1 {
        "1 area that could not be drawn as vector graphics was sent as a picture \
         inside its page."
            .to_owned()
    } else {
        format!(
            "{areas} areas that could not be drawn as vector graphics were sent as \
             pictures inside their pages."
        )
    }
}

/// After a job some of whose pages hold a colour profile that could not be
/// used.
#[must_use]
pub fn profiles_unused(pages: usize) -> String {
    if pages == 1 {
        "On 1 page a colour profile could not be used, so its colours printed \
         from the fallback colours the document names."
            .to_owned()
    } else {
        format!(
            "On {pages} pages a colour profile could not be used, so their colours \
             printed from the fallback colours the document names."
        )
    }
}

/// Why the switch makes no difference to this job: separations.
#[must_use]
pub const fn forced_separations() -> &'static str {
    "Separations send every page as an image."
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
