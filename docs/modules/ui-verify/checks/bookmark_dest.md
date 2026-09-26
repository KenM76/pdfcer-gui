# `ui-verify/checks/bookmark_dest`

`a_bookmark_lands_on_the_detail_it_names` — **the destination, not just the
page.**

# The report


> *"in Acrobat clicking on the nested bookmarks in the drawing package takes
> you to a zoomed in area of the page for the drawing bookmark that was
> clicked on. when we click on ours it just jumps us to the correct page,
> but doesn't send us to the spot on the page the bookmark actually points
> to."*

## Why his own drawing is the fixture

`TR-0461-1500-copy.pdf` is the case exactly, and its outline says so:

```text
bookmark level=0 title="TR-0461-1500"   dest=Page { page_index: 0, view: Fit }
  bookmark level=1 title="Drawing View64" dest=… FitR { left: 493, bottom: 119, right: 1104, top: 558 }
  bookmark level=1 title="Drawing View65" dest=… FitR { left:  76, bottom: 119, right:  687, top: 558 }
```

**Two nested bookmarks, one page, different rectangles.** Under the old
behaviour — page only — clicking either arrived in the same place, which is
indistinguishable from both being broken. That is why a check that asserted
"the page changed" would have passed against the defect, and why this one
asserts the ZOOM changed instead.

## The oracle, and why it is the zoom

`Fit` on an A1 sheet is about 0.39×; a `FitR` around one detail is several
times that. The `canvas` line carries `zoom=`, so *"did clicking a detail
bookmark actually take me to the detail"* reduces to *"did the zoom rise"* —
which no page-only navigation can produce.

Asserted as a RATIO against the zoom before the click rather than against
an absolute number. The absolute depends on the window size, and a check
that pinned it would fail on a different monitor while the feature worked.

## Item notes

### `const AT_LEAST`

1.5×, deliberately loose. The point is to separate "framed a detail" from
"did not move at all", and the exact ratio depends on the window; a tight
bound would be a test of the monitor.
