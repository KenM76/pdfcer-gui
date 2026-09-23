//! # `text::export_keeptext` — the SVG/EMF "Keep text as text" words
//!
//! The checkbox and its hint in the Export-image window, and the sentences an
//! SVG or EMF export owes when it kept text: how many runs stayed text, how
//! many fell back to outlines, and why, per reason, only where nonzero.

use pdfcer_render::emf::EmfTextOutcome;
use pdfcer_render::svg::SvgTextOutcome;

use crate::app::actions::imageexport::ImageFormat;

/// The heading over the checkbox.
#[must_use]
pub const fn heading() -> &'static str {
    "Text"
}

/// The checkbox.
#[must_use]
pub const fn checkbox() -> &'static str {
    "Keep text as text"
}

/// What the choice means for the selected vector format.
#[must_use]
pub const fn hint(format: ImageFormat, keep: bool) -> &'static str {
    match (format, keep) {
        (ImageFormat::Svg, true) => {
            "Words stay words, with their fonts carried inside the file, so \
             they can be selected and searched in a web browser. Word and \
             Inkscape ignore fonts carried this way and show the words in a \
             font of their own."
        }
        (ImageFormat::Emf, true) => {
            "Words stay words. A metafile cannot carry a font, so they are \
             drawn in whichever installed font has the same name, and may \
             look different on another computer."
        }
        (ImageFormat::Svg | ImageFormat::Emf, false) => {
            "Text is written as outlines. It looks the same in every program, \
             and cannot be selected or searched there."
        }
        (ImageFormat::Png | ImageFormat::Jpeg, _) => {
            "Pictures made of pixels have no text to keep."
        }
    }
}

/// The receipt for an SVG written with `SvgText::KeepText`.
#[must_use]
pub fn svg_kept(outcome: &SvgTextOutcome) -> Vec<String> {
    let kept = outcome.runs_as_text;
    let outlined = outcome.runs_as_outlines();
    let mut out = Vec::new();
    if kept > 0 {
        out.push(format!(
            "Text was kept as words: {kept} piece(s) of text, with {} font(s) \
             carried inside the file, so the words can be selected and searched \
             in a web browser.",
            outcome.fonts_embedded
        ));
        out.push(
            "Word and Inkscape do not use fonts carried inside an SVG, so there \
             the words may appear in a different font. For those programs, \
             clear Keep text as text."
                .to_owned(),
        );
        out.push(words_come_from_the_pdf().to_owned());
    }
    let reasons = [
        (
            outcome.fallback_not_sfnt,
            "in a kind of font that cannot be rebuilt for the web",
        ),
        (
            outcome.fallback_paint,
            "outlined, clipped, patterned, shaded or see-through",
        ),
        (
            outcome.fallback_unmapped,
            "with a character the PDF does not name",
        ),
        (
            outcome.fallback_conflict,
            "drawing one character two different ways in the same font",
        ),
        (
            outcome.fallback_geometry,
            "vertical, turned letter by letter, or raised part-way along",
        ),
        (
            outcome.fallback_font_build,
            "whose font could not be rebuilt",
        ),
        (
            outcome.fallback_restricted,
            "whose font's licence forbids carrying it",
        ),
    ];
    out.extend(fell_back(outlined, &reasons));
    if kept == 0 && outlined == 0 {
        out.push(no_text().to_owned());
    }
    out
}

/// The receipt for an EMF written with `EmfText::KeepText`.
#[must_use]
pub fn emf_kept(outcome: &EmfTextOutcome) -> Vec<String> {
    let kept = outcome.runs_as_text;
    let outlined = outcome.runs_as_outlines();
    let mut out = Vec::new();
    if kept > 0 {
        out.push(format!(
            "Text was kept as words: {kept} piece(s) of text. A metafile cannot \
             carry a font, so they are drawn in whichever installed font has \
             the same name, and on another computer the letters may look \
             different. Word and LibreOffice keep every letter in place; \
             Inkscape re-spaces the line."
        ));
        out.push(words_come_from_the_pdf().to_owned());
    }
    let reasons = [
        (
            outcome.fallback_paint,
            "outlined, see-through, patterned or blended",
        ),
        (
            outcome.fallback_unmapped,
            "with a character the PDF does not name",
        ),
        (
            outcome.fallback_geometry,
            "slanted, mirrored, stretched or not on one line",
        ),
        (
            outcome.fallback_symbol_face,
            "in a symbol font, which another computer would draw as different \
             characters",
        ),
    ];
    out.extend(fell_back(outlined, &reasons));
    if kept == 0 && outlined == 0 {
        out.push(no_text().to_owned());
    }
    out
}

/// The kept words are only as right as the PDF's own character mapping.
#[must_use]
pub const fn words_come_from_the_pdf() -> &'static str {
    "The words are the characters the PDF says it shows. A PDF that labels its \
     characters wrongly looks right on the page and gives wrong words here."
}

/// Keep text was on and the page had none.
#[must_use]
pub const fn no_text() -> &'static str {
    "Keep text as text was on, and this page has no text to keep."
}

/// The fallback sentence, naming only the nonzero reasons, or nothing.
fn fell_back(total: usize, reasons: &[(usize, &str)]) -> Option<String> {
    if total == 0 {
        return None;
    }
    let named: Vec<String> = reasons
        .iter()
        .filter(|(n, _)| *n > 0)
        .map(|(n, why)| format!("{n} {why}"))
        .collect();
    Some(format!(
        "{total} piece(s) of text could not be kept as words and were written \
         as outlines: {}.",
        named.join("; ")
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn svg_names_kept_fonts_and_only_the_nonzero_reasons() {
        let mut o = SvgTextOutcome::default();
        o.runs_as_text = 12;
        o.fonts_embedded = 2;
        o.fallback_paint = 3;
        o.fallback_restricted = 1;
        let joined = svg_kept(&o).join(" | ");
        assert!(joined.contains("12 piece(s)"), "{joined}");
        assert!(joined.contains("2 font(s)"), "{joined}");
        assert!(joined.contains("4 piece(s) of text could not"), "{joined}");
        assert!(joined.contains("3 outlined"), "{joined}");
        assert!(joined.contains("1 whose font's licence"), "{joined}");
        assert!(!joined.contains("does not name"), "{joined}");
        assert!(joined.contains("Word and Inkscape"), "{joined}");
    }

    #[test]
    fn emf_discloses_the_installed_font_and_its_reasons() {
        let mut o = EmfTextOutcome::default();
        o.runs_as_text = 5;
        o.fallback_symbol_face = 2;
        let joined = emf_kept(&o).join(" | ");
        assert!(joined.contains("5 piece(s)"), "{joined}");
        assert!(joined.contains("installed font"), "{joined}");
        assert!(joined.contains("2 piece(s) of text could not"), "{joined}");
        assert!(joined.contains("2 in a symbol font"), "{joined}");
        assert!(!joined.contains("slanted"), "{joined}");
    }

    #[test]
    fn nothing_kept_and_nothing_outlined_says_so() {
        assert_eq!(svg_kept(&SvgTextOutcome::default()), vec![no_text()]);
        assert_eq!(emf_kept(&EmfTextOutcome::default()), vec![no_text()]);
    }

    #[test]
    fn all_fallen_back_omits_the_kept_sentences() {
        let mut o = EmfTextOutcome::default();
        o.fallback_unmapped = 4;
        let notes = emf_kept(&o);
        assert_eq!(notes.len(), 1, "{notes:?}");
        assert!(notes[0].contains("4 with a character"), "{notes:?}");
    }
}
