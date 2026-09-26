# `renderstrip` — several pages at once, and what an undrawn one says

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

## What is rasterized, and when

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

## What an undrawn page shows — and why it is not a white rectangle

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

## The budget: several pages multiply the pixel cost, and this is the cap

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

## Why the current page keeps its own slot outside this cache

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

## Item notes

### `fn a_refusal_is_cached_so_it_is_not_retried_every_frame`

The behaviour this protects is not subtle: without it, a strip
containing one undecodable page would spawn a render for it sixty times
a second, and every one of those cancels whatever else was rendering —
so a single bad page would stop the *rest* of the strip from ever
filling in.

### `fn a_stale_key_or_a_stale_epoch_is_a_miss`

The epoch half is what lets a module this work may not edit —
`panels::forms::edit`, which clears the *current page's* texture and
knows nothing about a strip — invalidate every other page for free. If
the epoch were not compared, an edit would leave the pages either side
of the current one showing the document as it was before it.

### `fn the_current_page_is_pruned_out_even_when_it_is_visible`

The one rule the split with `OpenDoc::page_texture` costs, enforced
rather than remembered. A duplicate would be a second texture for the
one page that is always resident — the worst page in the document to
hold twice, since it is the largest one on screen.

### `fn pages_that_left_the_viewport_are_kept_until_the_budget_bites`

It read `pages_that_left_the_viewport_are_dropped`, drove
`retain(&[4, 5], 5)` over six cached pages, and asserted
`cache.len() == 1` with the note *"only page 4 is both visible and not
current"*. It passed for the whole life of the cache, and what it was
pinning was **the operator's complaint**: *"increase cache to maximum for
page view so they don't constantly redraw with larger files."*

A cache whose contents are the visible set is not a cache. Every page he
scrolled past was rendered again from the content stream the moment it
came back — 691 ms on a dense A1 (`BENCHMARK.md`) — and this test said
that was correct.

It is **reversed in place rather than deleted**, which is this
project's rule for a test that turned out to encode a wrong contract: a
reader who remembers the old behaviour must be able to find out what
replaced it, and a deleted test tells them nothing. The name changed
with the claim, because a name is a claim too.

### `fn a_budget_of_nothing_prunes_to_the_floor`

The assertion the old design could not make, because the visible-set
prune ran first and left the budget nothing to do. A refusal occupies
zero texels by definition, so this drives a real raster size through the
one lever a headless test has — `PageRaster::Failed` cannot carry a
count — by setting the budget to zero and checking that the cache prunes
itself down to the single entry the loop's own guard protects.

### `fn eviction_prefers_the_page_furthest_from_the_one_being_read`

Driven with `Failed` entries carrying a synthetic texel count is not
possible — a refusal is zero by definition — so this asserts the
*ordering* rule through the one lever a headless test has: the entry
list after a prune whose budget cannot bite. The texel arithmetic
itself is a `sum` and a `saturating_sub` with no branch worth a
fixture, and the eviction order is the part that would be wrong in a
way nobody notices.

### `const MAX_CACHED_TEXELS`

**It was 48 million and a backstop; it is now 256 million and the actual
limit**, and the change of role matters more than the change of number.

The old doc comment said, correctly for the code as it then stood: *"a
backstop rather than a working limit: the wanted set is already bounded by
what fits in the viewport."* [`StripRasters::retain`] pruned to the visible
set on every frame, so the budget could not bite: two or three fit-width
pages are ~8 M texels against a 48 M ceiling, and the eviction loop had
never run on any document this operator had opened. **Raising the number
alone would have changed nothing.**

With `retain` no longer discarding what scrolled off screen, this is what
bounds the cache — so it is now sized to *hold a working set* rather than to
catch a runaway.

256 million texels is about **1 GB** of RGBA: roughly 25 fit-width A1 sheets
on a 4K display, or well over a hundred pages of a report. Counted in texels
rather than pages because a page is not a unit of memory — a thumbnail and
an Annex C sheet differ by four orders of magnitude, and a page count that
admitted six of the latter would admit 1.5 GB without saying so.

It is a **default**, not a constant, as of 2026-08-19: the operator asked
for the maximum and the honest answer to *"how much of this machine's memory
may pdfcer spend on page pictures"* is that only they know. See
`crate::app::prefs::PageCache`, whose four steps each state their cost in
megabytes, because "Large" is not a number anybody can budget against.

### `enum PageRaster`

A failure is cached alongside a success **on purpose**: a page whose
content streams will not decode fails deterministically — same bytes, same
code — so retrying it on every frame would peg a core producing the same
error while the operator sits still. This is the same posture
`crate::app::state::PdfcerApp::settle_and_rasterize` takes towards the
current page's `render_error`, and holding the reason is what lets the page
say *why* rather than sitting undrawn forever with no explanation.

### `struct StripRasters`

Bounded by the operator's texel budget, whose default is
[`MAX_CACHED_TEXELS`], and ordered by distance from the page being read:
what leaves is always the furthest entry, never simply the invisible one.
Empty, and therefore free, under [`crate::viewer::PageDisplay::Single`] —
which is the mechanical form of "continuous is an option, not a
replacement": a single-page session allocates nothing here and runs the
same code path it ran before Phase 4.

