//! Tests for `crate::panels::objects::summary`, kept in the gui because they reach gui modules.

use crate::panels::objects::summary::*;
use pdfcer_core::content::ContentStream;
use pdfcer_core::vector::{Bounds, TextBoundsBasis};
use pdfcer_core::vector::{Matrix, NoXObjects, decompose};

/// Decompose a content stream and describe every object in paint order —
/// the seam these tests share, so each case is a content-stream literal
/// plus an assertion on the record.
fn describe_all(src: &[u8]) -> Vec<ObjectSummary> {
    let cs = ContentStream::parse(src.to_vec()).expect("parse");
    let objects = decompose(&cs, Matrix::IDENTITY, &NoXObjects);
    objects.objects.iter().map(describe_object).collect()
}

fn only(src: &[u8]) -> ObjectSummary {
    let mut all = describe_all(src);
    assert_eq!(all.len(), 1, "{all:?}");
    all.remove(0)
}

/// Describe every object on a FIXTURE's first page, through
/// `decompose_page` — i.e. with real font and XObject resolvers, which
/// is the path the GUI is actually on.
fn describe_fixture(rel: &str) -> Vec<ObjectSummary> {
    let path = crate::panels::objects::test_support::engine_fixture(rel);
    let doc = pdfcer_core::document::Document::load(&path).expect("the fixture loads");
    let pages = pdfcer_core::page_tree::pages(&doc).expect("a page tree");
    let model = pdfcer_core::vector::decompose_page(&doc.view(), &pages[0], Matrix::IDENTITY)
        .expect("the page decomposes");
    model.objects.iter().map(describe_object).collect()
}

/// End to end: a text object carries the string it shows and the
/// typeface that shows it, decoded through `text_extract`'s §9.10.2
/// ladder rather than a second decoder written here.
#[test]
fn a_text_object_reports_its_string_and_its_font() {
    let objects = describe_fixture("text/simple-winansi.pdf");
    let text = objects
        .iter()
        .find(|s| s.kind == ObjectKind::Text)
        .expect("the fixture has a text object");
    // SOURCED characters only, verbatim: the fixture's `TJ` opens the
    // gap between "Hello" and "world" with a -2000 kerning offset and NO
    // space glyph, and its second line is a `Td` with no line marker —
    // §14.8.2.5 S3/S5, neither of which the file states. `text_extract`
    // DERIVES both for `plain_text` and omits both for `sourced_text`;
    // a preview is the latter, because a row label is not the place to
    // present a reader's guess as the document's content.
    assert_eq!(text.text.as_deref(), Some("HelloworldSecond line"));
    assert!(!text.text_truncated);
    let font = text.font.as_ref().expect("a font was in effect");
    assert_eq!(font.base_font.as_deref(), Some("Helvetica"));
    assert_eq!(font.size, 24.0);
    // A decodable string earns no decode disclosure — only the
    // ever-present approximate-bounds one. The fixture's Helvetica is a
    // standard-14 face, so its widths and its ascent/descent are both
    // real metrics and the basis is the good one.
    assert_eq!(
        text.notes,
        vec![ObjectNote::ApproximateTextBounds(
            TextBoundsBasis::FontMetrics
        )]
    );
}

/// The honest-failure case: a font whose encoding defeats decoding
/// yields NO string and a note saying why — never a row of `\u{fffd}`,
/// which would read as a pdfcer defect rather than as a property of the
/// file.
#[test]
fn text_that_cannot_be_decoded_is_disclosed_not_mojibake() {
    let objects = describe_fixture("text/identity-h-no-tounicode.pdf");
    let text = objects
        .iter()
        .find(|s| s.kind == ObjectKind::Text)
        .expect("the fixture has a text object");
    assert_eq!(text.text, None, "no string may be fabricated or mangled");
    assert!(
        text.notes.contains(&ObjectNote::TextUndecodable),
        "{:?}",
        text.notes
    );
    // The FONT is still named: knowing which font cannot be read is
    // most of the value of the disclosure.
    assert!(text.font.is_some());
}

