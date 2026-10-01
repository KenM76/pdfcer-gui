# `ui-verify/checks/pan_refresh`

`panning_past_the_overscan_renders_the_new_area` — the operator's blank
strip, made falsifiable.

# The report


> *"if I pan to far to one side when I am beyond 800% zoom it doesn't always
> render the new exposed area, and the same thing happens usually when I
> zoom out."*

# What was actually wrong, and why every existing check was green

Above the pixmap ceiling a raster covers the **visible region** rather than
the page, so two textures of the same page at the same scale can be pictures
of *different places*. `app::settle`'s staleness test asked two questions
— has a discrete input changed (page, annotations, layers), and has the
scale changed — and **the region was in the cache key without being in
either**.

So a pan that changed nothing but which part of the page is on screen was
not stale by any measure, and no render was ever requested. The picture the
operator had kept being drawn correctly at its own region and simply slid
off, leaving the newly exposed area blank for as long as they cared to look
at it.

Every check passed throughout. `panning_at_deep_zoom_stays_where_it_was_put`
asks whether the view *moves* and whether the pixels are *placed* correctly
— both were perfect. `the_page_still_renders_at_every_decade_of_zoom`
photographs after a **zoom**, which does change the scale and therefore does
request a render. Nothing in the suite panned far enough to leave the
overscan and then looked at the screen.

# What this asserts

Pan by more than a whole viewport, so the destination is certainly outside
`render::strategy::OVERSCAN`'s half-viewport margin, then require **both**:

| | rules out |
|---|---|
| a render completes after the pan | the shell never asked, which is O25 |
| the canvas is not near-uniform afterwards | it asked, and what arrived is blank anyway |

Both, because either alone is satisfiable while the operator looks at
nothing: a render can complete for the region the view has already left, and
a canvas can be non-uniform because of the page's *edge* while its middle is
empty.

## Item notes

### `const RENDER_INLINE_EVENT`

Added 2026-08-28, after this check reported *"NO RENDER WAS REQUESTED"*
against a build that had spawned and completed **nineteen** of them.

`render::worker` has two completion paths and takes whichever is cheaper: a
raster that finishes fast enough is done **inline**, on the frame that asked
for it, and only a slow one goes to the thread and comes back as
`render-async-done`. A region raster above the pixmap ceiling covers the
viewport rather than the page, so it is *small* — 3 ms on the fixture this
check now uses — and it never takes the asynchronous path at all.

⇒ **A check that counts one completion path fails on a build that took the
other one**, and it fails by naming the feature rather than the instrument.
This one printed `app::settle`'s staleness test as the suspect, in detail,
down to `RenderKey::same_region` — and that mechanism was working perfectly.

The general rule, which this project has now met three times in one day:
**ask what the check SAMPLED before asking what is broken.** A failing
measurement is a claim about an instrument as much as about a program.

### `const ZOOM_NOTCHES`

Enough to be **past the pixmap ceiling**, which is where a raster stops
covering the page and starts covering the window — the tier this check is
about. Below it a pan is free and this check would be measuring nothing.
Twenty notches lands around 4,000 % on a Letter sheet, comfortably above the
~2,070 % crossover.

### `const PAN_NOTCHES`

`render::strategy::OVERSCAN` gives the raster half a viewport of margin on
every side, and `region_for` snaps to a half-viewport grid — so a pan has to
exceed a whole viewport before the operator is certainly looking at
something the current raster does not contain. This is the "too far to one
side" in his report, made specific.

### `const ZOOM_OUT_NOTCHES`

Enough to change the region substantially while staying **above the
crossover** — dropping below it in one step would put the raster back on the
whole-page path, where this defect cannot occur and the check would be
measuring the wrong tier.

### `fn renders_done`

The asynchronous line carries an `outcome`, because a thread can come back
with a cancellation or a failure; the inline one cannot fail asynchronously
and carries none, so it is counted unconditionally. Two shapes for one fact,
and the asymmetry is the worker's rather than this function's.

### `fn field`

Compared as text rather than parsed: this only needs to know *whether it
changed*, and the trace prints both regions from the same bits the cache
keys on, in the same format.

### `fn wanted_region`

`region=` is what the pixels on screen are a picture of, and on a build with
O25 present it never changes — no render is requested, so no new texture
arrives, so the field that describes the texture stands still. A check
watching it reads *"the view did not move"* and skips, which is what the
first version of this check did against a binary with the defect
deliberately restored.

`want=` moves the instant the view does. **The gap between the two is the
defect**, and it takes both fields to measure a gap.
