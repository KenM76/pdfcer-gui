# `ui-verify/checks/zoom_keeps_place`

`zooming_does_not_throw_away_where_the_operator_panned` — two reports, one
question: *does a zoom keep the view where I put it?*

# The reports


> *"if I am zoomed out to about page size, pan the cells to the center of
> the screen, then start to zoom, the page snaps back to near the center
> position."*

> *"I do lose the view at 2000000% magnification."*

The same failure at two scales, and the same shape at both: a zoom
**discards the position** instead of magnifying about it.

| where | cause |
|---|---|
| at fit-page zoom | `geometry::zoom_anchor_offset` clamped to `display - viewport`, which is **zero or negative** when the page is no larger than the viewport — so the offset was forced to 0, the centred position |
| at ~2,100,000 % | the `f64` tier's anchor was seeded from the **previous frame's** scroll offset, and then never moved on a zoom at all |

# Why this is not the same check as O24c's

`panning_at_deep_zoom_stays_where_it_was_put` asks whether a **pan** moves
the view and whether the pixels land in the right place. This asks whether a
**zoom** preserves it. They failed independently and were fixed
independently; one check covering both would have gone red for one reason
while the other was still broken, and the second would have been found later
and blamed on the first fix.

The measurement is the page point under the **viewport centre**, before and
after each zoom. Zoom-to-cursor holds the point under the *pointer*, and
this check puts the pointer at the centre, so a correct build keeps that
page point fixed however deep it goes.

Measured in page units rather than screen pixels, deliberately. Screen
pixels are what the defect happens in, but page units are what "the same
place on the drawing" means — and the tolerance has to shrink with the zoom,
or a check at a million percent would quietly accept a metre of drift.

## Item notes

### `const PAN_AT`

The pan is what makes this check able to fail. The centred position is
exactly where the O24e defect snapped **to**, so a check that zoomed without
panning first would have watched the view "stay" where the bug was about to
put it anyway — green, and measuring nothing.

### `const MAX_STAGES`

# A CAP, not a count — the loop climbs until the zoom SATURATES

The operator, 2026-08-22: *"can you test up to maximum zoom please?"* So the
run does not stop at a chosen depth; it keeps rolling until a whole stage
fails to increase the zoom, which is the application saying it has reached
its ceiling. With the default maximum of 10¹² % that is a long climb — eight
notches multiply the zoom by roughly five, so from a page-fit 76 % it takes
about fifteen stages.

The cap exists only so that a build broken in the *other* direction — one
that climbs by an epsilon for ever — ends the run instead of wedging the
suite. Reaching it is reported as a SKIP, not a pass: a run that never found
the ceiling has not tested the ceiling.

The saturation test asks the APPLICATION where its ceiling is rather than
comparing against a constant. The maximum is an operator setting, so a check
that hard-coded 10¹² % would silently stop testing the ceiling the day he
changed it — the same silently-inert control this whole request began with.

### `const SETTLE_ROUNDS`

`SETTLE_STEP × SETTLE_ROUNDS` frames is the worst case per notch — about
two seconds on an idle machine, and reached only when the view genuinely
never stops moving, which is a finding in itself and is reported as an
error rather than swallowed.
