//! Grid and circular arrangement arithmetic, ported from Inkscape 1.4's
//! Align-and-Distribute "Grid" and "Circular" tabs.
//!
//! Contract: every function is pure. Boxes are in canvas space, in points,
//! y down (min y is the top), in selection order; every result is indexed
//! like the input. Moves are translations `(dx, dy)` to add to the item.

use super::{Axis, Bx};

/// Two tops within this distance count as the same top band when finding a
/// grid row. Inkscape's constant, in canvas units.
const TOP_BAND_TOLERANCE: f64 = 2.0;

/// An arc whose length differs from a full turn by more than this, in
/// radians, is partial. Inkscape's constant.
const FULL_TURN_TOLERANCE: f64 = 0.01;

/// Radial tolerance for [`ellipse_from_points`], as a fraction of the larger
/// radius.
const ELLIPSE_TOLERANCE: f64 = 0.01;

// ---------------------------------------------------------------- grid ----

/// How the gaps between grid cells are chosen.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GridSpacing {
    /// Spread the cells so the grid fills the selection box.
    Fit,
    /// Fixed gaps between columns (`x`) and rows (`y`), in points; may be
    /// negative. Inkscape's default is 15 px each.
    Set { x: f64, y: f64 },
}

/// The Grid tab's settings.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GridParams {
    /// Requested rows. [`grid`] re-derives the pair from `rows` exactly as
    /// Inkscape does before arranging; see [`settle_rows_cols`].
    pub rows: usize,
    /// Requested columns; superseded by `rows` (see [`settle_rows_cols`]).
    pub cols: usize,
    /// Every row as tall as the tallest item overall.
    pub equal_height: bool,
    /// Every column as wide as the widest item overall.
    pub equal_width: bool,
    /// `(column, row)` of the anchor inside each cell, each `0..=2`:
    /// 0 left/top, 1 centre, 2 right/bottom. `(1, 1)` centres. Values above
    /// 2 are treated as 2.
    pub anchor: (u8, u8),
    pub spacing: GridSpacing,
}

/// Rows and columns for a fresh selection of `n`: both `ceil(√n)`, at
/// least 1.
#[must_use]
pub fn default_rows_cols(n: usize) -> (usize, usize) {
    let mut k = 1;
    while k * k < n {
        k += 1;
    }
    (k, k)
}

/// Columns needed for `n` items in `rows` rows: `ceil(n / rows)`, at least
/// 1. `rows` of 0 is treated as 1.
#[must_use]
pub fn cols_for_rows(n: usize, rows: usize) -> usize {
    n.div_ceil(rows.max(1)).max(1)
}

/// Rows needed for `n` items in `cols` columns: `ceil(n / cols)`, at least
/// 1. `cols` of 0 is treated as 1.
#[must_use]
pub fn rows_for_cols(n: usize, cols: usize) -> usize {
    n.div_ceil(cols.max(1)).max(1)
}

/// The `(rows, cols)` [`grid`] actually uses for `n` items.
///
/// Inkscape's Arrange re-runs both spinner handlers first: columns from
/// rows, then rows from those columns. So `rows` wins and `cols` is only
/// a request; the result always holds `n` items with no empty row.
#[must_use]
pub fn settle_rows_cols(n: usize, rows: usize, _cols: usize) -> (usize, usize) {
    let cols = cols_for_rows(n, rows);
    (rows_for_cols(n, cols), cols)
}

/// Reading order of an existing, roughly grid-shaped layout: indices into
/// `boxes`, row by row, each row left to right.
///
/// Per row: among the items whose top is within 2 units of the topmost
/// top, the tallest one's vertical midpoint is the row line; the row is
/// every item whose box strictly straddles that line (`y0 < line < y1`),
/// sorted by left edge. Repeat on the rest.
///
/// Ties in left edge keep selection order (Inkscape's sort is unstable).
/// Where no item straddles the line (all zero-height at the top), the row is
/// the top band itself; Inkscape instead warns and drops the remaining
/// items.
#[must_use]
pub fn spatial_order(boxes: &[Bx]) -> Vec<usize> {
    let mut rest: Vec<usize> = (0..boxes.len()).collect();
    let mut out = Vec::with_capacity(boxes.len());
    while !rest.is_empty() {
        let top = rest
            .iter()
            .map(|&i| boxes[i].y0)
            .fold(f64::INFINITY, f64::min);
        let in_band = |i: usize| (boxes[i].y0 - top).abs() <= TOP_BAND_TOLERANCE;
        let mut line = top;
        let mut tallest = 0.0;
        for &i in &rest {
            let h = boxes[i].extent(Axis::Y);
            if in_band(i) && h > tallest {
                tallest = h;
                line = f64::midpoint(boxes[i].y0, boxes[i].y1);
            }
        }
        let (mut row, mut left): (Vec<usize>, Vec<usize>) = rest
            .iter()
            .partition(|&&i| boxes[i].y0 < line && boxes[i].y1 > line);
        if row.is_empty() {
            (row, left) = rest.iter().partition(|&&i| in_band(i));
        }
        row.sort_by(|&a, &b| boxes[a].x0.total_cmp(&boxes[b].x0));
        out.extend(row);
        rest = left;
    }
    out
}

