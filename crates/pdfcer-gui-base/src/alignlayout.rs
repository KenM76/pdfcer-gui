//! # `alignlayout` — Align and Distribute's arithmetic: boxes in, one delta per box out.
//!
//! The formulas are Inkscape 1.4's, as `docs/reference/inkscape-align-and-distribute.md`
//! records them; `ALIGN_AND_DISTRIBUTE.md` says how the panel uses them.
//!
//! **Frame.** Every box and delta is in canvas space at scale 1: points, y
//! down, the page's `/Rotate` applied. So *min y* is the operator's top.
//! Items arrive in **selection order**, because *first* and *last selected*
//! are positions in that list. The result is indexed like the input, and an
//! item that does not move gets `(0, 0)`.

/// An axis-aligned box in canvas space. `x0 <= x1`, `y0 <= y1`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bx {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
}

impl Bx {
    pub fn new(x0: f64, y0: f64, x1: f64, y1: f64) -> Self {
        Self {
            x0: x0.min(x1),
            y0: y0.min(y1),
            x1: x0.max(x1),
            y1: y0.max(y1),
        }
    }

    /// The smallest box holding both.
    pub fn union(self, o: Bx) -> Bx {
        Bx {
            x0: self.x0.min(o.x0),
            y0: self.y0.min(o.y0),
            x1: self.x1.max(o.x1),
            y1: self.y1.max(o.y1),
        }
    }

    /// `(min, max)` along `axis`.
    pub fn span(self, axis: Axis) -> (f64, f64) {
        match axis {
            Axis::X => (self.x0, self.x1),
            Axis::Y => (self.y0, self.y1),
        }
    }

    /// Extent along `axis`.
    pub fn extent(self, axis: Axis) -> f64 {
        let (a, b) = self.span(axis);
        b - a
    }

    pub fn translated(self, (dx, dy): (f64, f64)) -> Bx {
        Bx {
            x0: self.x0 + dx,
            y0: self.y0 + dy,
            x1: self.x1 + dx,
            y1: self.y1 + dy,
        }
    }
}

/// The axis items move along. The panel's *horizontal* row moves along
/// [`Axis::X`]; its *vertical* row along [`Axis::Y`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    X,
    Y,
}

impl Axis {
    fn other(self) -> Axis {
        match self {
            Axis::X => Axis::Y,
            Axis::Y => Axis::X,
        }
    }

    fn vec(self, d: f64) -> (f64, f64) {
        match self {
            Axis::X => (d, 0.0),
            Axis::Y => (0.0, d),
        }
    }
}

/// Which edge of each item meets which edge of the target.
///
/// On [`Axis::X`] *min* is left; on [`Axis::Y`] it is top.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edge {
    /// Items' min edges to the target's min edge.
    Min,
    /// Items' centres to the target's centre.
    Centre,
    /// Items' max edges to the target's max edge.
    Max,
    /// Items' max edges to the target's min edge — beside it, outside.
    MaxToMin,
    /// Items' min edges to the target's max edge — beside it, outside.
    MinToMax,
}

impl Edge {
    /// `(target weights, item weights)`: a point is `w0·min + w1·max`.
    fn weights(self) -> ((f64, f64), (f64, f64)) {
        match self {
            Edge::Min => ((1.0, 0.0), (1.0, 0.0)),
            Edge::Centre => ((0.5, 0.5), (0.5, 0.5)),
            Edge::Max => ((0.0, 1.0), (0.0, 1.0)),
            Edge::MaxToMin => ((1.0, 0.0), (0.0, 1.0)),
            Edge::MinToMax => ((0.0, 1.0), (1.0, 0.0)),
        }
    }
}

/// What the items are aligned against.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RelativeTo {
    Last,
    First,
    Biggest,
    Smallest,
    Page,
    Drawing,
    #[default]
    Selection,
}

impl RelativeTo {
    /// Every choice, in the panel's order.
    pub const ALL: [Self; 7] = [
        Self::Last,
        Self::First,
        Self::Biggest,
        Self::Smallest,
        Self::Page,
        Self::Drawing,
        Self::Selection,
    ];

    /// The choices offered when one item is selected.
    pub const SINGLE: [Self; 2] = [Self::Page, Self::Drawing];
}

