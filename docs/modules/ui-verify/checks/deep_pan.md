# `ui-verify/checks/deep_pan`

`panning_at_deep_zoom_stays_where_it_was_put` — the operator's "it jumps
back" report, made falsifiable.

# The report


> *"is that the challenge I was running into trying to pan over a little bit
> at high zoom, but it would jump back to it's original location I panned
> from because I couldn't pan to the next point?"*

Two claims in one sentence and they need separating, because they have
different causes and only one of them is a bug:

| claim | would be |
|---|---|
| *"I couldn't pan to the next point"* | a **quantised** pan — the view refuses small movements and only moves in steps |
| *"it would jump back to its original location"* | a **reverting** pan — the view moves and is then put back |

A quantised pan is what an `f32` scroll offset does when its
representable spacing exceeds the drag: `last - pan` rounds straight back to
`last`, so the view does not move at all. It looks like the drag was
ignored. A reverting pan is something actively re-setting the offset after
the drag — a different fault with a different fix.

This check tells them apart by measuring the offset at three moments: before
the drag, immediately after, and several frames later.

# Why it rolls the wheel rather than dragging

The first version drag-panned with the primary button and reported the view
as stuck. That was a **harness** defect: `canvas::input::pan_delta` pans on
the middle button always and on the primary button only under the hand
tool, so a primary drag with the default Select tool correctly rubber-band
selected and correctly moved nothing. The check had measured a gesture the
application never offered, and blamed the application for not honouring it.

It is recorded here rather than quietly fixed because it is the same shape
as the false layout report in `checks::driving::declared`'s header: a
measurement of the wrong thing is indistinguishable, from the verdict line,
from a real defect. **Ask what the check sampled before asking what is
broken.**

The wheel is unconditional — no tool, no modifier, no button — so a view
that does not move under it is unambiguously the application's fault.

## Item notes

### `const CELLS_PT`

`banana.pdf` draws a banana at life size and, beside it, two banana cells at
**the same scale** — 0.85 pt and 0.17 pt across, with labels 0.085 pt tall.
At a zoom where the whole page fits a screen the pair covers roughly one
pixel. They are the reason the fixture exists and the only thing on it worth
magnifying.

The operator, 2026-08-22: *"You should try zooming into the two cells, then
try slightly panning to test. Right now you are just zooming into a blank
area on the canvas."* He was right — the check zoomed about the viewport
centre, which on this page is white paper, so every raster it exercised was
empty. The geometry assertions were still valid (a placement is a placement
whatever the pixels show), but nothing about the run resembled what he does
with it, and a screenshot oracle added later would have had nothing to look
at.

Located by rendering the region at 30x and measuring: the pair sits at
(540, 560), about 600 micrometres past the tip of the arrow that points at
them.

### `const POS_EVENT`

Not `canvas`'s `off=` and not its `rect=`. Both are `f32`, and at the
zoom this check works at their representable spacing is larger than the
drag, so both would read "unchanged" against an application that panned
perfectly. See `canvas::trace::position`.

### `const PRESSES`

# Where this has to land, and how badly it was got wrong

It must land in the band where the **region tier is engaged and the position
is still on the `scroll` tier** — because that band is the only place O24c
can exist, and it is narrower than it looks:

| zoom | raster | position |
|---|---|---|
| below ~2,070 % | whole page | `scroll` |
| ~2,070 % … ~1,000,000 % | **region** | **`scroll`** ← the band |
| above that | region | `deep` |

The lower edge is `MAX_PIXMAP_EDGE / page_height` = 16,383 / 792 on a Letter
sheet. Sixteen notches landed at **1,867 %** — just under it — so every run
traced `region=none`, the placement cross-check had nothing to compare, and
the check reported PASS twice against a binary with the defect deliberately
put back in.

It was defended with an argument, too: the operator's *"up to 800 %
things work perfect"* was read as evidence that 800 % is a mechanism
boundary. It is not one — 800 % is the **old maximum zoom**, and the plain
reading of his sentence is *"the range that existed before is fine; the new
range is not."* A sentence was promoted to a measurement, and it agreed with
a theory that the actual trace contradicts.

Hence [`REGION_TIER_REQUIRED`]: this check now refuses to pass a run in
which no region raster was ever placed.

