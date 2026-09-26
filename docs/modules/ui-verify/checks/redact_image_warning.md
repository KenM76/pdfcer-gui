# `ui-verify/checks/redact_image_warning`

`marking_over_an_image_says_so_before_apply` — a redaction mark that covers a
raster image discloses it when the rectangle is drawn, not when Apply is
pressed.

# The operator's report, `OPERATOR_REQUESTS.md` O103

> *"every time I've tried the redact feature it tells me it can't because
> there is objects that weren't redacted."*

Reproduced with `pdfcer` alone, so the refusal is the engine's:

```text
redaction refused: redaction region on page 1 intersects an image; pdfcer
cannot yet destroy image pixels … apply refused rather than producing a
false redaction
```

The refusal was right. What was wrong is *when* he found out — apply was
all-or-nothing for the document, so twelve careful marks and one that grazed
a logo produced a single refusal naming no region, after the work rather than
during it.

# THE REFUSAL IS GONE, and this check's subject changed with it


The disclosure did not become unnecessary — it changed subject, and to the
more important of the two. A raster redaction is irreversible in a way a text
one is not: the samples are overwritten and re-encoded, and what comes back
is a black block where the logo was. Learning that while the rectangle is
being drawn is the same argument the original made, applied to the opposite
outcome.

⇒ **The engine's reply told us to re-word it, by name.** Nothing in this
repository could have: a claim about an external limitation, phrased as a UI
string, compiles and passes for ever after the limitation lifts. See
`crate::checks::driving::declared_or_in_overflow` for the same shape found
the same morning in this harness's own code.

# Why this check asserts BOTH directions, and would be worthless with one

A check that only proves *"the warning appears on a document with an image"*
passes just as happily on a build that warns about **every** mark on every
document. That build is worse than no warning at all: a caveat attached to
every action is one an operator learns to scroll past, and the day it is
load-bearing they will scroll past it too.

So the second half is the real assertion: on a drawing with **no** image the
same gesture must produce **`disclosures=none`**. The two halves together
say the warning tracks the fact rather than the feature being on.

⇒ This is the same shape as `field_shading`'s three states and the tab-order
section's openness: a signal that cannot be absent is not a signal.

# What a passing run does NOT prove

That the count is right. The warning says how many images the region covers,
and this asserts only that it is non-zero on a document that has one and
absent on a document that has none — the arithmetic is
`app::actions::redactimg`'s and is a straight filter over the object model.
It also does not press Apply: what happens there is the engine's, was
reproduced with the CLI in O103, and is not this shell's to verify.

## Item notes

### `const CONSEQUENCE`

Matched on the CONSEQUENCE rather than on "image", because the sentence has
to tell the operator what will happen to him and not merely what is on the
page. A build that said "this region covers 1 image(s)" and stopped would
pass a looser check and leave him no wiser about what Apply will do.

**It read `"will be refused"` until 2026-09-03, and the words changed
because the OUTCOME did.** `pdfcer-core` v0.26.0 (`Pass 245.0`) destroys
image samples under a region instead of refusing the document, so the
disclosure stopped being a warning about a failure and became a warning
about an irreversible success. This constant is what made that a one-line
edit: the check asserts *"the consequence is stated"*, and only the
consequence moved.

⇒ A check pinned to a whole sentence would have gone red here and read as a
regression in the shell, when what had happened is that the engine got
better.
