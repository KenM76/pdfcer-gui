# `render::worker` — rasterization on a background thread

One job: keep a slow page from freezing the application. This module
owns the worker thread, the channel, the cancellation token and the
generation counter; [`crate::render::raster`] owns the texture upload
on the far side of that seam.

The measured numbers below are the evidence for the design and must not
be paraphrased away: **six rapid zoom steps start six generations and
complete one.**

## Why this exists, and what it is NOT

**It does not make anything faster.** A page that took 10 s still
takes 10 s. What changes is that the 10 s is spent on a thread the
operator is not waiting on, so the window keeps repainting, the
zoom keeps responding, and the render can be abandoned.

The evidence that justifies it: a real CAD sheet measures **~10 s at 1×
and ~58 s at 2×** when rasterized inline on the UI thread. At those
numbers the application does not render slowly — it stops answering.

## The three things that make it correct

**A generation counter.** A worker that finishes after its request
was superseded must have its result *discarded*, not painted. Every
spawn takes the next generation; a reply whose generation is not the
current one is dropped. Without this, releasing a zoom gesture would
paint whichever render happened to finish last rather than the one
that matches the screen.

**Cancellation that stops work.** [`RenderCancel`] is polled between
content-stream operators, so a superseded render abandons the page
rather than running to completion and having its output thrown away.
At 58 s a discarded result still occupies a core and still delays
whatever the operator asked for next. Measured: **28.9 ms** from
`cancel()` to thread exit mid-render, against **10,367 ms** to let
one finish.

**A bounded in-frame wait.** See [`RenderWorker::spawn`] — this is
what keeps a fast page indistinguishable from the synchronous
behaviour it replaces.

## What this module does not decide

Whether, and how, the canvas discloses that it is showing a stale
picture. That is a shell question and it lives in the shell, not here.
This module only reports, via [`RenderWorker::in_flight_since`], how
long the current render has been outstanding, so the shell can decide.

## The staleness keys, and the rule that governs adding one

**A key lands in the same commit as the surface that varies it, never
earlier and never later.** Both halves of that matter:

- Without the key, the cached texture does not invalidate and the
  operator-facing control **silently does nothing**. That is the failure
  mode to expect here — not a crash, a control that appears inert.
- With the key but no surface, the request carries a constant and the
  comparison carries an untriggerable branch, which is the "no state a
  surface can reach" invariant broken from the other side.

`font_env_generation` is the one input this module deliberately does not
key on. Nothing in this build lets an operator name a font folder, so the
bundled [`pdfcer_render::FontEnvironment`] is the only environment any
render can use and a generation counter over it would count to one and
stop. It lands with the font-folder surface, under the rule above.

### The other half of the invalidation, which is NOT in this module

A key on the *request* only stops the worker from de-duplicating two
genuinely different renders. It does not, on its own, make the shell ask
for the second one: the shell decides "the cached texture is stale" by
comparing the texture's own key against the one it wants. That comparison
lives in [`crate::app::state::PdfcerApp::settle_and_rasterize`], and it
reads **this same [`RenderKey`]**, recorded on
[`crate::render::raster::PageTexture`] when the pixels were uploaded.

That is deliberate and it is the structural half of the guarantee. Two
independent comparisons — one in the shell, one here — would let a third
key be added to one and not the other, compile, run, and produce exactly
the inert control described above. There is **one** key type, constructed
by **one** function ([`RenderKey::new`]), so adding a field to it changes
both sides at once.

`cmyk_intent`, `fonts` and `view_magnification` are not on the request at
all. All three have correct defaults in [`pdfcer_render::RenderOptions`]
(the operator-ruled `NeutralBlack` intent, the bundled font environment,
and `None` = the print-correct `/D`-initial optional-content state), and
no surface varies any of them, so they are left to that default and
travel on the request when a settings surface exists to move them.

**`view_magnification` deserves one extra sentence**, because it looks
adjacent to `layers_generation` and is not. §8.11.4.4's usage
applications recompute a layer's state from the zoom, and §8.11.4.5
forbids a print or aggregate path from applying them at all (core API
trap T-12.8). Leaving it `None` is therefore the *print-correct* answer
rather than a gap — and if a viewer ever opts in, it needs no new key of
its own, because it is a pure function of `raster_scale`, which is
already compared.

## Item notes

### `const IN_FRAME_BUDGET`

# Why a blocking wait is the right answer here

The requirement is that a page rasterizing in milliseconds behaves
exactly as it did when rendering was synchronous — no flash, no
spinner, no frame of stale content. Handing every render to a worker
and collecting it next frame would cost such a page one frame of
staleness for no benefit.

So the spawn waits briefly and collects the result inline when it
arrives. One frame at 60 Hz is ~16.7 ms; this is deliberately under
that, so even in the worst case the wait cannot itself drop a frame.
A page that beats the deadline never touches the asynchronous path
at all, and a page that misses it hands control back to the event
loop after a delay the operator cannot perceive.

This is the one place the UI thread blocks on rendering, it is
bounded by a constant, and the bound is the whole point.

### `fn the_same_request_twice_is_recognised_as_the_same_render`

# Why this is the load-bearing test and not bookkeeping

