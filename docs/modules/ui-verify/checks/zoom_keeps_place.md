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

### `const DRIFT_FRACTION`

# Per notch, not per stage, and the difference is not pedantry

The first version read the position once per stage of eight notches and
judged it against a one-notch tolerance. It failed by 3.17 pt against
2.57 pt — a real number and a meaningless one, because eight anchored zoom
steps each carry their own `f32` rounding and the accumulated slop was being
measured against the budget for one.

The tempting fix is to multiply the tolerance by the notch count. That is
loosening a threshold to fit an observation, which is exactly how a check
stops being able to see the defect it was written for. Reading after
**every** notch keeps the tolerance tight and localises a failure to the
notch that caused it.

A fraction rather than an absolute, because the tolerance must shrink with
the zoom. Two percent of what is on screen is generous against a defect that
discards the position outright — O24e moved the view by the whole pan, and
O24f by the whole zoom ratio.

### `const RESOLUTION_FLOOR`

# A floor on the TOLERANCE, which is not the same as loosening it

[`DRIFT_FRACTION`] is a fraction of what is on screen, so it shrinks with
the zoom — which is right, and which at the top of the climb takes it below
what any instrument here can resolve. A tolerance finer than the measurement
is not a strict check; it is a coin toss that reports whichever way the last
bit fell.

This is deliberately far below anything an operator could see: a ten
thousandth of a point is about a fortieth of the width of a banana cell's
label stroke on the fixture. Every defect this check exists for moves the
view by hundreds of points or by the whole pan. Nothing real hides under it.

It is a **floor on the tolerance**, applied only where the proportional
tolerance would be smaller — not a widening of it at the zooms where the
proportional one is meaningful. Those are different changes and only one of
them is honest.

### `struct Held`

`pub(crate)` because [`super::zoom_out_keeps_place`] measures the same
quantity on the way back down and must measure it with the **same
instrument**. Two spellings of "where is the view" would drift, and the one
that drifted would be the one whose check went green.

### `fn held`

# From the `f64` position line, because the `f32` one runs out

The first version derived this from the `canvas` line's `rect=` and `zoom=`:
`(centre − rect.min) / zoom`. Correct, and it stops working partway up the
climb. At 41,000,000 % a Letter page's rect holds a magnitude near 2.5 × 10⁸,
where an `f32`'s representable spacing is 32 — so the page point it yields
resolves to about 8 × 10⁻⁵ pt, while the drift tolerance at that zoom is
3 × 10⁻⁵. **The measurement became coarser than the thing being measured**,
and the check failed with "moved 0.0000 pt, where 0.0000 is the tolerance"
against a build that was holding the point perfectly.

That is the harness's floor, not the application's, and the tempting fix —
widening the tolerance — would have hidden a real defect at every zoom below
it. The `canvas-pos` line already carries the same quantity in `f64`
(`canvas::trace::position`, added for O24b for exactly this reason), so the
fix is to read the instrument that can still see.

`at=` is how far the view has been panned from the acting page's corner, in
screen pixels. The page point under a window point `p` is therefore
`(at + (p − viewport.min)) / zoom`, and the second term is small at every
depth — so no large intermediate is formed here either.

Falls back to the `f32` derivation when no position line has been emitted,
which is the case for a build older than that trace field. The fallback is
SILENT by design at shallow zooms, where the two agree to many decimals, and
is why [`RESOLUTION_FLOOR`] exists as a second guard.

### `fn settled`

`zooming_does_not_throw_away_where_the_operator_panned` failed sporadically
— three runs of the same binary failed at 2826 %, at 5150 %, and not until
6,898,097 % — with a drift of about 2.6 × the tolerance on one notch out of
a hundred and thirty, while every other notch sat at 18–33 % of it. A defect
that discards the position does not behave like that. A probe reading a
half-applied gesture does.

**egui smooths a `Ctrl`+wheel notch across about a dozen frames.** The wheel
event reaches `InputState` as a scroll delta, is smoothed there, and is
converted to `zoom_delta()` a slice at a time; `canvas::present` arms a
fresh anchor and pushes a fresh `ZoomBy` on **every one of those frames**.
Read out of the trace of one failing run, a single notch from 2826 % to
3452 % passed through 30.12, 30.53, 31.71, 32.53, 33.16, 33.58, 33.88,
34.08, 34.22, 34.32, 34.39 and only then 34.52, at which point the frames
began repeating byte for byte.

The check waited `settle(6)` after each notch — six frames of a twelve-frame
animation. So **every** reading it has ever taken was of a partially applied
zoom, and the quantity it compared was the anchor identity evaluated across
two arbitrary mid-flight frames, each of which had also just armed a new
anchor from a scroll offset the `ScrollArea` had not yet been handed. That
holds to within a fifth of the tolerance nearly always, and occasionally
does not.

The fix is to read the settled frame, **not to widen the tolerance**.
Those are opposite changes: one removes a variable the check never modelled,
the other blinds it to the defect it exists for. Waiting for the view to
stop makes the check strictly stronger — it now judges the position the
operator is actually left looking at, which is the only one they can see.

# How "settled" is decided

Exact equality of the zoom **and** of the page point, between two reads
`SETTLE_STEP` frames apart. Exact, not approximate: once the animation has
finished, the canvas re-emits the identical line every frame — the numbers
come from the same `f32` state through the same formatter — so equality is
reachable and is the strongest available statement. A tolerance here would
declare a slow tail settled while it was still moving, which is the failure
this function exists to remove.

Returns `Ok(None)` when the canvas publishes nothing at all, which is the
same "no reading" that [`held`] reports and is the caller's to interpret.
Returns `Err` when the view never stops moving inside the budget: that is
not a skip and it is not a pass, it is a claim about the application that
the caller must surface.
