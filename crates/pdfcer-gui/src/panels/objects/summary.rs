//! # `panels::objects::summary` — the ONE description of a page object
//!
//! Turns a `pdfcer_core::vector::VectorObject` into a small, GUI-shaped
//! **fact record** ([`ObjectSummary`]) that every surface which has to say
//! *"what is this thing?"* reads.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/objects/summary.md`.

use pdfcer_core::vector::{
    Bounds, ImageSource, PaintStyle, Rgb, TextBoundsBasis, TextFont, TextPreview, VectorObject,
};

/// Which kind of thing a page object is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObjectKind {
    /// A path object (`re`/`m`/`l`/`c` … then a painting operator).
    Path,
    /// A `BT`…`ET` text object.
    Text,
    /// A `BI`/`ID`/`EI` inline image.
    InlineImage,
    /// A `Do` on an image XObject.
    ImageXObject,
    /// A `Do` on a form XObject — one opaque object, not recursed into.
    FormXObject,
}

/// The SHAPE an object with a zero-extent bounding box actually is.
///
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Degeneracy {
    /// Zero width, non-zero height.
    VerticalRule,
    /// Zero height, non-zero width.
    HorizontalRule,
    /// Zero on both axes.
    Point,
}

/// A disclosable fact that explains an object the operator may not be able
/// to SEE (module docs' table).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObjectNote {
    /// The bounds of a text object are an approximation — of the kind the
    /// payload names.
    ///
    /// Parameterized rather than split into four sibling variants for the
    /// same reason [`ObjectNote::DegenerateBounds`] carries a
    /// [`Degeneracy`]: every consumer that asks *"is this box
    /// approximate?"* wants one answer for all four, and every consumer that
    /// EXPLAINS the box needs to know which one — so the question and the
    /// detail must not be two separate notes that could disagree.
    ///
    /// The ui-spec makes carrying the reason binding: shipping a better box
    /// with one sentence describing all four cases would leave some text
    /// objects disclosed by a sentence that no longer matches the box
    /// actually computed, which is a regression in honesty rather than the
    /// improvement the geometry work was for.
    ApproximateTextBounds(TextBoundsBasis),
    /// The path paints no pixels — an `n`-op clip or discarded construction.
    PaintsNothing,
    /// The bounds have zero extent on one or both axes.
    DegenerateBounds(Degeneracy),
    /// The object has no finite geometry at all, so no outline can be drawn.
    NoBounds,
    /// A form XObject: one opaque object covering a whole nested drawing.
    FormNotDecomposed,
    /// Not one character of the object's text could be recovered: every
    /// character code reached ISO 32000-1 §9.10.2's failure clause.
    ///
    /// A *document* fact, not a pdfcer limitation — the clause itself
    /// concedes that for such a font "there is no way to determine what the
    /// character code represents". Disclosed rather than shown as
    /// `\u{fffd}\u{fffd}\u{fffd}`, which would read as a defect in the
    /// reader.
    TextUndecodable,
    /// Some — not all — of the object's characters could not be recovered,
    /// so the shown string contains U+FFFD replacements.
    ///
    /// Distinct from [`ObjectNote::TextUndecodable`] because the operator's
    /// question is different: here there IS a readable string and the
    /// question is why part of it is `\u{fffd}`.
    TextPartlyUndecodable,
}

impl ObjectNote {
    /// Every note, for the tests that sweep the catalog.
    pub const ALL: [Self; 12] = [
        Self::ApproximateTextBounds(TextBoundsBasis::FontMetrics),
        Self::ApproximateTextBounds(TextBoundsBasis::MetricAdvancesNominalHeight),
        Self::ApproximateTextBounds(TextBoundsBasis::EstimatedAdvances),
        Self::ApproximateTextBounds(TextBoundsBasis::EmBox),
        Self::PaintsNothing,
        Self::DegenerateBounds(Degeneracy::VerticalRule),
        Self::DegenerateBounds(Degeneracy::HorizontalRule),
        Self::DegenerateBounds(Degeneracy::Point),
        Self::NoBounds,
        Self::FormNotDecomposed,
        Self::TextUndecodable,
        Self::TextPartlyUndecodable,
    ];
}

impl ObjectKind {
    /// Every kind, for the tests that sweep the catalog (see
    /// [`ObjectNote::ALL`] for the rationale).
    pub const ALL: [Self; 5] = [
        Self::Path,
        Self::Text,
        Self::InlineImage,
        Self::ImageXObject,
        Self::FormXObject,
    ];
}

