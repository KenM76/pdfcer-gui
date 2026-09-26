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
