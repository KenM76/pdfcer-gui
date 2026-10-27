//! # `clipboard::richtext` — text copied with its formatting and paragraphs
//!
//! A selection, a page or the whole document as the engine's formatted copy
//! (`EditableTextModel::rich_text`), placed as `HTML Format`, `Rich Text
//! Format` and `CF_UNICODETEXT` in one clipboard transaction, so a word
//! processor pastes bold, sizes, families and paragraphs and a plain-text
//! target still gets the words.
//!
//! The extraction is the operator's options with provenance on
//! ([`OpenDoc::provenance_page_text`]), so run indices match the canvas
//! selection's; paragraphs are the reflow recognition's.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/clipboard/richtext.md`.

use pdfcer_core::page_tree::Page;
use pdfcer_core::text_edit::decoration::page_decorations;
use pdfcer_core::text_edit::model::{PageFonts, RichText, RichTextSources};
use pdfcer_core::text_edit::{EditableTextModel, TextPosition, reflow_recognition_options};
use pdfcer_core::text_extract::PageText;
use pdfcer_core::view::DocumentView;

use super::place::{Refusal, put_entries};
use crate::opendoc::OpenDoc;

/// The registered name Word, LibreOffice and browsers read HTML from.
const HTML: &str = "HTML Format"; // ui-text-exempt: a clipboard format name
/// The registered name Word and WordPad read RTF from.
const RTF: &str = "Rich Text Format"; // ui-text-exempt: a clipboard format name
/// The predefined plain-text format's name, for the capture file and errors.
const UNICODE: &str = "CF_UNICODETEXT"; // ui-text-exempt: a clipboard format name

/// Page `page`'s text between `range`'s ends (either order, end exclusive),
/// or the whole page with `None`. `None` when the page does not exist or its
/// content does not extract.
#[must_use]
pub fn page(
    doc: &OpenDoc,
    page: usize,
    range: Option<(TextPosition, TextPosition)>,
) -> Option<RichText> {
    let text = doc.provenance_page_text(page)?;
    let page_ref = doc.pages.get(page)?;
    Some(build(&doc.session.view(), page_ref, &text, range))
}

/// Every page's text in page order, extracted afresh (not cached: a
/// whole-document copy is pressed rarely and would hold every page alive).
///
/// # Errors
///
/// The first page whose content does not extract, with the engine's reason.
pub fn document(doc: &OpenDoc) -> Result<RichText, String> {
    use crate::settings::SettingsExt;
    let options = doc.settings.extract_options().with_provenance(true);
    let view = doc.session.view();
    let mut all = RichText::default();
    for (index, page_ref) in doc.pages.iter().enumerate() {
        let text = pdfcer_core::text_extract::extract_page_view(&view, page_ref, index, &options)
            .map_err(|e| e.to_string())?;
        append(&mut all, build(&view, page_ref, &text, None));
    }
    Ok(all)
}

/// The formatted copy of one extracted page.
fn build(
    view: &DocumentView<'_>,
    page: &Page,
    text: &PageText,
    range: Option<(TextPosition, TextPosition)>,
) -> RichText {
    let model = EditableTextModel::recognize(text, &reflow_recognition_options());
    let fonts = PageFonts::read(view, page);
    // A page whose decorations do not read still copies, without underline
    // and strikethrough; the engine has no counter for that loss.
    let decorations = page_decorations(view, page).ok();
    let mut sources = RichTextSources::default().with_fonts(&fonts);
    if let Some(decorations) = &decorations {
        sources = sources.with_decorations(decorations);
    }
    model.rich_text(range, &sources)
}

/// `more`'s paragraphs after `all`'s, with the outcome counts summed.
fn append(all: &mut RichText, more: RichText) {
    all.paragraphs.extend(more.paragraphs);
    let (a, b) = (&mut all.outcome, more.outcome);
    a.paragraphs += b.paragraphs;
    a.weights_derived += b.weights_derived;
    a.families_missing += b.families_missing;
    a.colours_dropped += b.colours_dropped;
}

/// Place `rich` as HTML, RTF and plain text in one transaction. Returns the
/// names placed.
///
/// # Errors
///
/// The clipboard's refusal; nothing is placed when one is returned.
pub fn place(rich: &RichText) -> Result<Vec<&'static str>, Refusal> {
    let html = terminated(rich.to_cf_html());
    let rtf = terminated(rich.to_rtf());
    let text = utf16(&rich.plain_text());
    put_entries(&[
        native_clipboard::Entry {
            name: HTML,
            slot: native_clipboard::Slot::Registered,
            bytes: &html,
        },
        native_clipboard::Entry {
            name: RTF,
            slot: native_clipboard::Slot::Registered,
            bytes: &rtf,
        },
        native_clipboard::Entry {
            name: UNICODE,
            slot: native_clipboard::Slot::Predefined(native_clipboard::CF_UNICODETEXT),
            bytes: &text,
        },
    ])
}

/// `s`'s bytes with the NUL a registered text format ends in.
fn terminated(s: String) -> Vec<u8> {
    let mut bytes = s.into_bytes();
    bytes.push(0);
    bytes
}

/// `s` as NUL-terminated UTF-16LE, as `CF_UNICODETEXT` holds it.
fn utf16(s: &str) -> Vec<u8> {
    s.encode_utf16()
        .chain(std::iter::once(0))
        .flat_map(u16::to_le_bytes)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::opendoc::fixtures::{FOUR_PAGES, open_fixture};

    #[test]
    fn plain_text_is_nul_terminated_utf16() {
        assert_eq!(utf16("A\u{e9}"), vec![0x41, 0, 0xe9, 0, 0, 0]);
    }

    #[test]
    fn a_page_copies_as_paragraphs_with_html_and_rtf() {
        let doc = open_fixture(FOUR_PAGES);
        let rich = page(&doc, 0, None).expect("page 0 extracts");
        assert!(rich.outcome.paragraphs > 0, "no paragraph on page 0");
        assert!(!rich.plain_text().is_empty());
        assert!(rich.to_cf_html().starts_with("Version:"));
        assert!(rich.to_rtf().starts_with(r"{\rtf1"));
    }

    #[test]
    fn the_document_is_every_page_in_order() {
        let doc = open_fixture(FOUR_PAGES);
        let all = document(&doc).expect("every page extracts");
        let first = page(&doc, 0, None).expect("page 0 extracts");
        assert!(all.outcome.paragraphs > first.outcome.paragraphs);
        assert!(all.plain_text().starts_with(&first.plain_text()));
    }
}