/// An image reports its sample count from `/Width`/`/Height` (§8.9.5
/// Table 89) — and a form XObject reports none, because a form has no
/// samples.
#[test]
fn an_image_reports_its_pixel_size() {
    let objects = describe_fixture("vector/mixed.pdf");
    let image = objects
        .iter()
        .find(|s| s.kind == ObjectKind::ImageXObject)
        .expect("the fixture has an image XObject");
    // The fixture's image is 2x2 DeviceGray (its PROVENANCE entry).
    assert_eq!(image.pixels, Some((2, 2)));
    // The sample count is NOT the size on the page: the image is placed
    // by the CTM, so the two numbers differ and both are reported.
    assert_ne!(
        image.size().map(|(w, h)| (w as u32, h as u32)),
        image.pixels,
        "a 2x2 image placed at 2x2 pt would make this test prove nothing"
    );
}

/// Nothing is invented for the kinds that carry no such fact: a path has
/// no string or pixel size, and a text object with no `Tf` has no font.
#[test]
fn no_kind_gains_a_detail_it_does_not_have() {
    let path = only(b"0 0 1 rg 10 10 80 80 re f");
    assert_eq!(path.text, None);
    assert_eq!(path.font, None);
    assert_eq!(path.pixels, None);

    // A show operator with no preceding `Tf`: an object, but no font to
    // name and therefore none named.
    let text = only(b"BT 40 40 Td (Hi) Tj ET");
    assert_eq!(text.kind, ObjectKind::Text);
    assert_eq!(text.font, None);
}

#[test]
fn a_filled_path_reports_its_fill_colour_and_node_count() {
    let s = only(b"0 0 1 rg 10 10 80 80 re f");
    assert_eq!(s.kind, ObjectKind::Path);
    assert_eq!(s.nodes, Some(4));
    assert_eq!(s.colour.map(|c| c.b), Some(1.0));
    // Not stroked: no line width is reported, because none is used.
    assert_eq!(s.line_width, None);
    assert!(s.notes.is_empty(), "{:?}", s.notes);
    assert_eq!(s.size(), Some((80.0, 80.0)));
    // The winding rule travels, because the Properties panel reports it
    // and nothing else in the application can.
    assert!(s.paint.and_then(|p| p.fill).is_some());
}

/// A stroke-only path must report the STROKE colour: its fill colour is
/// never painted, so printing it would name a colour that is nowhere on
/// the page.
#[test]
fn a_stroked_path_reports_its_stroke_colour_and_line_width() {
    let s = only(b"1 0 0 RG 2 w 10 10 m 90 90 l S");
    assert_eq!(s.kind, ObjectKind::Path);
    assert_eq!(s.colour.map(|c| c.r), Some(1.0));
    assert_eq!(s.line_width, Some(2.0));
}

/// The `n`-op case — a real page object that paints no pixels. This is
/// one of the two headline "box over nothing" explanations.
#[test]
fn a_no_paint_path_reports_that_it_paints_nothing_and_no_colour() {
    let s = only(b"10 10 80 80 re n");
    assert_eq!(s.kind, ObjectKind::Path);
    assert_eq!(s.colour, None);
    assert!(
        s.notes.contains(&ObjectNote::PaintsNothing),
        "{:?}",
        s.notes
    );
}

/// The other headline case, and the one the operator most likely hit:
/// a text object is ALWAYS approximate, so its stated extent covers
/// whitespace around and above the glyphs.
#[test]
fn a_text_object_always_discloses_its_approximate_bounds() {
    let s = only(b"BT /F1 12 Tf 40 40 Td (Hi) Tj ET");
    assert_eq!(s.kind, ObjectKind::Text);
    assert_eq!(
        s.notes.first(),
        Some(&ObjectNote::ApproximateTextBounds(TextBoundsBasis::EmBox))
    );
    assert!(s.bounds_are_approximate());
    // Nothing is fabricated for text: no string, no font, no colour.
    assert_eq!(s.colour, None);
    assert_eq!(s.nodes, None);
    assert_eq!(s.paint, None);
}

