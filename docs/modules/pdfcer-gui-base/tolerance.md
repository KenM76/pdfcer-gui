# `tolerance` — the seven forms, and the two
things a panel must not do with them

## What a tolerance is, in this model

One more property of the `Pass 69.0` style cascade — not a parallel system.
It inherits factory → group → ce dimension exactly like a stroke width, uses
the same `Option` as its override checkbox, and has the same
clear-restores-inheritance semantics. That was the point of building the
cascade first.

Seven forms: `None`, `Basic`, `Symmetric { magnitude }`,
`Deviation { plus, minus }`, `Limit { upper, lower }`, `Min`, `Max`.

## Rule one: never build a preview by concatenation

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

## Rule two: nothing is clamped, swapped or absolutised

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

## Item notes

### `const SPEED`

An order of magnitude finer than the style module's point-valued
properties, because a tolerance is a manufacturing quantity: 0.05 is a
coarse fit and 0.005 is a bearing seat, and a spinner that skates past the
second is a spinner nobody uses twice.

### `const FORMS`

Constructed with placeholder magnitudes because the combo selects a
**shape**, and the value the operator was last editing is preserved by
[`reshape`] rather than by the list. Listing `Symmetric { magnitude: 0.0 }`
here and selecting it directly would zero a number the operator had
already typed.

### `fn same_form`

The combo's selected-state predicate. `PartialEq` compares magnitudes, which
would leave the combo showing nothing selected the moment the operator
changed a number — the control silently disowning the value it is editing.

### `fn reshape`

# Why this is not simply `*value = form`

Because an operator switching from *± 0.35* to *separate + and −* means
*"start from what I typed"*, not *"throw it away and give me 0.1"*. The
magnitude is the value they have been thinking about, and re-typing it is
the panel making them prove they meant it.

The mappings, and each is the reading a drafter would make:

| from → to | carried |
|---|---|
| symmetric → deviation | `± m` becomes `+m / −m`, which is the same tolerance written the other way |
| symmetric → limit | `± m` becomes `+m / −m` about the nominal, the same again |
| deviation → symmetric | the **larger** magnitude, because a symmetric tolerance that is tighter than the one it replaced would silently narrow a specification |
| limit → deviation | the pair, unchanged |
| anything → none / basic / min / max | nothing to carry; those forms hold no value |

Deviation → symmetric taking the larger is the only one that could be
argued, and it is argued the safe way round: a manufacturing tolerance that
gets **tighter** without the operator asking is a part that gets rejected.

### `fn the_combo_offers_each_form_exactly_once`

The count is asserted against the list rather than against a literal
seven **and** the discriminants are asserted distinct, so a form added
to the engine and forgotten here fails on the first assertion while a
form listed twice fails on the second.

### `fn collapsing_a_deviation_never_tightens_it`

The safe direction, and the reason is a manufactured part rather than a
preference: a tolerance that tightens without being asked produces
components that get rejected against a drawing nobody changed.

### `fn the_combo_stays_selected_while_the_number_changes`

Without this the combo would show nothing selected the instant the
operator dragged the magnitude — the control disowning what it is
editing.

### `fn an_inverted_limit_is_refused_rather_than_corrected`

The assertion is on the ENGINE's verdict, because that is what `show`
consults — a local re-implementation of the rule is exactly what would
let the panel and the file disagree.

### `fn show`

Returns `false` when the current fields do not validate, in which case the
caller must not raise an action — see the module header's rule two. The
refusal has already been drawn by the time this returns, so the caller does
not have to know why.

`unit` is the ce dimension's **resolved** display unit, used only for the
note saying which unit the numbers are in. It is resolved rather than the
group's own, because a ce dimension overriding its unit reads its tolerance
in the overridden one.
