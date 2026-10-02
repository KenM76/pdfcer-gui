//! # `blank` — where a new document comes from, and why it is a file
//!
//! `file.new` (`RIBBON_IA.md` §5.1, the File ▸ File band) makes a blank
//! document. This module holds the 443 bytes it makes it *out of*, the
//! decisions behind them, the one-page picture document a pasted stamp is
//! drawn from, and nothing else — the lifetime transition itself is
//! `pdfcer-gui`'s `PdfcerApp::new_document`'s, beside `open_path` and
//! `close_document`, because that is one subject and this is another.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/blank.md`.

use pdfcer_core::document::Document;
use pdfcer_core::page_tree::Page;

/// **The blank document, as bytes.**
pub const TEMPLATE: &[u8] = include_bytes!("assets/blank-a4.pdf");

/// The template page's width in PDF units. ISO 216 A4: 210 mm at 72/inch.
pub const WIDTH_PT: f64 = 595.276;

/// The template page's height in PDF units. ISO 216 A4: 297 mm at 72/inch.
pub const HEIGHT_PT: f64 = 841.89;

/// **Parse [`TEMPLATE`] into a document and its page vector.**
pub fn document() -> Result<(Document, Vec<Page>), String> {
    let doc = Document::from_bytes(TEMPLATE.to_vec()).map_err(|err| err.to_string())?;
    let pages = pdfcer_core::page_tree::pages(&doc).map_err(|err| err.to_string())?;
    Ok((doc, pages))
}

/// The blank document, **resized** to `rect` before anybody sees it.
///
/// # Why this exists at all
///
/// The module header's §3a sets out the alternative and why it is refused: one
/// checked-in template asset per size — ten with landscape, more with ANSI,
/// *and a custom size impossible at any count*. This function is the other
/// implementation, resting on `EditSession::set_media_box` and
/// `pdfcer_core::paper`: one asset and one dialog, every size, both
/// orientations, custom included, with [`TEMPLATE`] unchanged.
///
/// # Why the document is serialized and re-parsed rather than handed over
///
/// The obvious implementation is to build the [`EditSession`], resize page 0,
/// and give the caller the session. **That produces a document that is already
/// modified**: the undo stack holds one command, `save_pending()` is true, and
/// `Ctrl+Z` on a brand-new A1 sheet takes the operator back to A4 — a state
/// they never asked for and cannot name.
///
/// A new document is not an edited document. So the resize happens *before the
/// document exists as far as the shell is concerned*: the bytes are rewritten
/// with [`EditSession::to_full_bytes`] and parsed back, and what the caller
/// receives is an ordinary freshly-parsed `Document` whose page simply is that
/// size. Nothing is pending, nothing is undoable, and
/// `pdfcer-gui`'s `app::lifecycle` needs no special case.
///
/// The cost is one save and one parse of a ~450-byte file, which is not
/// measurable. The cost of the alternative is a permanent oddity in the undo
/// stack of every created document.
///
/// # Why `to_full_bytes` rather than `to_incremental_bytes`
///
/// An incremental save appends a revision, so the result would carry the A4
/// original *and* the resize — a two-revision file for a document that has no
/// history worth keeping and no signature to preserve. `to_full_bytes` writes
/// one revision. The engine's warning on it — that it destroys existing
/// digital signatures — cannot apply: the input is [`TEMPLATE`], which has
/// none.
///
/// # `SaveOptions::identity()`, and why this is not a hole in the funnel
///
/// `pdfcer-gui`'s `app::settings`' funnel exists because an option struct built at a
/// call site discards every setting the operator chose, and a test parses this
/// crate's syntax tree to enforce it. This call site is exempt, and the
/// argument is not "it is only a template" — it is that **no operator-visible
/// byte of this rewrite survives**.
///
/// `SaveOptions` has three fields. Two of them, `xref_entry_eol` and
/// `trailing_eol`, are byte-level spellings of the *written file*, and this
/// file is parsed back and discarded within the same statement — the document
/// the operator eventually saves is written by `pdfcer-gui`'s `app::save`, which does
/// read their settings. The third, `producer`, writes `/Producer` into an
/// **existing** `/Info` dictionary and explicitly does not create one; the
/// template has no `/Info` (read it — it is 443 bytes and hand-legible), so
/// the policy is inert here whichever way it is set.
///
/// `identity()` rather than `default()` regardless, because it is the value
/// that promises to change nothing, and a future template that grew an `/Info`
/// would then be preserved rather than silently stamped. That is R41's rule —
/// pdfcer does not write its own identity into a file the operator did not ask
/// it to mark — and a new document's construction is not an act of authorship
/// the operator directed at a file.
///
/// # Errors
///
/// A `String` for the caller to show, in three cases that are all worth
/// telling apart in the message rather than in a type:
///
/// - the template did not parse — a **build defect**, the same unreachable arm
///   [`document`] has;
/// - the size was degenerate — `EditError::MediaBoxDegenerate`, reachable from
///   a custom size the operator typed, which is why the dialog checks before
///   it asks for one;
/// - the rewrite failed — unreachable for a 443-byte unencrypted file with no
///   hybrid cross-reference, and reported rather than unwrapped.
pub fn document_sized(rect: pdfcer_core::page_tree::Rect) -> Result<(Document, Vec<Page>), String> {
    let base = Document::from_bytes(TEMPLATE.to_vec()).map_err(|err| err.to_string())?;
    let mut session = pdfcer_core::edit::EditSession::new(base);
    let change = session
        .set_media_box(0, rect)
        .map_err(|err| err.to_string())?;
    let (bytes, _report) = session
        .to_full_bytes(&pdfcer_core::writer::SaveOptions::identity())
        .map_err(|err| err.to_string())?;

    let written = bytes.len();
    let doc = Document::from_bytes(bytes).map_err(|err| err.to_string())?;
    let pages = pdfcer_core::page_tree::pages(&doc).map_err(|err| err.to_string())?;

    // Traced AFTER the re-parse, and reporting the page as the RE-PARSED
    // document states it rather than the rectangle that was asked for.
    //
    // The distinction is the whole value of the line. A trace of the request
    // says what this function was told; a trace of `pages[0].media_box` says
    // what a reader of the resulting file will see, which is what the operator
    // gets and the only thing worth asserting from outside the process.
    // `ui-verify`'s `new_document_sizes_the_page` reads `result_w`/`result_h`
    // for exactly that reason — a build that recorded the request and wrote
    // nothing would have a perfect `w=`/`h=` and a 595 × 842 page.
    let media = pages.first().map(|page| page.media_box);
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "new-document-sized w={:.2} h={:.2} change={change:?} bytes={} \
             result_w={:.2} result_h={:.2}",
            rect.width(),
            rect.height(),
            written,
            media.map_or(0.0, |m| m.urx - m.llx),
            media.map_or(0.0, |m| m.ury - m.lly),
        )
    });

    Ok((doc, pages))
}

