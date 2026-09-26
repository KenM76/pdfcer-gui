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
of *different places*. `render::settle`'s staleness test asked two questions
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
