# `text::measure` — what the measure tools say about what they inferred

## Rule 15

What these tools author is a **ce dimension** — one pdfcer writes. A **pdf
dimension** is CAD-exported page content pdfcer reads and must not alter.
This catalog avoids the bare word, exactly as [`crate::text::scale`] and
[`crate::text::dimension_groups`] do.

## ★ Why this module exists at all

Because the two-line tool's output is an **inference**, and the shell was
swallowing every statement about it.


| fact | what it means | what happened without it |
|---|---|---|
| `TwoLineRefusal` | collinear or zero-length lines — refused **by name** | the second click did nothing, silently, and the operator clicked again |
| `measured_angle_degrees` | the true angle, reported **even when forced parallel** | a checkbox that hides the number it is overriding |
| `apex_is_real() == Some(false)` | the lines meet only if extended | ordinary in CAD, and a fact the operator may not have noticed |

`docs/core-api/03-capabilities.md` §1.5 obligation 4 states the second of
those and quotes the engine's own reason: *"a checkbox that hides the number
it is overriding is withholding the fact that makes the decision a
decision."*

## Why the refusal is worded HERE and not taken from the error's `Display`

`TwoLineRefusal`'s `thiserror` messages are written for an operator, and the
capabilities doc says to surface them. It is still not right to print them:
`check-ui-strings.sh`'s exclusion 3 says in as many words that an error
type's `Display` output being exempt *"is not permission to route UI text
through an error type"*.

So the variant is matched and the sentence lives in this catalog — which is
the same resolution [`crate::text::markup::deleted_collateral`] reached for
`DeletionReport`. The **refusal is still surfaced by name**, which is what
the obligation actually asks for; what changes is which crate owns the
English, and that is decision 002 R1.
