# `render::strip` — several pages at once, and what an undrawn one says

Phase 4's continuous modes put more than one page on screen. Everything
about rasterization up to this point assumed exactly one — one texture, one
[`RenderKey`], one single-slot worker — and this module is the whole of what
changes, kept in one place so the single-page path is provably untouched.

Two things live here:

1. [`StripRasters`] — the cache of page textures **other than the current
   page's**, with the pixel budget that stops "several pages at once" from
   meaning "several times the memory, unbounded".
2. [`draw_page_state`] — what a page looks like when there is no texture
   for it yet, which is the honesty question this feature turns on.

## ★ What is rasterized, and when

**Only pages the operator can see, one at a time, nearest first.**

`BENCHMARK.md` measures a single page of the benchmark CAD sheet at over a
second. A continuous strip over a 400-page document must therefore never
behave like "rasterize the document"; it has to behave like "rasterize what
is on screen", and it has to keep behaving like that while the operator
scrolls. The rule the canvas applies, in full:

| step | rule | why |
|---|---|---|
| 1 | the wanted set is [`crate::viewer::strip::Strip::visible`] — pages whose rect **intersects** the scroll viewport | a page one pixel of which is showing is a page the operator can see |
| 2 | nothing outside that set is requested until everything inside it is drawn | precedence, as the operator asked for it, is structural: read-ahead is reached only down the branch where the visible scan found nothing to do, so it cannot delay a page he can see |
| 3 | at most **one** render is started per frame, and it is the visible page nearest the viewport centre that has no current texture | the worker is single-slot by design; asking for a second cancels the first, so asking for two would be asking for none |
| 4 | the request is stable while it runs — the same page stays nearest until it arrives | without this the per-frame staleness check would cancel and restart forever, which is [`RenderKey`]'s own documented livelock |
| 5 | the cache keeps what it has rendered, nearest the current page first, up to the operator's texel budget | see the budget section; a cache whose contents are the visible set is a frame buffer with extra steps |
| 6 | with the visible set drawn and budget left, the band around the current page is filled in, nearest first | O201, and nearest-first because eviction is furthest-first — any other order asks for the page the cache is about to drop |

The result is that a continuous strip fills in **from the middle outwards**,
one page at a time, and a scroll that outruns the renderer simply shows the
pages it has not reached yet as undrawn rather than blocking on them — then
keeps going outwards, on the frames it has nothing visible left to draw,
until the memory the operator allowed is spent.

## ★ What an undrawn page shows — and why it is not a white rectangle

`PROJECT_PLAN.md` §3 forbids placeholders. A white rectangle where a page
will be is exactly that: it is indistinguishable from a blank page, so the
operator cannot tell "pdfcer has not drawn this yet" from "this sheet is
empty" — and on a drawing set, "sheet 12 is blank" is a claim about their
document that pdfcer would be making falsely.

So [`draw_page_state`] draws three true things and no fourth:

* **the page's boundary**, at its real size and position. That is known —
  it comes from the page tree, not from the raster — so stating it is
  honest, and it is what makes the strip's geometry legible while it fills;
* **a fill that is visibly not paper** — the theme's *button face*, see
  [`undrawn_fill`], so no arrangement of it can be mistaken for content;
* **a sentence naming the page and its state** — being drawn now, waiting,
  or refused with the renderer's own reason.

The sentence is centred in the part of the page that is **on screen**, not
in the page, and is omitted only when even that is too small to hold it — in
which case the fill and the boundary still say "there is a page here and it
is not drawn". Both of those refinements came from screenshotting a driven
scroll rather than from a test; see [`draw_page_state`] and
[`undrawn_fill`], each of which records what the picture showed.

## ★ The budget: several pages multiply the pixel cost, and this is the cap

`crate::viewer::max_zoom_for_page` caps **one** pixmap at
`pdfcer_render::MAX_PIXMAP_EDGE`, accounting for `pixels_per_point`. That
guard is per page and is untouched — it still applies to every page this
module renders, because every page still goes through the same worker with
the same `raster_scale`.

What it does not do is cap the *sum*, and with several pages resident the
sum is what matters: four A1 sheets at 2× on a HiDPI display is roughly
4 × 15.5 M texels ≈ 250 MB of RGBA. So this cache carries a ceiling — the
operator's `crate::app::prefs::PageCache`, whose default is
[`MAX_CACHED_TEXELS`] — expressed in texels rather than pages because pages
are not a unit of memory — a thumbnail-sized page and an Annex C sheet
differ by four orders of magnitude. Since O201 that ceiling is reached in
ordinary use rather than only in theory: render-ahead spends what is left
of it on the pages either side of the one being read.

Eviction is **furthest from the current page first**, and the current
page's own texture is never in this cache to begin with (see
[`StripRasters`]'s header on the split), so the page the operator is
looking at can never be evicted to make room for one they are scrolling
past.

## ★ Why the current page keeps its own slot outside this cache

`crate::app::state::OpenDoc::page_texture` stays exactly what it was: the
current page's raster, `Option<PageTexture>`, invalidated by assigning
`None`. Three surfaces already depend on that field and on that spelling —
the status bar's render-notes disclosure reads its `diagnostics`,
`crate::app::actions`' `vector_edit` clears it after an edit, and
`crate::panels::forms::edit` clears it after a form change — and none of
them is about a strip.

Folding the current page into this cache would have meant rewriting three
call sites in three modules to ask a cache a question they currently answer
with a field, for no gain: the current page is exactly the one page that is
*always* wanted and *never* evicted, so it is the one page a cache buys
nothing for.

The cost of the split is one rule, and it is enforced rather than
remembered: **the current page is never in this cache**. [`StripRasters`]
is asked only for pages other than the current one (see
[`StripRasters::get`]'s contract), and
[`StripRasters::retain`] takes the current page so it can be excluded on
the way in as well as protected on the way out.

## Staleness needs no call site at all

Every entry carries the [`RenderKey`] it was rendered from **and** the
`edit_epoch` it was rendered at, and a lookup that does not match both
misses. That is deliberately stronger than what the current page's slot
does, and it is what lets a module this work may not edit —
`crate::panels::forms::edit`, which clears `page_texture` and knows nothing
about a strip — invalidate the whole strip for free: the edit bumps the
epoch, and every cached page misses on the next frame.
