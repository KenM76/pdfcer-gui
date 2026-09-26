# `canvas::scaling` — what rides along when a resize scales something

Three switches, on the Tool row, for the operator to set before a drag.
`OPERATOR_REQUESTS.md` **O51**:

> *"if that was the resize question about scaling line weight, etc with
> resize it got the answer wrong. default should be what it said, but there
> should be an option that they do scale with resize. Inkscape has options
> for this and I want the same."*

## The correction this module IS, because it is about reasoning

This project told `pdfcer-core` that a resize must **not** scale stroke
width, with three arguments: a CAD line weight is a drafting standard rather
than decoration; a non-uniform scale makes a single `/BS /W` scalar
ill-defined; and neither Acrobat nor Illustrator scales one by default.

**All three stand. The conclusion did not.**

⇒ *Convergence among reference implementations argues for a **default**, not
against an **option**.* The third argument contained its own refutation —
*Illustrator ships the toggle off* means **Illustrator has the toggle** —
and it was walked straight past. Inkscape puts four of them on the selector
tool's control bar.

So the defaults here are exactly what was argued for, and every one of
them is now something the operator can change.

## Why the Tool row and not Settings

Because it is a **per-drag modifier, not a preference**. Inkscape puts them
on the selector tool's control bar for the same reason: an operator decides
*for this resize* whether the border should thicken, the way they decide
whether to hold Shift. A settings dialog is where you say what pdfcer should
usually do; this is where you say what this gesture does.

## Why the third switch exists, and why it is NOT an Inkscape parity item

Because of a fact the engine established and neither program handles well:
**no per-axis stroke width exists**, in PDF or in SVG. `/BS /W` and `w` are
scalars. An annotation's artwork is placed through §12.5.5's matrix, which a
resize makes a scale, and that matrix is applied *after* stroking — so under
a non-uniform scale the drawn stroke becomes anisotropic and **no value of
`/BS /W` describes it**.

Inkscape hit the identical thing in SVG (Launchpad #1335376) and closed it
**Invalid** — a mathematical limit, not a defect. Its behaviour is to
silently produce a distorted stroke.

⇒ pdfcer refuses instead, **by name**, and this switch is the operator's way
to say *"proceed anyway"*. O51's ruling on that choice is explicit: the
honest options are refuse, or proceed and state the residual distortion —
*"never silently pick a fudge factor, which is the one thing the parity
reference does."*

It applies only where pdfcer did **not** author the appearance. An
appearance pdfcer built is rebuilt from the scaled geometry at the new size,
and both stroke-toggle states are then exactly satisfiable.

## The defaults, and why two of the three are `false` for opposite reasons

| switch | default | because |
|---|---|---|
| [`Modifiers::scale_stroke_width`] | **off** | a line weight is a drafting convention, not a length in the space being scaled |
| [`Modifiers::keep_rect_differences`] | **off**, i.e. `/RD` *does* scale | an inset **is** a length in the space being scaled; leaving it fixed while `/Rect` doubles changes the proportions |
| [`Modifiers::allow_distortion`] | **off** | a refusal that names its remedy beats artwork silently going oval |

The first two look inconsistent and are the same rule applied twice. The
engine promoted the discriminator out of this shell's own CAD argument:
**is the property a length in the space being transformed?** An inset is; a
line weight is not. Two opposite defaults, one question.

## Memory-backed, like the text pen

Same mechanism and same reason as [`crate::canvas::textedit::pen`]: the
value is read by the canvas and written by a panel, neither of which owns
the other, and it must survive a panel being closed. It is deliberately
**not** persisted to `preferences.txt` — a per-drag modifier that came back
set from last week would surprise somebody who has forgotten setting it.

## Item notes

### `fn the_defaults_are_the_arguments_that_were_accepted`

Not a tautology over `Default::default()`: it asserts the three engine
fields, through `to_options`, which is where an inverted mapping would
show up. `keep_rect_differences` is the one that reads backwards —
`false` means `/RD` **does** scale — and a shell that "fixed" that
reading would leave an inset fixed while the rectangle doubled.

### `fn each_switch_reaches_its_own_engine_field`

The failure this guards is a mapping that drops a field: three
checkboxes on the Tool row, two of which do something, and no error
anywhere. It is asserted by turning them on **one at a time**, because
all-three-on would pass on a build that ORed them together.
