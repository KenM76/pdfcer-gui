# `pdfcer-gui/canvas/strip`

## Item notes

### `fn the_render_order_starts_at_the_middle_of_the_viewport`

`app::settle` starts one render per frame and takes the first entry
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

### `struct DrawnPage`

Every visible page gets a `click_and_drag` response, not only the current
one, because pressing on a page is how the operator moves to it under a
continuous mode — see [`show`]. The response of exactly one of them becomes
the frame's interaction response.

### `struct PageView`

The Find wash's unit of work. Separate from [`DrawnPage`] because it
outlives the scroll-area closure and holds no `Response`, and because
[`Frame`] is `Copy` while a `Response` is not.

### `fn current_page_state`

The strip cache answers this for every other page ([`OpenDoc::strip_page_state`]);
the current page's answer comes from its own two fields, because its raster
lives in its own slot. See [`crate::render::strip`]'s header on why the
split exists.

`None` is unreachable in practice from the one call site — it is only asked
when there is no texture — and is answered as "waiting" rather than by
panicking, because the honest thing to draw on a page with no picture and
no stated reason is that it has not been drawn.

### `fn nearest_first`

The order [`crate::app::settle`] fills the strip in. Nearest-first rather
than top-down because the operator is looking at the middle of the viewport:
filling from the top means the page they are reading is the last one to
arrive whenever they have scrolled to a boundary, which is exactly the
moment a continuous mode is being used.

`pages` is `(page index, the page's vertical centre on screen)` and
`centre_y` is the viewport's own vertical centre, both in **screen**
coordinates — one space, so there is no conversion here to get wrong. The
pair is passed rather than the [`DrawnPage`] slice so the ordering rule is a
pure function with a test ([`tests::the_render_order_starts_at_the_middle_of_the_viewport`]);
a `Response` cannot be constructed headlessly, and an ordering rule that
could only be checked by running a window is an ordering rule nobody checks.

### `fn page_scroll_offset`

The third source of a forced scroll offset, and the one Phase 4 adds. Under
a continuous mode a page **command** — Next page, the status bar's page box,
a bookmark, a Find hit that landed with no geometry — changes
`view.page_index` and nothing else, so without this the operator would press
"Next page" and watch nothing happen.

# The gate is `page_index != tracked_page`, and nothing weaker works

The canvas writes `page_index` itself on every frame from the scroll
position, so "the index changed" is true of every frame of every scroll and
cannot be the test. `tracked_page` records what the *canvas* last derived;
a difference therefore means something else wrote the field, which is
precisely the definition of a navigation. See
[`crate::app::state::OpenDoc::tracked_page`].

# Where the page is put, and why it is the top rather than the centre

The page's top edge goes to the top of the viewport, less the strip's
row gap so the sheet does not sit flush against the edge. Not centred:
"Next page" means *show me that page*, and a reader expects to arrive at
the top of it and read downwards. Centring a page shorter than the viewport
would also scroll the previous page's foot into view above it, which reads
as having overshot.

Returns `None` in the paged modes, where a page command changes what is
laid out rather than where it is — there is nothing to scroll to.

### `fn track_current_page`

One of the four items of per-frame view bookkeeping the canvas is permitted
to write directly (see `canvas`'s module header): a scroll position cannot
be deferred into an `Action`, because the action would be applied after the
frame that has already drawn from it.

Lives here rather than in `canvas::show` because it is a question about the
**strip** — where the viewport falls across a column of pages — and because
R2's line limit is a prompt to find the seam rather than to raise the
limit. Its scroll-space conversion is the same one every other consumer of
`Strip` needs, and having it beside them is what makes the omission that
caused O26 visible next time.
