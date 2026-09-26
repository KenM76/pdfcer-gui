# `text::settings::look` — what changing it makes you see in the DOCUMENT

## The split is by BLAST RADIUS, which is the window's own taxonomy

Not by dialog group, and not alphabetically. Every setting in this window
carries a `*_radius` line stating *which way costs what*, and that line is
one of exactly four things:

| module | radius | settings |
|---|---|---|
| [`super::look`] | changes what the DOCUMENT looks like; the file is untouched | CMYK intent, CMYK JPEG polarity, CMYK ceiling, blending space, comment author, mask resampling, minification |
| [`super::extract`] | changes what you GET OUT — copy, search, redaction-by-pattern, new ce dimensions | word gap, unmappable codes, replacement text, parallel tolerance |
| [`super::bytes`] | changes what pdfcer WRITES | separations, missing appearance state, index line endings, trailing newline |
| [`super::shell`] | changes pdfcer's OWN window and touches no document at all | theme, UI scale, render quality, zoom settle, opening fit, wheel paging, paste chords, chrome, page cache, mesh padding, preset, auto-hide, Tab order |

That taxonomy is load-bearing rather than a filing convenience: it is the
distinction the window exists to make legible, and a test in [`super`]
asserts that exactly the byte-changing settings say they change the file —
in both directions, so a preview setting cannot quietly claim a consequence
it does not have.

The first three rows are answers to a silent standard and the fourth is
not, which is why [`super::shell`] is a separate module rather than a
section of this one: an operator reading *"the standard leaves this
undefined"* over a choice about how big pdfcer draws its own buttons is
being told something untrue.

One setting is filed by its radius rather than by its group and it is worth
naming: **CMYK JPEG polarity** is here under *look*, and its radius line
also says *"and the saved file if pdfcer re-compresses the image"*. It is
the only setting whose radius spans two of the four. It sits with the
others in its dialog group, where an operator looks for it.

## Item notes

### `fn cmyk_intent_neutral_note`

The last sentence is the one that matters: the divergence is **narrow by
construction**. Only the pure-K axis moves and every mixed colour still
uses the measured table, so an operator worrying that pdfcer has invented
its own colour science can be answered from the window.

### `fn polarity_radius`

**The only preview setting that can also change saved bytes**, and it is
the second half of the sentence that says so. A re-encode under the wrong
polarity bakes the inversion in permanently.

### `fn polarity_never_note`

Every other note either says "this is a guess" or says nothing about
provenance. This one claims the opposite, and it is entitled to: `"invert"`
occurs **zero times** in the Adobe technical note the standard makes
normative, the marker carries no polarity flag at all, and all four
reference engines make the same choice.

The distinction is the point. "pdfcer matched every other implementation"
and "pdfcer guessed" must not read alike, or the operator has no way to tell
which of thirteen defaults to trust.

### `fn cmyk_ceiling_title`

> *"can the size of the buffer be increased? Allow the user to set the size
> up to the maximum possible?"*

# Why this is worded as a SYMPTOM and not as a buffer

Nobody arrives at this window looking for a compositing buffer. They arrive
because *"the colours in this file change when I zoom"* — which is exactly
how it was reported, and which is what the title says. The mechanism is in
the note, underneath, for the reader who wants it.

# The three numbers this copy owes, and why each is here

All measured by the engine, not estimated:

* **20 bytes a pixel**, which is what makes the memory figure predictable;
* **about 50 % slower** than blending on screen at the same pixel count
  (1.4 s against 0.9 s at the boundary) — the trade is *correct colours are
  slower*, and an operator raising a ceiling deserves to know that before
  they notice it;
* **up to about four times the number chosen**, because the ceiling bounds
  ONE buffer and a page with nested transparency holds several page-sized
  ones at once — the page buffer, a group's child, a retained spare, and a
  whole backdrop copy for a knockout group.

The third is the one that must not be left out. The first version of the
advice given to the operator said "about 5 GB is the maximum possible",
which is right per buffer and **four times too low as a memory figure** —
and too low is the direction he could act on.

### `fn parsed_as`

The whole line is `= 256 MiB`, and it appears **only** when the operator's
spelling is not the canonical one — `256 MiB` echoed back as `256 MiB` is
noise, while `0.25gb` answered with `256 MiB` is the reason the line exists.
It is how somebody learns, without being told, that this field speaks binary
megabytes.

### `fn unparsed_value_note`

Deliberately says what is still true rather than what is wrong: the stored
value stands, nothing has been lost, and the operator is very likely
mid-keystroke. A message that read like a rejection would be false — the
field accepts every keystroke and always will.

### `fn author_name_title`

The title says **your comments**, not *"annotation title"* and not
*"the /T entry"*. §12.5.6.4 calls the field a title, which is a word from
the format rather than a word about the job, and an operator scanning
headings for *"why do my comments say nobody"* would not stop at it.

### `fn author_name_silence`

It states the consequence plainly rather than calling the empty value
a problem. An anonymous comment is legal, is what pdfcer wrote before this
existed, and is a reasonable choice for a drawing leaving the firm. The
window's job is to say what silence does, not to nag.

### `fn author_name_radius`

It names the two boundaries that matter: it goes into the file, and it
does not apply retroactively. Somebody who sets it after a review session
should not expect yesterday's comments to be signed.

### `fn author_name_note`

It answers the question the empty box provokes — *"why does pdfcer not
already know this?"* — because the answer is a decision rather than an
omission, and one an operator would agree with if told.

### `fn mask_box_note`

The second sentence is not in the source and is the case that actually
decides it: a mask at **higher** resolution than the base read one sample
per texel discards fifteen sixteenths of what the producer supplied.

### `fn minify_point_note`

`pdfcer-core` grades this default tier (d) — reasoned inference — as
explicitly as it grades the mask filter beside it, and the old note read as
a confident recommendation with no such admission. Obligation 1 was
therefore unmet for this setting for the whole of its shipped life.

The final sentence is more than a disclosure: it names the **specific
observation that would change the default**, which is what turns a
confessed guess into a piece of work somebody can do.
