//! # `textselection` — A run of selected page text: its two ends in the text model, the edit epoch it was resolved at, and its highlight geometry.

use egui::Rect;
use pdfcer_core::annot_author::Quad;
use pdfcer_core::text_edit::TextPosition;

/// **A range of characters on one page, and everything derived from it.**
#[derive(Debug, Clone, PartialEq)]
pub struct TextSelection {
    /// Which page the range is on. A selection is single-page — module header
    /// §4 — so this is a fact about the whole value rather than about one end.
    pub page: usize,
    /// Where the gesture started. Held so a drag or a Shift+click can extend
    /// **from** it: the anchor is the end the operator is not moving, and
    /// re-deriving it from the quads would be impossible once the focus has
    /// crossed it.
    anchor: TextPosition,
    /// Where the pointer is now. The end a drag moves.
    focus: TextPosition,
    /// The `pdfcer_gui::app::state::OpenDoc::edit_epoch` the positions above were
    /// resolved against. See the module header §7 — this is the whole of the
    /// staleness mechanism.
    epoch: u64,
    /// The selected glyphs' boxes, **in canvas space**, one per line of the
    /// selection.
    ///
    /// Canvas space (Y-down, page top-left, `/Rotate` applied) rather than PDF
    /// user space, and projected once here rather than per frame, for the
    /// reason `crate::find::Hit::canvas` gives for doing the same: page
    /// geometry cannot change while a document is open, so the answer is
    /// constant for the life of the selection, and the paint path becomes a
    /// projection with no PDF concepts in it at all.
    ///
    /// One box per line rather than one per glyph — a hundred adjacent
    /// rectangles paint as one band anyway, and merging them is what lets a
    /// selection over a paragraph cost four boxes instead of four hundred.
    pub quads: Vec<Rect>,
    /// **The same boxes, in PDF user space** — ready to become a text
    /// markup's `/QuadPoints`.
    ///
    /// One entry per entry of [`Self::quads`], in the same order, from the same
    /// accumulation in `resolve`. Not a conversion *of* that field and not a
    /// second walk: the walk produces one `Vec` of PDF-space rectangles and both
    /// of these are built from it, which is what makes *"what is highlighted is
    /// what is marked"* true by construction rather than by two functions
    /// agreeing. Module header §5.1 carries the argument, including why
    /// inverting the canvas projection at the authoring site is the wrong answer
    /// on a rotated page.
    ///
    /// `Quad` rather than `Rect` because that is the type
    /// [`pdfcer_core::annot_author::MarkupSpec::TextMarkup`] takes, and building
    /// it here — once, from the rectangle the glyphs actually produced — leaves
    /// the authoring site with nothing geometric to decide.
    pub page_quads: Vec<Quad>,
    /// **Exactly the characters those boxes cover**, ready for the clipboard.
    ///
    /// Includes the engine's derived word spaces and line breaks, because they
    /// are runs in their own right and the walk passes straight through them —
    /// which is what makes a copied paragraph read as a paragraph rather than
    /// as one unbroken word.
    pub text: String,
}
impl TextSelection {
    /// A selection between `anchor` and `focus`, resolved at `epoch`, with its
    /// highlight geometry already computed by the caller.
    #[must_use]
    pub const fn new(
        page: usize,
        anchor: TextPosition,
        focus: TextPosition,
        epoch: u64,
        quads: Vec<Rect>,
        page_quads: Vec<Quad>,
        text: String,
    ) -> Self {
        Self {
            page,
            anchor,
            focus,
            epoch,
            quads,
            page_quads,
            text,
        }
    }

    /// Where the drag began; the end a shift-click keeps.
    #[must_use]
    pub const fn anchor(&self) -> TextPosition {
        self.anchor
    }

    /// Where the drag is now.
    #[must_use]
    pub const fn focus(&self) -> TextPosition {
        self.focus
    }
    /// Whether this selection still describes the revision it was made
    /// against.
    #[must_use]
    pub fn live(&self, epoch: u64) -> bool {
        self.epoch == epoch
    }

    /// **Which runs of the page's extraction this selection covers**, low
    /// to high, or nothing when the revision has moved.
    #[must_use]
    pub fn runs(&self, epoch: u64) -> Vec<usize> {
        if !self.live(epoch) {
            return Vec::new();
        }
        let (start, end) = ordered(self.anchor, self.focus);
        (start.run..=end.run).collect()
    }

    /// The quads to paint on `page`, or nothing at all.
    #[must_use]
    pub fn highlights(&self, page: usize, epoch: u64) -> &[Rect] {
        if self.page == page && self.live(epoch) {
            &self.quads
        } else {
            &[]
        }
    }

    /// **The quads a text markup would be authored from**, or nothing at all.
    #[must_use]
    pub fn marks(&self, epoch: u64) -> &[Quad] {
        if self.live(epoch) {
            &self.page_quads
        } else {
            &[]
        }
    }

    /// How many characters are selected. For the trace line and for tests.
    #[must_use]
    pub fn len(&self) -> usize {
        self.text.len()
    }

    /// Whether the selection covers nothing.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }
}

/// The two positions in content order.
#[must_use]
pub fn ordered(a: TextPosition, b: TextPosition) -> (TextPosition, TextPosition) {
    if (a.run, a.byte_offset) <= (b.run, b.byte_offset) {
        (a, b)
    } else {
        (b, a)
    }
}