/// The boxes that are not items but can be targets.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Frame {
    /// The page's crop box.
    pub page: Bx,
    /// The union of every content object on the page.
    pub drawing: Bx,
}

/// The target box and the item that owns it, if an item does. That item
/// never moves.
fn target(items: &[Bx], axis: Axis, rel: RelativeTo, frame: Frame) -> Option<(Bx, Option<usize>)> {
    let first = *items.first()?;
    // Biggest and smallest compare the dimension PERPENDICULAR to the move:
    // aligning left edges picks the tallest.
    let size = |b: &Bx| b.extent(axis.other());
    let pick = |better: fn(f64, f64) -> bool| {
        let mut at = 0;
        for (i, b) in items.iter().enumerate() {
            if better(size(b), size(&items[at])) {
                at = i;
            }
        }
        (items[at], Some(at))
    };
    Some(match rel {
        RelativeTo::Last => (items[items.len() - 1], Some(items.len() - 1)),
        RelativeTo::First => (first, Some(0)),
        RelativeTo::Biggest => pick(|a, b| a > b),
        RelativeTo::Smallest => pick(|a, b| a < b),
        RelativeTo::Page => (frame.page, None),
        RelativeTo::Drawing => (frame.drawing, None),
        RelativeTo::Selection => (items.iter().fold(first, |u, b| u.union(*b)), None),
    })
}

/// **Align.** One delta per item, moving along `axis` only.
///
/// With `as_group`, the moving items' union is aligned and each item takes
/// that one delta, so their arrangement is kept.
pub fn align(
    items: &[Bx],
    axis: Axis,
    edge: Edge,
    rel: RelativeTo,
    frame: Frame,
    as_group: bool,
) -> Vec<(f64, f64)> {
    let mut out = vec![(0.0, 0.0); items.len()];
    let Some((t, anchor)) = target(items, axis, rel, frame) else {
        return out;
    };
    let ((m0, m1), (s0, s1)) = edge.weights();
    let (tmin, tmax) = t.span(axis);
    let mp = m0 * tmin + m1 * tmax;
    let point = |b: Bx| {
        let (a, z) = b.span(axis);
        s0 * a + s1 * z
    };
    let moving = |i: &usize| Some(*i) != anchor;
    if as_group {
        let union = (0..items.len())
            .filter(moving)
            .map(|i| items[i])
            .reduce(Bx::union);
        if let Some(u) = union {
            let d = axis.vec(mp - point(u));
            for i in (0..items.len()).filter(moving) {
                out[i] = d;
            }
        }
    } else {
        for i in (0..items.len()).filter(moving) {
            out[i] = axis.vec(mp - point(items[i]));
        }
    }
    out
}

/// **Align text anchors.** `anchors[i]` is item `i`'s baseline origin, or
/// `None` when it is not text; only text moves.
///
/// The reference point is the anchor item's baseline origin when it is text,
/// else the target box's min corner.
pub fn align_text(
    items: &[Bx],
    anchors: &[Option<(f64, f64)>],
    axis: Axis,
    rel: RelativeTo,
    frame: Frame,
) -> Vec<(f64, f64)> {
    let mut out = vec![(0.0, 0.0); items.len()];
    let Some((t, anchor)) = target(items, axis, rel, frame) else {
        return out;
    };
    let reference = anchor
        .and_then(|a| anchors.get(a).copied().flatten())
        .unwrap_or((t.x0, t.y0));
    let coord = |p: (f64, f64)| match axis {
        Axis::X => p.0,
        Axis::Y => p.1,
    };
    for (i, a) in anchors.iter().enumerate().take(items.len()) {
        if Some(i) == anchor {
            continue;
        }
        if let Some(p) = a {
            out[i] = axis.vec(coord(reference) - coord(*p));
        }
    }
    out
}

/// What [`distribute`] spaces evenly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Spacing {
    /// The min edges.
    Min,
    /// The centres.
    Centre,
    /// The max edges.
    Max,
    /// The gaps between neighbours.
    Gaps,
}

