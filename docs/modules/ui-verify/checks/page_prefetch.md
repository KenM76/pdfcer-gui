# `ui-verify/checks/page_prefetch`

`pages_are_drawn_before_he_scrolls_to_them` — **the page he scrolls onto is
already drawn**, measured on a real document.

# The operator's report, and why a unit test cannot answer it

> *"on multipage documents I noticed with scanned pdf I have to wait for
> pages to load as a I scroll to them. As many pages as we can should be
> rendered and ready to be shown as I scroll. The ones on screen should
> always take precedence to be rendered first."*

`render::prefetch`'s own tests pin the band's ORDER, which is arithmetic
over three integers. What they cannot see is whether anything ever calls
it. Render-ahead is reached down one arm of one `if` in a function that
runs only on a frame, against a live worker, a real cache and a wall clock,
and every way it can fail silently is on that side of the boundary: a
`settling` guard that never releases, a headroom test false on the first
frame, a candidate scan that finds everything resident because the visible
set is wider than the band. In each of those the unit tests stay green and
the operator keeps waiting.

# The oracle is his sentence, not the mechanism's

`strip-prefetch-requested` only says the band was asked for. What he asked
for is that a page is **already drawn when he arrives at it**, and the two
are different claims: a build that prefetched page 5 and evicted it before
he got there emits the first and fails the second.

So this check scrolls, and asserts that **no page prefetched before the
scroll is asked for again as a visible page after it**. A page in both
streams was rendered, forgotten, and rendered again — the waiting he
reported, with the extra cost of having rendered it twice.

That is also why the two trace names must stay distinct, and `page_cache`
records it from the other side: that check asserts no page appears twice in
the VISIBLE stream, and a prefetched page may legitimately be evicted and
re-offered, so folding the names together would make both oracles
unfalsifiable at once.

# The three subsidiary assertions

* **The band is non-empty.** If nothing was ever prefetched the main
  assertion is vacuously true, so that outcome is reported as SKIPPED. An
  instrument that can only return one answer is not an instrument.
* **`texels <= budget` on every disclosure line.** `prefetch_headroom` asks
  for room for one more page rather than for the cache to be under budget,
  precisely so render-ahead cannot fill past the ceiling, be trimmed back,
  and fill again for ever. A line over budget is that loop in progress.
* **Nearest-first, while the current page holds still.** The band's order
  must be the exact reverse of `StripRasters::retain`'s eviction order. Get
  it wrong and nothing fails — the two mechanisms grind against each other,
  rendering and evicting the same sheet, with nothing on screen to say so.
  It is the one property here whose defect is invisible from outside, which
  is why it is asserted from the running program and not only from the pure
  function.

# It needs a continuous mode and a document with pages to spare

Single-page mode keeps no strip at all, so the check runs in **Read**,
whose default page display is continuous. A band cannot exist on a one-page
fixture; the check says so rather than passing vacuously.
