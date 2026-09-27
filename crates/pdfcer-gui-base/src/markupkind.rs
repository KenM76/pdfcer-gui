//! # `markupkind` — which markup annotation the markup tool draws
//!
//! The kind enum and its gesture predicates; `pdfcer_gui::canvas::markup`
//! turns a kind and a gesture into a `MarkupSpec`.

/// Which markup annotation the markup tool is currently drawing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkupKind {
    /// `/Square` — a rectangle bounded by the drag. *"Drag from one corner to
    /// the other."*
    Rectangle,
    /// `/Circle` — an ellipse inscribed in the drag rectangle. *"Drag out the
    /// box it fits inside."*
    Ellipse,
    /// `/Line` with a head at the far end. *"Drag from the tail to the head."*
    Arrow,
    /// `/PolyLine` — an **open** run of segments. *"Click each corner; double-click
    /// the last."*
    ///
    /// See `pdfcer_gui::canvas::markup::vertex` for the gesture, its two endings, and what "open" costs the
    /// preview.
    PolyLine,
    /// `/Polygon` — the same run of clicks, **closed** back to the first vertex
    /// by the specification rather than by the operator.
    ///
    /// The difference from [`Self::PolyLine`] is *one segment in the file*, which
    /// is why they share a gesture, a state and a commit path and differ only in
    /// `pdfcer_gui::canvas::markup::spec` and in whether `pdfcer_gui::canvas::markup::vertex::preview` draws the closing segment.
    Polygon,
    /// A **revision cloud** — the same run of clicks as [`Self::Polygon`], with
    /// the cloudy border effect on it. *"Click each corner; double-click the
    /// last."*
    ///
    /// # It is a `/Polygon` in the file, and that is the specification's doing
    ///
    /// There is no `/Cloud` subtype in ISO 32000. A revision cloud **is** a
    /// polygon whose border is drawn cloudy — Table 181 declares `/BE` on
    /// Polygon and PolyLine with the qualifier *"meaningful only for polygon
    /// annotations"* — so `pdfcer_core::annot_author::MarkupSpec::Cloud` writes
    /// `/Subtype /Polygon` and differs from `MarkupSpec::Polygon` only by
    /// `/BE << /S /C /I n >>` and by the baked appearance.
    ///
    /// Which is why this is a **seventh kind rather than a Style property of
    /// the sixth**, and the decision is worth stating because the file format
    /// argues the other way. `RIBBON_IA.md` §5.5 gives the revision cloud its
    /// own row in Markup ▸ Shapes, and it is right: an operator drawing a
    /// revision cloud is not drawing a polygon and then styling it, they are
    /// reaching for the one tool this audience names first. A control an
    /// AEC reviewer has to *discover* by styling something else is a control
    /// they will conclude is missing — which is exactly what happened, three
    /// times, in the operator's own words: *"still no revision cloud tool."*
    ///
    /// # What it shares with Polygon, and what it does not
    ///
    /// Everything except `pdfcer_gui::canvas::markup::spec`: the same `pdfcer_gui::canvas::markup::vertex` gesture, the same
    /// three-vertex floor, the same two endings, the same closing segment in
    /// the preview. [`Self::is_vertex`] answers `true` for it and every reader
    /// of that predicate needed no change, which is the evidence that it is the
    /// same gesture rather than a similar one.
    Cloud,
    /// `/Ink` — a freehand stroke that follows the pointer. *"Press and draw."*
    ///
    /// Drag-shaped like the band kinds, and **not** describable by two points:
    /// see `pdfcer_gui::canvas::markup::ink`, which owns the trail, the simplification and the preview.
    Ink,
    /// `/Highlight` — a translucent band over the drag rectangle. *"Drag across
    /// what you want marked."*
    Highlight,
}

impl MarkupKind {
    /// Every variant, in the order the Markup ribbon tab lists them.
    pub const ALL: &'static [MarkupKind] = &[
        MarkupKind::Rectangle,
        MarkupKind::Ellipse,
        MarkupKind::Arrow,
        MarkupKind::PolyLine,
        MarkupKind::Polygon,
        MarkupKind::Cloud,
        MarkupKind::Ink,
        MarkupKind::Highlight,
    ];

    /// Whether this kind is drawn by dragging a bounding **rectangle**, as
    /// opposed to a pair of endpoints.
    #[must_use]
    pub fn is_rect(self) -> bool {
        matches!(self, Self::Rectangle | Self::Ellipse | Self::Highlight)
    }

    /// Whether this kind is gestured by the **two-point rubber band**.
    #[must_use]
    pub fn is_band(self) -> bool {
        matches!(
            self,
            Self::Rectangle | Self::Ellipse | Self::Arrow | Self::Highlight
        )
    }

    /// Whether this kind is gestured by a **run of clicks** — PolyLine,
    /// Polygon and Cloud.
    #[must_use]
    pub fn is_vertex(self) -> bool {
        // Cloud joins here and NOWHERE ELSE in this impl, which is the
        // property that made it a two-line change rather than a feature: every
        // reader of this predicate — `gesture::press_kind`'s live click,
        // `canvas::interact`'s routing away from the selection, `vertex`'s
        // whole state machine — is asking "is this a run of clicks", and the
        // cloud is one. The difference lives entirely in `spec`.
        matches!(self, Self::PolyLine | Self::Polygon | Self::Cloud)
    }

    /// Whether this kind follows the pointer freehand — Ink, and only Ink.
    #[must_use]
    pub fn is_freehand(self) -> bool {
        matches!(self, Self::Ink)
    }
}

