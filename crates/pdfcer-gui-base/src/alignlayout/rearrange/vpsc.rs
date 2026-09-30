//! A port of libvpsc's `removeoverlaps`: overlap removal by one horizontal,
//! one vertical and a second horizontal VPSC pass, each a quadratic program
//! (least squared movement under separation constraints) solved by block
//! merging and splitting.
//!
//! Contract: [`remove_overlaps`] moves each rectangle's centre and never its
//! size. Every variable has weight 1 and scale 1, and there are no equality
//! constraints and no fixed rectangles, which is all Inkscape's caller uses;
//! the formulas keep the weight term so they read like the original.

use std::cmp::Ordering;
use std::collections::BTreeSet;

const NONE: usize = usize::MAX;
/// Border added around every rectangle while its own axis is solved, so
/// rectangles made to touch in one pass do not count as overlapping in the
/// next.
const EXTRA_GAP: f64 = 1e-3;
/// A block is split across a constraint whose Lagrange multiplier is below
/// this.
const LAGRANGIAN_TOLERANCE: f64 = -1e-4;
/// `refine` gives up after this many splits.
const MAX_REFINE_TRIES: u32 = 100;

/// An axis-aligned rectangle, `x0 <= x1`, `y0 <= y1`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Rect {
    pub x0: f64,
    pub x1: f64,
    pub y0: f64,
    pub y1: f64,
}

impl Rect {
    fn centre(&self, d: Dim) -> f64 {
        let (lo, hi) = self.span(d);
        lo + (hi - lo) / 2.0
    }

    fn span(&self, d: Dim) -> (f64, f64) {
        match d {
            Dim::X => (self.x0, self.x1),
            Dim::Y => (self.y0, self.y1),
        }
    }

    fn len(&self, d: Dim) -> f64 {
        let (lo, hi) = self.span(d);
        hi - lo
    }

    fn move_centre(&mut self, d: Dim, c: f64) {
        let shift = c - self.centre(d);
        match d {
            Dim::X => {
                self.x0 += shift;
                self.x1 += shift;
            }
            Dim::Y => {
                self.y0 += shift;
                self.y1 += shift;
            }
        }
    }

    /// The rectangle as libvpsc's getters see it with borders `bx`, `by`.
    fn bordered(&self, bx: f64, by: f64) -> Rect {
        Rect {
            x0: self.x0 - bx,
            x1: self.x1 + bx,
            y0: self.y0 - by,
            y1: self.y1 + by,
        }
    }

