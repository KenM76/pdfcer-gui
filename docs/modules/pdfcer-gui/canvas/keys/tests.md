# `canvas::keys` tests — the Delete ladder enumerated, and the two keys
# that only pass through

## ★★ The seam, and why the half left behind is the interesting one

[`super`] is **two precedence ladders** — which claimant a Delete reaches,
and which a press of Escape does — and each is a short function whose whole
content is an ordered list of `if let` arms. The tests are bulky because a
ladder is exercised by enumerating it, not by three cases: every rung needs
a case that reaches it AND a case that proves the rung above did not swallow
it.

⇒ So this is a split between **the ladder** and **the enumeration of the
ladder**, a subject boundary rather than a size-driven cut — the same one
`gesture::meaning` took three commits earlier. The Escape ladder's
enumeration is [`escape_ladder`], one level down, for the same reason
applied once more: two ladders is two subjects, and the fixtures above are
all either one needs.

## The two keys here that are not a ladder at all

Delete and Escape choose between claimants. An arrow key and Tab do not —
they are routed straight through to the module that decides them, and the
only thing this file asserts about either is **that the route exists**:

* `an_arrow_key_reaches_the_nudge_through_canvas_keys` — the nudge's own
  enumeration lives in [`crate::canvas::moving::nudge`]'s tests, beside the
  module that decides it;
* `a_claimed_tab_reaches_the_object_ring_through_canvas_keys` — the ring's
  lives beside `canvas::objring`, where a narrowed [`PickFilter`] is the
  point rather than an incidental.

Both are single route assertions, and both are why the three paragraphs
that used to stand here — *no case presses Tab*, *no assertion presses an
arrow*, *every case passes `page: None`* — are gone rather than edited. Each
was true of the Delete and Escape ladders and was written as though it were
true of the file.

## ★★★ Every **ladder** case passes `targets: None` **and**
## `model_attempted: true`, and the pairing is deliberate

Those two fields answer different questions and a unit test is the one place
it is easy to set them inconsistently:

* `targets: None` — *there is no decomposition*, which is what lets these
  run without opening a file;
* `model_attempted: true` — *the frame asked for one and did not get it*,
  which is the **page-would-not-decompose** case.

The same cases pass [`PickFilter::all()`](PickFilter::all) and
`page: None`, and neither is a stub: `all()` is what a shell that has never
touched the filter hands over, and a Delete never crosses into PDF space, so
a frame with no page on screen genuinely has none to hand. The two route
assertions above are the exceptions, and each names its own reason.
