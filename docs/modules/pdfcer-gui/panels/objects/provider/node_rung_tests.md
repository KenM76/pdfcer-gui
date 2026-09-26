# `provider::node_rung_tests` — the Part and Node rungs, on real geometry

The second of `provider.rs`'s two inline test modules, moved out for R2
with its contents unchanged. Kept **separate** from
[`super::tests`](crate::panels::objects::provider::tests) rather than
merged, because it was separate before the move and merging two test
modules while relocating them would make a review of the move
indistinguishable from a review of a rewrite.

Its subject is the two rungs below the object: which subpath a click lands
in, which anchor is nearest, and — the law it exists for — that node
indices stay **object-scoped** across a part boundary, because that is the
space `vector::anchor_count` reports and `pdfcer node-move --node N`
addresses. A second numbering would make the number pdfcer shows disagree
with the number the operator can act on.

★ Everything here addresses the **page's** paint order, because
`part_hits`, `part_bounds` and `nearest_node` all index
`PageObjects::objects`. That is a fact about THIS provider's geometry
helpers and it is still true.

★★ The paragraph used to end *"the ladder stops at the Object rung for a
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


★ **The line gate still counts these lines.** `check-file-size.sh` counts
total lines, tests included, on purpose — its own header says so — so
this split is not a way of hiding lines from R2. It is the split R2 asked
for, taken on the seam that was already there.
