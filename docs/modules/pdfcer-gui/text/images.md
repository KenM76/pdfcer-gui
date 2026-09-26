# `text::images` — the words the Insert-image window shows

## What this surface is

`edit.insert_image` was registered, drawn on Edit ▸ Insert, listed in
`reach`'s `SCAFFOLDED` set with **no recorded reason at all**, and inert.
`EditSession::add_image` has shipped the whole time.

## This catalog's hardest job: saying what a resolution MEANS

An image placed into a PDF has no resolution of its own — §8.9.4 maps it
onto the **unit square** and the content stream's matrix scales that square
to whatever size the page asks for. So *"is this picture big enough?"* is a
question about the **placement**, not about the file, and it has no answer
until the operator has said how large the picture will be on paper.

`pdfcer-core` reports the answer as a number rather than as a warning, and
its own doc comment is the argument this catalog follows:

> Not a warning — a number. An operator dragging a 4000-pixel photo into a
> 2-inch box gets 2000 dpi (wasted bytes) and one dragging a 100-pixel logo
> across a page gets 12 (visibly soft), and **neither is visible on screen
> at editing zoom**.

That last clause is why the sentence exists: both mistakes look perfect
until the sheet is plotted.

## It IS previewed now, and the request that got it is worth the paragraph

This section used to say the resolution could not be shown before the
commit, because `NewImage` offered `placed_rect()` as a pure preview and
nothing for the resolution — and computing `pixels / (points / 72)` locally
would have been the second derivation `placed_rect()`'s own doc warns
about: *"re-deriving the arithmetic in the GUI is how a preview and a result
drift apart."*


**And the four-line version this shell nearly wrote would have been
wrong.** Under `ImageFit::Contain` the placed rectangle is the *letterboxed*
sub-rectangle, not the box the operator typed — so measuring `rect` reports
a resolution low by exactly the letterbox ratio. The pure sibling is not
saving four lines; it is saving the letterbox.

A **zero-area** placement reports `(0.0, 0.0)` and `below_screen_resolution`
is `true`. Zero is not a resolution, but it is a number a label can render,
and the engine chose it over `inf` and over `NaN` — the last of which would
have made its own preview/outcome equality test pass vacuously.

## Item notes

### `fn byte_change`

Both numbers, never a ratio alone. *"38 % larger"* on a 4 KB logo and on a
a 40 MB scan are the same sentence about very different documents, and the
operator's question is what happened to **their file**.

### `fn the_resolution_is_always_stated_and_a_soft_one_says_why_it_matters`

The number is always given because `pdfcer-core` insists it is *"not a
warning — a number"*: an operator placing a 4000-pixel photo in a 2-inch
box has done nothing wrong and has wasted several megabytes, and only
the figure tells them.

### `fn a_placement_is_letterboxed_or_distorted_and_not_both`

They are mutually exclusive by construction in the engine — one fit mode
produces each — and emitting both would describe a placement that cannot
happen.

### `fn nothing_worth_saying_produces_exactly_one_sentence`

The commonest path by far — a box the right shape, bytes passed through
— and it must not produce a paragraph. Three sentences on every insert is
how an operator learns to stop reading the one that matters.

### `fn every_known_recompression_reason_has_its_own_words`

The alarm `#[non_exhaustive]` takes away from the compiler. It cannot
catch a sixth variant — nothing downstream can — but it catches the
failure that is actually likely, which is an arm deleted or two variants
collapsed into one. `SourceCodecNotReusable` and `NoCompressedSource`
are asserted DIFFERENT for the engine's own reason: conflating them
tells a TIFF owner their file was uncompressed.

### `fn recompress_reason_fallback`

Named as a function rather than as a literal in the test above, so the
two cannot drift — which is the whole shape of the `NO_SURFACE.md`
finding about a test asserting a constant against a function returning
that constant. Here the relation is the assertion: *nothing known
reaches this*, whatever it says.

### `fn the_preview_and_the_outcome_state_the_resolution_alike`

They are two functions, and the engine went to the trouble of making the
two *sources* one — deleting its own copy of the formula so
`add_image` calls the pure sibling. This asserts the shell did not undo
that on the wording side: an operator who reads "150 dpi" in the window
and "150 dpi" in the status bar has been told one thing twice, which is
what makes the preview trustworthy.

The soft case is asserted in both, because that is the one where a
difference in phrasing would read as a difference in verdict.

### `fn source_size`

The **displayed** size, which for an EXIF-rotated photograph is not the
stored one — the engine transposes it and this reads the transposed value,
because the stored shape is not on screen anywhere.

### `fn format_name`

`ImageFormat` is `#[non_exhaustive]`, so a match here could not be
exhaustive and could never fail to compile when a format is added: the
wildcard the compiler forces is the wildcard that silences it for ever
(recorded in `D:/dev/rag/rust/` under that name). This function was first
written as four arms plus a fallback, and it did not need to be — the enum
carries `ImageFormat::name()`, which is `const`, is what the engine's own
refusal messages use, and gains a new format the moment `sniff` does.

