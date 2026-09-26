# `dialogs::print::position` — where the page sits on the paper

Operator request O208: *"can we add a control to our print preview screen
so that when we are printing at a scale that will lose content we have the
option to drag the drawing to a new position on the print page? That way we
can choose what gets cropped."*

## What this module owns

One quantity: a **displacement from the placement pdfcer chose**, per
document page. Everything else here is either arithmetic over that quantity
or the controls that set it. The copy is next door in
[`crate::text::print`]; the preview that draws the result is
[`super::preview`].

## Contract: a delta, keyed on the DOCUMENT page, sparse

- **A delta, not a position.** Zero means *where pdfcer put it*, which is
  what makes Reset a meaningful command distinct from Centre — see
  [`Positions::centre`] for why those two are not synonyms.
- **Keyed on `PagePlan::index`**, never on a position in the plan list. The
  job may be reversed, subset-filtered, or print a page more than once, so
  the two coincide only for a whole-document forward job.
- **Sparse.** An empty map means every placement is byte-identical to the
  one [`super::spooler::plan`] returned, which is what stops this feature
  from being able to change a job nobody has touched (`R6`).

## Where it is applied, and why not in the spooler

[`Positions::displace`] is called from `PrintDialog::show` on the value
`plan` returns, **before any reader**. Not inside [`super::spooler`],
because that module's header states that nothing in it computes a
placement, a sequence or a scale; the operator's displacement is a shell
decision and it belongs on the shell side of that line.

Applying it there rather than at each reader is what makes the preview, the
per-edge readout, the clip count, the commit and the trace agree by
construction — including [`super::verdicts`], whose cache key is the
[`Placement`] itself, so a displaced page correctly invalidates the ink
verdict measured at its old position.

## Item notes

### `const SETTLED_PT`

Drag arithmetic in `f32` screen points divided by a scale does not return to
exactly zero, and a page holding a delta of 1e-14 pt would read as *moved*
forever: Reset would stay live, the moved-page count would say "1 page", and
the operator would have no way to make either go away. Canonicalising in
[`Positions::set`] is what keeps "unmoved" reachable.

A hundredth of a point is four thousandths of a millimetre — two orders of
magnitude below this dialog's own unit, so nothing an operator can see is
rounded away.

### `const EPS_PT`

`pdfcer_print::place_page` compares against `const EPS: f64 = 0.5` when it
sets [`Placement::clipped`]. [`clips`] recomputes that flag after a
displacement, so it must use the engine's number: a tighter one here would
report a clip on a page the engine considers fitting, and the two verdicts
would differ by a hair on exactly the sheets that sit on the boundary.

### `const NUDGE_MM`

A millimetre rather than a point because the readouts beside the preview are
in whole millimetres. A point is about a third of one, so three presses of
an arrow key would leave every number on screen unchanged — which reads as a
control that is not listening, not as a fine adjustment.

### `fn line`

# Reported at the dialog's resolution, deliberately

An overhang that rounds to zero millimetres on all four edges is
reported as fitting. That is not a rounding error being hidden: whole
millimetres are the unit every length in this dialog is stated in, and
both alternatives are worse — a sentence reading *"extends past the
printable area — right 0 mm"* names a quantity the operator cannot act
on, and a decimal here would be the only decimal on the surface.

### `fn set`

The single writer. Every other mutator routes through it so that the
"a delta under [`SETTLED_PT`] is not a delta" rule cannot be bypassed by
a new command forgetting it.

### `fn publish`

`ui_rect_visible` and not `ui_rect`: the group sits in a scrolling options
column, so on a short dialog its lower controls are genuinely off screen,
and a driver handed a rectangle for an unreachable button would click
whatever is drawn over it and then report the wrong thing about the result.

Published even while a button is greyed. Whether **Reset** is enabled is
part of what a driven check asserts — an unmoved page must refuse it — and
a region that vanished when the control greyed would make "refused" and
"absent" the same reading.

### `fn the_engine_still_starts_an_oversized_page_flush_at_the_corner`

Every sentence in [`Positions::centre`] about why Reset and Centre
differ rests on one measured fact: `place_page` clamps an oversized
page's offset at zero, so pdfcer's own placement is the top-left corner.
If the engine ever centres an oversized page instead, Reset and Centre
become synonyms, three of this group's buttons become indistinguishable,
and the doc comments above become wrong — and nothing else in this
program would notice.