/// Everything the GUI can honestly say about one page object.
#[derive(Debug, Clone, PartialEq)]
pub struct ObjectSummary {
    /// What kind of object it is.
    pub kind: ObjectKind,
    /// Paint disposition — `Some` for paths only (nothing else has one).
    ///
    /// Carries the winding rule: `PaintStyle::fill` is `Some(FillRule)`, and
    /// the Properties panel reports nonzero-vs-even-odd from it. That is a
    /// fact the operator cannot get anywhere else in the application, and it
    /// is why the whole `PaintStyle` travels rather than a boolean pair.
    pub paint: Option<PaintStyle>,
    /// The colour a viewer actually SEES, resolved by paint disposition: the
    /// fill colour for a filled path, the stroke colour for a stroke-only
    /// path, `None` for a path that paints nothing (reporting its unused,
    /// default-black fill colour there would be a confidently wrong answer).
    pub colour: Option<Rgb>,
    /// Anchor count across every subpath — paths only.
    pub nodes: Option<usize>,
    /// Stroke width in user-space units at paint time — stroked paths only.
    pub line_width: Option<f64>,
    /// The object's decoded text preview — text objects only, and only when
    /// something was actually recovered (module docs' table).
    ///
    /// Already capped at `pdfcer_core::vector::MAX_TEXT_PREVIEW_CHARS` by the
    /// decomposition, which is the MEMORY bound; a display applies its own,
    /// shorter, LINE-LENGTH bound on top (see
    /// [`crate::text::panels::objects::object_row`]). Two caps because they
    /// answer two different questions, and collapsing them would tie a row's
    /// width to the object model's storage budget.
    pub text: Option<String>,
    /// Whether [`Self::text`] is a prefix of a longer string. A display
    /// marks the elision rather than presenting a prefix as the whole.
    pub text_truncated: bool,
    /// The font in effect at the object's first show operator — text objects
    /// only, `None` when no `Tf` was in effect.
    pub font: Option<TextFont>,
    /// The image's `(width, height)` in **samples** — image objects with a
    /// usable `/Width`+`/Height` only (ISO 32000-1 §8.9.5, Table 89).
    ///
    /// A sample count, not a size on the page: an image occupies the unit
    /// square under the CTM, so [`Self::bounds`] is where it is and this is
    /// what it is made of. Both are shown, and the pair is what lets an
    /// operator judge effective resolution.
    pub pixels: Option<(u32, u32)>,
    /// The object's page-space bounding box, verbatim from the model.
    pub bounds: Bounds,
    /// Every applicable disclosure, most-explanatory first (module docs).
    pub notes: Vec<ObjectNote>,
}

impl ObjectSummary {
    /// The bbox's width and height in PDF points, or `None` if it has no
    /// finite geometry. `(0.0, h)` and `(w, 0.0)` are legitimate answers —
    /// see [`Degeneracy`].
    #[must_use]
    pub fn size(&self) -> Option<(f64, f64)> {
        if self.bounds.is_empty() {
            return None;
        }
        Some((
            self.bounds.max.x - self.bounds.min.x,
            self.bounds.max.y - self.bounds.min.y,
        ))
    }

    /// Whether the object's stated extent is a deliberate APPROXIMATION
    /// rather than its measured extent.
    #[must_use]
    pub fn bounds_are_approximate(&self) -> bool {
        self.notes
            .iter()
            .any(|n| matches!(n, ObjectNote::ApproximateTextBounds(_)))
    }
}

/// Classify one object, and nothing more.
#[must_use]
pub fn object_kind(object: &VectorObject) -> ObjectKind {
    match object {
        VectorObject::Path(_) => ObjectKind::Path,
        VectorObject::Text(_) => ObjectKind::Text,
        VectorObject::Image(i) => match i.source {
            ImageSource::Inline => ObjectKind::InlineImage,
            ImageSource::XObject => ObjectKind::ImageXObject,
            ImageSource::Form => ObjectKind::FormXObject,
        },
    }
}

