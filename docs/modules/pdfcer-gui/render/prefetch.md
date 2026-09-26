# `render::prefetch` — filling in the pages he has not scrolled to yet

`OPERATOR_REQUESTS.md` O201:

> *"on multipage documents I noticed with scanned pdf I have to wait for
> pages to load as a I scroll to them. As many pages as we can should be
> rendered and ready to be shown as I scroll. The ones on screen should
> always take precedence to be rendered first."*

## Contract

[`OpenDoc::prefetch_candidate`] answers *"which page would render-ahead
like next, if anything"*, and is consulted by `settle::fill_strip` **only
when every visible page already has a picture**. [`disclose`] emits the
off-canvas report the feature owes. Nothing here spawns anything: the
caller owns the worker, so the single-slot rule and the zoom-settle
suppression stay in one place.

## Precedence is structural

His second sentence is not implemented as a priority number. The band is
reached only down the `None` arm of the visible scan, so there is no
ordering in which a page he can see loses to a page he cannot. A priority
comparison would have to be correct at every call site; an unreachable
branch is correct by construction.

## What bounds it

Three things, and they are independent on purpose:

* **[`PREFETCH_BAND`]** bounds how far a wrong guess about his direction
  can run, in pages.
* **[`OpenDoc::prefetch_headroom`]** bounds the memory, in texels, against
  the operator's own page-cache preference.
* **`OpenDoc::strip_page_orderable`** is applied unchanged, so a sheet
  above the renderer's pixmap ceiling is no more orderable ahead of time
  than it is on arrival.
