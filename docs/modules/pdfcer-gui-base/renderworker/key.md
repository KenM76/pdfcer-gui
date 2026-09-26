# `render::worker::key` — **what a render is OF**, as one comparable value


## Why this is the seam, and not "move the tests out"

The obvious way to get a file under the ceiling is to move its `#[cfg(test)]`
modules to a sibling, and it would have worked here — there are 375 lines of
them. It was rejected because it answers the gate without answering the
rule: R2 exists so that a reader can hold a file's subject in their head,
and a file whose tests live elsewhere has exactly the same subject it had
before, only harder to read.

[`super`] has two subjects and they change for different reasons:

* **the worker** — a thread, a channel, a cancellation token, one in-flight
  slot, and the mapping from `pdfcer-render`'s result to an [`super::Outcome`].
  It changes when the rasterisation contract does, which this week meant a
  new refusal variant.
* **the key** — which inputs make two pictures different. It changes when a
  new *control* is added: a layer override, a stroke-display mode, a region.

Nothing in this file mentions a thread, and nothing in [`super`]'s worker
half decides what makes a picture stale. Two subjects, two rates of change,
which is this project's test for a seam.

## What did NOT move

[`RenderKey`]'s tests. They stay in [`super`]'s test module beside the
worker's, because several of them assert the **pairing** — that the key the
worker spawns with is the key the texture is stamped with — and splitting
an assertion from one of its two subjects is how the pairing stops being
tested by either.

## Item notes

### `struct RenderKey`

# Why this is load-bearing rather than bookkeeping


`raster_scale` is compared by bit pattern rather than by `==`
because it comes from the same arithmetic each frame; an exact float
comparison is right here and a tolerance would be wrong, since any
difference at all means the shell wants a different picture.

# It is also the SHELL's staleness key, and that is the point

This type is public and is recorded on
[`crate::render::raster::PageTexture`] because the same comparison has to
be made in two places for a control to work:

1. **"Is the render already running the one I want?"** — here, in
   [`RenderWorker::spawn`], or a slow page never finishes.
2. **"Is the picture on screen still a picture of what I am looking
   at?"** — in [`crate::app::state::PdfcerApp::settle_and_rasterize`], or
   nothing ever *asks* for the second render.

Those were two independent field lists until S4, and the failure mode of
letting them drift is the one the module docs describe: a control that
ticks and changes nothing. One type, one constructor
([`Self::new`]), and a field added to it is compared on both sides
or on neither.

# The two categories of input, and why the split is here

[`Self::discrete_inputs`] and [`Self::scale_bits`] between them cover
every field, and the division is a **policy**, not a convenience:

- A **discrete** input (page, annotation visibility, layer override) is
  changed by a command or a click. There is no gesture in flight, no
  intermediate value on the way to it, and no stale picture worth
  showing, so it re-rasterizes at once.
- The **scale** is changed by a wheel gesture that emits dozens of values
  on the way to the one that was wanted, so it is debounced
  (`crate::app::state::ZOOM_SETTLE`) and the existing texture is drawn
  scaled in the meantime.

Stating it as two methods rather than as a comment means the shell reads
the categories off the key instead of re-deriving them, and a new key
added to neither accessor fails
[`tests::every_render_input_is_either_discrete_or_the_scale`].

See the module docs for the one further key this will grow
(`font_env_generation`) and the rule that it lands with the surface that
varies it.

### `fn new`

`stroke_display` is a **positional parameter and not a builder**,
unlike [`Self::with_region`], and the difference is deliberate. A
builder may be omitted, and an omission here would silently mean
`Actual` — which is the stale-raster bug this field exists to prevent,
wearing the shape of a call site that simply forgot. As a parameter,
every one of the five sites that computes a key has to answer the
question, and the compiler asks it.

### `fn with_region`

A builder rather than a second constructor, deliberately: this type's
own note warns that *"two constructors doing the same arithmetic is
how the two sides of the staleness comparison drift"*, and a builder
adds a field without repeating any of it. [`Self::new`] stays the one
place the base key is computed.

### `fn same_region`