So it is asserted against the engine directly, through the real
`place_page`, rather than restated here as a belief about it.

### `struct Offset`

Positive is right and down: the sense of the device context the offsets are
eventually handed to, and the sense of the preview on screen. It is the
opposite of a PDF page's own Y axis, which is why [`t::position_frame`]
states it rather than leaving it to be inferred.

### `enum Grab`

**Latched at `drag_started_by`, never re-derived mid-gesture.** The page
rectangle moves under the pointer while the page is being dragged, so a
per-frame hit test would classify the same gesture differently from one
frame to the next: drag the page far enough and the pointer leaves it, the
classification flips to `Paper`, and the rest of the stroke pans the view
instead. Deciding once is the only stable reading.

### `fn centre`

# Why the two commands differ, which is the whole feature

`pdfcer_print::place_page` centres a page that fits and then clamps:
`offset_x_pt: ((aw - w) / 2.0).max(0.0)`, *"clamped at zero so an
oversized page starts at the edge of the printable area rather than at a
negative offset"*. So for the sheets O208 is about — the ones losing
content — pdfcer's placement is flush to the top-left corner, and the
whole loss falls off the right and bottom.

Reset returns to that corner. Centre moves to the middle, which crops
the drawing evenly on all four edges. Both are wanted, and an operator
choosing what to lose off a big drawing wants the second far more often.

# The one primitive, stated once

> new delta = old delta + (target − current)

`current` is the placement as the preview is drawing it — already
displaced — so the undisplaced engine offset is never needed and
therefore can never be double-counted. The drag, the arrow keys and all
three centring commands are this one line, which is why there is no
second arithmetic path to keep level with the first.

A page whose engine placement is already centred lands on a delta of
zero, which [`Self::set`] canonicalises to *unmoved* — so Centre on a
page that fits correctly leaves Reset greyed rather than claiming a move
that did nothing.

### `fn displace`

Called once per frame on the value [`super::spooler::plan`] returned,
before any reader. Takes the job by value and hands it back so there is
no window in which a caller could hold the undisplaced one.

# `clipped` is recomputed, but only for a page that moved

The flag is a geometric verdict and a displacement changes the geometry,
so leaving it alone would leave the hatch, the caption and the commit
button's count describing the position the page used to be at.

The guard is not an optimisation. `place_page` returns
`clipped: true, scale: 1.0, offsets: 0` for degenerate input — a
zero-size page or sheet — which no purely geometric formula can
reproduce, so recomputing unconditionally would *clear* a flag the
engine set deliberately. An unmoved page keeps the engine's answer; only
a page the operator displaced gets ours.

### `fn clips`

The verdict `pdfcer_print::place_page` computes, restated over an arbitrary
offset rather than only over the centred-and-clamped one it produces. See
[`EPS_PT`] for why the tolerance is the engine's and not one chosen here,
and [`Positions::displace`] for the one case where the engine's own answer
must be preferred to this.

### `fn cropped`

The per-edge half of O208's second clause. The hatch answers *where* on the
picture; this answers *how much* as a number, which is the quantity the
operator is steering. Each edge is clamped at zero, so a page inside the
area on an edge reports nothing for it rather than a negative slack the
sentence would have to explain.

### `fn arrow_nudge`

The step lives here rather than at the call site because it is a fact about
the unit the readouts are in, not about the preview — see [`NUDGE_MM`].
Shift multiplies it, which is the convention every drawing program on this
machine uses for the same gesture.

Keys are **consumed**, so an arrow that moved the page cannot also reach a
sibling control on the same frame.

### `fn group`

Draws nothing at all when there is no job, or when the stepper is on a sheet
the job does not contain (`R9`: an unavailable capability renders nothing,
and a position control with no page to act on is unavailable rather than
temporarily disabled).

# Why a tab of its own, and not the preview strip

The strip under the preview already lays seven controls into a
`horizontal_wrapped` row that measures wider than the column holding it.
Four more would wrap it into a second row, and the strip's height is fixed
for the feedback-loop reason `preview::STRIP_HEIGHT_PTS` documents — so they
would be clipped, not merely cramped.

The remaining candidate was the foot of the Pages & Layout options column,
beside the scale radios, which is where scale and position belong together
conceptually. That was measured and does not fit; [`super::tabs::PrintTab`]
carries the numbers. What makes the split cheap is that the preview is a
separate column and is visible whichever tab is open, so the feedback these
controls need is never hidden behind the tab that owns them.