/// Translations placing `boxes` in a grid (Inkscape "Arrange in a grid").
///
/// Items fill cells in [`spatial_order`], row-major, in the
/// [`settle_rows_cols`] shape. A column is as wide as its widest item and a
/// row as tall as its tallest, or all as the overall maximum when Equal.
/// The grid's origin is the selection box's top-left, shifted so the
/// first row's tallest (first column's widest) item keeps its top (left)
/// when Equal widens that cell. Each item sits in its cell by `anchor`.
///
/// `Fit` gap = `(selection extent − total cells + last_pad) / (count − 1)`,
/// where `last_pad` is half the slack of the last column (row) under Equal,
/// else 0; a single column (row) gets gap 0.
#[must_use]
pub fn grid(boxes: &[Bx], params: &GridParams) -> Vec<(f64, f64)> {
    if boxes.is_empty() {
        return Vec::new();
    }
    let (rows, cols) = settle_rows_cols(boxes.len(), params.rows, params.cols);
    let sel = boxes.iter().copied().reduce(Bx::union).unwrap_or(boxes[0]);
    let order = spatial_order(boxes);
    let h_align = f64::from(params.anchor.0.min(2));
    let v_align = f64::from(params.anchor.1.min(2));

    let mut col_w = vec![0.0_f64; cols];
    let mut row_h = vec![0.0_f64; rows];
    for (slot, &item) in order.iter().enumerate() {
        col_w[slot % cols] = col_w[slot % cols].max(boxes[item].extent(Axis::X));
        row_h[slot / cols] = row_h[slot / cols].max(boxes[item].extent(Axis::Y));
    }
    let across = Track::new(
        col_w,
        sel.x0,
        sel.extent(Axis::X),
        params.equal_width,
        h_align,
    );
    let down = Track::new(
        row_h,
        sel.y0,
        sel.extent(Axis::Y),
        params.equal_height,
        v_align,
    );
    let (gap_x, gap_y) = match params.spacing {
        GridSpacing::Set { x, y } => (x, y),
        GridSpacing::Fit => (across.fit_gap(), down.fit_gap()),
    };
    let col_x = across.offsets(gap_x);
    let row_y = down.offsets(gap_y);

    let mut out = vec![(0.0, 0.0); boxes.len()];
    for (slot, &item) in order.iter().enumerate() {
        let (bx, col, row) = (boxes[item], slot % cols, slot / cols);
        let slack_x = across.sizes[col] - bx.extent(Axis::X);
        let slack_y = down.sizes[row] - bx.extent(Axis::Y);
        let new_x = across.origin + slack_x / 2.0 * h_align + col_x[col];
        let new_y = down.origin + slack_y / 2.0 * v_align + row_y[row];
        out[item] = (new_x - bx.x0, new_y - bx.y0);
    }
    out
}

/// One axis of the grid: cell sizes, origin and the numbers `Fit` needs.
struct Track {
    sizes: Vec<f64>,
    origin: f64,
    sel_extent: f64,
    total: f64,
    last_pad: f64,
}

impl Track {
    fn new(sizes: Vec<f64>, sel_min: f64, sel_extent: f64, equal: bool, align: f64) -> Self {
        let max = sizes.iter().copied().fold(0.0, f64::max);
        let count = sizes.len();
        if equal {
            Self {
                origin: sel_min - (max - sizes[0]) / 2.0 * align,
                last_pad: (max - sizes[count - 1]) / 2.0,
                total: max * count_f64(count),
                sizes: vec![max; count],
                sel_extent,
            }
        } else {
            Self {
                origin: sel_min,
                last_pad: 0.0,
                total: sizes.iter().sum(),
                sizes,
                sel_extent,
            }
        }
    }

