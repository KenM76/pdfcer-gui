//! # `ocr::layer_fixture` — what the engine makes of `fixtures/ocr-layer.pdf`
//!
//! `fixtures/ocr-layer.pdf` is written by `tools/gen-ocr-layer-fixture.py`,
//! which carries the argument for the document's shape. This module carries the
//! **numbers** — measured out of the engine, not computed by hand — and pins
//! them, so that a change in either the fixture or the extractor is a red test
//! rather than a check that quietly starts asserting something else.
//!
//! ## Why the fixture exists
//!
//! The overlay's selector is `ExtractedGlyph::invisible`, which the engine sets
//! from the ambient text rendering mode. Before this fixture there was nothing
//! in `fixtures/` with a mode-3 run on it, so the selector had no oracle:
//! `synthetic-image-only.pdf` has no text at all, and producing an OCR layer
//! from it costs the model weights, which are not in this repository.
//!
//! ## The three controls, and the wrong build each one catches
//!
//! | control | where | a build that fails it |
//! |---|---|---|
//! | a visible run on the same page | `VISIBLE CONTROL STAMP` | reports every glyph invisible |
//! | a hidden run outside the OCR stream | ladder, first run | decides invisibility by content stream |
//! | a visible run after a hidden one, same `BT`…`ET` | ladder, second run | treats mode 3 as latching to `ET` |
//!
//! The second and third are the ones the overlay turns on. A shell that
//! reimplements the mode judgement from the content stream — rather than
//! reading the flag the extractor already sets — gets the ladder wrong, and the
//! ladder is the only part of the page where getting it wrong is visible.
//!
//! ## Two instruments, not one
//!
//! The counts are asserted twice: once by filtering the glyphs, and once
//! against `TextDiagnostics::invisible_glyphs`, which the extractor tallies on
//! its own path. They are the same claim reached two ways, so a change that
//! moved one and not the other is a finding rather than a silent agreement.

#![cfg(test)]

use pdfcer_core::document::Document;
use pdfcer_core::text_extract::{ExtractOptions, ExtractedText, extract_document_view};

/// The fixture, relative to the workspace root.
const FIXTURE: &str = "fixtures/ocr-layer.pdf";

/// Glyphs shown at rendering mode 3: the nine OCR words, plus the ladder's
/// hidden run.
const INVISIBLE_GLYPHS: usize = 74;

/// Glyphs shown at rendering mode 0: the stamp, plus the ladder's visible run.
const VISIBLE_GLYPHS: usize = 51;

/// The fixture's bytes, or an explanation naming the path that was tried.
fn bytes() -> Vec<u8> {
    let path = super::fixture::workspace_root().join(FIXTURE);
    std::fs::read(&path).unwrap_or_else(|e| {
        panic!(
            "{} — regenerate with `python tools/gen-ocr-layer-fixture.py`: {e}",
            path.display()
        )
    })
}

/// Read and extract the fixture.
fn extracted() -> ExtractedText {
    let doc = Document::from_bytes(bytes()).expect("the fixture must parse");
    extract_document_view(&doc.view(), &ExtractOptions::default())
        .expect("the fixture must extract")
}

/// The characters one glyph contributed, read out of its run's text by span.
///
/// Not one character: a single code may produce several code points, which is
/// why `ExtractedGlyph` carries a byte range rather than a `char`.
fn glyph_text(run: &pdfcer_core::text_extract::TextRun, index: usize) -> &str {
    let g = &run.glyphs[index];
    let start = g.text_start as usize;
    &run.text[start..start + g.text_len as usize]
}

/// Every glyph on page 0, split by the flag the overlay selects on.
fn split(text: &ExtractedText) -> (String, String) {
    let page = text.pages.first().expect("one page");
    let mut hidden = String::new();
    let mut shown = String::new();
    for run in &page.runs {
        for i in 0..run.glyphs.len() {
            let s = glyph_text(run, i);
            if run.glyphs[i].invisible {
                hidden.push_str(s);
            } else {
                shown.push_str(s);
            }
        }
    }
    (hidden, shown)
}

