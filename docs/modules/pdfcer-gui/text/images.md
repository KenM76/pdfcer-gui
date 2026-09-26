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
