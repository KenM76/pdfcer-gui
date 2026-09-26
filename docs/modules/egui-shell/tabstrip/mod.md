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

### `const STRIP_HEIGHT`

A constant, and it must stay one. A surface whose height varies with its
content and which sits above a viewport that fits a page to itself forms a
measured feedback loop — `D:\dev\rag\egui\bottom_panel_height_change_retriggers_fit_to_viewport_zoom.md`
records a 230 % → 224 % → 215 % zoom drift from exactly that shape. The
caller is expected to give this an `exact_size` panel, not a
`default_height` one.

Two points taller than [`plan::TAB_BAR_HEIGHT`], deliberately: a document
tab carries a close glyph beside its label and a 24 pt row makes the two
touch.

### `struct TabItem`

Built fresh each frame by the caller from whatever it has open. There is no
retained model here on purpose: a strip that cached its own list would be a
second copy of *what is open*, and the two would disagree the first time a
close failed.

### `fn strip`

`active` is the index of the tab currently on screen; out-of-range is
treated as "none of them is active", which is a state a caller can reach
legitimately for one frame while a close is being confirmed.

Returns intents. Applies nothing.

# Layout, in the order that makes the reservation hold

The same six steps [`crate::dock::tabs`] documents, because it is the same
arithmetic:

1. measure every label (egui memoizes the galley, so this is a hash lookup)
2. add [`CLOSE_WIDTH`] to each, then clamp through [`plan::tab_width`]
3. subtract the overflow affordance's width from the strip's — **first**
4. choose the visible window inside what is left
5. lay the tabs into a rect that *is* that budget
6. lay the affordance into the space nothing else was allowed to touch

Step 3 before step 4 is what makes it impossible for the number of tabs to
eat the route to the tabs.

A seventh step is this module's own and has no counterpart there: the
reorder caret, resolved and painted after everything else, because the
boundary it marks does not exist until the tabs are laid out.