/// The one-page PDF of `image`: a page the picture's natural size, holding
/// only the picture, filling it — the artwork a pasted stamp is made from.
///
/// Exempt from the settings funnel for [`document_sized`]'s reason: the
/// template has no `/Info`, no annotation is authored, and the written bytes
/// become stamp artwork rather than a file the operator saves.
///
/// # Errors
///
/// The engine's sentence when the template does not parse, the picture is
/// refused, or the rewrite fails.
pub fn picture_page(image: &pdfcer_core::image_import::ImportedImage) -> Result<Vec<u8>, String> {
    let (w, h) = image.natural_size_pt();
    let sheet = pdfcer_core::page_tree::Rect::from_corners(0.0, 0.0, w, h);
    let base = Document::from_bytes(TEMPLATE.to_vec()).map_err(|e| e.to_string())?;
    let mut session = pdfcer_core::edit::EditSession::new(base);
    session.set_media_box(0, sheet).map_err(|e| e.to_string())?;
    session
        .add_image(&pdfcer_core::edit::NewImage::new(0, sheet, image))
        .map_err(|e| e.to_string())?;
    let (bytes, _report) = session
        .to_full_bytes(&pdfcer_core::writer::SaveOptions::identity())
        .map_err(|e| e.to_string())?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_picture_page_is_the_pictures_natural_size() {
        let mut b = Vec::new();
        for v in [40u32, 64, 32] {
            b.extend(v.to_le_bytes());
        }
        b.extend(1u16.to_le_bytes());
        b.extend(32u16.to_le_bytes());
        for v in [0u32, 0, 3780, 3780, 0, 0] {
            b.extend(v.to_le_bytes());
        }
        b.resize(40 + 64 * 32 * 4, 200);
        let image = crate::clippaste::image_from_dib(&b).unwrap();
        let doc = Document::from_bytes(picture_page(&image).unwrap()).unwrap();
        let pages = pdfcer_core::page_tree::pages(&doc).unwrap();
        assert_eq!(pages.len(), 1);
        let m = pages[0].media_box;
        assert!((m.width() - 48.0).abs() < 0.1 && (m.height() - 24.0).abs() < 0.1);
    }

    /// **The compiled-in template really is a document.**
    #[test]
    fn the_template_parses_and_holds_exactly_one_page() {
        let (_doc, pages) = document().expect("the compiled-in template must parse");
        assert_eq!(pages.len(), 1, "New makes a one-page document");
    }

    /// **The page is A4, to the tenth of a point.**
    #[test]
    fn the_template_page_is_a4() {
        let (_doc, pages) = document().expect("the template parses");
        let media = pages[0].media_box;
        let width = media.urx - media.llx;
        let height = media.ury - media.lly;
        assert!(
            (width - WIDTH_PT).abs() < 0.1 && (height - HEIGHT_PT).abs() < 0.1,
            "the template is {width} x {height} pt; A4 is {WIDTH_PT} x {HEIGHT_PT}. \
             If this default was changed deliberately, change `app::blank`'s header \
             argument with it — the reasoning is what makes the number defensible."
        );
    }

    /// **The page has a content stream, empty though it is.**
    #[test]
    fn the_template_page_carries_a_content_stream() {
        let (_doc, pages) = document().expect("the template parses");
        assert_eq!(
            pages[0].contents.len(),
            1,
            "the blank page must carry exactly one (empty) content stream"
        );
    }

    /// The asset stays a template rather than becoming a document.
    #[test]
    fn the_template_is_still_a_few_hundred_bytes() {
        assert!(
            TEMPLATE.len() < 2048,
            "the blank template is {} bytes; it was 443. Anything that big is \
             carrying content, and `assets/PROVENANCE.md` describes a file that \
             carries none.",
            TEMPLATE.len()
        );
    }
}
