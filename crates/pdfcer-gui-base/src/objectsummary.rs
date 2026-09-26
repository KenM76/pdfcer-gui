//! # `objectsummary` — the ONE description of a page object
//!
//! Turns a `pdfcer_core::vector::VectorObject` into a small, GUI-shaped
//! **fact record** ([`ObjectSummary`]) that every surface which has to say
//! *"what is this thing?"* reads.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/objectsummary.md`.

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
#[doc(hidden)]
pub fn degeneracy_note(bounds: Bounds) -> Option<ObjectNote> {
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
