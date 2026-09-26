# `ui-verify/checks/fit_places_the_view`

`a_fit_command_puts_the_page_on_screen` — the three fit modes, driven, with
the view deliberately thrown away first.

# The reports


> *"If I press the Fit width or fit page button the view should center to
> the width as well or center the page."*

> *"Adobe has fit height, so add that too."*

# ★★★ Why the run must pan into the pasteboard FIRST


So the run scrolls hard into the pasteboard, **asserts that it got there**
(a run that failed to displace the view has not set up its own precondition
and is a SKIP, not a pass), and only then presses the button.

# What is asserted, per mode

Read from the `canvas` trace line's `rect=`, which is the page's true drawn
rect on screen, against `canvas-viewport`'s:

| mode | the claim |
|---|---|
| **Fit page** | every edge of the page is inside the viewport, **and the margins are equal on both axes** |
| **Fit width** | both vertical edges are flush with the viewport's, so the full width is what is on screen |
| **Fit height** | the mirror: both horizontal edges flush |

★★ Fit page's claim is *contained and centred*, not "fills both axes". A
landscape sheet in a tall window fills the width and floats in the middle
vertically, and a check that demanded both axes fill would fail on every
page whose aspect differs from the window's. **Equal margins is the direct
statement of what the operator asked for**, and it is also the one thing a
fit that set only the scale cannot produce: a page can be entirely inside
the viewport and still jammed against one edge with a viewport of
pasteboard on the other.

★ "Flush" within a tolerance, not exactly: the fit divides in `f32` and the
page rect is rounded to the pixel grid, so demanding exact equality would
be a check that fails on arithmetic rather than on behaviour. A few points
— far below the *hundreds* the defect moves the page by, and far above the
rounding.
