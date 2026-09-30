//! Inkscape 1.4's Align-and-Distribute "Rearrange" and "Remove overlaps"
//! operations, and node-mode alignment, as pure arithmetic.
//!
//! Contract: canvas space in points, y down. Items arrive in selection order
//! as bounding boxes (or points, for nodes). Every function returns one
//! translation `(dx, dy)` per input, indexed like the input; an item that
//! does not move gets `(0.0, 0.0)`. An item's centre is its box's midpoint.

use super::{Axis, Bx};

mod vpsc;

fn centre(b: Bx) -> (f64, f64) {
    (f64::midpoint(b.x0, b.x1), f64::midpoint(b.y0, b.y1))
}

fn coord(p: (f64, f64), axis: Axis) -> f64 {
    match axis {
        Axis::X => p.0,
        Axis::Y => p.1,
    }
}

fn zeros(n: usize) -> Vec<(f64, f64)> {
    vec![(0.0, 0.0); n]
}

// ---- Exchange ----

/// Exchange positions: walking `order`, each listed item moves its centre to
/// the previous listed item's centre, and the first to the last's, so the
/// listed items cycle one step backwards through the list.
///
/// `order` holds indices into `boxes`, each at most once; items it omits
/// stay. Fewer than two boxes: nothing moves.
pub fn exchange(boxes: &[Bx], order: &[usize]) -> Vec<(f64, f64)> {
    let mut out = zeros(boxes.len());
    let (Some(&last), true) = (order.last(), boxes.len() >= 2) else {
        return out;
    };
    let mut p1 = centre(boxes[last]);
    for &i in order {
        let p2 = centre(boxes[i]);
        out[i] = (p1.0 - p2.0, p1.1 - p2.1);
        p1 = p2;
    }
    out
}

/// The "stacking order" list for [`exchange`]: positions `0..n` sorted by
/// ascending paint index, so bottom-most first. `paint_indices[i]` is item
/// `i`'s place in paint order. Equal indices keep selection order.
pub fn stacking_order(paint_indices: &[usize]) -> Vec<usize> {
    let mut order: Vec<usize> = (0..paint_indices.len()).collect();
    order.sort_by_key(|&i| paint_indices[i]);
    order
}

/// The "rotate around centre point" list for [`exchange`]: items sorted by
/// `atan2(dy, dx)` of their centre from the selection box's centre, ties by
/// distance, ties after that in selection order.
///
/// `atan2` runs over `(−π, π]` and y is down, so the list goes clockwise on
/// screen: starting just clockwise of due left, through up, right, down, and
/// ending with an item exactly due left. Exchanging along it therefore moves
/// each item one place anticlockwise on screen.
pub fn clockwise_order(boxes: &[Bx]) -> Vec<usize> {
    let mut order: Vec<usize> = (0..boxes.len()).collect();
    let Some(sel) = boxes.iter().copied().reduce(Bx::union) else {
        return order;
    };
    let c = centre(sel);
    let key = |i: usize| {
        let p = centre(boxes[i]);
        let (dx, dy) = (p.0 - c.0, p.1 - c.1);
        (dy.atan2(dx), dx.hypot(dy))
    };
    order.sort_by(|&a, &b| {
        let (ka, kb) = (key(a), key(b));
        ka.0.partial_cmp(&kb.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(ka.1.partial_cmp(&kb.1).unwrap_or(std::cmp::Ordering::Equal))
    });
    order
}

// ---- Randomize ----

/// `SplitMix64`: a small, fast, seedable generator; only its determinism per
/// seed matters here.
struct SplitMix64(u64);

impl SplitMix64 {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform in `0..n`, `n > 0`, by modulus like Inkscape's `rand() % n`.
    #[allow(
        clippy::cast_possible_truncation,
        reason = "the modulus is below n, a usize" // ui-text-exempt: lint reason
    )]
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }

    /// Uniform in `[lo, hi)`.
    #[allow(
        clippy::cast_precision_loss,
        reason = "53 random bits are exact in an f64" // ui-text-exempt: lint reason
    )]
    fn range(&mut self, lo: f64, hi: f64) -> f64 {
        let u = (self.next() >> 11) as f64 * (1.0 / (1u64 << 53) as f64);
        lo + u * (hi - lo)
    }
}

