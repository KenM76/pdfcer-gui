# `app::actions::redactimg` — say at MARK time that a region covers an image

## What this closes

**Ken:** *"every time I've tried the redact feature it tells me it can't
because there is objects that weren't redacted."*

A redaction that covers a raster is not a redaction that fails. The engine
gates on the image **samples** rather than on bounding boxes, so a region
that merely touches an image's rectangle destroys nothing; where the region
does cover samples it **destroys** them — decode, overwrite, clear the
matching part of any soft mask, re-encode losslessly — and removes an image
that is wholly covered. Only a mark over an image pdfcer cannot decode is
**retained** (`RedactionReport::marks_retained`), and only a document where
*every* mark would be retained is refused (`RedactError::ImageUndestroyable`).

**So the sentence this module supplies is about destruction, not refusal:**
*those pixels will be destroyed, not hidden*. A raster redaction is
irreversible in a way a text one is not — the samples are overwritten and
the image re-encoded — and the moment to learn that is while the rectangle
is being drawn.

⇒ **A claim about the engine that lives only in a UI string compiles and
passes for as long as it is false.** Where such a claim can be spelled as a
test assertion instead, spell it as one: `redact::tests`' image test goes
red the hour the engine changes underneath it, which is exactly the
behaviour a paragraph cannot have.

## This is DISCLOSURE, not a gate — and the distinction is load bearing

It refuses nothing and blocks nothing. The mark is authored exactly as
before, because a mark is reversible and costs nothing, and because pdfcer
must not decide on the operator's behalf that a region is not worth marking.
What changes is only that he is told, in the same breath as the success, and
can act while it is cheap — before the pixels go.

Rule 4: nothing is drawn on the canvas. The mark renders exactly as any
other mark renders, because it IS any other mark — a warning tint would be
pdfcer styling its own uncertainty into content, which is the thing the rule
forbids by name. The sentence goes where every other disclosure goes.

## Item notes

### `fn image_count`

Split out as its own function because it is the whole factual claim this
module makes, and because it is the part that could be wrong in a way an
operator would notice: over-counting invents a warning about a page that
would have redacted cleanly, and under-counting is the silence this module
exists to end.
