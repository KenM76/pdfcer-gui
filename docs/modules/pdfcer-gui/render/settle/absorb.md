# `render::settle::absorb` — what to do with the picture that came back

The parent module decides what the picture *should* be: what is stale, what
to re-rasterize now, what to debounce, which strip page is next. This one
handles the other half of that exchange — handing a page to the worker,
collecting the result, and turning whatever came back into a cached texture,
a promoted backdrop, an ink census or a learned zoom ceiling.

The two change for different reasons, which is the whole of why they are
separate files. The parent changes when the operator gains a gesture or the
strip gains a scheduling rule. This one changes when the **renderer** gains
a failure mode.

## What is here

- [`OpenDoc::rasterize`] — request a page, and absorb it inline if it beats
  the in-frame budget.
- [`OpenDoc::poll_render`] — collect a background render, once per frame.
- `absorb_render` — the single place a finished render becomes canvas state,
  shared by both of the above so the fast path and the slow path cannot
  drift. Routing is by the render's **own** page, never by which slot asked.
- `learn_raster_ceiling` — where a raster refusal becomes a zoom ceiling
  instead of an error banner (O186). The number can only come from a refusal
  that has already happened; see [`crate::render::ceiling`].

## Why `rasterize` is here and not with the scheduling

It reads as a scheduling verb, and moving it would have been the tidier cut.
It is here because it **absorbs**: it writes `render_in_flight`, whose only
reader is `absorb_render`, and the argument for why that stamp is taken
before the inline wait rather than after it lives inside `rasterize`. Split
along the name rather than along the subject and the stamp's two writers end
up on opposite sides of the cut, with the invariant documented on the far
side from the code that depends on it.

## Reach

`rasterize` and `poll_render` are `pub(super)` because the parent's
`settle_and_rasterize` and `fill_strip` call them; `absorb_render` and
`learn_raster_ceiling` stay private because their only callers moved here
too. ⚠ `super` is `render::settle`. If this file is ever re-homed as a
sibling of `settle.rs` rather than a child, both spellings have to widen to
`pub(in crate::render)` — the compiler will say so, but the reason will not
be obvious from the error.

## Item notes

### `fn absorb_render`

Shared by the in-frame fast path and the per-frame poll so the two
cannot drift: a render that beat the budget and one that took a minute
must produce exactly the same canvas state.

# The routing is by the render's own page, not by what asked for it

A render is labelled with the [`RenderKey`] it was run from, so the
page it is *of* is knowable from the result alone. That is what makes
the scroll case correct: a strip page whose render finishes after the
operator has scrolled onto it lands in the current page's slot, and the
current page's render that finishes after they have scrolled past it
lands in the strip. Routing by "which slot asked" would have to
remember the request, and would be wrong in exactly those two cases.

# It is also where a raster refusal becomes a zoom CEILING — O186

The operator, 2026-09-12: *"zoom should stop at the limit and not end up
showing an error — the canvas will just stop zooming in and can still
function."* This function is the one place in the shell a render refusal
is absorbed, so it is necessarily the place that clause is executed: see
the `Err` arm's own commentary for the learn-and-pull-back, and
[`crate::render::ceiling`] for why the number can only come from a
refusal that has already happened.

### `fn learn_raster_ceiling`

Returns whether the refusal was *fully absorbed*: a `true` means the
operator is standing at a zoom this page is believed able to draw, so
the caller must not also file an error. A `false` means no such zoom is
left — nothing was learned, or the ceiling is under
[`crate::viewer::MIN_ZOOM`] — and the ordinary refusal path must run.
The conditions are enumerated at the call site, which is the only
caller.

# It answers *where he stands*, not *did the zoom move*

O218. A refusal names the scale that was **ordered**, not the scale that
is wanted now, and the two separate whenever a deep render takes long
enough for the operator to wheel back out while it runs. Scoring such a
refusal on whether the clamp *moved* the view reports a failure on a view
that is already correct — and the caller answers a failure by blanking
the canvas and painting a sentence telling him to do the thing he has
just done.

The ceiling is read back from [`crate::render::ceiling::RasterCeiling`]
rather than taken from `learn`'s return, for the same reason. `learn`
declines a repeat, and a repeat is the commonest shape of a stale
refusal; what the caller needs is the ceiling **in force**, which
`for_page` gives whether or not this particular refusal moved it.

# Why the clamp is expressed in ZOOM and the ceiling in raster SCALE

They are different quantities and the conversion between them is the
display's density: a raster scale is *device pixels per PDF point*,
which is `zoom * pixels_per_point`. The ceiling is stored in scale
because that is what the renderer refused and the only quantity the
refusal is a fact about — move the window to a 200 % monitor and the
same zoom orders twice the pixels, so a ceiling stored in zoom would be
wrong by a factor of two on the other screen, silently, and only on the
machine it was not measured on.

So `pixels_per_point` is read here, at the moment of the clamp, and
again on every frame by [`crate::viewer::zoom_ceiling`]. Neither caches
it. A dragged window is a real gesture on this operator's desk — he runs
two monitors at different densities — and a cached density is a ceiling
that is wrong exactly after the drag.

# Why the clamp is GUARDED, when `set_zoom` clamps already

`set_zoom(target, target)` does not *lower* a zoom — it **assigns** one.
It clamps into `[MIN_ZOOM, max]` with the value equal to the bound, so it
lands on `target` from either side. Called unconditionally on a refusal
that arrived from a scale the operator has already left, it would carry
him back *up* toward a wall he had backed away from, and drop his fit
mode on the way. `before > target` is a correctness guard, not an
optimisation.

# Why it floors at [`crate::viewer::MIN_ZOOM`] and then checks again

`set_zoom` will not go below `MIN_ZOOM`, so a ceiling under it leaves the
view *above* the ceiling with nowhere further to go: the page cannot be
drawn at any zoom this viewer offers. That is the one case that must
report `false`, and it is read off the result rather than predicted
before the call, so the arithmetic that decides it is the arithmetic that
ran.
