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

### `enum RefusedCharacter`

A shell-side spelling of one field of `pdfcer_core::text_edit::Refusal`, and
deliberately not a copy of the engine's `RInvTrigger`: it carries the **two
cases that produce different sentences here**, and collapses the six that do
not. Widening it is how a second taxonomy starts, so widen it only when a
seventh trigger earns a seventh sentence.

`app::status::decline::textedit::refused_char_kind` is the one place that
builds it, and its doc comment carries the argument for why reading
`Refusal::trigger` there is a datum rather than a re-derivation.

### `fn of`

# The order of the arms is the design

The engine's category wins wherever it has one, and the shell's fact is
consulted **only** inside `NotFound` — the single bucket where the
engine's answer is true and unusable at the same time.

That ordering is what keeps this from being a second taxonomy. A split
run whose edit was stopped by a font gets
[`Self::UnsupportedFont`], not [`Self::SplitAcrossPieces`], because the
font is what actually stopped it and the split is merely also true. The
failure mode of the opposite ordering is precisely the one
`RefusalKind`'s own header warns about: *telling the operator the wrong
reason, which is strictly worse than the silence it replaced.*

# `one_operator` is a measurement, and `true` is its "not measured"

See [`crate::canvas::textedit::Plan::one_operator`]. It is `true` when
the plan could not read provenance at all, so an unmeasured run falls to
[`Self::TextMovedAway`] — a sentence that is honest about a page that
moved and never claims a structure this shell did not observe.
# `character` is the second fact the category cannot hold

`OPERATOR_REQUESTS.md` O141. `Some` when the engine's refusal carried a
`Refusal::character` — i.e. the inverse-encoding gate stopped on **one
scalar it has no code for** (R-INV-1, 6, 7, 8) rather than on the font's
whole code↔glyph relation being unreadable (R-INV-2, 3, 4). Both arrive
as `RefusalKind::UnsupportedFont`, and only the first has a remedy.


It is consulted **only** inside `UnsupportedFont`, on the same rule the
`NotFound` split follows: the engine's category wins wherever it has one,
and a shell-side fact is allowed to sharpen exactly the bucket where the
engine's answer is true and unusable at the same time.
# `occurrences` is the THIRD fact the category cannot hold

`OPERATOR_REQUESTS.md` O142. `Some(n)` when the plan had to reach the run
by `find` — the producer wrote it one glyph per show operator, so the
provenance pin would have confined the match to one character — and `n`
is how many times that text occurs on the page. `None` when the pin was
exact and no string was being matched at all.

It is consulted **only** inside `NotFound`, and **before**
`one_operator`, and that ordering is the design rather than an
implementation detail. Both facts are true at once on the page that
raises this: the run *is* split across pieces, and the text *does* occur
twice. Only the second is what stopped the edit —
[`crate::canvas::textedit::plan`] kept the pin **deliberately**, to make
the request unmatchable rather than let the engine pick an occurrence —
so reporting the split would be this shell explaining its own refusal
with somebody else's reason.

⇒ That is the same rule the arms below already follow, applied one level
deeper: *the thing that actually stopped it wins, and the things that are
merely also true stand aside.*
# `stale_pin`

Whether the refusal was `EditError::PinnedSpanNotFound` — *the pin
names no operator* — as distinct from `NoMatch`, *the pin is fine and
the text does not begin there*. Both arrive as
`RefusalKind::NotFound`, so only the caller, which holds the
`EditError`, can tell them apart. See the `NotFound` arm below for what
each one means to the operator and why conflating them misdirects him.

### `fn name`

# Why this exists rather than a derive

`app::status::decline::textedit` publishes `said=` beside `kind=` so a
build that read the engine's category correctly and then chose the wrong
sentence goes red, and `tools/ui-verify`'s
`a_refused_character_offers_a_face_that_can_type_it` and
`a_refused_typo_fix_says_why_it_was_refused` both match on that field.

It was `{why:?}`. The moment [`Self::FontLacksTheCharacter`] gained its
payload the field became `FontLacksTheCharacter('q')` and both checks
went red against a build that was working perfectly — the same failure,
in the same file, that the trace's own comment had already recorded once:
*"`{:?}` on a domain type makes the trace's vocabulary a consequence of a
Rust derive, so it changes silently when the type does."* The note did
not prevent the second occurrence; this function does, because a variant
added without a name is a compile error here.

The payload is deliberately **not** in the name. The character already
has its own `character=` field on the same line, and one datum spelled
two ways in one trace line is how a reader and a check come to disagree
about which is authoritative.

### `fn line`