/// Randomize centres in both dimensions, keeping the bounding box of the
/// centres: per axis (x then y), two distinct items chosen at random land
/// exactly on the minimum and maximum centre, and every other item's centre
/// is drawn uniformly from `[min, max)`. The same `seed` gives the same
/// result. Fewer than two boxes: nothing moves.
pub fn randomize(boxes: &[Bx], seed: u64) -> Vec<(f64, f64)> {
    let n = boxes.len();
    let mut out = zeros(n);
    if n < 2 {
        return out;
    }
    let mut rng = SplitMix64(seed);
    for axis in [Axis::X, Axis::Y] {
        let cs: Vec<f64> = boxes.iter().map(|&b| coord(centre(b), axis)).collect();
        let min = cs.iter().copied().fold(f64::INFINITY, f64::min);
        let max = cs.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let imin = rng.below(n);
        let mut imax = rng.below(n);
        while imin == imax {
            imax = rng.below(n);
        }
        for (i, &c) in cs.iter().enumerate() {
            let z = if i == imin {
                min
            } else if i == imax {
                max
            } else {
                rng.range(min, max)
            };
            match axis {
                Axis::X => out[i].0 = z - c,
                Axis::Y => out[i].1 = z - c,
            }
        }
    }
    out
}

// ---- Unclump ----

/// Inkscape's unclump state: each item's live centre and fixed size.
struct Unclump {
    c: Vec<(f64, f64)>,
    wh: Vec<(f64, f64)>,
}

fn sub(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    (a.0 - b.0, a.1 - b.1)
}

fn l2(p: (f64, f64)) -> f64 {
    p.0.hypot(p.1)
}

/// 2geom `unit_vector`: a zero or NaN-length vector comes back unchanged.
fn unit(p: (f64, f64)) -> (f64, f64) {
    let len = l2(p);
    if len == 0.0 || len.is_nan() {
        p
    } else {
        (p.0 / len, p.1 / len)
    }
}

/// Aspect ratios outside this band are "not circle-like", for which
/// nearest-edge distances are also tried.
fn stretched(wh: (f64, f64)) -> bool {
    let s = wh.1 / wh.0;
    !(0.66..=1.5).contains(&s)
}

/// The point of an item's box nearest `other` along each axis through its
/// centre: `[(cx, clamped y), (clamped x, cy)]`.
fn edge_points(c: (f64, f64), wh: (f64, f64), other: (f64, f64)) -> [(f64, f64); 2] {
    let clamp = |o: f64, c: f64, half: f64| {
        if o > c + half {
            c + half
        } else if o < c - half {
            c - half
        } else {
            o
        }
    };
    [
        (c.0, clamp(other.1, c.1, wh.1 / 2.0)),
        (clamp(other.0, c.0, wh.0 / 2.0), c.1),
    ]
}

impl Unclump {
    /// Edge-to-edge distance, each item taken as the ellipse inscribed in its
    /// box; negative when they interpenetrate. When both items are elongated
    /// the least of that and the four nearest-edge-point distances.
    fn dist(&self, i1: usize, i2: usize) -> f64 {
        let (c1, c2) = (self.c[i1], self.c[i2]);
        let (wh1, wh2) = (self.wh[i1], self.wh[i2]);
        let fold = |a: f64| {
            let a = a.abs();
            if a > std::f64::consts::FRAC_PI_2 {
                std::f64::consts::PI - a
            } else {
                a
            }
        };
        let d12 = sub(c2, c1);
        let d21 = sub(c1, c2);
        let a1 = fold(d12.1.atan2(d12.0 * wh1.1 / wh1.0));
        let a2 = fold(d21.1.atan2(d21.0 * wh2.1 / wh2.0));
        let r1 = 0.5 * (wh1.0 + (wh1.1 - wh1.0) * (a1 / std::f64::consts::FRAC_PI_2));
        let r2 = 0.5 * (wh2.0 + (wh2.1 - wh2.0) * (a2 / std::f64::consts::FRAC_PI_2));
        let dist_r = l2(d12) - r1 - r2;
        if !(stretched(wh1) && stretched(wh2)) {
            return dist_r;
        }
        let p1 = edge_points(c1, wh1, c2);
        let p2 = edge_points(c2, wh2, c1);
        let mut best = dist_r;
        for a in p1 {
            for b in p2 {
                best = best.min(l2(sub(a, b)));
            }
        }
        best
    }