    fn fit_gap(&self) -> f64 {
        let count = self.sizes.len();
        if count < 2 {
            return 0.0;
        }
        (self.sel_extent - self.total + self.last_pad) / count_f64(count - 1)
    }

    /// Start of each cell relative to `origin`.
    fn offsets(&self, gap: f64) -> Vec<f64> {
        let mut at = 0.0;
        self.sizes
            .iter()
            .map(|s| {
                let here = at;
                at += s + gap;
                here
            })
            .collect()
    }
}

#[allow(clippy::cast_precision_loss, reason = "item counts are far below 2^52")] // ui-text-exempt: lint reason
fn count_f64(n: usize) -> f64 {
    n as f64
}

// ------------------------------------------------------------ circular ----

/// The point of each item that lands on the ellipse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircleAnchor {
    /// `(column, row)` on the item's box, each `0..=2` as in
    /// [`GridParams::anchor`].
    BoxPoint(u8, u8),
    /// The item's rotational centre, taken to be its box centre.
    Centre,
}

/// An axis-aligned ellipse or arc in canvas space (points, y down).
///
/// Angles are in degrees, measured from +x toward +y, so on a y-down
/// canvas they run clockwise on screen: 90° is straight below the centre.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ellipse {
    pub cx: f64,
    pub cy: f64,
    pub rx: f64,
    pub ry: f64,
    pub start_deg: f64,
    pub end_deg: f64,
}

impl Ellipse {
    /// The Parameterized defaults: centre (0, 0), radii 100, 0°–180°.
    pub const DEFAULT: Self = Self {
        cx: 0.0,
        cy: 0.0,
        rx: 100.0,
        ry: 100.0,
        start_deg: 0.0,
        end_deg: 180.0,
    };
}

/// How one item moves: translate by `(dx, dy)`, then rotate by
/// `rotate_rad` about the translated box's centre.
///
/// `rotate_rad` is in the canvas frame: positive turns +x toward +y, which
/// on a y-down canvas is clockwise on screen.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placement {
    pub dx: f64,
    pub dy: f64,
    pub rotate_rad: f64,
}

impl Placement {
    const STAY: Self = Self {
        dx: 0.0,
        dy: 0.0,
        rotate_rad: 0.0,
    };
}

/// Placements arranging `boxes` along `ellipse` (Inkscape "Arrange on
/// ellipse").
///
/// Items other than `skip` take slots in selection order. With `count`
/// such items, item `i` goes to angle `start + i/slots · (end − start)`,
/// where `slots = count` on a full turn and `count − 1` on a partial arc so
/// both ends are used; its anchor lands on `(cx + rx·cos, cy + ry·sin)`.
/// When `slots` is 0 (one item on a partial arc) the item goes to the
/// start angle.
///
/// With `rotate`, each item turns so its top faces away from the centre:
/// Inkscape's `−atan2(−(px − cx), −(py − cy))`, pivoting on the placed
/// anchor. The pivot is folded into `dx, dy` so that rotating about the
/// box centre, as [`Placement`] specifies, gives the same result.
///
/// `skip` (the reference ellipse item) gets a zero placement and does not
/// take a slot; an out-of-range `skip` is ignored.
#[must_use]
pub fn circular(
    boxes: &[Bx],
    ellipse: Ellipse,
    anchor: CircleAnchor,
    rotate: bool,
    skip: Option<usize>,
) -> Vec<Placement> {
    let skip = skip.filter(|&s| s < boxes.len());
    let count = boxes.len() - usize::from(skip.is_some());
    let begin = ellipse.start_deg.to_radians();
    let arc = ellipse.end_deg.to_radians() - begin;
    let partial = (arc.abs() - std::f64::consts::TAU).abs() > FULL_TURN_TOLERANCE;
    let slots = if partial {
        count.saturating_sub(1)
    } else {
        count
    };

    let mut out = Vec::with_capacity(boxes.len());
    let mut slot = 0;
    for (idx, bx) in boxes.iter().enumerate() {
        if Some(idx) == skip {
            out.push(Placement::STAY);
            continue;
        }
        let angle = if slots == 0 {
            begin
        } else {
            begin + count_f64(slot) / count_f64(slots) * arc
        };
        slot += 1;
        let (tx, ty) = (
            ellipse.cx + ellipse.rx * angle.cos(),
            ellipse.cy + ellipse.ry * angle.sin(),
        );
        let (ax, ay) = anchor_point(*bx, anchor);
        let (mut dx, mut dy) = (tx - ax, ty - ay);
        let mut rotate_rad = 0.0;
        if rotate {
            rotate_rad = -f64::atan2(-(tx - ellipse.cx), -(ty - ellipse.cy));
            // Rotating about the target T equals rotating about the moved
            // box centre C after an extra shift of (T − C) − R(T − C).
            let arm_x = tx - (f64::midpoint(bx.x0, bx.x1) + dx);
            let arm_y = ty - (f64::midpoint(bx.y0, bx.y1) + dy);
            let (sin, cos) = rotate_rad.sin_cos();
            dx += arm_x - (arm_x * cos - arm_y * sin);
            dy += arm_y - (arm_x * sin + arm_y * cos);
        }
        out.push(Placement { dx, dy, rotate_rad });
    }
    out
}

