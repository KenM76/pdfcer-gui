# `text::editrefusal` — **why a text edit the operator committed did not
happen**, in his words

`OPERATOR_REQUESTS.md` **O140**, **O141** and **O142**. One enum, one
classification function, one sentence per cause — and the whole of what this
shell says when `EditSession::edit_text` comes back `Err`.

## Why it is its own file

**R2.** [`super::textedit`] crossed 1,500 lines on 2026-09-06, when O142's
ambiguity refusal and `Pass 256.1`'s ambiguous-character refusal arrived
together. The seam was already drawn and already labelled — that file carried
a banner reading *"Why an edit the operator committed did not happen — O140"*
— and the two subjects either side of it are genuinely different questions:

* above it, **what an edit COSTS**: the pinned-tail disclosure, the
  multi-run note, the reflow refusals. Sentences about an edit that
  happened, or about a re-wrap that was declined before any verb ran.
* here, **why a commit was REFUSED**: the engine answered `Err`, and this is
  the joining of its coarse `RefusalKind` with the facts only the shell
  holds.

Every item is re-exported from [`super::textedit`], so no call site moved
and nothing outside this pair needs to know the split happened.

## The rule that governs every arm in here

**The engine's category wins wherever it has one, and a shell-side fact is
allowed to sharpen exactly the bucket where the engine's answer is true and
unusable at the same time.** Three shell-side facts are consulted, each in
one bucket only:

| fact | bucket | what it separates |
|---|---|---|
| `character` ([`RefusedCharacter`]) | `UnsupportedFont` | a font that cannot spell the letter, from one that spells it two ways |
| `occurrences` | `NotFound` | a page holding the words twice, from a run written one glyph at a time |
| `one_operator` | `NotFound` | a run written in pieces, from a page that moved under the caret |

⚠ **Nothing here greps a `Display` string or keys on a trigger id to
reconstruct a category.** `RefusedCharacter` reads one *field* of a
structured refusal, on the same licence the character itself is read on, and
its constructor's doc comment carries that argument in full. Building a
second copy of the engine's taxonomy is what `RefusalKind`'s own header
exists to forbid, and the cost of getting it wrong is named there: telling
the operator the WRONG reason, which is strictly worse than the silence it
replaced.

## Item notes

### `fn no_edit_refusal_opens_by_naming_what_the_operator_did`

Checked on the first clause because that is the part the status bar
actually shows — `disclosure_line` truncates and hangs the rest on
hover.

### `fn the_engines_four_buckets_map_to_six_sentences_and_two_of_them_split`

The compiler already proves `EditRefusal::of` handles every
`RefusalKind` — that is what `RefusalKind` not being `#[non_exhaustive]`
buys, and it is why the engine committed to it. What the compiler cannot
prove is that the **`NotFound` split is wired the right way round**, and
getting it backwards is the failure mode that matters most here: the
operator would be told his page had moved when his line is written one
letter at a time, or the reverse.

### `fn a_font_refusal_that_names_a_character_is_the_one_with_a_way_out`

`EditError::Refused(_)` maps to `RefusalKind::UnsupportedFont`
**wholesale**, so R-INV-1 (*"this font has no glyph for '€'"* — a
character the subset does not carry, and a face swap fixes it) and
R-INV-2 (*"its code↔glyph relation lives inside the embedded program,
which pdfcer-core does not parse"* — and no face swap helps, because the
run cannot be re-encoded at all) arrive as the same category.

Getting it round the wrong way costs in both directions: an operator two
clicks from their `€` is told pdfcer cannot edit the text, or an operator
with an unreadable font is sent to a chooser that will refuse every row.
`Refusal::character` is `Some` for exactly the first family and `None`
for the second, which is why the datum is read rather than the trigger id.

Asserted across `one_operator` as well, because the run's provenance
has nothing to do with the font's repertoire and a condition that crept
in would make the sentence depend on how the producer emitted the line.

### `fn the_missing_character_sentence_names_the_surface_that_answers_it`

Without the last clause O141 is answered with a better diagnosis and no
route — which is exactly the state O141 was filed about: *"That last
clause is the answer to your question, and it is buried in an error
message."*

### `fn the_two_content_refusals_explain_why_his_own_added_lines_edit`

*"the lines I added below `price)` are editable, but everything else
that existed when I got the pdf is not."* He had the diagnosis before
the program did. A sentence that explains the refusal and says nothing
about why his own lines behave differently leaves the one question he
actually asked unanswered.

### `fn no_edit_refusal_offers_a_remedy_this_build_does_not_have`

⚠ Verified before it was written, not assumed: there is no verb in this
shell that deletes a text run, and `add_text` writes the engine's
bundled Helvetica — so *"delete it and retype it"* would cost the
operator his typography and his position, and is not offered. A remedy
named in a decline is a promise, and a promise that does not resolve is
worse than the silence it replaced.

### `fn the_split_run_sentence_says_the_document_is_unchanged`

The failure this ends is not *"I was not told why"* — it is *"I do not
know whether it took"*. `edit_declined_by_engine`'s own documentation
carries the argument; this variant inherits the obligation.