One function over the enum rather than one per variant, for
[`refusal`]'s and [`ReflowRefusal::line`]'s reason: a variant added
without a sentence is a compile error rather than a commit that refuses
silently.

# Every sentence is FRONT-LOADED, and that is a layout fact

`app::status::disclosure::disclosure_line` draws the decline with
`.truncate()` and hangs the whole text on hover. So the first clause is
what most operators read, and it must carry the claim that matters:
**pdfcer cannot**, not *you did something wrong*. The cause, the
contrast and the remedy follow it in that order.

# Two of them name his contrast explicitly

He noticed it before the program told him: *"the lines I added below
`price)` are editable, but everything else that existed when I got the
pdf is not."* A sentence that explains the refusal and ignores the
contrast reads as evasive, because he has already worked out that the
two cases differ and is waiting to hear why.

# Why this returns a [`Cow`] and every other catalog function does not


[`Cow::Borrowed`] is what five of the six arms return, so the change
costs no allocation on any frame that is not reporting this one refusal —
and the bar redraws every frame, which is why that mattered enough to
state. The alternative, a `String` return, would have allocated the same
five static sentences over and over for the life of the process.

### `fn font_lacks_the_character`

The operator, 2026-09-05: *"if the character isn't available in a pdf are we
able to change to a different font?"*

# Why the character is worth the one allocation this catalog makes

Because the generic form of this sentence is what he meets first, and it is
indistinguishable from the five other declines he might have earned. *"pdfcer
cannot type that character"* asks him to remember what he typed and to
believe pdfcer knows; *"pdfcer cannot type a `q`"* is a report he can check
against the keyboard in front of him. The engine handed the character over
(`Refusal::character`) and two surfaces already had it; only the bar could
not say it, and only because of a return type.

# The clauses, in the order the bar truncates them

`app::status::disclosure::disclosure_line` draws at most 45 % of the bar and
hangs the rest on hover, so the order is not style:

1. **what pdfcer cannot do, with the character** — the claim, which must
   survive truncation;
2. **why**, in his words rather than the format's: the font was built with
   only the letters the page already prints. O141's framing is that he should
   not have to learn the word *subset* to understand his own file;
3. **his document is unchanged** — the reassurance every decline in this
   catalog carries, because the operator's first question after a refusal is
   whether something happened anyway;
4. **where the route is.** The bar cannot hold a chooser; Properties can, and
   the sentence says so by name.

⚠ It deliberately does not say *"choose another font"* on its own. That is
the answer, and an answer with no control beside it is O141 filed all over
again — *"that last clause is the answer to your question, and it is buried
in an error message."*

### `fn font_has_two_glyphs_for`

# What is true here, and it is the opposite of the sentence beside it

[`font_lacks_the_character`] tells the operator the font *"was built with
only the letters your page already prints"*. That is exactly wrong for this
refusal: the character **is** on his page, in this font, drawn by two
different glyphs — and pdfcer declines to choose between them, because
choosing would silently swap one shape for another that happens to spell the
same letter.


The remedy is the same control as [`font_lacks_the_character`]'s and the
sentence ends by naming it, because the face offer really is raised for this
case too — `app::status::decline::textedit` calls
`panels::properties::refusedchar::record` on any refusal that named a
character, and this one names one.

### `fn run_cannot_take`

# The clause this sentence has and the other two do not

**"Nothing you have already typed is lost."** It is not politeness and it is
not shared with [`font_lacks_the_character`], because it is the one fact
that is *different* about this refusal. The commit-time refusals arrive
after `Ctrl+Enter` has called `commit_into` and then `abandon`, so the draft
really is gone and the catalog's usual reassurance — *your document is
unchanged* — is the whole of what can be said. This one arrives at a
keystroke, with the draft alive and the rest of the word still in the box.

An operator who has just watched one key do nothing does not know which of
those two worlds he is in, and the difference decides whether he keeps
typing or starts again. So the sentence says it.

# Why the second clause reports a MEASUREMENT rather than a cause

Rule 4's surviving half: *inferences the operator cannot see still owe an
off-canvas report.* Nothing visible happened here — a key was pressed and
no letter appeared — and the reason is a set pdfcer computed silently when
the caret landed. Saying *"pdfcer checked this run's font"* discloses that
the check happened; naming a cause the set cannot carry would be the
invented reason [`EditRefusal::RunCannotTake`]'s own docs argue against.

The clause order is [`EditRefusal::line`]'s, because
`app::status::disclosure::disclosure_line` truncates at 45 % of the bar:
the claim with the character first, the measurement second, the
reassurance third, the route last.