    fn average(&self, item: usize, others: &[usize]) -> f64 {
        let mut n = 0u32;
        let mut sum = 0.0;
        for &o in others.iter().filter(|&&o| o != item) {
            n += 1;
            sum += self.dist(item, o);
        }
        if n == 0 { 0.0 } else { sum / f64::from(n) }
    }

    /// The first other at the least distance; distances of magnitude 1e6 or
    /// more (degenerate boxes) are ignored.
    fn closest(&self, item: usize, others: &[usize]) -> Option<usize> {
        let mut min = f64::INFINITY;
        let mut best = None;
        for &o in others.iter().filter(|&&o| o != item) {
            let d = self.dist(item, o);
            if d < min && d.abs() < 1e6 {
                min = d;
                best = Some(o);
            }
        }
        best
    }

    fn farthest(&self, item: usize, others: &[usize]) -> Option<usize> {
        let mut max = f64::NEG_INFINITY;
        let mut best = None;
        for &o in others.iter().filter(|&&o| o != item) {
            let d = self.dist(item, o);
            if d > max && d.abs() < 1e6 {
                max = d;
                best = Some(o);
            }
        }
        best
    }

    /// `rest` without the items behind `closest` as seen from `item`: those
    /// not strictly on `item`'s side of the line through `closest`
    /// perpendicular to the direction between them.
    fn remove_behind(&self, item: usize, closest: usize, rest: &[usize]) -> Vec<usize> {
        let it = self.c[item];
        let p1 = self.c[closest];
        let d = sub(it, p1);
        // 2geom `rot90`: (x, y) -> (-y, x).
        let p2 = (p1.0 - d.1, p1.1 + d.0);
        let (la, lb, lc) = (p1.1 - p2.1, p2.0 - p1.0, p2.1 * p1.0 - p1.1 * p2.0);
        let val_item = la * it.0 + lb * it.1 + lc;
        rest.iter()
            .copied()
            .filter(|&o| {
                let q = self.c[o];
                o != item && val_item * (la * q.0 + lb * q.1 + lc) > 1e-6
            })
            .collect()
    }

    /// Moves `what` by `d` along the unit direction `dir` and returns nothing;
    /// Inkscape's push and pull differ only in the direction.
    fn shift(&mut self, what: usize, dir: (f64, f64), d: f64) {
        let u = unit(dir);
        self.c[what].0 += d * u.0;
        self.c[what].1 += d * u.1;
    }
}

/// One application of Unclump ("try to equalize edge-to-edge distances").
///
/// Items are visited in selection order and each moves before the next is
/// visited, so later items see earlier moves. For an item, neighbours are
/// picked greedily nearest first, each pick discarding the remaining items
/// behind it. With at least two neighbours the item is pushed away from the
/// closest by `0.3·(avg − dmin)`, then pulled towards the farthest by
/// `0.35·(dmax − avg)`, where distances are [`Unclump::dist`] edge-to-edge
/// distances to the neighbours. Repeating the call keeps moving things; only
/// a hexagonal grid is a fixed point.
pub fn unclump(boxes: &[Bx]) -> Vec<(f64, f64)> {
    let n = boxes.len();
    if n < 2 {
        return zeros(n);
    }
    let start: Vec<(f64, f64)> = boxes.iter().map(|&b| centre(b)).collect();
    let mut u = Unclump {
        c: start.clone(),
        wh: boxes.iter().map(|b| (b.x1 - b.x0, b.y1 - b.y0)).collect(),
    };

    for item in 0..n {
        let mut rest: Vec<usize> = (0..n).filter(|&i| i != item).collect();
        let mut found = Vec::new();
        while !rest.is_empty() {
            let Some(closest) = u.closest(item, &rest) else {
                break;
            };
            found.push(closest);
            rest.retain(|&i| i != closest);
            rest = u.remove_behind(item, closest, &rest);
        }
        if found.len() < 2 {
            continue;
        }
        // Inkscape prepends each neighbour, so ties resolve in reverse
        // discovery order.
        found.reverse();
        let nei = found;
        let ave = u.average(item, &nei);
        let (Some(closest), Some(farthest)) = (u.closest(item, &nei), u.farthest(item, &nei))
        else {
            continue;
        };
        let dist_closest = u.dist(closest, item);
        let dist_farthest = u.dist(farthest, item);
        if ave.abs() < 1e6 && dist_closest.abs() < 1e6 && dist_farthest.abs() < 1e6 {
            u.shift(
                item,
                sub(u.c[item], u.c[closest]),
                0.3 * (ave - dist_closest),
            );
            u.shift(
                item,
                sub(u.c[farthest], u.c[item]),
                0.35 * (dist_farthest - ave),
            );
        }
    }
    u.c.iter().zip(&start).map(|(&c, &s)| sub(c, s)).collect()
}

