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
