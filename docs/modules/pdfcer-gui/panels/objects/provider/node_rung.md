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

## ★★★ Why this is a module and not more methods in [`super`]


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

★ The Point rung's **tests do not live here**, and that is deliberate
rather than an omission. [`super::node_rung_tests`] covers the Part rung
(`part_hits`, `part_bounds`) as well as the Node rung, and the Part rung
stayed in [`super`]; splitting that file to follow this one would make a
review of a pure move indistinguishable from a review of a rewrite, which
is the reason its own header gives for not having been merged with
[`super::tests`] in the first place.