/// Dump every run the extractor reports, with its invisibility, so the
/// constants above can be re-derived rather than trusted.
///
/// `#[ignore]`d because it exists to be read, not to assert:
///
/// ```text
/// cargo test -p pdfcer-gui --lib dump_the_ocr_layer_fixture -- --ignored --nocapture
/// ```
#[test]
#[ignore = "prints; run deliberately"]
fn dump_the_ocr_layer_fixture() {
    let text = extracted();
    for page in &text.pages {
        println!("page {}: {} runs", page.page_index, page.runs.len());
        println!("  invisible_glyphs = {}", page.diagnostics.invisible_glyphs);
        for (n, run) in page.runs.iter().enumerate() {
            let invisible = run.glyphs.iter().filter(|g| g.invisible).count();
            println!(
                "  run {n}: {:?} — {} glyphs, {invisible} invisible",
                run.text,
                run.glyphs.len()
            );
        }
        let (hidden, shown) = split(&text);
        println!("  invisible ({}): {hidden:?}", hidden.chars().count());
        println!("  visible   ({}): {shown:?}", shown.chars().count());
    }
}

/// ★ **The counts the overlay's selector will be measured against.**
///
/// Both numbers, not one: a build that reported everything invisible passes the
/// first assertion alone, and a build that reported nothing invisible passes
/// the second alone.
#[test]
fn the_fixture_splits_into_the_invisible_and_visible_counts_it_claims() {
    let text = extracted();
    let page = text.pages.first().expect("one page");
    let invisible: usize = page
        .runs
        .iter()
        .map(|r| r.glyphs.iter().filter(|g| g.invisible).count())
        .sum();
    let visible: usize = page
        .runs
        .iter()
        .map(|r| r.glyphs.iter().filter(|g| !g.invisible).count())
        .sum();
    assert_eq!(
        invisible, INVISIBLE_GLYPHS,
        "invisible glyph count moved; re-run dump_the_ocr_layer_fixture"
    );
    assert_eq!(
        visible, VISIBLE_GLYPHS,
        "visible glyph count moved; re-run dump_the_ocr_layer_fixture"
    );
}

/// The extractor's own tally agrees with the per-glyph filter.
///
/// A second instrument on the same claim. `invisible_glyphs` is counted on the
/// extractor's own path, so agreement here is evidence and disagreement is a
/// finding about the engine rather than about this fixture.
#[test]
fn the_engines_own_diagnostic_counts_the_same_invisible_glyphs() {
    let text = extracted();
    let page = text.pages.first().expect("one page");
    assert_eq!(
        page.diagnostics.invisible_glyphs as usize, INVISIBLE_GLYPHS,
        "TextDiagnostics::invisible_glyphs disagrees with the per-glyph flag"
    );
}

/// ★★ **The ladder, which is the whole point of the fixture.**
///
/// A hidden run in a stream that is not the OCR layer, followed inside the same
/// `BT`…`ET` by a visible one. Asserted by reading the text back off the glyphs
/// rather than by position, because a position assertion would pass on a build
/// that had the two runs' flags swapped.
#[test]
fn rendering_mode_is_ambient_and_the_ladder_proves_both_directions() {
    let text = extracted();
    let (hidden, shown) = split(&text);

    assert!(
        hidden.contains("HIDDEN RUN IN A VISIBLE STREAM"),
        "a mode-3 run outside the OCR stream was not reported invisible: {hidden:?}"
    );
    assert!(
        shown.contains("VISIBLE RUN AFTER A HIDDEN ONE"),
        "a run reached by `0 Tr` after a `3 Tr` was reported invisible: {shown:?}"
    );
    assert!(
        shown.contains("VISIBLE CONTROL STAMP"),
        "the visible control run was not reported visible: {shown:?}"
    );
    assert!(
        !shown.contains("SCANNED"),
        "an OCR-layer word was reported visible: {shown:?}"
    );
    assert!(
        hidden.contains("SCANNED"),
        "an OCR-layer word was not reported invisible: {hidden:?}"
    );
}

