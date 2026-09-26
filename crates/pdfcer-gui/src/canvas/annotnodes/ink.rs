//! # `canvas::annotnodes::ink` — **a freehand mark's points, addressed two
//! ways at once**
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/annotnodes/ink.md`.

use pdfcer_core::edit::InkEdit;
use pdfcer_core::vector::Point;

use crate::canvas::dimdrag::VertexIntent;

/// **Where each stroke of an `/InkList` sits in the flat anchor list.**
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrokeTable {
    /// Points per stroke, in `/InkList` order.
    lengths: Vec<usize>,
}

impl StrokeTable {
    /// Flatten an `/InkList` into the anchor list and the table that indexes
    /// it.
    ///
    /// The points come back in page space (PDF user space, y-up), exactly as
    /// `Annotation::ink_list` holds them; nothing is converted here.
    #[must_use]
    pub fn flatten(strokes: &[Vec<(f64, f64)>]) -> (Vec<Point>, Self) {
        let points = strokes
            .iter()
            .flatten()
            .map(|&(x, y)| Point::new(x, y))
            .collect();
        let lengths = strokes.iter().map(Vec::len).collect();
        (points, Self { lengths })
    }

    /// How many strokes the table describes.
    #[must_use]
    pub fn strokes(&self) -> usize {
        self.lengths.len()
    }

    /// How many anchors in total — the length of the flat list this indexes.
    #[must_use]
    pub fn total(&self) -> usize {
        self.lengths.iter().sum()
    }

    /// The flat index of stroke `stroke`'s first point.
    fn start_of(&self, stroke: usize) -> Option<usize> {
        (stroke < self.lengths.len()).then(|| self.lengths[..stroke].iter().sum())
    }

    /// **Flat anchor index → `(stroke, point)`**, the address the engine's
    /// `InkEdit` variants carry.
    #[must_use]
    pub fn address(&self, flat: usize) -> Option<(usize, usize)> {
        let mut start = 0;
        for (stroke, &len) in self.lengths.iter().enumerate() {
            if flat < start + len {
                return Some((stroke, flat - start));
            }
            start += len;
        }
        None
    }

    /// **`(stroke, point)` → flat anchor index** — the inverse of
    /// [`Self::address`].
    ///
    /// `None` when either half is out of range, so a caller cannot build an
    /// anchor index for a point the engine does not have.
    #[must_use]
    pub fn flat(&self, stroke: usize, point: usize) -> Option<usize> {
        let start = self.start_of(stroke)?;
        (point < self.lengths[stroke]).then_some(start + point)
    }

    /// **The index pairs a preview joins** — every consecutive pair **within**
    /// a stroke, and never a pair that spans two strokes.
    #[must_use]
    pub fn segment_pairs(&self) -> Vec<(usize, usize)> {
        let mut out = Vec::with_capacity(self.total().saturating_sub(self.strokes()));
        let mut start = 0;
        for &len in &self.lengths {
            for i in start..(start + len).saturating_sub(1) {
                out.push((i, i + 1));
            }
            start += len;
        }
        out
    }

    /// The table **after** an edit at flat index `flat`, so the preview's
    /// point list and its stroke boundaries move together.
    #[must_use]
    pub fn after_edit(&self, intent: VertexIntent, flat: usize) -> Option<Self> {
        let (stroke, _) = self.address(flat)?;
        let mut lengths = self.lengths.clone();
        match intent {
            VertexIntent::Move => {}
            VertexIntent::Insert => lengths[stroke] += 1,
            VertexIntent::Remove => lengths[stroke] -= 1,
        }
        Some(Self { lengths })
    }

    /// **The [`InkEdit`] one frame's intent asks the engine for**, addressed
    /// through this table.
    #[must_use]
    pub fn plan(
        &self,
        intent: VertexIntent,
        flat: usize,
        from: Point,
        target: Point,
    ) -> Option<InkEdit> {
        let (stroke, point) = self.address(flat)?;
        Some(match intent {
            VertexIntent::Move => InkEdit::MovePoint {
                stroke,
                point,
                dx: target.x - from.x,
                dy: target.y - from.y,
            },
            VertexIntent::Insert => InkEdit::InsertPoint {
                stroke,
                after: point,
                at: target,
            },
            VertexIntent::Remove => InkEdit::RemovePoint { stroke, point },
        })
    }
}
