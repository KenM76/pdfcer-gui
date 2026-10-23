//! # `text::compact_images` — what the compact window says about pictures
//!
//! Every word of the Pictures fold in Save a compacted copy, and of its
//! report. The fold starts off: every picture change it offers is visible on
//! the page, so it is asked for, never assumed.

use pdfcer_core::optimize::{ImageConvert, ImageSkip, ImageStep, QUALITY_LEVELS};

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

/// One "Store as" choice, as the wording names it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatWord {
    /// Keep each picture's stored format.
    Keep,
    /// Flate.
    Lossless,
    /// The smallest exact storage, tried per picture.
    Smallest,
    /// JPEG.
    Jpeg,
    /// LZW.
    Lzw,
    /// Run length.
    RunLength,
    /// CCITT Group 4.
    Ccitt,
    /// JBIG2.
    Jbig2,
}

/// A choice's name in the list.
#[must_use]
pub const fn format(word: FormatWord) -> &'static str {
    match word {
        FormatWord::Keep => "Keep how each is stored",
        FormatWord::Lossless => "Lossless (exact)",
        FormatWord::Smallest => "Smallest exact (tries each)",
        FormatWord::Jpeg => "JPEG (smaller, loses detail)",
        FormatWord::Lzw => "LZW (exact)",
        FormatWord::RunLength => "Run length (exact)",
        FormatWord::Ccitt => "CCITT Group 4 (exact, black and white)",
        FormatWord::Jbig2 => "JBIG2 (exact, black and white)",
    }
}

/// Hover on a choice, saying what it is for.
#[must_use]
pub const fn format_tooltip(word: FormatWord) -> &'static str {
    match word {
        FormatWord::Keep => "Each picture stays in the kind of storage it has now.",
        FormatWord::Lossless => "Exact; the usual choice when nothing may change.",
        FormatWord::Smallest => {
            "Exact. Each picture is stored every exact way that suits it (Lossless, run \
             length, and for black-and-white pictures CCITT Group 4 and JBIG2), and the \
             smallest is kept. A JPEG picture stays JPEG, and nothing raises the PDF \
             version."
        }
        FormatWord::Jpeg => "Much smaller for photographs; detail around edges is lost.",
        FormatWord::Lzw => {
            "Exact, usually larger than Lossless; for programs that expect LZW. A PDF/A \
             document's pictures are left as they are, since PDF/A forbids it."
        }
        FormatWord::RunLength => "Exact; small only for pictures with large areas of one colour.",
        FormatWord::Ccitt => {
            "Exact, and the smallest storage for black-and-white pictures such as \
             scanned drawings. Other pictures are left as they are; choose Black and white \
             under Colours to convert them first."
        }
        FormatWord::Jbig2 => {
            "Exact, and often smaller than CCITT Group 4 for black-and-white scans. Other \
             pictures are left as they are. A document older than PDF 1.4 is raised to \
             1.4, which JBIG2 needs, and the report says so."
        }
    }
}

/// Hover on a choice the colour setting rules out.
#[must_use]
pub const fn format_ruled_out(word: FormatWord) -> &'static str {
    match word {
        FormatWord::Ccitt | FormatWord::Jbig2 => {
            "This stores black-and-white pictures only. Choose Black and white or Keep \
             their colours to use it."
        }
        _ => {
            "JPEG cannot store black-and-white or reduced-colour pictures. Choose another \
             colour setting to use JPEG."
        }
    }
}

/// Hover on JPEG among the black-and-white settings.
#[must_use]
pub const fn format_one_bit_ruled_out() -> &'static str {
    "JPEG cannot store black-and-white pictures; they would be left as they are."
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

/// Hover on a colour choice the chosen storage rules out.
#[must_use]
pub const fn colour_ruled_out(word: FormatWord) -> &'static str {
    match word {
        FormatWord::Ccitt | FormatWord::Jbig2 => {
            "The chosen storage keeps black-and-white pictures only. Choose another \
             storage to use this."
        }
        _ => "JPEG cannot store this. Choose Lossless or Keep how each is stored to use it.",
    }
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

/// Report lines for pictures written by a closeness search: `matched` reached
/// the target, `short` did not and were stored at the highest quality allowed.
#[must_use]
pub fn closeness(matched: usize, short: usize) -> Vec<String> {
    let mut lines = Vec::new();
    if matched > 0 {
        lines.push(format!(
            "{matched} picture(s) were stored at the lowest quality that matches the \
             original as closely as asked."
        ));
    }
    if short > 0 {
        lines.push(format!(
            "{short} picture(s) could not match the original as closely as asked within the \
             highest JPEG quality allowed, and were stored at that quality."
        ));
    }
    lines
}

/// The closeness row's label.
#[must_use]
pub const fn target_label() -> &'static str {
    "Match the original"
}

/// Hover on the closeness row.
#[must_use]
pub const fn target_tooltip() -> &'static str {
    "Stores each picture at the lowest JPEG quality that still looks this close to the \
     original, measured by structural similarity (SSIM), and never above the JPEG quality \
     chosen above. Busy photographs keep more quality and plain ones shrink further. \
     Each picture is encoded several times to find its quality, so this takes longer."
}

