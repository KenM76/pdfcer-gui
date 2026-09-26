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

## Item notes

### `const CURRENT_UNFILLABLE_SLOT`

Its own slot for the same reason [`BEYOND_RASTER_SLOT`] has one: the
de-duplication is per slot, so two lines sharing one suppress each other and
each appears only when the two happen to alternate.

The line is emitted on the way OUT of the regime as well as into it, and
that is not decoration. The subject is an absence — no order, no refusal, no
learned ceiling — and a run that cannot tell *"never entered it"* from
*"still in it"* cannot assert an absence at all. Same argument as
`strip-beyond-raster pages=0`.

### `const BEYOND_RASTER_SLOT`

Its own slot, not shared with any other line in this module, because
`trace_changed` de-duplicates **per slot**: sharing one would make each line
suppress the other and the count would appear only when it happened to
alternate. See `canvas::trace`'s slot table for the same rule stated once for
the canvas.

### `fn zoom_settle`

**The operator's, as of 2026-08-17.** [`ZOOM_SETTLE`] was the whole
answer and is now only the *default* — `manifest::DIRECTED` carried this
as *"partial G — `ZOOM_SETTLE` is a compiled-in constant today"*, and
that was accurate: the control was missing, not the value.

Read from the document's preferences **snapshot** rather than from the
application, for the same reason its settings snapshot exists: this is a
per-frame read inside a `&mut doc` borrow, and reaching back to
`PdfcerApp` would be a second borrow of the whole struct.

The snapshot cannot be meaningfully stale here — `adopt_settings` writes
it and drops every raster in the same statement, so a settle read after
a change is a settle for a cache that no longer exists.

### `fn rehome_current_page`

Called when the scroll position has made a different page current. See
the module header, step 2: without this, every page of a continuous
strip would re-render at the moment it passed the middle of the
viewport — visibly flashing undrawn on the way through, which is the
opposite of what a continuous mode exists for.

`wanted` is the key the *current* page needs this frame. The incoming
page is taken out of the cache only if its raster matches that key,
because a raster at a stale zoom is not a raster the current page can
use — leaving it in the cache costs nothing and the ordinary staleness
path re-renders it.

The outgoing texture is filed at `self.edit_epoch`, and that is exact
rather than approximate: the current page's slot is cleared outright by
every edit (`crate::app::actions`' `vector_edit` and
`crate::panels::forms::edit` both assign `page_texture = None`), so a
texture that is still here has not survived an edit.

### `fn fill_strip`

Step 3 of the priority. Does nothing at all when `strip_visible` is
empty or holds only the current page, which is every frame of every
single-page session — so this whole feature costs a `Vec::is_empty`
check on the default path.

# Why exactly one render per frame, and why "nearest" is the order

`RenderWorker` is single-slot by design: a second `spawn` cancels the
first. So "start every missing page" would start the last one and
abandon the rest, and a strip would fill in from the *bottom* of the
viewport at one page per frame with every earlier page's work thrown
away. One request per frame, always the nearest missing page, fills the
strip outwards from where the operator is looking and never discards
completed work.

# Why it waits for the current page

The current page is the largest thing on screen and the one being read.
A strip page requested while it is still stale would cancel its render.
The gate is `!doc.render_worker.is_rendering()` plus a settled current
page: while a zoom is in flight, the strip stops asking entirely, so a
wheel gesture over a continuous document costs the same one debounced
render it costs over a single page.

# And why it must ASK FOR A FRAME while it is waiting

**Found by driving the binary, not by a test.** Every gate was green and
the strip did not fill: the trace showed `visible=2 drawn=1` and then
nothing at all, for as long as the window was left alone.

The cause is that egui is **event-driven**. Opening a document resolves
the fit mode, which moves the zoom, which arms the 150 ms settle
deadline; the current page renders inside the in-frame budget and
requests one more frame to draw itself; on that frame the strip is still
inside the settle window, so it asks for nothing — and nothing else
wakes the process. The deadline passes with no frame to notice it, and
page 2 stays undrawn until the operator moves the mouse.