Deriving the string from the value rather than from a table beside it is the
first of that finding's four remedies, and where an upstream accessor exists
it is the only one needed.

### `fn natural_size`

**The provenance is half the fact.** `ImportNotes::dpi_source`
distinguishes *"the file said 300 dpi"* from *"pdfcer assumed 72"*, and the
engine keeps them apart deliberately. A natural size derived from an assumed
72 dpi is not a claim about the picture — it is one pixel per point, which
is the PDF default and nothing the file asked for.

### `fn placement_page_hint`

The image goes on the page the operator is looking at, which is the answer
every other page-scoped verb in this application gives, and stating it is
what makes that checkable — the window is centred over a document they may
have scrolled. The same reasoning the Insert-from-file dialog gives for
naming its destination by number.

### `fn placement_y`

**From the BOTTOM**, because PDF user space has its origin at the
bottom-left and y increases upward (§8.3.2.3). Measuring from the top here
would be friendlier for one field and would disagree with every coordinate
the Properties panel, the object tree and the rulers report — and an
operator comparing two numbers that mean different things is worse off than
one learning a convention their drawing package already uses.

### `fn fit_name`

Named by **what happens to the picture**, not by the engine's identifier.
"Contain" and "Stretch" are precise and are words about a box; an operator
deciding this is thinking about their photograph.

### `fn dpi_preview`

The number `pdfcer-core` insists is *"not a warning — a number"*, shown
beside the spinners that decide it rather than after the commit that fixes
it. Both mistakes it can report look perfect on screen at editing zoom: a
4000-pixel photo in a 2-inch box wastes megabytes, and a 100-pixel logo
across a page plots soft.

### `fn placed_note`

`NewImage::placed_rect()` is public *for this*, and its doc says why:
*"a front end drawing a preview must draw the same rectangle the edit will
produce, and re-deriving the arithmetic in the GUI is how a preview and a
result drift apart."* Nothing here computes a rectangle.

Shown only when it differs from what was asked for — under `Stretch` it
never does, and a line restating the two numbers above it would be noise.

### `fn off_the_page`

Refused rather than clamped. A picture silently moved back onto the sheet
is a placement the operator did not make, and they would find it by looking
at the drawing rather than at this window. The same posture
`Tolerance::validate` takes: *"a corrected value the operator never saw is
exactly the sneaky case."*

### `fn recompress_reason`

`RecompressReason` carries no `Display`, and that absence is a decision
rather than an omission: these are *pdfcer's* reasons, in pdfcer's vocabulary
— an alpha channel split out into an `/SMask`, a TIFF codec with no encoder
on this side — and the engine leaves the English to the front end because
only the front end knows who is reading it.

The engine draws one distinction this catalog keeps, because it is the one
that changes what an operator should do:

| class | variants | how it reads |
|---|---|---|
| **your file forced this** | `AlphaSplit`, `NoCompressedSource`, `SourceCodecNotReusable` | a fact, nothing to decide |
| **you asked for this** | `LosslessRequested`, `JpegRequested` | *"a chosen reason is not a substitution, so a front end should not apologise for it"* — the engine's own words |

`SourceCodecNotReusable` is kept apart from `NoCompressedSource` for the
reason its own doc gives: conflating them *"tells a TIFF owner their file
was uncompressed"*. There were bytes; they were simply not reusable.

# The wildcard is forced, not chosen

`RecompressReason` is `#[non_exhaustive]`, so this match cannot be
exhaustive and can never fail to compile when pdfcer grows a sixth reason —
see `D:/dev/rag/rust/`'s finding of that name. There is no upstream
accessor to delegate to here (unlike `ImageFormat::name`), so the fallback
is a true sentence that says a re-encode happened without inventing a
reason for it, and the test below asserts none of the five known variants
reaches it.

### `fn import_failed`

Passed through, unlike a `TwoLineRefusal`, and the difference is worth
stating because the two look like the same case. `ImageImportError`'s
messages **name the operator's file** — *"pdfcer does not place GIF images —
it places PNG, JPEG, BMP and TIFF"*, *"this image uses {feature}, which
pdfcer cannot place"* — so the specific half is the whole value and a
catalog sentence would have to discard it. `crate::text::canvas_render_failed`
makes the same call for the same reason.

### `fn placement_disclosures`

Every one of these is a fact the operator **cannot see on screen at editing
zoom**, which is the rule-4 test in its purest form for this feature: the
picture looks identical whether it was stored at 12 dpi or 2000, whether its
bytes passed through unchanged or were re-encoded, and whether a lossy
source was re-compressed lossily a second time.

Returns them in the order they matter to a drawing:

1. **resolution**, because it decides whether the sheet plots acceptably;
2. **shape**, because it decides whether the picture is honest;
3. **bytes**, because it decides how big the file got and why.

A clause is emitted only when it has something to say. `letterboxed` on a
box the operator drew to the picture's own shape is false, and a sentence
about it would be noise on the commonest path.
