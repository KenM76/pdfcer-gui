# `pdfcer-gui-base/text/textedit`

## Item notes

### `fn the_multi_run_note_says_what_happens_rather_than_refusing`

Its ancestor asserted `s.contains("Click directly on the word")` — advice
that could not work, because the refusal was about the *line* and not
about where on it the operator clicked. The property that replaces it is
the one that matters now: the sentence must name **the consequence the
operator cannot otherwise see**, which is that the neighbouring pieces
will not move.

### `fn sharing_the_line_pins_the_neighbours`

If this reason ever reflowed, editing one cell of a SolidWorks parts
table would slide every cell after it sideways — content the operator did
not touch, moved by an edit that did not mention it. Asserted here rather
than only in `disposition`'s own tests because this module is where the
sentence promising it lives, and a sentence and a behaviour that disagree
is worse than either alone.

### `fn each_pinning_reason_explains_itself_differently`

A single generic sentence would be the cheaper implementation and would
be wrong for both cases: "right-aligned" is something the operator's
document is, and "rotated" is something they can see, and the remedy
differs.

### `fn every_reflow_cause_says_something_of_its_own`

The property the enum exists for. Before O127 the four shell-side causes
went to one channel and the four engine-side ones collapsed into nine
generic words — so the operator could press Reflow for four genuinely
different reasons and be told the same nothing. A duplicate here would
be that failure re-arriving with a type in front of it.

### `fn the_two_stale_plan_causes_name_the_remedy`

A refusal naming a cause with no route is half a sentence — the rule
`text::embed`'s blocker rows already follow — and *"save this file and
open it again"* is not guessable from *"cannot reflow"*.

⚠⚠ **Both of this test's subjects are UNREACHABLE at engine `025d703d`**
and it is kept anyway. Said plainly so it is not mistaken for coverage
of something an operator can meet: `PageSetChanged` has been unreachable
since the engine's `Pass 257.0`, and `PageAlreadyEdited` since `G015`.
The test guards the two SENTENCES, which are deliberately retained
against a future engine reinstating either guard by name — and a
retained sentence with no test is how a retained sentence rots. What it
is NOT is evidence that either refusal can be produced.

**The two have DIFFERENT instruments for that question, and saying
"the gate covers it" would be wrong about one of them.**

* `PageAlreadyEdited` is dead because of something in the ENGINE —
  nothing constructs `ReflowApplyError::PageEditedThisSession`. This
  side of the boundary cannot see that change happen, so it is watched
  by `tools/gates/check-unreachable-refusals`, which re-measures the
  engine source at the pinned revision on every commit.
* `PageSetChanged` is dead because of something in THIS crate — no arm
  of `app::actions::textstyle::reflow_refusal` produces it. That is a
  fact about a twelve-line function, and the instrument is the unit
  test beside it (`…::tests`, which asserts the `Unsupported` arm has
  not drifted back onto it). A gate is the wrong tool for a question
  `cargo test` already answers, and claiming one covers it would be an
  unevidenced excuse — which reads as an answered question, so nobody
  investigates.

### `fn the_enter_refusal_offers_a_keyboard_route_and_a_gesture_route`

The sentence has to carry the keyboard route as well as the gesture
route, because O127's brief is explicit that commit must not be reachable
only by mouse: an operator told *"use Add text"* and nothing else has
been given a way to place text and no way to finish the edit they are
already in.

### `fn the_point_text_disclosure_names_the_edge_it_wraps_at`

Rule 4's obligation, and the reason the sentence is longer than *"placed
as a block"*: the operator can see two lines of text and cannot see the
rectangle they were laid into, so the one fact they need is what decides
where a long line will break.

### `fn every_refusal_says_something`

The whole point of the module: the old shell's answer to the cross-run
case was no sentence at all.


It is tolerable here for the same one reason [`EditRefusal`]'s
`EVERY` gives: [`refusal`]'s own `match` is exhaustive, so a fifth
variant is a **compile error** in the catalog before it can be a silent
gap in this list. The list is a convenience over a closed set, not the
closure itself — which is why the compile error is the tripwire and this
paragraph is only the reminder to extend the array when it fires.

### `fn shares_the_line_note`

Until 2026-08-19 this sentence's ancestor was a *refusal*: a click whose
visual line was made of more than one show operator placed no caret at all,
and the sentence told the operator to *"click directly on the word you want
to change"* — advice that could not work, because the refusal was about the
**line**, not about where on it they clicked.

On a SolidWorks sheet — one show operator per table cell, one per title-block
field — that refused nearly every click. The operator reported the feature as
not working twice, weeks apart, and **he was right both times**.

The refusal is gone and **the disclosure is the half that was always
useful**. It says the same true thing in the same operator's terms — *a run
is not a thing anyone can see on a page; what they can see is that the line
is made of separate pieces* — and then says what pdfcer is going to do about
it instead of stopping.

Shown when the caret **lands**, not when the edit commits: rule 4's
*"announced before it is picked, not after"*, applied to a layout
consequence rather than to a geometric inference. The commit-time half is
[`pinned_tail_disclosure`], which says the same fact in the past tense.

### `fn pinned_tail_disclosure`

Two sentences and no more, because it shares the status row with everything
else and R128 forbids that row growing. The first says what happened; the
second says what to watch for.

### `fn reflow_after_edit`

The remedy is the sentence, not the refusal. `reflow_block` re-emits the
page's FIRST content stream and its commit sweep empties every other one,
so a page carrying a non-empty EXTRA stream is refused by name rather than
having the text in that stream silently deleted.

