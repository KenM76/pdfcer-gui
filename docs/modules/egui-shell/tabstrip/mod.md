# `egui-shell/tabstrip/mod`

## Item notes

### `const CLOSE_WIDTH`

Fixed rather than measured, because it is a fixed glyph in a fixed size and
measuring it would make the tab-width arithmetic depend on the font — which
is exactly the dependency that makes an overflow reservation drift.

### `const CLOSE_GLYPH`

**U+00D7 MULTIPLICATION SIGN**, not U+2715 MULTIPLICATION X and not U+2716.
It is in Latin-1, so it is present in every font this application could
possibly fall back to, and `epaint`'s `has_glyph` is not a coverage oracle
(`D:\dev\rag\egui\epaint_has_glyph_is_resolved_face_vs_replacement_face_not_a_coverage_oracle.md`)
— so "will this render?" is a question best answered by not asking it.

### `struct TabDrag`

In `egui::Memory` rather than in a field on [`TabStrip`], because
[`TabStrip`] is built fresh every frame — the strip is deliberately
stateless (see [`TabItem`]) and a drag has to outlive a frame. This is the
only thing about the strip that does.

### `fn settle_reorder`

Runs after every tab is laid out, because the boundary the caret marks does
not exist until they are — the same reason a page grid resolves its drop
target inside its layout pass.

# The gap is resolved by CENTRES, not by edges

A tab whose centre is left of the pointer is a tab the dragged one has
passed. That makes the boundary flip when the pointer crosses the middle of
a neighbour, which is what every tab strip does and what stops the caret
jittering between two gaps while the pointer sits over the seam between two
tabs.

### `const CARET_PTS`

The same weight the dock's and the page grid's carets use: thin enough to
read as a boundary rather than as a tab, thick enough not to look like a
rendering artefact on a dense strip.

### `fn draw_overflow`

A plain menu of the open tabs, shown only once some of them do not fit —
see the loop below on why it lists every tab rather than only the hidden
ones. Unlike the dock's, it offers only
activation: closing a document you cannot see is not a gesture any
application offers, and offering it here would be inventing one.

### `fn a_crowded_strip_reserves_room_for_the_overflow_affordance`

`MODES_AND_PANELS.md` failure mode #8, asserted from this side of the
reuse: the arithmetic is `dock::plan`'s and is tested there, and this
is the check that this module actually *uses* it rather than laying
tabs out itself and reporting a plausible `hidden`.

The assertion is that the drawn tabs stop short of the strip's right
edge by at least the affordance's width — measured from the published
rectangles, not recomputed.
