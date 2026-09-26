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