/// **Distribute.** The outermost two items stay; the rest are spaced so the
/// chosen measure steps evenly between them. With two items nothing moves.
pub fn distribute(items: &[Bx], axis: Axis, spacing: Spacing) -> Vec<(f64, f64)> {
    let n = items.len();
    let mut out = vec![(0.0, 0.0); n];
    if n < 2 {
        return out;
    }
    let key = |b: &Bx| {
        let (a, z) = b.span(axis);
        match spacing {
            Spacing::Min => a,
            Spacing::Max => z,
            Spacing::Centre | Spacing::Gaps => (a + z) / 2.0,
        }
    };
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&a, &b| key(&items[a]).total_cmp(&key(&items[b])));
    let (first, last) = (items[order[0]], items[order[n - 1]]);
    if spacing == Spacing::Gaps {
        let total: f64 = items.iter().map(|b| b.extent(axis)).sum();
        let step = (last.span(axis).1 - first.span(axis).0 - total) / (n - 1) as f64;
        let mut pos = first.span(axis).0;
        for &i in &order {
            out[i] = axis.vec(pos - items[i].span(axis).0);
            pos += items[i].extent(axis) + step;
        }
    } else {
        let (k0, k1) = (key(&first), key(&last));
        for (rank, &i) in order.iter().enumerate() {
            let want = k0 + rank as f64 * (k1 - k0) / (n - 1) as f64;
            out[i] = axis.vec(want - key(&items[i]));
        }
    }
    out
}

/// **Distribute text anchors.** Among the text items, the lowest and highest
/// baseline origins stay and the rest are spaced evenly between them.
pub fn distribute_text(anchors: &[Option<(f64, f64)>], axis: Axis) -> Vec<(f64, f64)> {
    let mut out = vec![(0.0, 0.0); anchors.len()];
    let coord = |p: (f64, f64)| match axis {
        Axis::X => p.0,
        Axis::Y => p.1,
    };
    let mut text: Vec<(usize, f64)> = anchors
        .iter()
        .enumerate()
        .filter_map(|(i, a)| a.map(|p| (i, coord(p))))
        .collect();
    if text.len() < 3 {
        return out;
    }
    text.sort_by(|a, b| a.1.total_cmp(&b.1));
    let (lo, hi) = (text[0].1, text[text.len() - 1].1);
    let steps = (text.len() - 1) as f64;
    for (rank, &(i, c)) in text.iter().enumerate() {
        out[i] = axis.vec(lo + rank as f64 * (hi - lo) / steps - c);
    }
    out
}

/// Whether a delta is too small to be worth an edit — Inkscape's 1e-9.
pub fn negligible((dx, dy): (f64, f64)) -> bool {
    dx.abs() <= 1e-9 && dy.abs() <= 1e-9
}

#[cfg(test)]
mod tests {
    use super::*;

    const FRAME: Frame = Frame {
        page: Bx {
            x0: 0.0,
            y0: 0.0,
            x1: 600.0,
            y1: 800.0,
        },
        drawing: Bx {
            x0: 10.0,
            y0: 20.0,
            x1: 500.0,
            y1: 700.0,
        },
    };

    fn boxes() -> Vec<Bx> {
        vec![
            Bx::new(100.0, 100.0, 150.0, 130.0), // 50 wide, 30 tall
            Bx::new(200.0, 300.0, 220.0, 400.0), // 20 wide, 100 tall
            Bx::new(50.0, 500.0, 130.0, 510.0),  // 80 wide, 10 tall
        ]
    }

    #[test]
    fn left_edges_to_the_selection_area() {
        let d = align(
            &boxes(),
            Axis::X,
            Edge::Min,
            RelativeTo::Selection,
            FRAME,
            false,
        );
        assert_eq!(d, [(-50.0, 0.0), (-150.0, 0.0), (0.0, 0.0)]);
    }

    #[test]
    fn the_anchor_item_never_moves() {
        let d = align(
            &boxes(),
            Axis::X,
            Edge::Max,
            RelativeTo::First,
            FRAME,
            false,
        );
        assert_eq!(d, [(0.0, 0.0), (-70.0, 0.0), (20.0, 0.0)]);
        let d = align(&boxes(), Axis::Y, Edge::Min, RelativeTo::Last, FRAME, false);
        assert_eq!(d[2], (0.0, 0.0));
        assert_eq!(d[0], (0.0, 400.0));
    }

