# `provider::node_rung` — the Point rung's pick sets: anchors and handles

Everything this shell knows about **the points a path is made of**: which
anchors belong to which subpath, what number each one answers to, which
Bézier control points shape the curve on either side of it, and which of
those a press lands on. It is the third rung of the selection ladder —
*object → part → point* — answered for the page's own paint order.

## What lives here

Six inherent methods on [`ObjectModelProvider`], in two pairs and a pair of
picks:

| method | answers |
|---|---|
| [`ObjectModelProvider::subpath_node_points`] | the anchors of ONE subpath, each with its **object-scoped** index |
| [`ObjectModelProvider::object_node_points`] | the same, flattened across every subpath of the object |
| [`ObjectModelProvider::subpath_handle_points`] | every cubic control point of one subpath, tagged with the node it shapes and which side |
| [`ObjectModelProvider::node_handles`] | the at-most-two handles of ONE anchor — the per-selection form |
| [`ObjectModelProvider::nearest_node`] | which anchor a canvas-space press picks, within a tolerance |
| [`ObjectModelProvider::nearest_handle`] | which handle a page-space press picks, checked **before** nodes |

Their own doc comments carry the reasoning — the object-scoped/subpath-scoped
index split of decision 025 §1.3(b) and decision 028 §Q1, the segment↔anchor
off-by-one that decides which side a handle is drawn on, why handles are
hit-tested ahead of nodes (028 §Q3), and why a closed subpath's closing
segment deliberately offers no handle. Every word of that came across with
the code it is about; nothing was summarised on the way.

## Why this is a module and not more methods in [`super`]


**The seam chosen is the Point rung.** [`super`] answers *"what is on this
page, and where is it?"* — it owns the decomposition, the canvas↔PDF
projection, the object- and part-level hit tests, the marquee, and the
`TargetId` encoding. This module answers the strictly narrower question
*"what points is one already-identified part made of, and which one did the
operator just grab?"* That is one coherent responsibility with one index
convention to keep straight, and the index convention is the reason it is
worth reading alone: **a node index is object-scoped even though the pick
set is subpath-scoped**, and the running-offset arithmetic that makes that
true appears in three of the six methods. Keeping those three within one
screen of each other is the point — they must agree, and now they can be
checked against each other without scrolling past the marquee.

It is the same cut [`super::geometry`] made for the `_of` family, in the
other direction: `geometry` generalises the *address space* (page index vs
[`TargetId`]), this one isolates the *rung*.

## How it fits the provider

A pure `impl ObjectModelProvider` block over the same struct `mod.rs`
defines — no new type, no new state, no re-export, no change to any
signature or any call site. Rust's privacy rules make a child module see
its parent's private items, so these methods still read `self.objects`
directly and still reach [`super`]'s private `canvas_to_pdf` for the one
canvas-space input among them ([`ObjectModelProvider::nearest_node`]). The
move is therefore invisible from outside `provider`: the panel, the canvas
and the measure tools call exactly what they called before.

The Point rung's **tests do not live here**, and that is deliberate
rather than an omission. [`super::node_rung_tests`] covers the Part rung
(`part_hits`, `part_bounds`) as well as the Node rung, and the Part rung
stayed in [`super`]; splitting that file to follow this one would make a
review of a pure move indistinguishable from a review of a rewrite, which
is the reason its own header gives for not having been merged with
[`super::tests`] in the first place.

## Item notes

### `fn node_handles`

Returns at most two: the control point governing the curve as it
*arrives* at the anchor and the one governing it as it *leaves*. Either
or both are absent when the neighbouring segment is a straight line or
there is no neighbouring segment at all — which is the ordinary case on
a CAD drawing, where almost every path is polygonal.

# Why this is per-ANCHOR and every other point accessor is per-subpath

Because handles are only ever drawn for the anchors the operator has
selected, and that is not a cosmetic decision. A subpath's anchors are
its skeleton and are worth showing all at once; its handles are two per
anchor and are *inside* the shape, so drawing every one turns a curve
into a thicket and hides the outline the operator is working on. Every
vector editor draws them for the selection alone, and this accessor's
shape is what makes that the cheap path rather than a filter over a
list that was expensive to build.

