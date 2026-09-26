# `ui-verify/checks/measure_perimeter`

`measure_perimeter_traces_and_closes` — the perimeter tool arms, takes
vertices, closes on the first one, and the dimension reaches the engine.

# Why this check exists at all, given `measure_linear` already passes

Because the perimeter tool is the first gesture on this tab with **no fixed
arity**, and every link that differs from the linear tool is a link no
existing check crosses:

| # | link | new here? |
|---|---|---|
| 1 | the ribbon item arms `Measure(Perimeter)` | new command, new variant |
| 2 | a click becomes a **vertex** rather than one of three fixed picks | new |
| 3 | the running total accumulates across an unbounded run | new |
| 4 | a click on the **first vertex** closes the ring | new — no other tool has this ending |
| 5 | closing **commits**, and the engine accepts it | the same `add_dimension` the others reach |

Links 2–4 are the ones that cannot be unit-tested: `PerimeterPick` can be
driven from a test without a window and is, but *whether a click on the page
reaches it* and *whether the ring test converts screen pixels to the right
vertex* are properties of call sites and of a coordinate conversion, and
both are only observable in a running process.

# The assertion that matters most is the CLOSING one

`closes_the_ring` compares the click against the first vertex **in canvas
space**, which means it crosses the page→canvas bridge — the conversion
`canvas::mapping`'s header calls *the classic silent defect*, because the
canvas is Y-down from the page's top-left with `/Rotate` applied and every
point pdfcer publishes is Y-up from the un-rotated CropBox.

Get it wrong and the ring never closes: the operator clicks the first corner
of a footprint they have just traced and gets a fifth vertex on top of it.
No unit test can see that, because the arithmetic is individually correct on
both sides and it is the *caller* that mixes two spaces of the same type.

# What this check does NOT assert, and where that is recorded

**That the number is scaled.** The value goes through `Group` by
construction — it is a `DimensionKind`, so `format_measurement` handles it
with the same code path every other dimension uses, and the engine pins that
with its own tests. Asserting it here would need the fixture to carry a
calibrated group, which is a second gesture (Set scale) inside a check about
a first one. The tool-panel running total is formatted through the same
function and is the surface where a scale defect would show.