    /// libvpsc `overlapX`/`overlapY`: how far `self` and `o` overlap along
    /// `d`, judged from which centre is lower; 0 when apart.
    fn overlap(&self, o: &Rect, d: Dim) -> f64 {
        let (mine, theirs) = (self.span(d), o.span(d));
        let self_lower = self.centre(d) <= o.centre(d);
        if self_lower && theirs.0 < mine.1 {
            return mine.1 - theirs.0;
        }
        if o.centre(d) <= self.centre(d) && mine.0 < theirs.1 {
            return theirs.1 - mine.0;
        }
        0.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Dim {
    X,
    Y,
}

impl Dim {
    fn other(self) -> Dim {
        match self {
            Dim::X => Dim::Y,
            Dim::Y => Dim::X,
        }
    }
}

/// Moves `rects` so that no two overlap, each pass moving them as little as
/// possible in the least-squares sense. Fewer than two rectangles are left
/// alone.
pub(super) fn remove_overlaps(rects: &mut [Rect]) {
    if rects.len() < 2 {
        return;
    }
    let init_x: Vec<f64> = rects.iter().map(|r| r.centre(Dim::X)).collect();
    pass(rects, Dim::X, EXTRA_GAP, EXTRA_GAP, true);
    pass(rects, Dim::Y, 0.0, EXTRA_GAP, false);
    // The third pass restores the original x so rectangles whose overlap the
    // vertical pass resolved need not have moved horizontally.
    for (r, &x) in rects.iter_mut().zip(&init_x) {
        r.move_centre(Dim::X, x);
    }
    pass(rects, Dim::X, EXTRA_GAP, 0.0, false);
}

/// One axis: generate separation constraints with the given borders, solve,
/// and move every centre on `d` to its solved position.
fn pass(rects: &mut [Rect], d: Dim, bx: f64, by: f64, neighbour_lists: bool) {
    let seen: Vec<Rect> = rects.iter().map(|r| r.bordered(bx, by)).collect();
    let desired: Vec<f64> = seen.iter().map(|r| r.centre(d)).collect();
    let cons = generate_constraints(&seen, d, neighbour_lists);
    let solved = Solver::new(&desired, &cons).solve();
    for (r, p) in rects.iter_mut().zip(solved) {
        r.move_centre(d, p);
    }
}

/// A scanline key: position on the solved axis, then node index. libvpsc
/// breaks position ties by node address, which is allocation order.
#[derive(Debug, Clone, Copy)]
struct Key(f64, usize);

impl PartialEq for Key {
    fn eq(&self, o: &Self) -> bool {
        self.cmp(o) == Ordering::Equal
    }
}
impl Eq for Key {}
impl PartialOrd for Key {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}
impl Ord for Key {
    fn cmp(&self, o: &Self) -> Ordering {
        if self.0 < o.0 {
            Ordering::Less
        } else if o.0 < self.0 {
            Ordering::Greater
        } else {
            self.1.cmp(&o.1)
        }
    }
}

#[derive(Default)]
struct Node {
    above: Option<usize>,
    below: Option<usize>,
    left: BTreeSet<Key>,
    right: BTreeSet<Key>,
}

/// `(left, right, gap)`: `pos[left] + gap <= pos[right]`.
type Con = (usize, usize, f64);

/// libvpsc `generateXConstraints` (`d == X`) and `generateYConstraints`
/// (`d == Y`, never with neighbour lists): sweep across the other axis and
/// separate rectangles that are open on the scanline at the same time.
/// Neighbour lists also separate a rectangle from every scanline neighbour
/// whose overlap along `d` is no larger than across it.
fn generate_constraints(rs: &[Rect], d: Dim, neighbour_lists: bool) -> Vec<Con> {
    let across = d.other();
    let key = |i: usize| Key(rs[i].centre(d), i);
    let sep = |a: usize, b: usize| f64::midpoint(rs[a].len(d), rs[b].len(d));

    let mut events: Vec<(f64, bool, usize)> = Vec::with_capacity(rs.len() * 2);
    for (i, r) in rs.iter().enumerate() {
        let (lo, hi) = r.span(across);
        events.push((lo, false, i));
        events.push((hi, true, i));
    }
    // Opens before closes at one position; libvpsc's `qsort` leaves other
    // ties platform-defined, and this keeps them in rectangle order.
    events.sort_by(|a, b| {
        a.0.partial_cmp(&b.0)
            .unwrap_or(Ordering::Equal)
            .then(a.1.cmp(&b.1))
    });

    let mut nodes: Vec<Node> = (0..rs.len()).map(|_| Node::default()).collect();
    let mut scan: BTreeSet<Key> = BTreeSet::new();
    let mut cs = Vec::new();
    for (_, close, v) in events {
        let kv = key(v);
        let opening = !close;
        if opening {
            scan.insert(kv);
            if neighbour_lists {
                let near = |u: usize, set: &mut BTreeSet<Key>| {
                    let ov = rs[u].overlap(&rs[v], d);
                    if ov <= 0.0 {
                        set.insert(key(u));
                        return true;
                    }
                    if ov <= rs[u].overlap(&rs[v], across) {
                        set.insert(key(u));
                    }
                    false
                };
                let mut left = BTreeSet::new();
                for k in scan.range(..kv).rev() {
                    if near(k.1, &mut left) {
                        break;
                    }
                }
                let mut right = BTreeSet::new();
                for k in scan.range((std::ops::Bound::Excluded(kv), std::ops::Bound::Unbounded)) {
                    if near(k.1, &mut right) {
                        break;
                    }
                }
                for k in &left {
                    nodes[k.1].right.insert(kv);
                }
                for k in &right {
                    nodes[k.1].left.insert(kv);
                }
                nodes[v].left = left;
                nodes[v].right = right;
            } else {
                if let Some(u) = scan.range(..kv).next_back() {
                    nodes[v].above = Some(u.1);
                    nodes[u.1].below = Some(v);
                }
                if let Some(u) = scan
                    .range((std::ops::Bound::Excluded(kv), std::ops::Bound::Unbounded))
                    .next()
                {
                    nodes[v].below = Some(u.1);
                    nodes[u.1].above = Some(v);
                }
            }
        } else {
            if neighbour_lists {
                for k in std::mem::take(&mut nodes[v].left) {
                    cs.push((k.1, v, sep(v, k.1)));
                    nodes[k.1].right.remove(&kv);
                }
                for k in std::mem::take(&mut nodes[v].right) {
                    cs.push((v, k.1, sep(v, k.1)));
                    nodes[k.1].left.remove(&kv);
                }
            } else {
                let (l, r) = (nodes[v].above, nodes[v].below);
                if let Some(l) = l {
                    cs.push((l, v, sep(v, l)));
                    nodes[l].below = r;
                }
                if let Some(r) = r {
                    cs.push((v, r, sep(v, r)));
                    nodes[r].above = l;
                }
            }
            scan.remove(&kv);
        }
    }
    cs
}

struct Var {
    desired: f64,
    weight: f64,
    offset: f64,
    block: usize,
    visited: bool,
    ins: Vec<usize>,
    outs: Vec<usize>,
}

struct Constraint {
    left: usize,
    right: usize,
    gap: f64,
    time_stamp: i64,
    active: bool,
    lm: f64,
}

/// Variables held rigidly relative to one another by active constraints.
/// `posn` is the weighted-optimal position given the offsets:
/// `(AD − AB) / A2` with `AB = Σw·offset`, `AD = Σw·desired`, `A2 = Σw`.
struct Block {
    vars: Vec<usize>,
    posn: f64,
    ab: f64,
    ad: f64,
    a2: f64,
    deleted: bool,
    time_stamp: i64,
    heap_in: Option<usize>,
    heap_out: Option<usize>,
}

#[derive(Clone, Copy)]
struct Heap {
    root: usize,
}

/// A pairing-heap node. libvpsc's heap compares keys that change after
/// insertion (slacks and time stamps), so its exact link order is ported
/// rather than replaced by a standard priority queue.
#[derive(Clone, Copy)]
struct PNode {
    elem: usize,
    child: usize,
    next: usize,
    prev: usize,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Dir {
    In,
    Out,
}

/// libvpsc `Solver` (not `IncSolver`): `satisfy` then `refine`.
struct Solver {
    vars: Vec<Var>,
    cons: Vec<Constraint>,
    blocks: Vec<Block>,
    live: Vec<usize>,
    heaps: Vec<Heap>,
    nodes: Vec<PNode>,
    ctr: i64,
}

impl Solver {
    fn new(desired: &[f64], cs: &[Con]) -> Self {
        let mut s = Solver {
            vars: desired
                .iter()
                .map(|&d| Var {
                    desired: d,
                    weight: 1.0,
                    offset: 0.0,
                    block: NONE,
                    visited: false,
                    ins: vec![],
                    outs: vec![],
                })
                .collect(),
            cons: cs
                .iter()
                .map(|&(left, right, gap)| Constraint {
                    left,
                    right,
                    gap,
                    time_stamp: 0,
                    active: false,
                    lm: 0.0,
                })
                .collect(),
            blocks: Vec::new(),
            live: Vec::new(),
            heaps: Vec::new(),
            nodes: Vec::new(),
            ctr: 0,
        };
        for (i, c) in cs.iter().enumerate() {
            s.vars[c.0].outs.push(i);
            s.vars[c.1].ins.push(i);
        }
        for v in 0..desired.len() {
            let b = s.new_block();
            s.vars[v].offset = 0.0;
            s.add_variable(b, v);
            s.live.push(b);
        }
        s
    }

    fn solve(mut self) -> Vec<f64> {
        self.satisfy();
        self.refine();
        (0..self.vars.len()).map(|v| self.pos(v)).collect()
    }

    fn pos(&self, v: usize) -> f64 {
        self.blocks[self.vars[v].block].posn + self.vars[v].offset
    }

    fn slack(&self, c: usize) -> f64 {
        let c = &self.cons[c];
        self.pos(c.right) - c.gap - self.pos(c.left)
    }

    fn block_of(&self, v: usize) -> usize {
        self.vars[v].block
    }

    // ---- Blocks ----

    /// Reverse post-order of a depth-first walk over the constraint DAG.
    fn total_order(&mut self) -> Vec<usize> {
        for v in &mut self.vars {
            v.visited = false;
        }
        let mut order = Vec::with_capacity(self.vars.len());
        for root in 0..self.vars.len() {
            if !self.vars[root].ins.is_empty() {
                continue;
            }
            self.vars[root].visited = true;
            let mut stack = vec![(root, 0usize)];
            while let Some(&mut (v, ref mut k)) = stack.last_mut() {
                if let Some(&c) = self.vars[v].outs.get(*k) {
                    *k += 1;
                    let w = self.cons[c].right;
                    if !self.vars[w].visited {
                        self.vars[w].visited = true;
                        stack.push((w, 0));
                    }
                } else {
                    order.push(v);
                    stack.pop();
                }
            }
        }
        order.reverse();
        order
    }

    fn satisfy(&mut self) {
        for v in self.total_order() {
            let b = self.block_of(v);
            if !self.blocks[b].deleted {
                self.merge_left(b);
            }
        }
        self.cleanup();
    }

    fn refine(&mut self) {
        let mut solved = false;
        let mut tries = MAX_REFINE_TRIES;
        while !solved && tries > 0 {
            solved = true;
            tries -= 1;
            let length = self.live.len();
            for i in 0..length {
                let b = self.live[i];
                self.set_up_heap(b, Dir::In);
                self.set_up_heap(b, Dir::Out);
            }
            for i in 0..length {
                let b = self.live[i];
                if let Some(c) = self.find_min_lm(b)
                    && self.cons[c].lm < LAGRANGIAN_TOLERANCE
                {
                    self.split_blocks(b, c);
                    self.cleanup();
                    solved = false;
                    break;
                }
            }
        }
    }

    /// Merges `r` with the blocks to its left across its most violated
    /// incoming constraints until none is violated.
    fn merge_left(&mut self, r: usize) {
        self.ctr += 1;
        self.blocks[r].time_stamp = self.ctr;
        self.set_up_heap(r, Dir::In);
        let mut r = r;
        let mut c = self.find_min_in(r);
        while let Some(ci) = c
            && self.slack(ci) < 0.0
        {
            let h = self.blocks[r].heap_in.expect("set up"); // ui-text-exempt: panic message
            self.h_delete_min(h);
            let mut l = self.block_of(self.cons[ci].left);
            if self.blocks[l].heap_in.is_none() {
                self.set_up_heap(l, Dir::In);
            }
            let con = &self.cons[ci];
            let mut dist = self.vars[con.right].offset - self.vars[con.left].offset - con.gap;
            if self.blocks[r].vars.len() < self.blocks[l].vars.len() {
                dist = -dist;
                std::mem::swap(&mut l, &mut r);
            }
            self.ctr += 1;
            self.merge_blocks(r, l, ci, dist);
            self.merge_heaps(r, l, Dir::In);
            self.blocks[r].time_stamp = self.ctr;
            self.blocks[l].deleted = true;
            c = self.find_min_in(r);
        }
    }

    /// Mirror of [`Self::merge_left`] over outgoing constraints. libvpsc
    /// merges the larger block into the smaller here; ported as is.
    fn merge_right(&mut self, l: usize) {
        self.set_up_heap(l, Dir::Out);
        let mut l = l;
        let mut c = self.find_min_out(l);
        while let Some(ci) = c
            && self.slack(ci) < 0.0
        {
            let h = self.blocks[l].heap_out.expect("set up"); // ui-text-exempt: panic message
            self.h_delete_min(h);
            let mut r = self.block_of(self.cons[ci].right);
            self.set_up_heap(r, Dir::Out);
            let con = &self.cons[ci];
            let mut dist = self.vars[con.left].offset + con.gap - self.vars[con.right].offset;
            if self.blocks[l].vars.len() > self.blocks[r].vars.len() {
                dist = -dist;
                std::mem::swap(&mut l, &mut r);
            }
            self.merge_blocks(l, r, ci, dist);
            self.merge_heaps(l, r, Dir::Out);
            self.blocks[r].deleted = true;
            c = self.find_min_out(l);
        }
    }

    /// libvpsc `Blocks::split`: split `b` across `c`, then re-merge each half
    /// with its neighbours.
    fn split_blocks(&mut self, b: usize, c: usize) {
        let (l, r) = self.split(b, c);
        self.live.push(l);
        self.live.push(r);
        self.blocks[r].posn = self.blocks[b].posn;
        self.merge_left(l);
        let r = self.block_of(self.cons[c].right);
        self.update_weighted_position(r);
        self.merge_right(r);
        self.blocks[b].deleted = true;
    }

    fn cleanup(&mut self) {
        let blocks = &self.blocks;
        self.live.retain(|&b| !blocks[b].deleted);
    }

    // ---- Block ----

    fn new_block(&mut self) -> usize {
        self.blocks.push(Block {
            vars: Vec::new(),
            posn: 0.0,
            ab: 0.0,
            ad: 0.0,
            a2: 0.0,
            deleted: false,
            time_stamp: 0,
            heap_in: None,
            heap_out: None,
        });
        self.blocks.len() - 1
    }

    fn add_stats(&mut self, b: usize, v: usize) {
        let var = &self.vars[v];
        let blk = &mut self.blocks[b];
        blk.ab += var.weight * var.offset;
        blk.ad += var.weight * var.desired;
        blk.a2 += var.weight;
    }

    fn add_variable(&mut self, b: usize, v: usize) {
        self.vars[v].block = b;
        self.blocks[b].vars.push(v);
        self.add_stats(b, v);
        let blk = &mut self.blocks[b];
        blk.posn = (blk.ad - blk.ab) / blk.a2;
    }

    fn update_weighted_position(&mut self, b: usize) {
        let blk = &mut self.blocks[b];
        blk.ab = 0.0;
        blk.ad = 0.0;
        blk.a2 = 0.0;
        for k in 0..self.blocks[b].vars.len() {
            let v = self.blocks[b].vars[k];
            self.add_stats(b, v);
        }
        let blk = &mut self.blocks[b];
        blk.posn = (blk.ad - blk.ab) / blk.a2;
    }

    /// A fresh heap of `b`'s constraints that cross its boundary in `dir`;
    /// stamps every constraint of `b`'s variables with the current time.
    fn set_up_heap(&mut self, b: usize, dir: Dir) {
        let heap = self.h_new();
        for k in 0..self.blocks[b].vars.len() {
            let v = self.blocks[b].vars[k];
            let count = match dir {
                Dir::In => self.vars[v].ins.len(),
                Dir::Out => self.vars[v].outs.len(),
            };
            for idx in 0..count {
                let c = match dir {
                    Dir::In => self.vars[v].ins[idx],
                    Dir::Out => self.vars[v].outs[idx],
                };
                self.cons[c].time_stamp = self.ctr;
                let far = match dir {
                    Dir::In => self.cons[c].left,
                    Dir::Out => self.cons[c].right,
                };
                if self.block_of(far) != b {
                    self.h_insert(heap, c);
                }
            }
        }
        match dir {
            Dir::In => self.blocks[b].heap_in = Some(heap),
            Dir::Out => self.blocks[b].heap_out = Some(heap),
        }
    }

    /// Moves `b`'s variables into `this` so that `c` is exactly tight;
    /// `dist` is added to each of their offsets.
    fn merge_blocks(&mut self, this: usize, b: usize, c: usize, dist: f64) {
        self.cons[c].active = true;
        for k in 0..self.blocks[b].vars.len() {
            let v = self.blocks[b].vars[k];
            self.vars[v].offset += dist;
            self.add_variable(this, v);
        }
        self.blocks[b].deleted = true;
    }

    fn merge_heaps(&mut self, this: usize, b: usize, dir: Dir) {
        match dir {
            Dir::In => {
                self.find_min_in(this);
                self.find_min_in(b);
            }
            Dir::Out => {
                self.find_min_out(this);
                self.find_min_out(b);
            }
        }
        let (a, bh) = match dir {
            Dir::In => (self.blocks[this].heap_in, self.blocks[b].heap_in),
            Dir::Out => (self.blocks[this].heap_out, self.blocks[b].heap_out),
        };
        self.h_merge(a.expect("set up"), bh.expect("set up")); // ui-text-exempt: panic message
    }

    /// The most violated incoming constraint, dropping internal ones and
    /// re-stamping those whose left block moved since they were stamped.
    fn find_min_in(&mut self, b: usize) -> Option<usize> {
        let h = self.blocks[b].heap_in.expect("set up"); // ui-text-exempt: panic message
        let mut out_of_date = Vec::new();
        while self.heaps[h].root != NONE {
            let v = self.h_find_min(h);
            let lb = self.block_of(self.cons[v].left);
            let rb = self.block_of(self.cons[v].right);
            if lb == rb {
                self.h_delete_min(h);
            } else if self.cons[v].time_stamp < self.blocks[lb].time_stamp {
                self.h_delete_min(h);
                out_of_date.push(v);
            } else {
                break;
            }
        }
        for v in out_of_date {
            self.cons[v].time_stamp = self.ctr;
            self.h_insert(h, v);
        }
        (self.heaps[h].root != NONE).then(|| self.h_find_min(h))
    }

    fn find_min_out(&mut self, b: usize) -> Option<usize> {
        let h = self.blocks[b].heap_out.expect("set up"); // ui-text-exempt: panic message
        while self.heaps[h].root != NONE {
            let v = self.h_find_min(h);
            if self.block_of(self.cons[v].left) != self.block_of(self.cons[v].right) {
                return Some(v);
            }
            self.h_delete_min(h);
        }
        None
    }

    fn can_follow_left(&self, bk: usize, c: usize, last: usize) -> bool {
        let c = &self.cons[c];
        self.block_of(c.left) == bk && c.active && last != c.left
    }

    fn can_follow_right(&self, bk: usize, c: usize, last: usize) -> bool {
        let c = &self.cons[c];
        self.block_of(c.right) == bk && c.active && last != c.right
    }

    /// d(cost)/dv over the active tree hanging from `v` (not back over `u`),
    /// storing each active constraint's Lagrange multiplier and the least
    /// in `min_lm`.
    fn compute_dfdv(&mut self, bk: usize, v: usize, u: usize, min_lm: &mut Option<usize>) -> f64 {
        let mut dfdv = 2.0 * self.vars[v].weight * (self.pos(v) - self.vars[v].desired);
        for k in 0..self.vars[v].outs.len() {
            let c = self.vars[v].outs[k];
            if self.can_follow_right(bk, c, u) {
                let lm = self.compute_dfdv(bk, self.cons[c].right, v, min_lm);
                self.cons[c].lm = lm;
                dfdv += lm;
                if min_lm.is_none_or(|m| lm < self.cons[m].lm) {
                    *min_lm = Some(c);
                }
            }
        }
        for k in 0..self.vars[v].ins.len() {
            let c = self.vars[v].ins[k];
            if self.can_follow_left(bk, c, u) {
                let lm = -self.compute_dfdv(bk, self.cons[c].left, v, min_lm);
                self.cons[c].lm = lm;
                dfdv -= lm;
                if min_lm.is_none_or(|m| lm < self.cons[m].lm) {
                    *min_lm = Some(c);
                }
            }
        }
        dfdv
    }

    fn reset_active_lm(&mut self, bk: usize, v: usize, u: usize) {
        for k in 0..self.vars[v].outs.len() {
            let c = self.vars[v].outs[k];
            if self.can_follow_right(bk, c, u) {
                self.cons[c].lm = 0.0;
                self.reset_active_lm(bk, self.cons[c].right, v);
            }
        }
        for k in 0..self.vars[v].ins.len() {
            let c = self.vars[v].ins[k];
            if self.can_follow_left(bk, c, u) {
                self.cons[c].lm = 0.0;
                self.reset_active_lm(bk, self.cons[c].left, v);
            }
        }
    }

    /// The active constraint of `b` that most wants to split.
    fn find_min_lm(&mut self, b: usize) -> Option<usize> {
        let v0 = self.blocks[b].vars[0];
        self.reset_active_lm(b, v0, NONE);
        let mut min_lm = None;
        self.compute_dfdv(b, v0, NONE, &mut min_lm);
        min_lm
    }

    fn populate_split_block(&mut self, old: usize, nb: usize, v: usize, u: usize) {
        self.add_variable(nb, v);
        for k in 0..self.vars[v].ins.len() {
            let c = self.vars[v].ins[k];
            if self.can_follow_left(old, c, u) {
                self.populate_split_block(old, nb, self.cons[c].left, v);
            }
        }
        for k in 0..self.vars[v].outs.len() {
            let c = self.vars[v].outs[k];
            if self.can_follow_right(old, c, u) {
                self.populate_split_block(old, nb, self.cons[c].right, v);
            }
        }
    }

    /// Deactivates `c` and returns new blocks holding the variables on its
    /// left and right sides of `b`'s active tree.
    fn split(&mut self, b: usize, c: usize) -> (usize, usize) {
        self.cons[c].active = false;
        let (cl, cr) = (self.cons[c].left, self.cons[c].right);
        let l = self.new_block();
        self.populate_split_block(b, l, cl, cr);
        let r = self.new_block();
        self.populate_split_block(b, r, cr, cl);
        (l, r)
    }

    // ---- Pairing heap ----

    /// libvpsc `CompareConstraints`: by slack, where a constraint whose left
    /// block moved since it was stamped, or which is internal, sorts first;
    /// ties by `(left, right)` variable id.
    fn less(&self, a: usize, b: usize) -> bool {
        let key = |c: usize| {
            let con = &self.cons[c];
            let lb = self.block_of(con.left);
            if self.blocks[lb].time_stamp > con.time_stamp || lb == self.block_of(con.right) {
                -f64::MAX
            } else {
                self.slack(c)
            }
        };
        let (sa, sb) = (key(a), key(b));
        // ui-text-exempt: lint reason
        #[allow(clippy::float_cmp, reason = "libvpsc breaks exact slack ties by id")]
        if sa == sb {
            let (ca, cb) = (&self.cons[a], &self.cons[b]);
            return (ca.left, ca.right) < (cb.left, cb.right);
        }
        sa < sb
    }

    fn h_new(&mut self) -> usize {
        self.heaps.push(Heap { root: NONE });
        self.heaps.len() - 1
    }

    fn h_find_min(&self, h: usize) -> usize {
        self.nodes[self.heaps[h].root].elem
    }

    fn h_insert(&mut self, h: usize, elem: usize) {
        self.nodes.push(PNode {
            elem,
            child: NONE,
            next: NONE,
            prev: NONE,
        });
        let n = self.nodes.len() - 1;
        let root = self.heaps[h].root;
        self.heaps[h].root = if root == NONE { n } else { self.link(root, n) };
    }

    fn h_delete_min(&mut self, h: usize) {
        let old = self.heaps[h].root;
        let child = self.nodes[old].child;
        self.heaps[h].root = if child == NONE {
            NONE
        } else {
            self.combine_siblings(child)
        };
    }

    /// Moves all of `rhs` into `h`.
    fn h_merge(&mut self, h: usize, rhs: usize) {
        let broot = std::mem::replace(&mut self.heaps[rhs].root, NONE);
        let root = self.heaps[h].root;
        self.heaps[h].root = if root == NONE {
            broot
        } else {
            self.link(root, broot)
        };
    }

    /// libvpsc `compareAndLink`; returns the node that is now `first`.
    fn link(&mut self, first: usize, second: usize) -> usize {
        if second == NONE {
            return first;
        }
        if self.less(self.nodes[second].elem, self.nodes[first].elem) {
            self.nodes[second].prev = self.nodes[first].prev;
            self.nodes[first].prev = second;
            self.nodes[first].next = self.nodes[second].child;
            let n = self.nodes[first].next;
            if n != NONE {
                self.nodes[n].prev = first;
            }
            self.nodes[second].child = first;
            second
        } else {
            self.nodes[second].prev = first;
            self.nodes[first].next = self.nodes[second].next;
            let n = self.nodes[first].next;
            if n != NONE {
                self.nodes[n].prev = first;
            }
            self.nodes[second].next = self.nodes[first].child;
            let n = self.nodes[second].next;
            if n != NONE {
                self.nodes[n].prev = second;
            }
            self.nodes[first].child = second;
            first
        }
    }

    /// libvpsc `combineSiblings`: pair left to right, then fold right to
    /// left.
    fn combine_siblings(&mut self, first: usize) -> usize {
        if self.nodes[first].next == NONE {
            return first;
        }
        let mut arr = Vec::new();
        let mut s = first;
        while s != NONE {
            arr.push(s);
            let p = self.nodes[s].prev;
            if p != NONE {
                self.nodes[p].next = NONE;
            }
            s = self.nodes[s].next;
        }
        let num = arr.len();
        let mut i = 0;
        while i + 1 < num {
            arr[i] = self.link(arr[i], arr[i + 1]);
            i += 2;
        }
        // `j` runs over even indices; `i >= 2` here since `num >= 2`.
        let mut j = i - 2;
        if j + 3 == num {
            arr[j] = self.link(arr[j], arr[j + 2]);
        }
        while j >= 2 {
            arr[j - 2] = self.link(arr[j - 2], arr[j]);
            j -= 2;
        }
        arr[0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn overlapping(a: &Rect, b: &Rect) -> bool {
        a.x0 < b.x1 - 1e-9 && b.x0 < a.x1 - 1e-9 && a.y0 < b.y1 - 1e-9 && b.y0 < a.y1 - 1e-9
    }

    #[test]
    fn solver_single_constraint_meets_in_the_middle() {
        // Both want 0; x1 must be 10 right of x0: least squares puts them at ∓5.
        let p = Solver::new(&[0.0, 0.0], &[(0, 1, 10.0)]).solve();
        assert!(
            (p[0] + 5.0).abs() < 1e-12 && (p[1] - 5.0).abs() < 1e-12,
            "{p:?}"
        );
    }

    #[test]
    fn solver_chain_of_three() {
        // x0 + 4 <= x1, x1 + 4 <= x2, all want 0: -4, 0, 4.
        let p = Solver::new(&[0.0, 0.0, 0.0], &[(0, 1, 4.0), (1, 2, 4.0)]).solve();
        for (got, want) in p.iter().zip([-4.0, 0.0, 4.0]) {
            assert!((got - want).abs() < 1e-12, "{p:?}");
        }
    }

    #[test]
    fn solver_splits_a_block_that_should_not_hold() {
        // x0 wants 0, x1 wants 0, x2 wants 100; x0+10<=x1, x1+10<=x2.
        // The optimum keeps only the first constraint active: -5, 5, 100.
        let p = Solver::new(&[0.0, 0.0, 100.0], &[(0, 1, 10.0), (1, 2, 10.0)]).solve();
        for (got, want) in p.iter().zip([-5.0, 5.0, 100.0]) {
            assert!((got - want).abs() < 1e-9, "{p:?}");
        }
    }

    #[test]
    fn solver_satisfied_constraints_move_nothing() {
        let p = Solver::new(&[0.0, 50.0], &[(0, 1, 10.0)]).solve();
        assert_eq!(p, vec![0.0, 50.0]);
    }

    #[test]
    fn pairing_heap_orders_by_slack() {
        // Constraints with slacks 5, -3, 1 between separate single-variable blocks.
        let mut s = Solver::new(
            &[0.0, 10.0, 20.0, 30.0],
            &[(0, 1, 5.0), (1, 2, 13.0), (2, 3, 9.0)],
        );
        let h = s.h_new();
        for c in 0..3 {
            s.h_insert(h, c);
        }
        let mut got = vec![];
        while s.heaps[h].root != NONE {
            got.push(s.h_find_min(h));
            s.h_delete_min(h);
        }
        assert_eq!(got, vec![1, 2, 0]);
    }

    #[test]
    fn grid_of_overlaps_is_resolved() {
        let mut rs: Vec<Rect> = (0..12)
            .map(|i| {
                let x = f64::from(i % 4) * 7.0;
                let y = f64::from(i / 4) * 6.0;
                Rect {
                    x0: x,
                    x1: x + 10.0,
                    y0: y,
                    y1: y + 10.0,
                }
            })
            .collect();
        remove_overlaps(&mut rs);
        for i in 0..rs.len() {
            assert!((rs[i].len(Dim::X) - 10.0).abs() < 1e-9);
            for j in i + 1..rs.len() {
                assert!(
                    !overlapping(&rs[i], &rs[j]),
                    "{i} {j}: {:?} {:?}",
                    rs[i],
                    rs[j]
                );
            }
        }
    }
}
