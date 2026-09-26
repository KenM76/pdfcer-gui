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

★ Every item is re-exported from [`super::textedit`], so no call site moved
and nothing outside this pair needs to know the split happened.

## ★★★ The rule that governs every arm in here

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
