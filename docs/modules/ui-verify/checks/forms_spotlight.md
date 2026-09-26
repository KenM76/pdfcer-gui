# `ui-verify/checks/forms_spotlight`

`clicking_a_form_row_lights_the_field_on_the_page` — focusing a row in the
Forms panel outlines that field's box on the canvas.

# The operator's ask, `OPERATOR_REQUESTS.md` O98

> *"when we have the fill form panel visible and I click on fields in it
> instead it should highlight the field on the canvas that is being
> filled."*

On a drawing with a dozen fields you fill one in the panel and cannot see
where it went. Note the direction — **panel → canvas**. The other direction
shipped as O53.

# Why this needs driving, and why it needed two new instruments first

The feature is a **handshake across two surfaces inside one frame**: the
panel writes a field name into `egui`'s temp store when a row has focus, and
the canvas reads it while painting. Both halves are a few lines and both are
individually trivial. What can break is the *join*:

* the panel writes a name the canvas cannot match — the identity bug the
  channel was built on fully-qualified names rather than indices to avoid;
* the panel writes and the canvas never reads, because the canvas painted
  before the panel drew;
* the row never takes focus at all, so nothing is ever written.

**No unit test can reach any of the three**, because all three require a
real frame with both surfaces in it and a real pointer press that moves
keyboard focus.

Two instruments had to be built before this check could exist, and that
is worth naming because it keeps happening:

1. **The spotlight published no trace.** Drawing an outline is invisible to
   the harness. It now traces `canvas-form-spotlight field= drawn=
   candidates=` — and it traces `field=none` too, so "pointing at nothing"
   and "pointing at something the canvas cannot place" are distinguishable
   rather than both being silence.
2. **The fill rows published no region.** There was nothing to aim a pointer
   at. They now publish `forms.fill.row.<index>`.

A feature that cannot be observed cannot be verified, and the fix is to give
it an oracle rather than to weaken the assertion.

# The assertion that distinguishes working from plausible

`drawn >= 1` **with the field named**. A build where the panel writes and
the canvas silently fails to match would trace `field=Subscribe drawn=0`,
which is a specific, diagnosable state — and it looks exactly like a working
build to anyone assert­ing only that the trace line appeared.

`candidates=` is carried alongside so the two reasons for `drawn=0` can be
told apart: zero candidates means the canvas has no box for that name at all
(an identity mismatch), and candidates with `drawn=0` means the field exists
but sits on a page that is not on screen (not a defect).