/// ★★★ **The OCR layer contributes nothing to the raster.**
///
/// This is the property the whole overlay rests on: the operator sees the
/// invisible layer only because the shell draws it, never because the
/// rasterizer does. If mode-3 text reached the pixmap, the overlay would be a
/// second rendering of content the page already shows, the slider would be
/// compositing a picture with itself, and R8b's "applied content renders
/// exactly as saved content will render" would be false of every OCR'd page.
///
/// Measured against a control built from the fixture's own bytes: the page's
/// `/Contents` array is rewritten to drop the OCR stream, padded to the same
/// length so every offset in the cross-reference table stays right. The two
/// rasters must be identical, pixel for pixel.
///
/// The control also guards the assertion from the other side. A build that
/// rasterized nothing at all would satisfy "identical", so the ink count is
/// asserted non-zero first: the page has a picture on it and a visible stamp,
/// and both must be in the pixmap for the comparison to mean anything.
#[test]
fn the_invisible_layer_reaches_no_pixel_of_the_raster() {
    // `[4 0 R 6 0 R 9 0 R]` → `[4 0 R 9 0 R]`, space-padded to the same width
    // so no byte after it moves and the classic xref table stays correct.
    const WITH: &[u8] = b"[4 0 R 6 0 R 9 0 R]";
    const WITHOUT: &[u8] = b"[4 0 R 9 0 R]      ";
    assert_eq!(
        WITH.len(),
        WITHOUT.len(),
        "the patch must not move any offset"
    );

    let original = bytes();
    let at = original
        .windows(WITH.len())
        .position(|w| w == WITH)
        .expect("the page's /Contents array is not the shape this test patches");
    let mut control = original.clone();
    control[at..at + WITHOUT.len()].copy_from_slice(WITHOUT);

    let ink_and_pixels = |data: Vec<u8>| -> (u64, Vec<u8>) {
        let doc = Document::from_bytes(data).expect("parses");
        let pages = pdfcer_core::page_tree::pages(&doc).expect("a page tree");
        let rendered =
            pdfcer_render::render_page(&doc, &pages[0], 2.0).expect("the page rasterizes");
        let pixmap = rendered.pixmap;
        let ink = pixmap
            .pixels()
            .iter()
            .filter(|p| p.red() < 128 && p.green() < 128 && p.blue() < 128)
            .count() as u64;
        (ink, pixmap.data().to_vec())
    };

    let (with_ink, with_px) = ink_and_pixels(original);
    let (without_ink, without_px) = ink_and_pixels(control);

    // A floor rather than a figure, so it cannot go stale: the checkerboard
    // alone is a quarter of the sheet in solid black, which is hundreds of
    // thousands of pixels at this scale. A blank page gives zero and a stray
    // antialiased fleck gives a handful; either would make the comparison
    // below true and meaningless.
    assert!(
        with_ink > 1_000,
        "only {with_ink} dark pixels rasterized — the checkerboard and the visible stamp are \
         effectively missing, so an identical-pixels result below would prove nothing"
    );
    // One assertion, not two. Equal pixels imply an equal ink count, so an ink
    // assertion beside this one would be unreachable while this is green and
    // would shadow it while it is red — the counts belong in the message
    // instead, where they say how much of the layer reached the page.
    assert!(
        with_px == without_px,
        "dropping the OCR content stream changed the raster — mode-3 text is reaching the \
         pixels. Dark pixels: {with_ink} with the layer, {without_ink} without it"
    );
}

/// The one `Tz` that is not 100 survives into the glyph advances.
///
/// The engine sets a per-word horizontal scale so the glyphs span the
/// recognised box. A reader that drops `Tz` gets every OCR run's extent wrong
/// without getting anything visibly wrong, which is the failure this catches.
///
/// The fixture writes `SCANNED` twice — same text, same size, `Tz` 100 on the
/// upper baseline and 86.4 on the lower — so the ratio below has exactly one
/// cause. Comparing two *different* words would confound the scale with the
/// letters.
#[test]
fn the_horizontal_scale_reaches_the_advances_rather_than_being_dropped() {
    let text = extracted();
    let page = text.pages.first().expect("one page");

    // Every occurrence of the control word, with the baseline it sits on.
    let mut widths: Vec<(f32, f32)> = Vec::new();
    for run in &page.runs {
        for i in 0..run.glyphs.len() {
            if run.text[run.glyphs[i].text_start as usize..].starts_with("SCANNED") {
                let span: f32 = run.glyphs[i..i + 7].iter().map(|g| g.advance).sum();
                widths.push((run.glyphs[i].y, span));
            }
        }
    }
    assert_eq!(
        widths.len(),
        2,
        "the fixture must contain the control word exactly twice, got {widths:?}"
    );
    widths.sort_by(|a, b| b.0.total_cmp(&a.0));
    let full = widths[0].1;
    let narrowed = widths[1].1;
    let ratio = narrowed / full;
    assert!(
        (ratio - 0.864).abs() < 0.001,
        "the same word at Tz 86.4 spans {narrowed} against {full} at Tz 100 — ratio {ratio}, \
         expected 0.864; the horizontal scale is not reaching the advances"
    );
}
