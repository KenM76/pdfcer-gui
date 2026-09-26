# `egui-shell/dock/geometry`

## Item notes

### `fn hit`

Last match wins: compartments are recorded in draw order and do not overlap,
so the choice is immaterial for stacks — but a later entry is the one drawn
on top, which is the answer a pointer gesture wants if that ever changes.

### `struct StackAddr`

Positional for [`PanelAddress`]'s reason: this model has no stable handle
for a stack and deliberately does not want one. The consequence is the same
one [`super::float::DockHome`] carries — an address can go stale across an
edit — and the answer is the same: addresses are used **within** a frame, or
rebuilt against the layout before they are trusted.

### `fn is_empty`

True on the first frame, and on any frame where both sides are empty or
collapsed. A caller that hit-tests an empty geometry gets `None`
everywhere, which is the correct answer rather than a special case.

### `fn push_tab`

Only tabs that were **actually drawn** are recorded: a tab in the
overflow menu has no rect on the strip, and inventing one from its index
and width would be deriving a coordinate the layout already knows — the
rule [`crate::tabstrip::TabStrip::drawn`] carries, for the reason that a
harness computing a position from an index can be wrong in the same
direction as the code under test.

### `fn strip_at`

Separate from [`Self::stack_at`] because the two answer different
questions for a drop: over the strip means *"between these two tabs"*,
over the body means *"into this group, or splitting it"*.

### `fn gap_in`

`0` is before the first drawn tab and `n` is after the last, which is the
vocabulary an insertion caret is drawn in and the one
[`crate::tabstrip::TabIntent::Reorder`] and the dock's own reorder
intent both use. It is deliberately not
*"the index it ends up at"*: those differ by one whenever a tab moves
rightward, because the tab is removed before it is re-inserted, and a
caller that got the convention wrong would be off by one in one
direction only — the hardest kind of off-by-one to see.

## Resolved by CENTRES, not by edges

A tab whose centre is left of the pointer is a tab the dragged one has
passed. The boundary therefore flips when the pointer crosses the middle
of a neighbour, which is what every tab strip on this desktop does, and
which is what stops the caret jittering between two gaps while the
pointer rests on the seam between two tabs.

## Seeded from the FIRST DRAWN tab, not from zero

A strip whose leading tabs are in the overflow menu starts at a non-zero
index. Seeding at zero would let a pointer at the left edge report a
boundary left of everything on screen — a caret drawn in one place and a
move committed somewhere else.

`None` when the stack drew no tabs at all.