    #[test]
    fn biggest_compares_the_perpendicular_dimension() {
        // Aligning along X picks the TALLEST (item 1), not the widest (item 2).
        let d = align(
            &boxes(),
            Axis::X,
            Edge::Min,
            RelativeTo::Biggest,
            FRAME,
            false,
        );
        assert_eq!(d[1], (0.0, 0.0));
        assert_eq!(d[0], (100.0, 0.0));
        // Aligning along Y compares widths.
        let d = align(
            &boxes(),
            Axis::Y,
            Edge::Min,
            RelativeTo::Smallest,
            FRAME,
            false,
        );
        assert_eq!(d[1], (0.0, 0.0), "narrowest is item 1");
    }

    #[test]
    fn beside_the_anchor_outside() {
        let d = align(
            &boxes(),
            Axis::X,
            Edge::MinToMax,
            RelativeTo::First,
            FRAME,
            false,
        );
        assert_eq!(d[1], (-50.0, 0.0), "left edge 200 to anchor right 150");
        let d = align(
            &boxes(),
            Axis::X,
            Edge::MaxToMin,
            RelativeTo::First,
            FRAME,
            false,
        );
        assert_eq!(d[2], (-30.0, 0.0), "right edge 130 to anchor left 100");
    }

    #[test]
    fn centre_on_the_page_and_as_a_group() {
        let d = align(
            &boxes(),
            Axis::X,
            Edge::Centre,
            RelativeTo::Page,
            FRAME,
            false,
        );
        assert_eq!(d[0], (175.0, 0.0));
        let g = align(
            &boxes(),
            Axis::X,
            Edge::Centre,
            RelativeTo::Page,
            FRAME,
            true,
        );
        // Union x 50..220, centre 135 → 300.
        assert!(g.iter().all(|&v| v == (165.0, 0.0)));
    }

    #[test]
    fn distribute_centres_keeps_the_ends() {
        let b = vec![
            Bx::new(0.0, 0.0, 10.0, 10.0),
            Bx::new(80.0, 0.0, 90.0, 10.0),
            Bx::new(20.0, 0.0, 30.0, 10.0),
        ];
        let d = distribute(&b, Axis::X, Spacing::Centre);
        assert_eq!(d, [(0.0, 0.0), (0.0, 0.0), (20.0, 0.0)]);
    }

    #[test]
    fn distribute_gaps_equalises_the_space_between() {
        let b = vec![
            Bx::new(0.0, 0.0, 10.0, 10.0),
            Bx::new(12.0, 0.0, 42.0, 10.0),
            Bx::new(90.0, 0.0, 100.0, 10.0),
        ];
        // Span 100, extents 50 → two gaps of 25; the middle starts at 35.
        let d = distribute(&b, Axis::X, Spacing::Gaps);
        assert_eq!(d, [(0.0, 0.0), (23.0, 0.0), (0.0, 0.0)]);
    }

    #[test]
    fn text_anchors_move_only_text() {
        let b = boxes();
        let a = [Some((105.0, 125.0)), None, Some((60.0, 508.0))];
        let d = align_text(&b, &a, Axis::X, RelativeTo::First, FRAME);
        assert_eq!(d, [(0.0, 0.0), (0.0, 0.0), (45.0, 0.0)]);
        // A non-text anchor item gives its box's min corner.
        let d = align_text(&b, &a, Axis::Y, RelativeTo::Page, FRAME);
        assert_eq!(d, [(0.0, -125.0), (0.0, 0.0), (0.0, -508.0)]);
    }

    #[test]
    fn distribute_text_keeps_the_extreme_anchors() {
        let a = [
            Some((0.0, 0.0)),
            None,
            Some((100.0, 0.0)),
            Some((10.0, 0.0)),
        ];
        let d = distribute_text(&a, Axis::X);
        assert_eq!(d, [(0.0, 0.0), (0.0, 0.0), (0.0, 0.0), (40.0, 0.0)]);
    }

    #[test]
    fn nothing_selected_moves_nothing() {
        assert!(align(&[], Axis::X, Edge::Min, RelativeTo::Page, FRAME, false).is_empty());
        assert!(negligible((1e-10, -1e-10)));
    }
}
