# `ui-verify/checks/page_cache`

`pages_stay_drawn_when_you_scroll_back` — **a page pdfcer has already drawn
is not drawn again**, measured by scrolling a real drawing set away and
back.

# The operator's report, and why it needed driving

> *"increase cache to maximum for page view so they don't constantly redraw
> with larger files."*

The report is right and the cause is not a size.
`render::strip::StripRasters::retain` prunes the cache once a frame, and a
prune that drops **every entry not in the visible set** leaves the cache
holding exactly what is on screen. Scroll a sheet off the top and it is
gone; scroll back and it is rendered again from the content stream, which
`BENCHMARK.md` measures at **691 ms** for a dense A1.

**The texel budget is not the lever it looks like.** 48 M texels is
roughly eighteen fit-width pages, so on a visible set of two or three the
eviction loop had nothing to do and *raising the number alone changed
nothing at all* — which is exactly what "increase the cache" invites a
reader to do, and it is why this check measures **re-requests** rather than
the cache's size.

O201's render-ahead changed the second half of that: the strip now fills
the band around the current page up to the budget, so the cache does sit
near its ceiling and eviction does run. What keeps this check's oracle
intact is the direction: `retain` drops the entry FURTHEST from the current
page, and the pages this check scrolls away and back are the nearest ones,
so a band page is always evicted before a page he can see. **A visible page
re-requested is still the defect, and now it also means the two mechanisms
have got their orders crossed.**

# What it asserts, and why that is the only honest oracle

`strip-raster-requested page=N` is emitted where the strip asks for a page
**the operator can see**. Render-ahead has its own line,
`strip-prefetch-requested`, and that separation is load-bearing rather
than tidy: a prefetched page may legitimately be evicted and asked for
again, so folding the two lines together would make this check's oracle
unfalsifiable. **A page number appearing twice in the visible stream is
the defect**, verbatim: it means pdfcer drew a page, forgot it, and drew
it again.

Nothing else would do. The cache's *size* is not the claim — a build that
held a gigabyte and still re-requested would pass a size assertion and fail
the operator. A screenshot is worse than useless here, because a re-rendered
page and a remembered one are **the same picture**. That is why a cache
that forgets goes unnoticed, and why the only symptom is a person waiting.

# It needs a continuous mode and a document with pages to spare

Single-page mode keeps no strip at all (`fill_strip` clears the cache when
`strip_visible` is empty), so the check runs in **Read**, whose default is
continuous — `viewer::display::default_for_mode`. A document of at least a
few pages is required for a scroll to take one off screen; the check says so
rather than passing vacuously on a one-page fixture. **An instrument that
can only return one answer is not an instrument.**
