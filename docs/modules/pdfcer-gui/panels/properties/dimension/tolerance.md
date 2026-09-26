# `panels::properties::dimension::tolerance` — the seven forms, and the two
things a panel must not do with them

## What a tolerance is, in this model

One more property of the `Pass 69.0` style cascade — not a parallel system.
It inherits factory → group → ce dimension exactly like a stroke width, uses
the same `Option` as its override checkbox, and has the same
clear-restores-inheritance semantics. That was the point of building the
cascade first.

Seven forms: `None`, `Basic`, `Symmetric { magnitude }`,
`Deviation { plus, minus }`, `Limit { upper, lower }`, `Min`, `Max`.

## ★ Rule one: never build a preview by concatenation

`docs/core-api/03-capabilities.md` §1.6 trap (b), and it is the trap that
cost `pdfcer` a shipped defect of its own:

> A panel that previews *"nominal + tolerance"* by concatenation **will
> disagree with the bytes in the page** for every limit tolerance.

Because `Tolerance::suppresses_nominal()` is true for `Limit`, and the
appearance-stream baker branches on it: the label becomes `50.20/49.90`
with **the nominal gone**, not `50.00 50.20/49.90`. And `Basic`'s caption is
the **empty string** — the box is the notation.

So this module renders **fields**, never a specimen. What each form will do
to the printed label is stated in a sentence
([`crate::text::panels::dimension::tolerance_suppresses_nominal`],
[`crate::text::panels::dimension::tolerance_is_a_box`]) which is a claim
about behaviour rather than a second derivation of a string the engine
already owns. `Pass 68.0`'s defect — the pane reading `77.5°` while the
`/AP` read `77.47 pt` — was exactly two independent derivations of one
display value, and the fix was *one producer, always*.

## ★ Rule two: nothing is clamped, swapped or absolutised

`Tolerance::validate` refuses and says why, and `tolerance.rs`'s own comment
is the reason: *"a corrected value the operator never saw is exactly the
sneaky case."* A negative symmetric magnitude, an inverted limit pair and a
non-finite value are all refusals **by name**, with `ToleranceError`'s own
`Display` — written for an operator, e.g. *"a symmetric tolerance's
magnitude must not be negative (write ±0.1, not ±-0.1)"*.

This module therefore lets the operator type an invalid pair, shows the
engine's sentence, and **raises no action** until it validates. The
alternative — silently swapping an inverted limit — would produce a drawing
that says something the operator did not ask for and would not notice.

## Why the values carry no unit suffix

`Tolerance::caption` emits none in any branch, deliberately: a tolerance is
read in the nominal's unit, and `"50.00 mm ±0.10 mm"` is not how a drawing
is written. The fields match, and a note beneath names the unit once.
