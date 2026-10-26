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

## Item notes

### `const FIXTURE`

Not one form fixture in `D:\Dev\pdfcer\fixtures\synthetic\forms\` carries a
plain text field with an `/AP` `/N` appearance stream. Measured 2026-09-02
across all eighteen: `demo-form` and `radio-choice-form` have text fields
with **no** appearance, `rich-field-form`'s one paint-ready text field is
**rich text** (which the canvas declines by design), and every other
paint-ready widget is a 12 x 12 check box or radio.

That matters because the canvas census refuses a widget with no appearance
(`NotOnCanvas::NoAppearance`) — the page draws nothing there, so there is
nothing to outline. On every engine fixture the spotlight is therefore
*unable* to light the one row the panel offers, and the check could only
ever have failed. It is the check being unable to reach the feature, not
the feature being broken: exactly the shape this whole afternoon has been
about.

So this is 1,129 hand-written bytes: one page, one text field, one real
appearance stream. `fixtures/off-page-object.pdf` is the precedent.

### `fn select_tool`

Silent when the item is not on screen: the caller's own assertions report
the consequence, and a missing ribbon item is a different finding from a
spotlight that does not light.