/// The bug found while observing: a horizontal rule is a correct object
/// whose bbox has zero height, so an outline rect around it strokes
/// nothing at all. The note is what lets a panel explain it.
#[test]
fn a_zero_height_path_is_disclosed_as_degenerate() {
    let s = only(b"100 200 m 300 200 l S");
    assert_eq!(s.size(), Some((200.0, 0.0)));
    assert!(
        s.notes
            .contains(&ObjectNote::DegenerateBounds(Degeneracy::HorizontalRule)),
        "{:?}",
        s.notes
    );
}

#[test]
fn a_zero_width_path_is_disclosed_as_degenerate() {
    let s = only(b"200 100 m 200 300 l S");
    assert_eq!(s.size(), Some((0.0, 200.0)));
    assert!(
        s.notes
            .contains(&ObjectNote::DegenerateBounds(Degeneracy::VerticalRule)),
        "{:?}",
        s.notes
    );
}

/// A single-point path — degenerate on both axes at once.
#[test]
fn a_point_path_is_disclosed_as_degenerate_on_both_axes() {
    let s = only(b"150 150 m 150 150 l S");
    assert_eq!(s.size(), Some((0.0, 0.0)));
    assert!(
        s.notes
            .contains(&ObjectNote::DegenerateBounds(Degeneracy::Point)),
        "{:?}",
        s.notes
    );
}

/// An inline image is a distinct answer from an image XObject, and both
/// are distinct from a form XObject — see [`ObjectKind`]'s own
/// rationale.
#[test]
fn an_inline_image_is_reported_as_inline() {
    let s = only(b"q 100 0 0 50 10 10 cm BI /W 1 /H 1 /CS /G /BPC 8 ID \x00 EI Q");
    assert_eq!(s.kind, ObjectKind::InlineImage);
    assert_eq!(s.size(), Some((100.0, 50.0)));
    // Honest ceiling: no pixel size exists in the model for an inline
    // image.
    assert!(s.notes.is_empty(), "{:?}", s.notes);
}

/// The census is what the Objects panel's header line is built from.
#[test]
fn the_census_tallies_each_kind() {
    let c = census([
        ObjectKind::Path,
        ObjectKind::Path,
        ObjectKind::Text,
        ObjectKind::InlineImage,
        ObjectKind::ImageXObject,
        ObjectKind::FormXObject,
    ]);
    assert_eq!(c.total, 6);
    assert_eq!(c.paths, 2);
    assert_eq!(c.texts, 1);
    // Inline and XObject images are one bucket in a census.
    assert_eq!(c.images, 2);
    assert_eq!(c.forms, 1);
    assert_eq!(census([]), SelectionCensus::default());
}

/// An empty bbox yields `NoBounds` and a `None` size — the case where no
/// outline can be drawn anywhere, which must be disclosed rather than
/// looking like a dead click.
#[test]
fn an_object_with_no_finite_geometry_reports_no_bounds() {
    assert_eq!(
        degeneracy_note(Bounds::EMPTY),
        Some(ObjectNote::NoBounds),
        "{:?}",
        Bounds::EMPTY
    );
}

/// **The catalogs are complete and free of duplicates.**
#[test]
fn the_note_and_kind_catalogs_hold_no_duplicates() {
    let mut notes = ObjectNote::ALL.to_vec();
    let n = notes.len();
    notes.sort_by_key(|note| format!("{note:?}"));
    notes.dedup();
    assert_eq!(notes.len(), n, "ObjectNote::ALL lists a note twice");

    let mut kinds = ObjectKind::ALL.to_vec();
    let k = kinds.len();
    kinds.sort_by_key(|kind| format!("{kind:?}"));
    kinds.dedup();
    assert_eq!(kinds.len(), k, "ObjectKind::ALL lists a kind twice");
}
