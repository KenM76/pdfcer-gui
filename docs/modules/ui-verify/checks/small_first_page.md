# `a_small_first_page_is_the_current_page`

**Defect it catches:** in continuous mode, a small page shown whole above a
larger one is not the current page, because the larger page covers more of
the view. Page commands (rotate, extract, delete) then act on the page the
operator was not looking at.

## What it drives

Nothing is clicked. The app is launched off-screen (`-4200,-4200,1400,900`,
scripted pointer, `--no-input` safe) on `fixtures/small-page-first.pdf`: a
200 × 280 pt page 1 above a Letter page 2 (`small-page-first.PROVENANCE.py`).
The frame is shot for the record.

## Oracle

The last `canvas` line's `page=` (0-based) is `0`.

SKIP when the document did not open in `continuous`, or fewer than two pages
are visible, since then no larger page is competing.

Falsified by restoring the raw-area rule in `Strip::page_at_view`: the canvas
reported page index 1, FAIL.

## Not covered

Scrolling is not driven. The unit test
`a_small_page_seen_whole_is_the_current_page` covers the scrolled case, where
the larger page fills the view and becomes current.