// ---- Remove overlaps ----

/// Remove overlaps: moves items as little as possible so that their boxes,
/// each grown by `gap_x / 2` left and right and `gap_y / 2` top and bottom,
/// do not overlap. Gaps may be negative; a box that would invert collapses to
/// its centre on that axis instead. Each item moves its centre to its solved
/// rectangle's centre, by libvpsc's horizontal, vertical, then second
/// horizontal least-squares pass. Fewer than two boxes: nothing moves.
pub fn remove_overlaps(boxes: &[Bx], gap_x: f64, gap_y: f64) -> Vec<(f64, f64)> {
    if boxes.len() < 2 {
        return zeros(boxes.len());
    }
    let grow = |lo: f64, hi: f64, gap: f64| {
        let (a, b) = (lo - 0.5 * gap, hi + 0.5 * gap);
        if b < a {
            let m = f64::midpoint(a, b);
            (m, m)
        } else {
            (a, b)
        }
    };
    let start: Vec<vpsc::Rect> = boxes
        .iter()
        .map(|b| {
            let (x0, x1) = grow(b.x0, b.x1, gap_x);
            let (y0, y1) = grow(b.y0, b.y1, gap_y);
            vpsc::Rect { x0, x1, y0, y1 }
        })
        .collect();
    let mut rs = start.clone();
    vpsc::remove_overlaps(&mut rs);
    rs.iter()
        .zip(&start)
        .map(|(r, s)| {
            (
                f64::midpoint(r.x0, r.x1) - f64::midpoint(s.x0, s.x1),
                f64::midpoint(r.y0, r.y1) - f64::midpoint(s.y0, s.y1),
            )
        })
        .collect()
}

// ---- Node mode ----

/// What selected nodes are aligned to. Inkscape's default is [`Self::First`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NodeRelative {
    /// The last node in selection order.
    Last,
    /// The first node in selection order.
    #[default]
    First,
    /// The middle of the nodes' range on the aligned axis.
    Middle,
    /// The least coordinate on the aligned axis.
    Min,
    /// The greatest coordinate on the aligned axis.
    Max,
}

/// Align nodes: sets every point's `axis` coordinate to the target's, so
/// [`Axis::X`] lines the nodes up on a common vertical line and [`Axis::Y`]
/// on a common horizontal one. (Inkscape's action names the other axis: its
/// `alignNodes(X)` is "align to a horizontal line" and sets y.)
pub fn align_nodes(points: &[(f64, f64)], axis: Axis, rel: NodeRelative) -> Vec<(f64, f64)> {
    let (Some(&first), Some(&last)) = (points.first(), points.last()) else {
        return Vec::new();
    };
    let cs = points.iter().map(|&p| coord(p, axis));
    let min = cs.clone().fold(f64::INFINITY, f64::min);
    let max = cs.fold(f64::NEG_INFINITY, f64::max);
    let target = match rel {
        NodeRelative::Last => coord(last, axis),
        NodeRelative::First => coord(first, axis),
        NodeRelative::Middle => f64::midpoint(min, max),
        NodeRelative::Min => min,
        NodeRelative::Max => max,
    };
    points
        .iter()
        .map(|&p| axis.vec(target - coord(p, axis)))
        .collect()
}

