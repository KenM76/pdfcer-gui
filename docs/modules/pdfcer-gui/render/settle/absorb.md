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
