# `viewer::ladder` — the zoom levels the `+` and `−` buttons step through

One subject: **given a zoom, what is the next one up or down?** Split out
of [`super`] under R2 when that file reached 1,540 lines, and the seam is a
real one — everything here answers that question and nothing here knows
what a page, a viewport or a raster is.

## Why a ladder at all

A fixed list of round percentages makes zoom-in and zoom-out **exactly
reversible** and makes every step land somewhere a person can name. A zoom
that arrived from somewhere else — Ctrl+wheel, a fit mode, a saved
document — is off the ladder, and the two functions here take the next rung
strictly above or below it, so the ladder doubles as a *snap back to
sanity*.

## Past the ladder's end, the two must stay inverses

The named rungs stop at 800 %, which was the maximum zoom until O24 raised
it. Above that [`ladder_step_up`] doubles and [`ladder_step_down`] halves —
a constant ratio, so a constant number of presses per decade, and the same
number of presses back.

Both branches were needed and only one was written. `ladder_step_up`
grew its doubling when the ceiling was raised; `ladder_step_down` kept a
plain reverse search and therefore returned **800 %** from anywhere above
it. One press discarded a hundred-fold magnification, which is
`OPERATOR_REQUESTS.md` O24g. A pair of controls that disagree about what a
step is breaks the one property an operator relies on to explore without
losing their place, so the reversibility is pinned as a round-trip test
rather than against fixed numbers.