So a wait has to schedule its own wake-up, exactly as the zoom debounce
already does for the current page (`ctx.request_repaint_after`). The
symptom of getting this wrong is not a crash and not a wrong pixel; it
is a feature that works perfectly whenever anyone is watching it and
stalls the moment they stop, which is the single hardest kind of defect
to see from a test suite.

### `mod tests`

Only [`OpenDoc::raster_order_fillable`] is covered here, and deliberately
only it. Everything else in this file is a frame's worth of sequencing
against a live `egui::Context`, a worker thread and a wall clock; the one
part that is a pure question about a document is the predicate O186's third
route turns on, and that is the part a unit test can actually pin.

⚠ **What these tests do NOT establish is that the guard is in the right
place.** They prove the predicate answers correctly; they cannot see the
`if !current_held && fillable` that consults it, and a build with that
`&& fillable` deleted passes every one of them — measured, not assumed. The
driven coverage that is owed, and the check that will eventually supply it,
is named on [`OpenDoc::raster_order_fillable`] itself. This paragraph exists
so that a reader who finds four green tests here does not conclude the route
is covered.

## Why the fixture is this repository's and not the engine's

`open_local_fixture("four-pages.pdf")`, **not**
`open_fixture(FOUR_PAGES)` — and the difference is not cosmetic. There are
two documents on this machine called `four-pages.pdf`:

| path | pages |
|---|---|
| engine `synthetic/pageops/four-pages.pdf` | four sheets, **all US Letter** |
| this repo's `fixtures/four-pages.pdf` | `2383.937 × 1683.78`, `612 × 792`, `612 × 792`, `306 × 396` |


Opened rather than hand-built, for the reason `app::status::rasterstop`'s
tests give: [`Self::strip_page_orderable`] reaches `page_extent_pts`, which
reads the real `/MediaBox` and `/Rotate`, so a synthesised page would check
the arithmetic against a number this test invented rather than against a
document.

### `const BIG_SHEET_FITS`

Both constants sit a wide factor either side of that on purpose. A test
that straddled 6.87 closely would be measuring the engine's rounding,
which is the engine's business and not this predicate's — and it would
go red the day `MAX_PIXMAP_EDGE` changes, reporting a defect here that
is not here.

### `fn region_on`

The rectangle's own size is irrelevant and deliberately small: what the
predicate asks is whether a region *exists for this page*, because the
request a region produces is viewport-sized rather than page-sized. A
test that made the rectangle large would imply the size mattered.

### `fn with_no_region_an_order_is_fillable_only_while_the_whole_sheet_fits`

Asserted in one test because either alone is satisfied by a constant: a
predicate hard-wired to `true` passes the first assertion and one
hard-wired to `false` passes the second, so a suite holding only one of
them would be green against a function that had stopped reading its
arguments.

The third assertion is the one that proves the answer is about the
**page**. Page 3 is a sixth of page 0, so `SMALL_SHEET_SCALE` is fine for
it and far past page 0's limit; an implementation written against a
single document-wide extent — the most likely wrong version of this —
fails here and nowhere else.

### `fn a_region_belonging_to_another_page_does_not_make_this_one_fillable`

This is the assertion that pins [`Self::region_for`] rather than the
`raster_region` field inside the predicate. Both rectangles are valid, so
reading the field directly would answer `true` for every page in the
document the moment any one page had a region — and nothing would report
it, because the consequence is simply that the wrong sheet gets ordered
whole and refused, which is the defect this guard exists to stop.

The second half is what makes the first half a statement about *whose*
region it is rather than about regions being ignored altogether.

### `fn a_page_past_the_end_is_never_fillable`

Reachable in practice on the frame after a page is deleted, before the
view index has been brought back into range. The answer is `false` rather
than `true` because there is no sheet to order at all — a `true` here
would place a request the worker could only discard, spending a thread on
a page that does not exist.
