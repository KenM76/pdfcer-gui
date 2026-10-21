//! # `text::compact_images` — what the compact window says about pictures
//!
//! Every word of the Pictures fold in Save a compacted copy, and of its
//! report. The fold starts off: every picture change it offers is visible on
//! the page, so it is asked for, never assumed.

use pdfcer_core::optimize::{ImageConvert, QUALITY_LEVELS};

/// The fold's heading, saying when pictures will be changed.
#[must_use]
pub fn heading(on: bool) -> &'static str {
    if on {
        "Make pictures smaller (on)"
    } else {
        "Make pictures smaller"
    }
}

/// The switch inside the fold.
#[must_use]
pub const fn toggle() -> &'static str {
    "Change the pictures in the copy"
}

/// Hover on the switch.
#[must_use]
pub const fn toggle_tooltip() -> &'static str {
    "Lowers the resolution of pictures, stores them more compactly or with fewer colours. \
     Unlike everything else in this window, this changes how the pictures look. Your open \
     document is not changed."
}

/// The resolution row's label.
#[must_use]
pub const fn resolution_label() -> &'static str {
    "Resolution"
}

/// Hover on the resolution row.
#[must_use]
pub const fn resolution_tooltip() -> &'static str {
    "Pictures sharper than this, at the largest size they are drawn on any page, are \
     scaled down to it. Pictures already at or below it are left their size."
}

/// One resolution choice; `None` keeps every picture's size.
#[must_use]
pub fn resolution(ppi: Option<u16>) -> String {
    ppi.map_or_else(
        || "Keep their size".to_owned(),
        |ppi| format!("{ppi} pixels per inch"),
    )
}

/// The format row's label.
#[must_use]
pub const fn format_label() -> &'static str {
    "Store as"
}

/// The choice that keeps each picture's stored format.
#[must_use]
pub const fn format_keep() -> &'static str {
    "Keep how each is stored"
}

/// The lossless choice.
#[must_use]
pub const fn format_lossless() -> &'static str {
    "Lossless (exact)"
}

/// The JPEG choice.
#[must_use]
pub const fn format_jpeg() -> &'static str {
    "JPEG (smaller, loses detail)"
}

/// Hover on the JPEG choice while a colour choice rules it out.
#[must_use]
pub const fn jpeg_ruled_out() -> &'static str {
    "JPEG cannot store black-and-white or reduced-colour pictures. Choose another colour \
     setting to use JPEG."
}

/// The quality row's label.
#[must_use]
pub const fn quality_label() -> &'static str {
    "JPEG quality"
}

/// Hover on the quality row.
#[must_use]
pub const fn quality_tooltip() -> &'static str {
    "Lower quality makes a smaller file and shows more blur and blockiness around edges and \
     text in the pictures."
}

/// The name of one of the engine's named JPEG qualities.
#[must_use]
pub fn quality(value: u8) -> String {
    let name = match QUALITY_LEVELS.iter().position(|&(_, q)| q == value) {
        Some(0) => "Minimum",
        Some(1) => "Low",
        Some(2) => "Medium",
        Some(3) => "High",
        Some(4) => "Maximum",
        _ => return format!("{value}"),
    };
    format!("{name} ({value})")
}

/// The colour row's label.
#[must_use]
pub const fn colour_label() -> &'static str {
    "Colour"
}

/// One colour choice.
#[must_use]
pub fn colour(convert: ImageConvert) -> &'static str {
    match convert {
        ImageConvert::Gray => "Greyscale",
        ImageConvert::Bilevel => "Black and white",
        ImageConvert::Indexed => "Fewer colours",
        _ => "Keep their colours",
    }
}

/// Hover on a colour choice JPEG rules out.
#[must_use]
pub const fn colour_ruled_out() -> &'static str {
    "JPEG cannot store this. Choose Lossless or Keep how each is stored to use it."
}

/// The palette-size row's label.
#[must_use]
pub const fn colors_label() -> &'static str {
    "At most"
}

/// One palette size.
#[must_use]
pub fn colors(n: u16) -> String {
    format!("{n} colours")
}

/// The full-size line said when the copy's pictures lose detail.
#[must_use]
pub const fn lossy_line() -> &'static str {
    "The pictures in the copy will look different from the original: they lose detail that \
     cannot be got back. Your open document and its file are not changed."
}

/// What the picture changes came to.
#[must_use]
pub fn done(resized: usize, jpeg: usize, converted: usize) -> Vec<String> {
    let mut lines = Vec::new();
    if resized > 0 {
        lines.push(format!("{resized} picture(s) were scaled down."));
    }
    if jpeg > 0 {
        lines.push(format!("{jpeg} picture(s) were stored as JPEG."));
    }
    if converted > 0 {
        lines.push(format!("{converted} picture(s) had their colours changed."));
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every engine quality level has a name, and an unnamed value is said
    /// as a number rather than misnamed.
    #[test]
    fn every_named_quality_is_named_and_others_are_numbers() {
        for (_, q) in QUALITY_LEVELS {
            assert!(quality(q).contains('('), "{q}");
        }
        assert_eq!(quality(77), "77");
    }

    /// Nothing is reported for a change that did not happen.
    #[test]
    fn the_report_names_only_what_happened() {
        assert!(done(0, 0, 0).is_empty());
        assert_eq!(done(2, 0, 1).len(), 2);
    }
}
