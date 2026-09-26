# `ui-verify/checks/deep_zoom`

`zooming_past_the_pixmap_ceiling_still_renders` — the operator's own
failure, reproduced and then asserted away.

# The report


> *"I got a requested raster size 14580x18868 is empty or exceeds
> MAX_PIXMAP_EDGE when I got to 2382% zoom."*

A US Letter page at 2382 % is 18,868 device pixels tall against a 16,384
cap. The whole-page raster cannot be made, and until the region tier was
wired into the canvas nothing else was tried — so the page simply stopped
rendering and said so.

# Why this needs driving rather than a unit test

Every part of the mechanism already had unit tests before he hit this:
`strategy::for_page` knew the ceiling, `render::region` converted the
rectangles, the worker could call `render_page_region`, and the cache keyed
on it. **All of them passed, and the feature did not exist**, because
nothing called the strategy — `strategy::for_page` appeared in the shell
only inside comments.

That is the defect this project keeps finding and the reason R1 exists: a
mechanism that is complete and unreachable looks identical, from a test
suite, to one that works. The only thing that can tell them apart is a
zoom driven past the ceiling on a running application.

# What it asserts

* no render reports `outcome=failed`, which is what the raster-size refusal
  produces, **and**
* a raster actually arrives afterwards.

Both, because either alone is satisfiable by a build that renders nothing
at all: a canvas that never asks cannot fail, and a canvas that fails
silently still drew something earlier.

## Item notes

### `const PRESSES`

Enough to SATURATE, deliberately. The ladder runs to 800 % and then doubles,
so this walks all the way to the ceiling and keeps pressing — which means
the check exercises the whole range rather than a point in the middle of it,
and would catch a build that renders at 25,000 % and fails at 2,000,000 %.

### `const MUST_EXCEED`

A Letter page's whole-page raster fails at about 26×. Reported as SKIPPED
below this rather than passed: a run that never left the whole-page tier has
not exercised the region tier at all.