fn anchor_point(b: Bx, anchor: CircleAnchor) -> (f64, f64) {
    let (h, v) = match anchor {
        CircleAnchor::BoxPoint(h, v) => (f64::from(h.min(2)), f64::from(v.min(2))),
        CircleAnchor::Centre => (1.0, 1.0),
    };
    (
        b.x0 + (b.x1 - b.x0) * h / 2.0,
        b.y0 + (b.y1 - b.y0) * v / 2.0,
    )
}

// -------------------------------------------------- ellipse detection ----

/// The full axis-aligned ellipse a closed cubic Bézier chain draws, if it
/// draws one.
///
/// `points` is the chain `[a0, c, c, a1, c, c, a2, …, aK]` in canvas space:
/// anchors at every third index, two control points between each pair, at
/// least two segments (`len = 3K + 1`, `K ≥ 2`). The centre and radii come
/// from the anchors' bounding box. Accepted when every anchor and every
/// segment's `t = ¼, ½, ¾` point lies within 1% of the larger radius of
/// that ellipse, measured radially. Checking the curve points, not just
/// the anchors, rejects polygons whose corners sit on the ellipse (a
/// diamond); a square is rejected anyway, since its corners are the bbox
/// corners, √2 radii out. The result spans 0°–360°; partial arcs are not
/// recognised.
#[must_use]
pub fn ellipse_from_points(points: &[(f64, f64)]) -> Option<Ellipse> {
    if points.len() < 7 || points.len() % 3 != 1 {
        return None;
    }
    let anchors = || points.iter().step_by(3).copied();
    let bbox = anchors()
        .map(|(x, y)| Bx::new(x, y, x, y))
        .reduce(Bx::union)?;
    let (rx, ry) = (bbox.extent(Axis::X) / 2.0, bbox.extent(Axis::Y) / 2.0);
    let usable = rx.is_finite() && ry.is_finite() && rx > 0.0 && ry > 0.0;
    if !usable {
        return None;
    }
    let e = Ellipse {
        cx: f64::midpoint(bbox.x0, bbox.x1),
        cy: f64::midpoint(bbox.y0, bbox.y1),
        rx,
        ry,
        start_deg: 0.0,
        end_deg: 360.0,
    };
    let tol = ELLIPSE_TOLERANCE * rx.max(ry);
    let on = |q: (f64, f64)| radial_error(&e, q) <= tol;
    let curves_on = points.windows(4).step_by(3).all(|s| {
        [0.25, 0.5, 0.75]
            .into_iter()
            .all(|t| on(cubic_at(s[0], s[1], s[2], s[3], t)))
    });
    (anchors().all(on) && curves_on).then_some(e)
}

/// Distance from `q` to where the ray from the centre through `q` meets the
/// ellipse.
fn radial_error(e: &Ellipse, q: (f64, f64)) -> f64 {
    let (u, v) = (q.0 - e.cx, q.1 - e.cy);
    let rho = ((u / e.rx).powi(2) + (v / e.ry).powi(2)).sqrt();
    if rho == 0.0 {
        return e.rx.min(e.ry);
    }
    u.hypot(v) * (1.0 - 1.0 / rho).abs()
}

