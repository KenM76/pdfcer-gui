# `pdfcer-gui/canvas/moving/refusal`

## Item notes

### `enum Refusal`

Reported rather than silently absorbed, and reported with enough detail to
act on, because *"nothing happened"* has several causes with opposite
responses: a drag that ended where it started is correct behaviour, a text
object at the Part rung is a missing verb, and a degenerate page is a
broken document.

### `fn worded`

# Exhaustive on purpose — a `_ => None` would be the defect

Four arms return a sentence and the rest return `None`, and it would be
shorter to write the four and catch the rest with a wildcard. That
shorter version has one property this one does not: **a thirteenth
refusal would join the silent ones without anybody deciding that it
should.**

Written out, adding a variant to [`Refusal`] is a compile error until
somebody answers *does this one owe the operator a sentence?* — which is
the question that was never asked about `NoVerbForPart`, and
`OPERATOR_REQUESTS.md` **O188** is what that costs: a box drawn round one
label, a drag across the sheet, and nothing happening with no sentence
anywhere.

# Why the other nine stay silent, and why it is still the right default

They describe states the operator put themselves in and can see: nothing
selected, a drag that travelled no distance, a rung entered with nothing
named in it. A status bar that narrates the obvious is a status bar that
stops being read, which would cost the two sentences that matter. The
test is not *is this a refusal* but **can the operator see the cause**.

[`PartKind::Subpath`] is answered explicitly rather than folded in
with `NoVerbForPart(_)`. `eligible` routes a subpath at the Part rung to
`move_subpath`, so that combination is unreachable today — and an
unreachable arm that is written down is a claim the next reader can
check, where one hidden behind a wildcard is an assumption.