**Re-measured 2026-09-14; what stood here was two revisions out of
date.** It said `reflow_block` is planned against the **base** document and
that *"one typed character is enough to trip it"*. Engine `Pass 257.0`
(2026-09-06) moved the planner onto the session view and both clauses went
with it: an ordinary text EDIT no longer trips this, because that edit's own
sweep has already consolidated the page. Adding text does.

The guard is also structural rather than provenance-based, so it fires on
a page NOBODY edited if the producer split its content across streams —
`request_G015`, O198, measured on a sheet carrying eight. The sentence below
is then false about the cause and its remedy does not work. It is still the
engine's sentence and this shell will not invent a better one; see
`app::actions::textstyle::reflow` for why a second predicate here is refused.

It says **save and reopen**, in those words, because that is the whole of
what an operator has to do and it is not guessable from *"cannot reflow"*.
A refusal naming a cause with no remedy is a sentence that leaves somebody
trying things — the rule `text::embed`'s blocker rows already follow.

It does not apologise or call it a limitation. It is a correctness
property: the alternative to refusing is splicing base-relative byte offsets
into a stream that has moved, which corrupts the page silently.

### `fn reflow_unchanged`

A correct outcome that reads as a failure without a sentence: the
paragraph already fitted its box, so re-wrapping it changed nothing visible.
Silence here is indistinguishable from a command that did not work, which is
the shape this project keeps finding.

### `fn reflow_needs_caret`

**The three reflow refusals below are one design decision**: a
paragraph command whose operand is the caret has three ways to find no
operand, and each of them leaves the operator in a different place. Merging
them into one *"nothing to reflow"* would be shorter and would tell somebody
with the text tool armed but unclicked exactly nothing.

It names the tool by the word on its button — *Edit text* — because
"place the caret" is our language and not theirs.

### `fn reflow_needs_existing_text`

`Anchor::Origin` and `Anchor::Box` mean the operator clicked bare page:
they are composing text that is not on the page yet, so there is no
paragraph to re-wrap and there will not be one until they commit. The
sentence says that rather than implying they mis-clicked — they did not.

### `fn reflow_no_block`

The honest one, and the one most likely to be met on the drawings this
program is for. A CAD title block is isolated cells, not prose: pdfcer finds
no paragraph because there is none, and a re-wrap of a two-word cell would
be meaningless even if it ran.

It says what pdfcer concluded about the text rather than that something
failed, because nothing did.

### `enum ReflowRefusal`

# Why this enum exists, when five `&'static str` functions already did

Because the five functions were being written to **the wrong slot**, and
nothing in the type system could say so. `app::dispatch::text` and
`app::actions::textstyle` both called `crate::app::actions::record_note`,
which the status bar renders under **`⚑ About your last edit:`** — and
`app::status::decline`'s own header forbids exactly that for a decline:

> *"an operator who reads 'About your last edit' after a gesture that did
> nothing has been told a small lie confidently."*

The operator's report is the plain consequence. He pressed Reflow, the shell
declined every time, wrote a correct sentence into a slot that reads as a
footnote about something *earlier*, truncated it to 45 % of the bar, and he
reported *"I haven't seen the reflow option actually work with anything when
I press it."* **It was answering him. In the wrong voice, in the wrong
place, in the wrong tense.**

⇒ Routing every cause through one enum makes the channel a property of the
type: `Declined::Reflow` can only be shown by `decline::show`, which wears
`⊗` and means *nothing happened*. A sixth cause added tomorrow cannot pick
the wrong slot, because there is no longer a slot to pick.

# The two halves, and why both are here

### `fn line`

One function over the enum rather than one per variant, for
[`refusal`]'s reason: a variant added without a sentence is a compile
error rather than a control that declines silently.

The first four forward to the free functions that already existed and
are already tested, so no sentence is written twice. What changed for
them is the **channel**, not the words.

### `fn enter_cannot_split_existing_text`

The operator: *"can the enter key create new lines when we are editing or
creating text?"*

**Creating: yes, everywhere, as of this change.** A dragged box and a
clicked point both take a line break on Enter and commit on Ctrl+Enter.

**Editing text already on the page: no, and it is the FILE that says so.**
`EditSession::edit_text` replaces the string inside one show operator, and a
show operator cannot contain a line break — `\n` has no code in any of the
standard encodings, so the engine refuses it by name (`Refusal`,
`TargetAbsent`, character `'\n'`) rather than dropping it. A PDF has no
paragraph: each visible line is its own operator at its own absolute
position, so splitting a line in two is not an edit, it is authoring a
second line somewhere.

So this is a **decline with a route**, not an apology. It says what
cannot happen, why, and the two things that can: finish the edit, or place
new text in a box that wraps. Silence here — which is what the shell did
before, by quietly committing instead — is the founding defect class of this
project: the key was pressed, something else happened, and nothing said so.

### `fn point_text_became_a_block`

# Rule 4: an inference the operator cannot see owes an off-canvas report

A click has no extent, so a point text has no width to wrap against — which
is why dragging a box was the multi-line gesture in the first place. Once
Enter inserts a line break at a clicked caret, the commit needs a box
anyway, and this shell derives one: from the click across to the right edge
of the sheet, and down to the bottom.

That width is **not invented** — it is the page's own crop box, which is a
fact about the operator's document rather than a number this shell chose —
but it is still an inference, and the operator cannot see a rectangle that
is not drawn. What they *can* see is the consequence: a line long enough
will wrap at the sheet edge rather than running off it.

It is shown once, at the commit, and not while typing: the draft is still
a draft until then, and a sentence about how something will be placed is
noise until it has been placed.
