# `pdfcer-gui/canvas/strip`

## Item notes

### `fn the_render_order_starts_at_the_middle_of_the_viewport`

`render::settle` starts one render per frame and takes the first entry
of this list that has no raster, so the order **is** the fill order. Top
down would mean that whenever the operator has scrolled to a page
boundary — which is when a continuous mode is being used — the page they
are reading is the last one to arrive.

### `fn a_page_command_scrolls_the_strip_and_a_scroll_does_not`

The gate this function exists for, from both sides. Without the first
half, "Next page" in a continuous document does nothing at all; without
the second, every frame of every scroll would fight the operator by
snapping back to the page the last frame derived.

### `fn the_forced_offset_stays_inside_the_scroll_range`

This asserted `Vec2::ZERO` until 2026-08-21, on the reasoning that a
viewport taller than the whole strip has nowhere to scroll. O23 made
that false on purpose: there is a pasteboard of one viewport on every
side, so even a document that fits entirely on screen can be moved
around — which is the operator's *"move the view of the corner of the
page to the center of the screen"* for a small document.

So the assertion is the INVARIANT rather than the number. Pinning the
new number would say nothing about whether it is reachable, and this
function's whole job is that its answer is inside the range egui will
accept — an offset beyond it is silently clamped, and the page then
does not appear where the navigation promised.