/// Describe one object — **the single description path** (module docs).
#[must_use]
pub fn describe_object(object: &VectorObject) -> ObjectSummary {
    let bounds = object.page_bbox();
    let kind = object_kind(object);
    let mut notes = Vec::new();
    if let Some(note) = degeneracy_note(bounds) {
        notes.push(note);
    }
    match object {
        VectorObject::Path(p) => {
            let nodes = p.subpaths.iter().map(|sp| sp.anchors().count()).sum();
            if p.style.is_invisible() {
                notes.push(ObjectNote::PaintsNothing);
            }
            ObjectSummary {
                kind,
                paint: Some(p.style),
                colour: visible_colour(p.style, p.fill_color, p.stroke_color),
                nodes: Some(nodes),
                line_width: p.style.stroke.then_some(p.line_width),
                text: None,
                text_truncated: false,
                font: None,
                pixels: None,
                bounds,
                notes,
            }
        }
        VectorObject::Text(t) => {
            // The decode disclosures come BEFORE the approximation one is
            // inserted at the head, so the final order is: approximation
            // first (it explains the box, which is what the operator is
            // looking at), then why the string reads as it does.
            if let Some(note) = decode_note(&t.preview) {
                notes.push(note);
            }
            if t.approximate {
                // Insert FIRST: for text this is the whole explanation, and a
                // degenerate text bbox (possible for an empty `BT`/`ET`) is
                // the lesser fact. The basis travels with the note so the
                // sentence shown always describes the box actually computed.
                notes.insert(0, ObjectNote::ApproximateTextBounds(t.bounds_basis));
            }
            let (text, text_truncated) = match &t.preview {
                // An all-U+FFFD string is withheld: `ObjectNote::
                // TextUndecodable` says the same thing in words, and a row
                // of replacement characters reads as a pdfcer defect rather
                // than as a property of the file (module docs' table).
                TextPreview::Decoded {
                    text, truncated, ..
                } => (Some(text.clone()), *truncated),
                TextPreview::Undecodable | TextPreview::Unavailable | TextPreview::Empty => {
                    (None, false)
                }
            };
            ObjectSummary {
                kind,
                paint: None,
                colour: None,
                nodes: None,
                line_width: None,
                text,
                text_truncated,
                font: t.font.clone(),
                pixels: None,
                bounds,
                notes,
            }
        }
        VectorObject::Image(i) => {
            if kind == ObjectKind::FormXObject {
                notes.push(ObjectNote::FormNotDecomposed);
            }
            ObjectSummary {
                kind,
                paint: None,
                colour: None,
                nodes: None,
                line_width: None,
                text: None,
                text_truncated: false,
                font: None,
                pixels: i.pixel_size,
                bounds,
                notes,
            }
        }
    }
}

/// The disclosure, if any, a text preview's decoding outcome earns.
fn decode_note(preview: &TextPreview) -> Option<ObjectNote> {
    match preview {
        TextPreview::Undecodable => Some(ObjectNote::TextUndecodable),
        TextPreview::Decoded { lossy: true, .. } => Some(ObjectNote::TextPartlyUndecodable),
        TextPreview::Decoded { lossy: false, .. }
        | TextPreview::Unavailable
        | TextPreview::Empty => None,
    }
}

/// The colour a viewer actually sees for a path, per its paint disposition.
fn visible_colour(style: PaintStyle, fill: Rgb, stroke: Rgb) -> Option<Rgb> {
    if style.fill.is_some() {
        Some(fill)
    } else if style.stroke {
        Some(stroke)
    } else {
        None
    }
}

/// Classify a bounding box's degeneracy, if any.
fn degeneracy_note(bounds: Bounds) -> Option<ObjectNote> {
    if bounds.is_empty() {
        return Some(ObjectNote::NoBounds);
    }
    let zero_w = bounds.max.x - bounds.min.x == 0.0;
    let zero_h = bounds.max.y - bounds.min.y == 0.0;
    match (zero_w, zero_h) {
        (true, true) => Some(ObjectNote::DegenerateBounds(Degeneracy::Point)),
        (true, false) => Some(ObjectNote::DegenerateBounds(Degeneracy::VerticalRule)),
        (false, true) => Some(ObjectNote::DegenerateBounds(Degeneracy::HorizontalRule)),
        (false, false) => None,
    }
}

/// How many of each kind a group of objects contains.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SelectionCensus {
    /// Total objects counted.
    pub total: usize,
    /// Path objects.
    pub paths: usize,
    /// Text objects.
    pub texts: usize,
    /// Inline images and image XObjects, together — the distinction matters
    /// when describing ONE object and is noise in a census.
    pub images: usize,
    /// Form XObjects.
    pub forms: usize,
}

/// Tally a group of objects by kind.
#[must_use]
pub fn census(kinds: impl IntoIterator<Item = ObjectKind>) -> SelectionCensus {
    let mut c = SelectionCensus::default();
    for kind in kinds {
        c.total += 1;
        match kind {
            ObjectKind::Path => c.paths += 1,
            ObjectKind::Text => c.texts += 1,
            ObjectKind::InlineImage | ObjectKind::ImageXObject => c.images += 1,
            ObjectKind::FormXObject => c.forms += 1,
        }
    }
    c
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfcer_core::content::ContentStream;
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
}