/// Distribute nodes along `axis`: points sorted by that coordinate (ties in
/// input order) are spaced evenly from the least to the greatest, which stay
/// put.
pub fn distribute_nodes(points: &[(f64, f64)], axis: Axis) -> Vec<(f64, f64)> {
    let n = points.len();
    let mut out = zeros(n);
    if n < 2 {
        return out;
    }
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&a, &b| {
        coord(points[a], axis)
            .partial_cmp(&coord(points[b], axis))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let min = coord(points[order[0]], axis);
    let max = coord(points[order[n - 1]], axis);
    #[allow(
        clippy::cast_precision_loss,
        reason = "selection sizes are far below 2^52" // ui-text-exempt: lint reason
    )]
    let step = (max - min) / (n - 1) as f64;
    for (k, &i) in order.iter().enumerate() {
        #[allow(
            clippy::cast_precision_loss,
            reason = "selection sizes are far below 2^52" // ui-text-exempt: lint reason
        )]
        let target = min + k as f64 * step;
        out[i] = axis.vec(target - coord(points[i], axis));
    }
    out
}

#[cfg(test)]
#[allow(clippy::float_cmp, reason = "expected values are exact in binary")]
mod tests {
    use super::*;

    fn sq(cx: f64, cy: f64, s: f64) -> Bx {
        Bx::new(cx - s / 2.0, cy - s / 2.0, cx + s / 2.0, cy + s / 2.0)
    }

    fn close(a: (f64, f64), b: (f64, f64)) -> bool {
        (a.0 - b.0).abs() < 1e-9 && (a.1 - b.1).abs() < 1e-9
    }

    fn moved(boxes: &[Bx], d: &[(f64, f64)]) -> Vec<Bx> {
        boxes
            .iter()
            .zip(d)
            .map(|(&b, &d)| b.translated(d))
            .collect()
    }

    fn overlaps(a: Bx, b: Bx) -> bool {
        a.x0 < b.x1 - 1e-9 && b.x0 < a.x1 - 1e-9 && a.y0 < b.y1 - 1e-9 && b.y0 < a.y1 - 1e-9
    }

    #[test]
    fn exchange_cycles_three_boxes() {
        let b = [sq(0.0, 0.0, 2.0), sq(10.0, 0.0, 4.0), sq(0.0, 20.0, 6.0)];
        let d = exchange(&b, &[0, 1, 2]);
        // 0 → 2's centre, 1 → 0's, 2 → 1's.
        assert_eq!(d, vec![(0.0, 20.0), (-10.0, 0.0), (10.0, -20.0)]);
    }

    #[test]
    fn exchange_follows_the_given_order_and_skips_unlisted() {
        let b = [sq(0.0, 0.0, 2.0), sq(10.0, 0.0, 2.0), sq(20.0, 0.0, 2.0)];
        let d = exchange(&b, &[2, 0]);
        assert_eq!(d, vec![(20.0, 0.0), (0.0, 0.0), (-20.0, 0.0)]);
        assert_eq!(exchange(&b[..1], &[0]), vec![(0.0, 0.0)]);
    }

    #[test]
    fn stacking_order_sorts_by_paint_index() {
        assert_eq!(stacking_order(&[7, 2, 9, 4]), vec![1, 3, 0, 2]);
    }

    #[test]
    fn clockwise_order_at_compass_points() {
        // Selection order: east, south, west, north around (0, 0), y down.
        let b = [
            sq(10.0, 0.0, 2.0),
            sq(0.0, 10.0, 2.0),
            sq(-10.0, 0.0, 2.0),
            sq(0.0, -10.0, 2.0),
        ];
        // atan2: north −π/2, east 0, south π/2, west π.
        assert_eq!(clockwise_order(&b), vec![3, 0, 1, 2]);
        // North takes west's centre: one step anticlockwise on screen.
        let d = exchange(&b, &clockwise_order(&b));
        assert_eq!(d[3], (-10.0, 10.0));
        assert_eq!(d[0], (-10.0, -10.0));
    }

    #[test]
    fn clockwise_order_breaks_angle_ties_by_distance() {
        let b = [sq(20.0, 0.0, 2.0), sq(-20.0, 0.0, 2.0), sq(5.0, 0.0, 2.0)];
        assert_eq!(clockwise_order(&b), vec![2, 0, 1]);
    }