/// **The geometry one completed markup gesture produced**, in PDF user space.
#[derive(Debug, Clone, PartialEq)]
pub enum Geometry {
    /// The two **raw** endpoints of a rubber-band drag, in drag order.
    ///
    /// Un-normalised on purpose: `pdfcer_gui::canvas::markup::spec` normalises per kind, at the last point
    /// at which the raw pair is still available, because an arrow's head is at
    /// `end` and a normalised rect cannot say which corner the operator started
    /// at. See `pdfcer_gui::canvas::markup::spec`'s own section.
    Band {
        /// Where the press landed. For [`MarkupKind::Arrow`] this is the **tail**.
        start: (f64, f64),
        /// Where the release landed. For [`MarkupKind::Arrow`] this is the **head**.
        end: (f64, f64),
    },
    /// A run of clicked vertices, in click order — PolyLine and Polygon.
    ///
    /// **Never carries the closing vertex for a polygon.** `/Polygon` closes
    /// back to the first entry of `/Vertices` by §12.5.6.13, so appending the
    /// first point again would author a duplicate vertex and a zero-length
    /// closing segment — visible on a rounded join as a blob, and invisible
    /// everywhere else, which is the worst of both.
    Vertices(Vec<(f64, f64)>),
    /// One or more freehand strokes — Ink, and only Ink.
    ///
    /// A list of lists because `/InkList` is one, even though the shipped
    /// gesture always produces exactly one stroke: `pdfcer_gui::canvas::markup::ink`'s header records that
    /// **one drag is one annotation**, and the outer list is the engine's shape
    /// rather than a promise about a gesture that does not exist yet.
    Strokes(Vec<Vec<(f64, f64)>>),
}
impl Geometry {
    /// Every coordinate this geometry carries, in no particular order.
    pub fn coordinates(&self) -> impl Iterator<Item = f64> + '_ {
        // A boxed iterator rather than three branches at the call site: the arms
        // have three different concrete types and the alternative is repeating
        // the predicate per arm, which is the thing this exists to avoid.
        let it: Box<dyn Iterator<Item = f64> + '_> = match self {
            Self::Band { start, end } => Box::new([start.0, start.1, end.0, end.1].into_iter()),
            Self::Vertices(points) => Box::new(points.iter().flat_map(|&(x, y)| [x, y])),
            Self::Strokes(strokes) => Box::new(
                strokes
                    .iter()
                    .flat_map(|s| s.iter().flat_map(|&(x, y)| [x, y])),
            ),
        };
        it
    }
}
/// Which of the three selection-marking subtypes a command authors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextMarkKind {
    /// `/Highlight` — a translucent wash over each quad.
    ///
    /// The one kind in this enum that is *also* a [`MarkupKind`] — the
    /// armed tool that draws an area highlight by dragging a box.
    ///
    /// That is not a duplication, and the reason is what these two enums
    /// actually encode: **not identity, but GEOMETRY.** `MarkupKind` is *"kinds
    /// whose operand is a shape the pointer draws"*; this one is *"kinds whose
    /// operand is a run of text"*. Highlight is the only kind that is honestly
    /// both, because a highlight over text follows the lines and a highlight
    /// over a scan is an area — and Acrobat's own tool does exactly that.
    ///
    /// ⇒ A kind reachable by two gestures needs an entry in both tables. The
    /// alternative — one enum with a geometry field — would put a branch in
    /// every arm that today cannot be wrong, to express a thing that is true of
    /// one variant.
    ///
    /// It takes the **highlighter**, not the ink, unlike the other three —
    /// see [`Self::rgb`]. Same instrument, same swatch, whichever gesture
    /// reached it.
    Highlight,
    /// `/Underline` — a line near each quad's baseline.
    Underline,
    /// `/StrikeOut` — a line through each quad's vertical middle.
    StrikeOut,
    /// `/Squiggly` — a wavy line at each quad's baseline.
    ///
    /// pdfcer authors this natively **even though Acrobat's own UI does not** —
    /// a deliberate exceed-Acrobat choice recorded in `pdfcer-core`'s
    /// `TextMarkupKind::Squiggly`: the subtype is fully spec-legal (§12.5.6.10)
    /// and Acrobat displays it. It is offered here for that reason and not by
    /// oversight; the standing instruction is to *match* the reference
    /// applications, and matching does not mean declining something the engine
    /// already writes correctly.
    Squiggly,
}
impl TextMarkKind {
    /// Every variant, in the order the Markup ribbon lists them.
    pub const ALL: &'static [TextMarkKind] = &[
        TextMarkKind::Underline,
        TextMarkKind::StrikeOut,
        TextMarkKind::Squiggly,
    ];

    /// The `pdfcer-core` subtype this kind authors.
    #[must_use]
    pub fn subtype(self) -> pdfcer_core::annot_author::TextMarkupKind {
        match self {
            Self::Highlight => pdfcer_core::annot_author::TextMarkupKind::Highlight,
            Self::Underline => pdfcer_core::annot_author::TextMarkupKind::Underline,
            Self::StrikeOut => pdfcer_core::annot_author::TextMarkupKind::StrikeOut,
            Self::Squiggly => pdfcer_core::annot_author::TextMarkupKind::Squiggly,
        }
    }

    /// **EACH OF THE FOUR TAKES ITS OWN PEN.**
    ///
    /// A two-instrument partition — lines take the biro, the wash takes the
    /// marker — answers *"which of these two?"* correctly and is still the
    /// intuition a reader arrives with. It cannot answer the question the
    /// operator actually asked, which is *"which colour does Adobe use?"*:
    ///
    /// | kind | Acrobat key | measured |
    /// |---|---|---|
    /// | Highlight | `cHighlight` | `#FF6200`, an **orange** |
    /// | Underline | `cUnderline` | `#1373E8`, a **blue** |
    /// | StrikeOut | `cStrikeOut` | `#F86464`, a light red |
    /// | Squiggly | `cSquiggly` | `#DB3425`, the shape red |
    ///
    /// ⇒ Four kinds, four keys, three distinct colours. A partition into "biro"
    /// and "marker" cannot express that, so the pen carries a slot per key and
    /// this function is a routing table rather than a two-arm decision. See
    /// [`crate::markuppalette`] for where those four readings come from and
    /// [`crate::markuppen::PenSlot`] for the slots.
    ///
    /// The property a partition gave for free — **no kind reaching two
    /// colours and no kind reaching none** — is asserted instead, from both
    /// sides: `pen::tests::every_kind_takes_the_slot_it_is_documented_to_take`
    /// and `pdfcer_gui::canvas::markup::text::tests::each_text_kind_takes_its_own_pen`, each sweeping its enum's
    /// full list rather than a hand-written subset.
    ///
    /// # Why this is a routing table and NOT a hard-coded triple
    ///
    /// A constant here — `fn rgb(self) -> (f64, f64, f64)` returning one red for
    /// every line kind — compiles, and a test that asserts the constant against
    /// the constant passes, and the pen control in Markup ▸ Style goes on
    /// working perfectly for every kind that reaches `pdfcer_gui::canvas::markup::spec`. Nothing is
    /// red anywhere. What the operator sees is the inconsistency: set the pen to
    /// blue, draw a rectangle, get blue; underline a word, get red.
    ///
    /// The generalisable part is not "remember to update duplicates". It is that
    /// **a doc comment naming its own seam is an asset only if something checks
    /// the seam when it is filled.** A prose seam marker is a note to a human,
    /// and a sweep reading it will record *"no surface"* long after the surface
    /// exists. What catches it is a test asserting the two paths agree, which is
    /// `tests::the_ink_reaches_every_text_kind`.
    ///
    /// # The constraint the measurement has to pass
    ///
    /// *"A line must be seen against the text it marks; a yellow underline under
    /// black glyphs on white paper is very nearly invisible."* That is a **check
    /// on the measurement rather than a reason for a value**: Acrobat's
    /// underline blue and strikeout red both pass it comfortably, which is
    /// evidence that the registry readings are a designed set rather than an
    /// accident of this machine.
    ///
    /// # Why an operator's Highlight comes out of one swatch
    ///
    /// It is the reason [`Self::Highlight`] and
    /// [`MarkupKind::Highlight`] both route to
    /// [`crate::markuppen::PenSlot::Highlighter`]: a highlight is a wash whichever
    /// gesture drew it, so a text-following one and an area one must come out of
    /// the same swatch. The alternative is one feature that changes colour
    /// depending on how it was reached.
    #[must_use]
    pub fn rgb(self, pen: crate::markuppen::Pen) -> (f64, f64, f64) {
        pen.colour_of(self.slot())
    }

    /// **Which pen draws this kind** — see [`Self::rgb`] for the table.
    #[must_use]
    pub const fn slot(self) -> crate::markuppen::PenSlot {
        match self {
            Self::Highlight => crate::markuppen::PenSlot::Highlighter,
            Self::Underline => crate::markuppen::PenSlot::Underline,
            Self::StrikeOut => crate::markuppen::PenSlot::StrikeOut,
            Self::Squiggly => crate::markuppen::PenSlot::Squiggly,
        }
    }
}