### `fn get`

**Contract: `page` is never the current page.** The current page's
raster lives in `OpenDoc::page_texture` — see the module header for
why — and asking here for it would always miss, which would be a
silent second render of the one page that is definitely already
rendered.

A miss on any of page, key or epoch is a miss, and the caller's answer
to a miss is to draw the page's state rather than to draw nothing.

### `fn has`

The predicate the render scheduler asks, and it is deliberately true
for a **failure** as well as a success: a page that will not draw must
not be requested again on the next frame, or the strip spends every
frame re-failing it. See [`PageRaster`].

### `fn insert`

Replaces any previous entry for the same page, whatever key it carried:
a page has one raster, and keeping the old one at a stale zoom would be
memory held for a picture nothing will ever ask for.

`texels` is the raster's pixel count, supplied by the caller because it
is knowable for a *failure* too (zero) and because deriving it from the
texture handle would tie this type to egui's texture metadata for a
number the caller already has.

### `fn retain`

Called once per frame with the current page and the budget the operator
chose. Two passes, and the order matters:

1. **drop the current page**, whose raster belongs in
   `OpenDoc::page_texture` and must not be duplicated here;
2. **while over `budget`, drop the entry furthest from the current
   page** — furthest in page-index terms, which on a vertical strip is
   furthest in scroll terms.


The first pass read
`self.entries.retain(|e| e.page != current && visible.contains(&e.page))`,
so **the cache held exactly what was on screen and nothing else.**

That makes the name a misnomer and the budget decorative. A cache whose
contents are the visible set is not a cache — it is a frame buffer with
extra steps. Scroll a page off the top and it is gone; scroll back and it
is rendered again from the content stream, which on a dense A1 sheet is
`BENCHMARK.md`'s 691 ms. Do that in a 36-sheet set and every sheet is
re-rendered every time it comes back into view, for ever.

The operator's words, 2026-08-19: *"increase cache to maximum for page
view so they don't constantly redraw with larger files."* He had
diagnosed it exactly. **The budget was never the limit** — 48 M texels is
~18 fit-width pages and the visible set is two or three, so the eviction
loop below had never run on any document he had ever opened. Raising the
number without this change would have done nothing at all.

# What bounds it now

The budget, which is now the operator's (`crate::app::prefs`), and the
distance rule below. Together they mean *"keep what you have rendered,
nearest to where I am, until the memory runs out"* — which is what a
page cache is for and what every other viewer does.

`visible` is no longer a parameter. It had one other job — proving a
page had been *wanted* — and nothing needed that: an entry only exists
because something rendered it, and something only renders a page the
strip asked for.

### `fn take`

The other half of the rehoming `crate::render::settle` performs when a
scroll makes a different page current: the incoming page's texture
leaves this cache and takes up the current page's dedicated slot, so
scrolling never re-renders a page whose picture is already in memory.

A stale key or epoch is left in place rather than removed. It costs
nothing to keep — the next [`Self::retain`] drops it if it is not
wanted, and [`Self::insert`] replaces it if it is — and removing it
here would silently discard a raster that is still a perfectly good
answer for the zoom the operator is about to return to.

### `fn clear`

For a mode change back to a single-page arrangement, where the strip's
extra pages are not merely unwanted but cannot be reached at all — so
holding their textures would be memory kept for a picture nothing can
draw.

### `enum PageState`

Four states rather than a boolean, because the operator's response to each
differs: *wait*, *wait*, **zoom out**, and *there is something wrong with
this page*. Collapsing the first two would be tolerable; collapsing any of
them with the last would tell somebody to wait for a picture that is never
coming, or to look for damage that is not there.

### `fn draw_page_state`

See the module header for the argument. In one sentence: a white rectangle
would be a claim that the sheet is blank, so this draws the page's real
boundary, a fill that is visibly not paper, and a sentence naming the page
and its state.

`rect` is the page's rect **on screen**. `page_number` is 1-based, because
this string is read by an operator and the UI is 1-based everywhere — the
conversion happens at the call site's edge, exactly as the status bar's
page box does it.

# Why the sentence can be omitted and the frame cannot

At a zoom that fits twenty pages in the viewport, a page rect is a
thumbnail and a sentence would not fit in it. A *truncated* sentence is
worse than none — "Page 1…" reads as a label, not as a state — so the text
is drawn only when the laid-out galley fits inside the rect with room to
breathe. The fill and the boundary are always drawn, and between them they
already say "there is a page here and it has no picture yet", which is the
load-bearing half.

### `fn undrawn_fill`

It was, and a screenshot of a driven scroll is what corrected it: in the
light theme `faint_bg_color` is a hair off white, so an undrawn page read as
**a blank sheet of paper** — which is precisely the claim about the
operator's document this function exists not to make. Every gate was green
and every test passed; the failure was only visible in a picture.

`widgets.inactive.bg_fill` is the theme's *button face*: a surface the
operator already reads as chrome rather than as content, distinct from paper
in the light theme and from the canvas surround in the dark one. Taken from
the visuals rather than written as a literal —
`tools/gates/check-theme-colors.sh` — so a restyle carries it.
