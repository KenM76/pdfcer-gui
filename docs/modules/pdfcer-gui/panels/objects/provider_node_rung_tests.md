# `panels::objects::provider_node_rung_tests` — the Part and Node rungs, on real geometry

The second of `provider.rs`'s two inline test modules, moved out for R2
with its contents unchanged. Kept **separate** from
`panels::objects::provider_tests` rather than
merged, because it was separate before the move and merging two test
modules while relocating them would make a review of the move
indistinguishable from a review of a rewrite.

Its subject is the two rungs below the object: which subpath a click lands
in, which anchor is nearest, and — the law it exists for — that node
indices stay **object-scoped** across a part boundary, because that is the
space `vector::anchor_count` reports and `pdfcer node-move --node N`
addresses. A second numbering would make the number pdfcer shows disagree
with the number the operator can act on.

Everything here addresses the **page's** paint order, because
`part_hits`, `part_bounds` and `nearest_node` all index
`PageObjects::objects`. That is a fact about THIS provider's geometry
helpers and it is still true.

The paragraph used to end *"the ladder stops at the Object rung for a
leaf, and it stops there because the address space runs out"*, citing
`FormLeaf::is_editable` being `false` for every leaf. **Corrected
2026-09-11: the ladder does not stop.** `pdfcer-core`'s `Pass 188.0`
shipped form-scoped part and node verbs, `canvas::moving::eligible`
routes to them, and `is_editable` now means *"this leaf is a path"*
rather than *"nothing in a form can be edited"*.

What is genuinely absent is a form-interior equivalent of the three
helpers above, and the coverage for it lives where the routing does
rather than here. Saying *"there is no equivalent"* and saying *"the
capability does not exist"* read the same in a test header and are not
the same claim; this one was making the second while meaning the first.

## `#![cfg(test)]` at the top, and why it is the marker rather than the name

Two gates recognise the **inner attribute** as meaning *"none of this is in
the shipped binary"* — `check-ui-strings.sh` and `check-theme-colors.sh`
— and both state why they match on that rather than on a filename: the
property that earns the exemption is not being in the binary, and a
filename is a restatement of it that goes stale the moment a third such
module is written.


**The line gate still counts these lines.** `check-file-size.sh` counts
total lines, tests included, on purpose — its own header says so — so
this split is not a way of hiding lines from R2. It is the split R2 asked
for, taken on the seam that was already there.

## Item notes

### `fn the_second_parts_points_keep_counting_from_the_first`

This is decision 025 §1.3(b) made testable. The pick set is scoped to
one part, but the numbering is not — because the number pdfcer shows
and the number `pdfcer node-move --node N` addresses have to be the
same number. A subpath-scoped index would restart at 0 on the second
part and quietly address a point in the first.

The Objects panel's point rows print these numbers, which is what
makes this a live invariant at S3 rather than an S4 one.

### `fn the_object_wide_point_list_matches_the_parts_concatenated`

Two functions walk the same anchors in the same order and both hand
out object-scoped indices. If they ever disagreed, a multi-node drag
would move a different point from the one the panel row named — and
nothing about that looks wrong at the moment it happens.

### `fn a_parts_pick_set_excludes_every_other_part`

The whole reason the rung exists: a measured CAD object holds 6,681
anchors, and offering all of them as a grab target is what made the
old ungated gesture unpredictable.

### `fn a_cubics_two_handles_belong_to_the_nodes_at_its_two_ends`

Segment k runs from anchor k to anchor k+1, so `c1` shapes the curve
LEAVING anchor k and `c2` shapes the curve ARRIVING at anchor k+1.
Assigning both to one node would look plausible, draw two handles in
roughly the right place, and make every handle drag move the wrong end
of the curve.

### `fn a_straight_part_has_no_handles_at_all`

pdfcer refuses to turn a line into a curve without being asked, so the
absence must show up as nothing drawn — not as a placeholder sitting
on the node, which would advertise an edit that will be refused.

### `fn the_short_curve_spellings_still_yield_two_handles`

Worth pinning because the GUI would otherwise need to know about the
short spellings, and getting `v` (first control = current point) and
`y` (second control = endpoint) confused is the classic error in this
operator family.