#[allow(clippy::many_single_char_names, reason = "Bernstein-form cubic")] // ui-text-exempt: lint reason
fn cubic_at(p0: (f64, f64), p1: (f64, f64), p2: (f64, f64), p3: (f64, f64), t: f64) -> (f64, f64) {
    let s = 1.0 - t;
    let (a, b, c, d) = (s * s * s, 3.0 * s * s * t, 3.0 * s * t * t, t * t * t);
    (
        a * p0.0 + b * p1.0 + c * p2.0 + d * p3.0,
        a * p0.1 + b * p1.1 + c * p2.1 + d * p3.1,
    )
}

// --------------------------------------------------------------- tests ----

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::{FRAC_PI_2, PI};

    const EPS: f64 = 1e-9;

    fn near(a: f64, b: f64) -> bool {
        (a - b).abs() < EPS
    }

    #[track_caller]
    fn assert_pts(got: &[(f64, f64)], want: &[(f64, f64)]) {
        assert_eq!(got.len(), want.len());
        for (g, w) in got.iter().zip(want) {
            assert!(
                near(g.0, w.0) && near(g.1, w.1),
                "got {got:?}, want {want:?}"
            );
        }
    }

    /// New top-left corners after applying `grid`'s moves.
    fn corners(boxes: &[Bx], p: &GridParams) -> Vec<(f64, f64)> {
        grid(boxes, p)
            .into_iter()
            .zip(boxes)
            .map(|(d, b)| (b.x0 + d.0, b.y0 + d.1))
            .collect()
    }

    /// A (10×12), B (20×20) on top; C (10×6), D (5×10) below.
    fn four() -> [Bx; 4] {
        [
            Bx::new(0.0, 0.0, 10.0, 12.0),
            Bx::new(50.0, 0.0, 70.0, 20.0),
            Bx::new(0.0, 40.0, 10.0, 46.0),
            Bx::new(50.0, 40.0, 55.0, 50.0),
        ]
    }

    fn params(anchor: (u8, u8), spacing: GridSpacing, eq_w: bool, eq_h: bool) -> GridParams {
        GridParams {
            rows: 2,
            cols: 2,
            equal_height: eq_h,
            equal_width: eq_w,
            anchor,
            spacing,
        }
    }

    const SET: GridSpacing = GridSpacing::Set { x: 5.0, y: 7.0 };

    #[test]
    fn rows_cols_helpers() {
        assert_eq!(default_rows_cols(0), (1, 1));
        assert_eq!(default_rows_cols(1), (1, 1));
        assert_eq!(default_rows_cols(4), (2, 2));
        assert_eq!(default_rows_cols(5), (3, 3));
        assert_eq!(default_rows_cols(10), (4, 4));
        assert_eq!(cols_for_rows(7, 2), 4);
        assert_eq!(cols_for_rows(7, 0), 7);
        assert_eq!(rows_for_cols(7, 3), 3);
        assert_eq!(rows_for_cols(0, 3), 1);
        // Rows win; 10 rows for 7 items settles to 7×1.
        assert_eq!(settle_rows_cols(7, 3, 9), (3, 3));
        assert_eq!(settle_rows_cols(7, 10, 1), (7, 1));
        assert_eq!(settle_rows_cols(5, 4, 1), (3, 2));
    }

    #[test]
    fn spatial_order_of_shuffled_3x2() {
        // Row 1 at y≈0 with jitter, row 2 at y≈30; shuffled input.
        let boxes = [
            Bx::new(40.0, 31.0, 50.0, 41.0), // r2 c3
            Bx::new(20.0, 1.0, 30.0, 11.0),  // r1 c2
            Bx::new(0.0, 30.0, 10.0, 40.0),  // r2 c1
            Bx::new(40.0, 0.0, 50.0, 14.0),  // r1 c3, tallest of top band
            Bx::new(20.0, 29.0, 30.0, 38.0), // r2 c2
            Bx::new(0.0, 2.0, 10.0, 10.0),   // r1 c1
        ];
        assert_eq!(spatial_order(&boxes), vec![5, 1, 3, 2, 4, 0]);
    }

    #[test]
    fn spatial_order_requires_strict_straddle() {
        // The row line is B's midpoint y=10; A ends exactly there, so A
        // starts the next row (Inkscape's strict test).
        let boxes = [
            Bx::new(0.0, 0.0, 10.0, 10.0),
            Bx::new(20.0, 0.0, 30.0, 20.0),
        ];
        assert_eq!(spatial_order(&boxes), vec![1, 0]);
    }

    #[test]
    fn spatial_order_ties_and_flat_items() {
        let same_x = [Bx::new(0.0, 0.0, 10.0, 10.0), Bx::new(0.0, 1.0, 5.0, 9.0)];
        assert_eq!(spatial_order(&same_x), vec![0, 1]);
        // Zero-height items never straddle: fall back to the top band.
        let flat = [
            Bx::new(30.0, 0.0, 40.0, 0.0),
            Bx::new(0.0, 1.0, 10.0, 1.0),
            Bx::new(0.0, 20.0, 10.0, 20.0),
        ];
        assert_eq!(spatial_order(&flat), vec![1, 0, 2]);
        assert!(spatial_order(&[]).is_empty());
    }

    #[test]
    fn grid_set_spacing_top_left() {
        let got = corners(&four(), &params((0, 0), SET, false, false));
        // Columns 10 and 20 wide, rows 20 and 10 tall.
        assert_pts(&got, &[(0.0, 0.0), (15.0, 0.0), (0.0, 27.0), (15.0, 27.0)]);
    }

    #[test]
    fn grid_set_spacing_centre_and_bottom_right() {
        let b = four();
        let c = corners(&b, &params((1, 1), SET, false, false));
        assert_pts(&c, &[(0.0, 4.0), (15.0, 0.0), (0.0, 29.0), (22.5, 27.0)]);
        let br = corners(&b, &params((2, 2), SET, false, false));
        assert_pts(&br, &[(0.0, 8.0), (15.0, 0.0), (0.0, 31.0), (30.0, 27.0)]);
        let tr = corners(&b, &params((2, 0), SET, false, false));
        assert_pts(&tr, &[(0.0, 0.0), (15.0, 0.0), (0.0, 27.0), (30.0, 27.0)]);
    }

    #[test]
    fn grid_follows_spatial_order_not_selection_order() {
        let b = four();
        let shuffled = [b[3], b[0], b[2], b[1]];
        let got = corners(&shuffled, &params((0, 0), SET, false, false));
        assert_pts(&got, &[(15.0, 27.0), (0.0, 0.0), (0.0, 27.0), (15.0, 0.0)]);
    }

    #[test]
    fn grid_equal_width_keeps_first_column_left() {
        let got = corners(&four(), &params((1, 1), SET, true, false));
        // Columns 20 wide from x = −5, so A stays at x = 0.
        assert_pts(&got, &[(0.0, 4.0), (20.0, 0.0), (0.0, 29.0), (27.5, 27.0)]);
    }

    #[test]
    fn grid_equal_height() {
        let got = corners(&four(), &params((0, 2), SET, false, true));
        // Rows 20 tall; origin y = 0 − (20 − 20)/2·2 = 0.
        assert_pts(&got, &[(0.0, 8.0), (15.0, 0.0), (0.0, 41.0), (15.0, 37.0)]);
    }

    #[test]
    fn grid_fit_fills_selection_box() {
        let b = four();
        let got = corners(&b, &params((0, 0), GridSpacing::Fit, false, false));
        // Gaps (70 − 30)/1 = 40 and (50 − 30)/1 = 20.
        assert_pts(&got, &[(0.0, 0.0), (50.0, 0.0), (0.0, 40.0), (50.0, 40.0)]);
        // B's right edge and D's cell bottom meet the selection box.
        assert!(near(got[1].0 + 20.0, 70.0));
        assert!(near(got[3].1 + 10.0, 50.0));
    }

    #[test]
    fn grid_fit_with_equal_cells() {
        let got = corners(&four(), &params((1, 1), GridSpacing::Fit, true, true));
        // Cells 20×20; gap x = (70 − 40 + 0)/1 = 30, gap y = (50 − 40 + 5)/1 = 15;
        // origin x = −5.
        assert_pts(&got, &[(0.0, 4.0), (45.0, 0.0), (0.0, 42.0), (52.5, 40.0)]);
    }

    #[test]
    fn grid_single_row_and_edge_cases() {
        let b = [
            Bx::new(0.0, 0.0, 10.0, 10.0),
            Bx::new(100.0, 0.0, 110.0, 10.0),
        ];
        let p = GridParams {
            rows: 1,
            cols: 5,
            ..params((0, 0), GridSpacing::Fit, false, false)
        };
        // One row: gap y is 0 rather than Inkscape's division by zero.
        assert_pts(&corners(&b, &p), &[(0.0, 0.0), (100.0, 0.0)]);
        assert!(grid(&[], &p).is_empty());
        let one = [Bx::new(3.0, 4.0, 5.0, 6.0)];
        assert_pts(&grid(&one, &p), &[(0.0, 0.0)]);
    }

    fn circle() -> Ellipse {
        Ellipse {
            cx: 100.0,
            cy: 100.0,
            rx: 50.0,
            ry: 50.0,
            start_deg: 0.0,
            end_deg: 360.0,
        }
    }

    fn unit_boxes(n: usize) -> Vec<Bx> {
        (0..n).map(|_| Bx::new(0.0, 0.0, 10.0, 10.0)).collect()
    }

    #[track_caller]
    fn assert_place(p: Placement, dx: f64, dy: f64, rot: f64) {
        assert!(
            (p.dx - dx).abs() < 1e-9
                && (p.dy - dy).abs() < 1e-9
                && (p.rotate_rad - rot).abs() < 1e-9,
            "got {p:?}, want ({dx}, {dy}, {rot})"
        );
    }

    #[test]
    fn four_on_full_circle_with_rotation() {
        let got = circular(&unit_boxes(4), circle(), CircleAnchor::Centre, true, None);
        // Box centre (5, 5) moves to 0°, 90°, 180°, 270°; tops face outward:
        // right, down, left, up.
        assert_place(got[0], 145.0, 95.0, FRAC_PI_2);
        assert_place(got[1], 95.0, 145.0, PI);
        assert_place(got[2], 45.0, 95.0, -FRAC_PI_2);
        assert_place(got[3], 95.0, 45.0, 0.0);
    }

    #[test]
    fn rotation_off_is_zero() {
        let got = circular(&unit_boxes(4), circle(), CircleAnchor::Centre, false, None);
        assert_place(got[1], 95.0, 145.0, 0.0);
    }

    #[test]
    fn corner_anchor_pivots_on_the_placed_corner() {
        // Top-left to (150, 100), then 90° clockwise about that corner:
        // the 10×20 box ends at x 130..150, y 100..110, centre (140, 105).
        let b = [Bx::new(0.0, 0.0, 10.0, 20.0)];
        let e = Ellipse {
            end_deg: 360.0,
            ..circle()
        };
        let got = circular(&b, e, CircleAnchor::BoxPoint(0, 0), true, None);
        assert_place(got[0], 135.0, 95.0, FRAC_PI_2);
        let plain = circular(&b, e, CircleAnchor::BoxPoint(2, 2), false, None);
        assert_place(plain[0], 140.0, 80.0, 0.0);
    }

    #[test]
    fn partial_arc_uses_both_ends() {
        let e = Ellipse {
            end_deg: 180.0,
            ..circle()
        };
        let got = circular(&unit_boxes(3), e, CircleAnchor::Centre, false, None);
        assert_place(got[0], 145.0, 95.0, 0.0);
        assert_place(got[1], 95.0, 145.0, 0.0);
        assert_place(got[2], 45.0, 95.0, 0.0);
    }

    #[test]
    fn nearly_full_turn_counts_as_full() {
        // 359.9° is within 0.01 rad of a turn: four slots of 89.975°.
        let e = Ellipse {
            end_deg: 359.9,
            ..circle()
        };
        let got = circular(&unit_boxes(4), e, CircleAnchor::Centre, false, None);
        let a = 2.0 * 359.9_f64.to_radians() / 4.0;
        assert_place(got[2], 95.0 + 50.0 * a.cos(), 95.0 + 50.0 * a.sin(), 0.0);
    }

    #[test]
    fn reference_item_is_skipped() {
        let e = Ellipse {
            end_deg: 180.0,
            ..circle()
        };
        let got = circular(&unit_boxes(4), e, CircleAnchor::Centre, false, Some(1));
        assert_eq!(got[1], Placement::STAY);
        assert_place(got[0], 145.0, 95.0, 0.0);
        assert_place(got[2], 95.0, 145.0, 0.0);
        assert_place(got[3], 45.0, 95.0, 0.0);
        // An out-of-range skip is ignored.
        let all = circular(&unit_boxes(3), e, CircleAnchor::Centre, false, Some(9));
        assert_place(all[2], 45.0, 95.0, 0.0);
    }

    #[test]
    fn degenerate_counts() {
        let e = Ellipse {
            start_deg: 90.0,
            end_deg: 180.0,
            ..circle()
        };
        // One item on a partial arc: no slots to divide, goes to the start.
        let one = circular(&unit_boxes(1), e, CircleAnchor::Centre, false, None);
        assert_place(one[0], 95.0, 145.0, 0.0);
        assert!(circular(&[], e, CircleAnchor::Centre, false, None).is_empty());
        let only_ref = circular(&unit_boxes(1), e, CircleAnchor::Centre, true, Some(0));
        assert_eq!(only_ref, vec![Placement::STAY]);
    }

    #[test]
    fn default_parameters() {
        let got = circular(
            &unit_boxes(2),
            Ellipse::DEFAULT,
            CircleAnchor::Centre,
            false,
            None,
        );
        assert_place(got[0], 95.0, -5.0, 0.0);
        assert_place(got[1], -105.0, -5.0, 0.0);
    }

    /// A 4-segment cubic circle, as PDF producers write one.
    fn bezier_circle(cx: f64, cy: f64, rx: f64, ry: f64) -> Vec<(f64, f64)> {
        let k = 0.552_284_749_8;
        vec![
            (cx + rx, cy),
            (cx + rx, cy + k * ry),
            (cx + k * rx, cy + ry),
            (cx, cy + ry),
            (cx - k * rx, cy + ry),
            (cx - rx, cy + k * ry),
            (cx - rx, cy),
            (cx - rx, cy - k * ry),
            (cx - k * rx, cy - ry),
            (cx, cy - ry),
            (cx + k * rx, cy - ry),
            (cx + rx, cy - k * ry),
            (cx + rx, cy),
        ]
    }

    /// A closed polygon as a cubic chain with controls on the edges.
    fn polygon(corners: &[(f64, f64)]) -> Vec<(f64, f64)> {
        let mut out = vec![corners[0]];
        for i in 0..corners.len() {
            let (a, b) = (corners[i], corners[(i + 1) % corners.len()]);
            let lerp = |t: f64| (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
            out.extend([lerp(1.0 / 3.0), lerp(2.0 / 3.0), b]);
        }
        out
    }

    #[test]
    fn detects_circle_and_ellipse() {
        let c = ellipse_from_points(&bezier_circle(100.0, 50.0, 30.0, 30.0)).expect("circle");
        assert!(near(c.cx, 100.0) && near(c.cy, 50.0) && near(c.rx, 30.0) && near(c.ry, 30.0));
        assert!(near(c.start_deg, 0.0) && near(c.end_deg, 360.0));
        let e = ellipse_from_points(&bezier_circle(0.0, 0.0, 80.0, 20.0)).expect("ellipse");
        assert!(near(e.rx, 80.0) && near(e.ry, 20.0));
    }

    #[test]
    fn rejects_square_and_diamond() {
        // Square corners are the bbox corners, √2 radii from the centre.
        let square = polygon(&[(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)]);
        assert_eq!(ellipse_from_points(&square), None);
        // Diamond corners are on the inscribed circle; its edges are not.
        let diamond = polygon(&[(5.0, 0.0), (10.0, 5.0), (5.0, 10.0), (0.0, 5.0)]);
        assert_eq!(ellipse_from_points(&diamond), None);
    }

    #[test]
    fn rejects_malformed_and_flat() {
        let c = bezier_circle(0.0, 0.0, 10.0, 10.0);
        assert_eq!(ellipse_from_points(&c[..12]), None);
        assert_eq!(ellipse_from_points(&c[..4]), None);
        assert_eq!(ellipse_from_points(&[]), None);
        let flat = polygon(&[(0.0, 0.0), (10.0, 0.0)]);
        assert_eq!(ellipse_from_points(&flat), None);
    }

    #[test]
    fn tolerance_is_one_percent() {
        // Bulge the first quarter by lengthening its handles: the anchors and
        // the fit stay put while the midpoint moves to √2·(4 + 3k)/8 · r.
        let bulged = |k: f64| {
            let mut c = bezier_circle(0.0, 0.0, 100.0, 100.0);
            c[1] = (100.0, 100.0 * k);
            c[2] = (100.0 * k, 100.0);
            ellipse_from_points(&c)
        };
        assert!(bulged(0.5617).is_some(), "0.5% out is accepted");
        assert_eq!(bulged(0.58), None, "1.5% out is rejected");
    }
}