### `const REGION_TIER_REQUIRED`

The single most important line in this file. Without it the check is
satisfied by a run that never reached the tier it is named after — which is
not a hypothetical, it is what happened. A check that cannot fail is not
evidence, and this one was being quoted as evidence.

### `const MORE_PRESSES`

# Why the check probes twice

The position is owned by two different mechanisms at two different depths —
an `f32` scroll offset below the deep threshold and an `f64`
`viewer::deep::DeepAnchor` above it — and they are a hard branch, not a
re-parameterisation. A pan that works on one says nothing about the other.

Probing only the deepest would have been the tempting choice and it would
have missed the operator's actual case, which was on the shallow tier. One
probe per mechanism is the minimum that can honestly claim panning works
"at high zoom".

Sized to SATURATE, on the operator's request of 2026-08-22 — *"can you
test up to maximum zoom please?"* A Ctrl+wheel notch multiplies the zoom by
about 1.22, so reaching the default ceiling of 10¹² % from the first probe's
4,155 % takes roughly a hundred notches. Overshooting costs a few seconds
and is what makes the second probe a statement about the **ceiling** rather
than about some arbitrary depth on the way to it.

### `const NOTCHES`

# Why the wheel and not a drag

The first version of this check drag-panned with the primary button and
reported a stuck view. It was wrong: `canvas::input::pan_delta` pans on the
**middle** button always, and on the primary button only while the hand
tool is active. The default tool is Select, so a primary drag correctly
rubber-band selected and correctly did not move the view. The harness had
measured a gesture the application never claimed to honour, and blamed the
application.

The wheel is unconditional — no tool, no modifier, no button — so a view
that does not move under it is unambiguously the application's. It is also
what an operator reaches for first.

Three notches: small enough to be the "little bit" he described, large
enough that a working build moves visibly.

### `const ROLLS`

# Why one small roll is not enough, and how that was found out

`render::strategy::region_for` quantises the wanted region to a **half
viewport** grid, and O24c only exists while a *new* cell's raster is in
flight. A single three-notch roll moves about 120 points, stays inside the
cell it started in, requests nothing, and therefore cannot reproduce the
defect at all.

That was not reasoned out — it was measured. With one roll this check passed
**twice out of two** against a binary with the defect deliberately put back
in. A check that green-lights the bug it is named after is worse than no
check, because it is quoted as evidence.

Enough rolls to cross at least one grid line, with a placement reading taken
between each, so whichever roll crosses is sampled inside its transient.

### `const PLACEMENT_TOLERANCE_PT`

Not zero. The application computes the placement in `f32` from a page rect
that is itself `f32`, and the trace prints three decimals; the harness
recomputes in `f64` from those printed values. A fraction of a point of
disagreement is the arithmetic, not the defect. The defect it is looking for
is `render::strategy::region_for`'s grid step — half a viewport, several
hundred points — so there is four orders of magnitude between the noise and
the signal and no need to tune this finely.

### `fn position`

# Why the two must be read together

The first version of this check read the position here and the page rect at
the end of the probe. They came from different frames, so the placement
cross-check compared a paint rectangle recorded before the wheel against a
page rectangle recorded after it — and reported a 120-point placement error
that was exactly the wheel movement, against a correct build.

That is the third time in two days this harness has produced a confident,
specific, entirely wrong verdict by sampling two quantities at two moments.
The pairing is now structural: one function, one return value, and no way
for a caller to hold one without the other.

### `fn probe`

Returns `Ok(None)` when the view moved and stayed moved, and
`Ok(Some(verdict))` when it did not — the two failure shapes described in
this module's header.

Takes the report so its notes carry BOTH probes' numbers. A verdict that
says "the view did not move" without saying which tier it was on sends the
next reader to whichever of the two files they guess first.

### `fn check_placement`

Checked at every reading, because the window in which it fails is exactly
the window in which a new region's raster is in flight: check only the
settled one and the defect is invisible, which is how it shipped.

`scroll` tier only. Above the deep threshold the placement comes from the
`f64` anchor rather than from the page's rect, and `placement_error`'s
formula does not describe it — comparing anyway would report a defect that
is only a wrong model.