# How an anchor index maps onto segments, and the off-by-one in it

`Subpath` is `start` plus a list of `segments`, and `anchors()` yields
`start` first and then each segment's end. So for object-scoped anchor
`k` **within this subpath** (0-based):

- its **incoming** handle is `segments[k - 1].c2` — the second control
  point of the segment that ends *here*. Absent for `k == 0`, which has
  no segment before it.
- its **outgoing** handle is `segments[k].c1` — the first control point
  of the segment that starts *here*. Absent for the last anchor.

Getting that backwards produces handles that are drawn on the wrong side
of the anchor and drag the wrong curve, which looks like a coordinate
bug rather than an indexing one. It is stated here because
`pdfcer_core::vector::Handle`'s own doc comment states it, and the two
must agree: the `Handle` value returned here is passed straight to
`EditSession::move_handle`.

A **closed** subpath's first anchor also has an incoming handle — from
the closing segment — and that is deliberately NOT returned. The closing
segment of an `h`-terminated subpath has no operands of its own in the
content stream, so there is nothing for `move_handle` to rewrite, and
offering a handle the engine will refuse is the "visible control,
silently inert" failure this project keeps finding.

### `fn object_node_points`

# Why the whole object and not one subpath

A multi-node **selection** is object-scoped: nothing stops an operator
Ctrl-clicking one anchor on a shape's outer subpath and another on a
hole inside it, and a selection set holds both by their object-scoped
index. A multi-node **drag** therefore has to look up positions across
the whole object — asking per-subpath would mean the caller
re-deriving which subpath each selected index falls in, which is
exactly the offset arithmetic [`Self::subpath_node_points`] exists to
keep in one place.

Empty for a non-path object, for the same reason
[`Self::subpath_count`] returns `0`.

### `fn subpath_handle_points`

# Which handle belongs to which node

A cubic segment carries two control points, and they belong to
*different* nodes — this is the part that is easy to get backwards.
Segment `k` runs from anchor `k` to anchor `k+1`, so its `c1` shapes
the curve LEAVING anchor `k` and its `c2` shapes the curve ARRIVING
at anchor `k+1`. That is exactly the split
[`pdfcer_core::vector::Handle`] names, and it is why the enum is worded
by direction of travel rather than "first/second": first-and-second
are properties of a *segment*, and a segment says nothing about which
node the operator selected.

Straight segments contribute nothing. pdfcer refuses to invent a handle
for a line — turning a line into a curve is a different operation with
a different name — so a node with no curve on a side simply has no
mark there, and the absence is stated in the readout rather than drawn
as a ghost (decision 028 §Q2).

`v`/`y` implicit control points need no special handling here: the
decomposition already resolves them into explicit `c1`/`c2`
(`Segment::Cubic`'s own doc comment), so this sees one uniform shape
and the promotion-to-`c` happens far downstream in the planner.

### `fn nearest_handle`

# Why handles are hit-tested BEFORE nodes

A handle sits close to its own node exactly when the curve is nearly
flat there. If the node won ties, the handle would be unreachable
precisely in the case where the operator most wants it — to pull a
flat segment into a curve. Checking the smaller target first is the
standard resolution and the one decision 028 §Q3 specifies.

`point` is in **PDF page space**, unlike [`Self::nearest_node`]'s
canvas-space input: the only caller is the drag classifier, which has
already converted the press origin to page space to compute the drag's
reference point. Converting back to canvas just to convert forward
again would be two chances to disagree with itself for no benefit.

### `fn nearest_node`

Takes canvas space and converts internally, exactly as
[`Self::subpath_hits`] does, so the canvas→PDF frame conversion stays
in the one place that owns it rather than being re-derived by each
caller. `tolerance` is in PDF units, already converted from screen
pixels by the caller.

**Ties resolve to the lower index**, which is the same rule the vector
edit tool's own nearest-anchor search uses — so a point equidistant
from two anchors picks the same one whether it was reached by clicking
or by dragging. (That tool lands at S5; the rule is stated here rather
than cross-referenced so it survives being read alone.)