    #[test]
    fn randomize_keeps_centre_extremes_and_is_deterministic() {
        let b: Vec<Bx> = (0..6)
            .map(|i| sq(f64::from(i) * 10.0, f64::from(i * i), 2.0))
            .collect();
        let d1 = randomize(&b, 42);
        assert_eq!(d1, randomize(&b, 42));
        assert_ne!(d1, randomize(&b, 43));
        let cs: Vec<(f64, f64)> = moved(&b, &d1).iter().map(|&m| centre(m)).collect();
        for (axis, lo, hi) in [(Axis::X, 0.0, 50.0), (Axis::Y, 0.0, 25.0)] {
            let v: Vec<f64> = cs.iter().map(|&c| coord(c, axis)).collect();
            assert_eq!(v.iter().filter(|&&x| x == lo).count(), 1, "{v:?}");
            assert_eq!(v.iter().filter(|&&x| x == hi).count(), 1, "{v:?}");
            assert!(v.iter().all(|&x| (lo..=hi).contains(&x)));
        }
    }

    #[test]
    fn randomize_handles_negative_coordinates() {
        let b = [
            sq(-30.0, -5.0, 2.0),
            sq(-10.0, -1.0, 2.0),
            sq(-20.0, -3.0, 2.0),
        ];
        let cs: Vec<(f64, f64)> = moved(&b, &randomize(&b, 7))
            .iter()
            .map(|&m| centre(m))
            .collect();
        assert!(
            cs.iter()
                .all(|c| (-30.0..=-10.0).contains(&c.0) && (-5.0..=-1.0).contains(&c.1))
        );
        assert!(cs.iter().any(|c| c.0 == -30.0) && cs.iter().any(|c| c.0 == -10.0));
    }

    #[test]
    fn unclump_distance_between_circles_and_bars() {
        let u = Unclump {
            c: vec![(0.0, 0.0), (10.0, 0.0)],
            wh: vec![(2.0, 2.0), (4.0, 4.0)],
        };
        // Round items: 10 − 1 − 2.
        assert!((u.dist(0, 1) - 7.0).abs() < 1e-12);
        // Two tall bars offset diagonally: the nearest-edge points (1, 0) and
        // (10, 5) beat the ellipse estimate of about 12.6.
        let u = Unclump {
            c: vec![(0.0, 0.0), (10.0, 15.0)],
            wh: vec![(2.0, 20.0), (2.0, 20.0)],
        };
        assert!(
            (u.dist(0, 1) - 106f64.sqrt()).abs() < 1e-12,
            "{}",
            u.dist(0, 1)
        );
    }

    #[test]
    fn unclump_moves_a_clumped_item_away() {
        // Item 1 sits 2 from item 0 and 18 from item 2, all on a line.
        let b = [sq(0.0, 0.0, 2.0), sq(4.0, 0.0, 2.0), sq(24.0, 0.0, 2.0)];
        let d = unclump(&b);
        // Item 0 has one neighbour (1 hides 2) and stays.
        assert_eq!(d[0], (0.0, 0.0));
        // Item 1: avg 10, push 0.3·(10−2)=2.4 right, pull 0.35·(18−10)=2.8 right.
        assert!(close(d[1], (5.2, 0.0)), "{:?}", d[1]);
        // Item 2 sees only item 1 (item 0 is behind it): one neighbour.
        assert_eq!(d[2], (0.0, 0.0));
    }

    #[test]
    fn unclump_leaves_single_neighbour_items() {
        assert_eq!(
            unclump(&[sq(0.0, 0.0, 2.0), sq(3.0, 0.0, 2.0)]),
            vec![(0.0, 0.0); 2]
        );
    }

    #[test]
    fn remove_overlaps_separates_two_squares_minimally() {
        let b = [Bx::new(0.0, 0.0, 10.0, 10.0), Bx::new(6.0, 1.0, 16.0, 11.0)];
        let d = remove_overlaps(&b, 0.0, 0.0);
        let m = moved(&b, &d);
        assert!(!overlaps(m[0], m[1]), "{m:?}");
        // Overlap is 4 wide and 9 tall: split horizontally, 2 each way (plus
        // libvpsc's 1e-3 border), order kept, no vertical movement.
        assert!(
            (d[0].0 + 2.001).abs() < 1e-9 && (d[1].0 - 2.001).abs() < 1e-9,
            "{d:?}"
        );
        assert_eq!((d[0].1, d[1].1), (0.0, 0.0));
        assert!(m[0].x1 <= m[1].x0);
    }