The shell re-runs its staleness check every frame, and while a
background render is in flight the cached texture has not been
replaced — so the check keeps saying "stale" and keeps asking
for the same render. `spawn` recognises that request as the one
already running *only* through this equality.

If it fails, every frame cancels the render the previous frame
started and begins an identical one. A page slower than a single
frame then **never finishes at all** — which is strictly worse
than the freeze this module was written to remove, and it would
look like a hang rather than a bug.

### `fn changing_any_single_render_input_makes_a_different_key`

The test above cannot distinguish a correct `RenderKey` from one
that compares nothing at all and reports every pair as equal — and
that failure is not hypothetical. A key that ignored a field would
make the guard swallow a *genuine* new request: change the zoom,
and the shell would decline to re-render because it believes the
in-flight job already covers it. The page would stop responding to
zoom entirely.

So each field is varied one at a time. Dropping any single field
from `RenderKey`'s `PartialEq` fails exactly one of these — and
each key this struct grows (see the module docs) must add its own
line here in the same commit.

### `fn a_region_is_part_of_the_key`

The region tier rasterizes the viewport rather than the page, so two
rasters of the same page at the same scale can show different parts of
it. If the region were not in the key the cache would serve the first
for every position: **the operator pans and the picture does not move**,
with nothing reporting an error, because from the cache's side every
request was a hit.

That is the worst shape of defect this project keeps finding — silent,
and indistinguishable from a frozen canvas.

### `fn every_render_input_is_either_discrete_or_the_scale`

[`RenderKey::discrete_inputs`] and [`RenderKey::scale_bits`] are how
the shell decides whether a change re-rasterizes **now** or waits out
the zoom debounce. A field that appears in neither is a change the
shell cannot see at all: the key would compare unequal, the worker
would happily run the new render — and nothing would ever ask for it,
because the texture would still look current.

That is not the same failure as an uncompared field, and it is worse:
the module's own key would be *correct* while the picture stayed
wrong, so the obvious place to look would be the innocent one.

Each field is varied one at a time and the pair is asserted to move.
A key this struct grows must add its line here in the same commit,
exactly as it must to the test above.

### `fn the_render_key_moves_when_line_weights_are_turned_off`

# The vacuous test this must not become, and it is the likeliest mistake

A test that `view.line_weights` is *plumbed* — that the request carries
it and the worker assigns it — **passes on a build where the cache
serves the old picture.** The operator presses the button, the strip
reports a hit, the texture drawn under `Actual` is drawn again, and
nothing anywhere reports an error. From his chair that is *"the button
never worked"*, which is the sentence O137 exists to answer, arriving
for a second reason.

So the property asserted is not "the field reaches the renderer". It is
**the key moves**, in both of the ways the shell compares keys:

* `RenderKey` equality, which `render::strip` uses to decide whether a
  cached raster may be served at all; and
* [`RenderKey::discrete_inputs`], which `render::settle` uses to decide
  whether to re-rasterize **at once** rather than after `ZOOM_SETTLE`.

The second matters on its own: a stroke display that landed in the
*scale* category would make the toggle take 150 ms to do anything, on a
control whose entire complaint history is that it did nothing.

And the reverse, so the test cannot pass by making every key unequal:
two keys that agree about line weights and about everything else must
still be equal.

### `fn only_the_raster_scale_is_debounced`

The other half of the split, asserted from the other side: if a
discrete input leaked into the scale category it would inherit the
150 ms zoom debounce, and a click on the annotation toggle would take
a fifth of a second to do anything for no reason an operator could
see. If the scale leaked into the discrete category, every notch of a
wheel gesture would rasterize a CAD sheet — the behaviour
`ZOOM_SETTLE` exists to remove.

### `fn a_one_bit_scale_difference_is_a_different_render`

Comparing `f32` by bit pattern rather than by a tolerance is
deliberate. The shell derives `raster_scale` from the same
arithmetic each frame, so an unchanged zoom yields bit-identical
values and the guard holds; but any difference at all means the
shell has asked for a different picture, and a tolerance would
silently serve it the wrong one.

### `fn an_idle_worker_reports_nothing_in_flight`

Guards the status-bar staleness disclosure against its most
embarrassing failure mode: announcing that the canvas is behind
when nothing is rendering.

### `fn cancelling_an_idle_worker_is_a_harmless_no_op`

The `Drop` impl exists because closing a document must not leave a
58-second render running against a session nobody can see. There is
no page to render in a unit test, so what is checked is the weaker
but still meaningful property that the teardown path is reachable
and idempotent on an idle worker — a `cancel_in_flight` that
panicked or blocked on an empty slot would hang every close.

### `fn the_region_round_trips_bit_exactly`

The placement is computed from what comes back out, and the render was
run from what went in. A rounding step between them is a rounding step
between the pixels and where they are drawn — which at a high zoom is a
visible offset (O24c).

### `fn neighbouring_regions_stay_distinguishable`

This is what lets a held texture be placed by the region it is a
picture of while the shell is already asking for the next one. If the
accessor returned the shell's wanted region — or if the builder
mutated in place and both keys ended up agreeing — the placement would
silently follow the request instead of the pixels, which is the exact
shape of the defect this pair exists to prevent.
