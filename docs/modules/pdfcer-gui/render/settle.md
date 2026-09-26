# `render::settle` — is the picture on screen still a picture of what I am looking at?

The per-frame raster decision: what is stale, what to re-rasterize now,
what to debounce until a zoom gesture stops, and which of a continuous
strip's several visible pages to render next.

## Where the boundaries are, on both sides

[`crate::app::state`] answers *"what is open, and what is the operator
looking at?"*. This module answers *"what does the picture need to be, and
what should be done about it this frame?"*. The two change for different
reasons — the first when a document gains a property, the second when
rendering gains a strategy — and only the second belongs in `render/`,
beside the worker it schedules and the texture cache it prunes.

The child module `absorb` takes the other half of that second question:
not what the picture should be, but what to do with the one that came back.
It changes when the **renderer** gains a failure mode, which is a third
reason again, and it is where a raster refusal becomes a zoom ceiling.

## Rendering happens on state change, never per frame

egui redraws continuously; rasterizing a PDF page at 60 Hz would be absurd.
Staleness is a [`RenderKey`] comparison — page, raster scale, annotation
visibility, layer-override generation — and there is deliberately no second
field list to keep in step with it: the key the worker labelled a texture
with is compared against the key the current view wants, and a field added
to the type changes both sides at once.

**Two staleness policies apply**, split by the key's own
`discrete_inputs` / `scale_bits` categories, and the difference is the
whole of why zoom feels smooth:

- **Discrete change — commit immediately.** A page step, an annotation
  toggle, a layer toggle. None has a gesture in flight and none has an
  intermediate value on the way to it, so any delay is pure latency; for a
  page change there is not even a stale texture worth showing, because it
  is a picture of a different page.
- **Zoom change — debounce by [`ZOOM_SETTLE`]**, drawing the existing
  texture scaled to the new size in the meantime. A Ctrl+wheel gesture
  emits dozens of zoom values on the way to the one the operator wants;
  rasterizing each would burn CPU producing images nobody sees. The interim
  scaled texture is soft, not blank or blocky — which is exactly what every
  other document viewer does, so it reads as normal rather than as a
  glitch. A **discrete** command (Ctrl+0, Ctrl+Plus) bypasses the debounce
  through `ViewFrame::zoom_commanded`: there is no gesture in flight, so
  waiting would just feel unresponsive.

## The strip, and the priority that keeps it affordable

Under a continuous mode several pages are visible at once, and
[`crate::render::strip`]'s header sets out the whole scheduling rule. This
module is where it is enforced, in one order that is a **priority** rather
than a sequence:

1. **the current page, always first.** It is the largest thing on screen
   and the one the operator is reading. If it is stale, the worker is
   pointed at it — cancelling a strip page mid-render if necessary, which
   is exactly what `RenderWorker::spawn` does when handed a different key.
2. **rehome, before anything is requested.** Scrolling changes which page
   is "current" without changing any *picture*, so the outgoing page's
   texture is moved into the strip cache and the incoming page's is moved
   out of it. Without this, scrolling a continuous strip would re-render
   every page at the moment it became current — the pages would flash
   undrawn as they passed the middle of the viewport, which is the exact
   opposite of what a continuous mode is for.
3. **then one strip page**, the nearest visible page that has no current
   raster. One, because the worker is single-slot: asking for two would
   cancel the first and deliver neither.

Steps 2 and 3 do nothing at all when the strip is empty, which is every
frame of every single-page session.

**A consequence that surprises every driven check written against step 3:**
the strip prefetch only has work to do after a *discontinuity*. Because step
2 rehomes the outgoing current page's texture into the strip cache, a smooth
continuous scroll makes every page it passes current in turn, and so fills
the cache as it travels. The step-3 scan then never finds a visible page
that is both not-current and unrastered, and emits no request. A check that
wants to observe strip rastering must create the discontinuity deliberately
— set the page number, or Ctrl+End. Scrolling to a page is precisely the
gesture that guarantees the page is already cached.