    #[test]
    fn remove_overlaps_honours_gaps() {
        let b = [Bx::new(0.0, 0.0, 10.0, 10.0), Bx::new(6.0, 0.0, 16.0, 10.0)];
        let m = moved(&b, &remove_overlaps(&b, 4.0, 0.0));
        assert!(m[1].x0 - m[0].x1 >= 4.0 - 1e-9, "{m:?}");
        assert_eq!(remove_overlaps(&b[..1], 4.0, 4.0), vec![(0.0, 0.0)]);
    }

    #[test]
    fn remove_overlaps_uses_vertical_pass_for_wide_overlap() {
        // Wide boxes overlapping 2 vertically and 18 horizontally: the third
        // pass lets them keep x and separate in y.
        let b = [Bx::new(0.0, 0.0, 20.0, 10.0), Bx::new(2.0, 8.0, 22.0, 18.0)];
        let d = remove_overlaps(&b, 0.0, 0.0);
        let m = moved(&b, &d);
        assert!(!overlaps(m[0], m[1]), "{m:?}");
        assert!(d[0].0.abs() < 1e-9 && d[1].0.abs() < 1e-9, "{d:?}");
        assert!(
            (d[0].1 + 1.001).abs() < 1e-9 && (d[1].1 - 1.001).abs() < 1e-9,
            "{d:?}"
        );
    }

    #[test]
    fn remove_overlaps_many_boxes_end_disjoint() {
        let b: Vec<Bx> = (0..20)
            .map(|i| {
                let f = f64::from(i);
                sq((f * 3.7) % 17.0, (f * 5.3) % 13.0, 6.0 + f % 3.0)
            })
            .collect();
        let m = moved(&b, &remove_overlaps(&b, 1.0, 1.0));
        for i in 0..m.len() {
            for j in i + 1..m.len() {
                assert!(!overlaps(m[i], m[j]), "{i} {j}");
            }
        }
    }

    #[test]
    fn remove_overlaps_negative_gap_collapses_not_inverts() {
        let b = [Bx::new(0.0, 0.0, 2.0, 2.0), Bx::new(1.0, 0.0, 3.0, 2.0)];
        let d = remove_overlaps(&b, -10.0, -10.0);
        assert!(d.iter().all(|p| p.0.is_finite() && p.1.is_finite()));
    }

    #[test]
    fn align_nodes_each_reference() {
        let p = [(1.0, 5.0), (4.0, 2.0), (3.0, 9.0)];
        assert_eq!(
            align_nodes(&p, Axis::X, NodeRelative::First),
            vec![(0.0, 0.0), (-3.0, 0.0), (-2.0, 0.0)]
        );
        assert_eq!(
            align_nodes(&p, Axis::X, NodeRelative::Last),
            vec![(2.0, 0.0), (-1.0, 0.0), (0.0, 0.0)]
        );
        assert_eq!(
            align_nodes(&p, Axis::Y, NodeRelative::Middle),
            vec![(0.0, 0.5), (0.0, 3.5), (0.0, -3.5)]
        );
        assert_eq!(
            align_nodes(&p, Axis::Y, NodeRelative::Min),
            vec![(0.0, -3.0), (0.0, 0.0), (0.0, -7.0)]
        );
        assert_eq!(
            align_nodes(&p, Axis::X, NodeRelative::Max),
            vec![(3.0, 0.0), (0.0, 0.0), (1.0, 0.0)]
        );
        assert!(align_nodes(&[], Axis::X, NodeRelative::Min).is_empty());
    }

    #[test]
    fn distribute_nodes_spaces_evenly_between_extremes() {
        let p = [(10.0, 0.0), (0.0, 1.0), (1.0, 2.0), (2.0, 3.0), (7.0, 4.0)];
        let d = distribute_nodes(&p, Axis::X);
        // Sorted x: 0, 1, 2, 7, 10 → 0, 2.5, 5, 7.5, 10.
        assert_eq!(
            d,
            vec![(0.0, 0.0), (0.0, 0.0), (1.5, 0.0), (3.0, 0.0), (0.5, 0.0)]
        );
        assert_eq!(distribute_nodes(&p[..1], Axis::Y), vec![(0.0, 0.0)]);
    }
}