# Why this had to become its own question


> *"if I pan to far to one side when I am beyond 800% zoom it doesn't
> always render the new exposed area, and the same thing happens
> usually when I zoom out."*

The staleness test in `render::settle` asked two things — has a
**discrete input** changed (page, annotations, layers), and has the
**scale** changed — and the region was in the key without being in
either. So a pan that changed nothing but *which part of the page is on
screen* was not stale by any measure, and **no render was ever
requested**. The picture the operator had kept being drawn correctly at
its own region and simply slid off, leaving the newly exposed area
blank for as long as they cared to look at it.

The zoom-out half is the same fault arriving by a different route. A
zoom does change the scale, so a render *is* requested — but the
request is built from whatever region was current when it spawned, and
by the time it lands the gesture has moved on. Once the scale settles,
nothing notices the region it arrived with is the wrong one. Both
symptoms are one missing comparison.

Compares the **stored bits**, not the reconstructed rectangles: `f64`
is not `Eq`, and a comparison with a tolerance would make "the same
view" a matter of degree in the one place that must answer yes or no.

### `fn region`

# Why a texture must be placed by ITS OWN region


> *"As I drag using the middle mouse button the pan will follow and
> work, but if I pan a little too far it jumps back in the opposite
> direction I was moving … if I pan the other direction and cross the
> same area where I experienced the jump the pan location jumps back
> to being correct."*

The current page's texture is served from its slot **without a
staleness check** — deliberately, so a zoom or a pan shows the last
good picture instead of blank paper while the next one renders. That
is the behaviour the operator asked for by name: *"I don't want the
affect that other readers have where you always have to wait for
detail to render after panning to a new area."*

But the destination rectangle was computed from the region the shell
**now wants**, and `render::strategy::region_for` quantises that to a
half-viewport grid. So the instant a pan crossed a grid line the
destination jumped a whole grid step while the pixels were still the
previous cell's — the picture lurched backwards, held there until the
new raster landed, and snapped right again when the operator panned
back over the same line. Every detail of the report follows from that,
including *"it isn't exactly in the same place as it started"* (the
step is the grid, not the drag) and the page occasionally leaving the
screen entirely (two grid steps at once, at a zoom where the grid is
most of the window).

The fix is **not** to reject the stale texture. That would blank the
page on every grid crossing — the exact behaviour he ruled out. It is
to draw the stale pixels *where they belong*, so they slide off
naturally as the pan continues and the new raster replaces them in
place. This accessor is what makes that possible: the key already
carried the region, and nothing ever read it back.

The round-trip through [`f64::to_bits`] is exact, so the rectangle
returned is bit-identical to the one the request was built from — a
placement derived from it cannot disagree with the render by a
rounding step.

### `fn page`

Added at Phase 4, and it is what makes a strip's routing correct: a
finished render is labelled with the key it was run from, so
`crate::render::settle` can file it against the page it is *of* rather
than against whatever slot asked for it. Those differ exactly when the
operator scrolled while it was running, which under a continuous mode
is the common case rather than the rare one.

It is deliberately a separate accessor from [`Self::discrete_inputs`]
even though that tuple's first element is the same number: that method
is a *staleness category* and its shape belongs to the debounce policy,
while this is an identity. A caller that reached for `.0` would be
reading a policy decision as a fact.

### `fn discrete_inputs`

See the type docs: none of these has a gesture behind it, so waiting
out the zoom debounce would make a click feel unresponsive for no
benefit — and for a page change there is not even a stale picture
worth showing, because it is a picture of a different page.

### `fn raster_scale`

The exact inverse of [`Self::new`]'s `to_bits`, and here rather than at
the one call site (`tools.render_diagnostics`) because a bit pattern
reinterpreted by hand is the kind of arithmetic that is right once and
then copied. Staleness still compares [`Self::scale_bits`]: a bit
comparison is total where `f32` equality is not, which is the whole
reason the field is stored as bits.

**Device pixels per PDF user-space unit** — the operator's zoom already
multiplied by the display's `pixels_per_point`, per this type's own
docs — so it is not the percentage the status bar shows.
