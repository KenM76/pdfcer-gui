# `ui-verify/checks/off_page_press`

`a_band_that_starts_in_the_margin_reaches_an_object_off_the_page` — **the
half of O23 that had nothing to enable.**

# The report


> *"also objects should still be reachable even if they are off the page."*


> *"how do I view and edit objects that are off of the page? we added this
> feature but I didn't see how to enable it."*


# How this differs from `off_page_marquee`, and why that is not a
duplicate

`off_page_marquee` is the sibling check for O92, and it drags a band **from
blank paper** into the margin. That gesture already worked, and it worked
for a reason that looks like this one and is not: an `egui::Response` keeps
reporting `interact_pointer_pos` after the pointer has left the widget, so a
drag that BEGINS on the sheet stays with the sheet for its whole life no
matter how far out it goes.


⇒ The discriminating assertion here is **`canvas-surface surface=pasteboard
… pastegesture=true`, followed by a band**. Either alone is weak: the trace
line without a band would say the decision ran and the gesture still died,
and a band without the line would not say WHICH surface produced it.

## ⚠ …and `surface=pasteboard` alone is not the assertion, for a measured
reason

`canvas::trace::surface` is emitted from the pointer's **position**, every
frame, so `surface=pasteboard` appears whenever the pointer is off the sheet
— *including on a build whose scroll content senses nothing but hover*. The
first draft of this check asserted on exactly that, and when it was run
against a deliberately falsified build (the `Sense` put back to `hover()`,
2026-09-10) it went red with a message beginning **"the `Sense` is right"**.
It was not right; it was the one thing that had been broken.

`pastegesture=true` is the field that cannot exist without a drag sense, so
it is the field this check turns on. The general lesson, recorded because it
has bitten this suite before: **a falsification is a measurement of the
check as well as of the feature.** A check that goes red for the right
reason while naming the wrong cause is a check that will send the next
reader to the wrong file.

# The fixture, and why `hits == 1` is an oracle

`fixtures/off-page-object.pdf` — the same 485 bytes `off_page_marquee` uses,
deliberately, because two fixtures for one property is two chances for one of
them to stop having it. A 200 × 200 page with exactly two filled squares:

| | where | on the page? |
|---|---|---|
| **A** | x 40–100, y 40–100 | yes |
| **B** | x −160 – −40, y 100–140 | **no — entirely left of the media box** |

The band runs from `(−20, 190)` — **grey, above and left of everything** —
leftward and downward to `(−100, 120)`, covering x −100…−20, y 120…190.

* It **misses A twice over**: A's right edge is x = 40 and the band's right
  is x = −20; A's top is y = 100 and the band's bottom is y = 120. Either
  miss alone would do; both is deliberate, so a change to one axis of the
  fixture cannot quietly make the count ambiguous.
* It **touches B** — x −100…−40 and y 120…140 are inside both.
* It **cannot enclose B**, which reaches x = −160 where the band stops at
  x = −100. So this is still a *crossing* window and the check still
  discriminates between the two marquee modes rather than passing under
  either.

⇒ **One hit can only be B.** The unit tests at the foot of this file assert
every clause of that paragraph against the fixture's own coordinates, so the
argument fails at build time if the geometry is ever edited out from under
it.

# Why the origin is at y = 190 rather than beside the square

`canvas::presspick`'s rule is that a press on *ink* starts a move and a press
on empty paper starts a band. The pick tolerance is several page points wide
and is measured in screen pixels, so it grows in page terms as the zoom
falls. `(−20, 190)` is 36 pt from B's nearest corner at `(−40, 140)` — far
enough at any zoom this check can be driven at, and the alternative failure
is silent: the press would select B directly, no band would run, and the
check would report a marquee defect that does not exist. `off_page_marquee`
learned this at 92 pt; this one has less room and states the margin.

# What this check does NOT claim

**It does not claim the operator can SEE the object.** What this measures is
that it can be *reached*: banded, selected, and therefore outlined (the
selection overlay is clipped to the canvas viewport, not to the page) and
dragged home — every one of which works whether or not a single pixel of
square B was ever painted.

Making it visible is the render half of part B and it has its own check,
`off_page_visible`, which counts ink in a screenshot at square B's centre
against a paper control 60 pt below it. Conflating the two would produce one
check that cannot say which half broke — and they did break separately: the
reach half shipped on 2026-09-10 with the raster still sized to the crop
box, which is exactly the state the operator reported as *"I didn't see how
to enable it"*.

# Every way this reports SKIP

No binary, `--no-input`, no diagnostic channel, no page on screen, the Select
tool unreachable from the ribbon, or **not enough grey margin on screen to
reach x = −100** — a property of the window size on the day, reported as a
skip that names the geometry and never as a pass.