/// A closeness choice; `None` is off, otherwise ten-thousandths of SSIM.
#[must_use]
pub fn target(value: Option<u16>) -> String {
    let name = match value {
        None => return "Off: every picture at the quality above".to_owned(),
        Some(9900..) => "Indistinguishable",
        Some(9800..) => "Very close",
        Some(_) => "Close",
    };
    let score = value.map_or(0.0, |n| f64::from(n) / 10_000.0);
    format!("{name} ({score:.2})")
}

/// Report lines for pictures stored as CCITT Group 4, JBIG2, LZW and run length.
#[must_use]
pub fn stored_exact(counts: [usize; 4]) -> Vec<String> {
    ["CCITT Group 4", "JBIG2", "LZW", "run length"]
        .into_iter()
        .zip(counts)
        .filter(|(_, n)| *n > 0)
        .map(|(name, n)| format!("{n} picture(s) were stored as {name}."))
        .collect()
}

/// Format ▸ Make pictures smaller: the window's title.
#[must_use]
pub const fn shrink_title() -> &'static str {
    "Make pictures smaller"
}

/// The window's opening line, naming how many pictures it changes.
#[must_use]
pub fn shrink_intro(pictures: usize) -> String {
    if pictures == 1 {
        "Change the selected picture:".to_owned()
    } else {
        format!("Change the {pictures} selected pictures:")
    }
}

/// Said before Apply: a picture is one stored object, however often drawn.
#[must_use]
pub const fn shrink_everywhere() -> &'static str {
    "A picture drawn in more than one place changes everywhere it is drawn. Undo puts it back."
}

/// The button that applies.
#[must_use]
pub const fn shrink_button() -> &'static str {
    "Apply"
}

/// The button that closes without changing anything.
#[must_use]
pub const fn shrink_cancel() -> &'static str {
    "Cancel"
}

/// Reported after a change the engine calls lossy.
#[must_use]
pub const fn shrink_lossy() -> &'static str {
    "The pictures lost detail. Undo puts them back."
}

/// Reported when the storage chosen raised the document's PDF version.
#[must_use]
pub fn shrink_raised(version: &str) -> String {
    format!("The document now declares PDF {version}, which the storage chosen needs.")
}

/// Reported when the plan changed no picture.
#[must_use]
pub const fn shrink_nothing() -> &'static str {
    "No picture was changed."
}

/// Pictures left as stored, for one reason.
#[must_use]
pub fn shrink_left(reason: &str, count: usize) -> String {
    format!("{count} picture(s) left as they were: {reason}.")
}

/// Why a picture was left as stored, in the operator's words. Carries no
/// per-picture numbers, so equal reasons group into one report line.
#[must_use]
pub fn picture_left_because(why: &ImageSkip) -> String {
    let s = match why {
        ImageSkip::MatteMask | ImageSkip::MatteParent | ImageSkip::MatteMaskFailed { .. } => {
            "their see-through edge is tied to the picture's size"
        }
        ImageSkip::ColourKey(ImageStep::Resample) => {
            "they hide one exact colour, which scaling would blur"
        }
        ImageSkip::ColourKey(_) => "they hide one exact colour, which re-encoding would shift",
        ImageSkip::SoftMaskConvert | ImageSkip::MatteParentConvert => {
            "their transparency keeps them in the colours they have"
        }
        ImageSkip::OneBitAsJpeg | ImageSkip::PaletteAsJpeg | ImageSkip::ReducedAsJpeg => {
            "they are black-and-white or few-colour pictures, which JPEG would spoil"
        }
        ImageSkip::JpegComponents => "their colours cannot be stored as JPEG",
        ImageSkip::AlreadyJpeg { .. } => "they are already JPEG at that quality or lower",
        ImageSkip::NoDefinedGrey { .. } => "their spot colours have no grey to convert to",
        ImageSkip::SizeChanged => "they changed while being planned; try again",
        ImageSkip::NotBilevel(_) | ImageSkip::NotBilevelOverride(_) => {
            "they are not black and white, the only pictures that storage keeps"
        }
        ImageSkip::LzwUnderPdfa => "the document is PDF/A, which forbids LZW",
        ImageSkip::AutoNotSmaller { .. } => "no exact storage made them smaller",
        ImageSkip::VersionTooLow { .. } => {
            "the document is an older PDF version than that storage needs"
        }
        _ => return format!("they are pictures that cannot be rewritten ({why})"),
    };
    s.to_owned()
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

    /// The report counts skips by their wording, so a reason carrying a
    /// per-picture number must still word identically.
    #[test]
    fn equal_reasons_with_different_numbers_word_alike() {
        let a = ImageSkip::AlreadyJpeg {
            source_estimate: Some(40),
            asked: 60,
        };
        let b = ImageSkip::AlreadyJpeg {
            source_estimate: None,
            asked: 80,
        };
        assert_eq!(picture_left_because(&a), picture_left_because(&b));
    }
}
