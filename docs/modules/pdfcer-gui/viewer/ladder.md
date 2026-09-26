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

## Item notes

### `fn stepping_up_then_down_returns_to_where_it_started_above_the_ladder`

The operator: *"clicking the negative button to zoom back snaps me back
to 800% when I am over 800%."* `ladder_step_up` grew a doubling branch
when the maximum zoom was raised; `ladder_step_down` did not, so from
4,155 % one press discarded a hundred-fold magnification.

Asserted as a ROUND TRIP rather than against fixed numbers. The
property this module's header promises is reversibility, and a test of
two constants would keep passing if both were changed together in a way
that broke it.

### `fn descending_past_the_ladders_end_lands_on_its_top_rung`

Halving from 8.5 gives 4.25, which is between two named rungs — so the
next press down would go to 4.00 and the 600 % rung would never be
reachable from above. Clamping the halving at the top rung hands the
descent to the named percentages cleanly.

### `fn ladder_stepping_climbs_past_its_end_and_still_saturates_downward`

So the property changes shape rather than disappearing: **the step
keeps climbing, and what stops it is the CEILING** — `ViewState::zoom_in`
clamps against `zoom_ceiling`, which is where the limit belongs. A
stepper that enforced its own maximum would be a second opinion about
how far the operator may zoom.

The downward half is unchanged: `MIN_ZOOM` is a floor with nothing
below it, and 10 % of a page is not a number anybody has asked to go
under.

### `fn ladder_step_down`

# Why the search is not simply reversed


> *"clicking the negative button to zoom back snaps me back to 800% when I
> am over 800%."*

Exactly what a plain reverse search does. The ladder ends at 8.0, so from
4,155 % the highest rung *below* is 8.00 — one press and a hundred-fold
magnification is gone. [`ladder_step_up`] had already grown the doubling
branch for the same reason on the way up; **only one half of the pair was
given it**, which made the two buttons stop being inverses of each other
exactly where the new range begins.

That asymmetry is the defect, more than the snap itself. This module's
own header promises *"zoom-in/zoom-out exactly reversible"*, and a pair of
controls that disagree about what a step is breaks the one property an
operator relies on to explore without losing their place.

Halving mirrors the doubling above, so eleven presses out of a million
percent is eleven presses back in, and the last halving hands over to the
named rungs at the top of the ladder rather than jumping past them.
