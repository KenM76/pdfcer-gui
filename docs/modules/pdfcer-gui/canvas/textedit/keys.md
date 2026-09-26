# `canvas::textedit::keys` — what every key means inside a draft

## What this is


## Why it is its own file

R2. `textedit/mod.rs` reached 1,571 lines the day the selection landed, and
the seam was already drawn: everything else in that module is about *what a
draft is* and *where it came from*, and this is about *what happens when a
key goes down*. The old shell's 25,005-line `main.rs` is the argument, and
the rule that prevents it is to split at the seam rather than to raise the
limit.

## The four selection rules, and where each is enforced

| # | rule | enforced by |
|---|---|---|
| 1 | a selection is the range between the mark and the caret | [`caret::range`] |
| 2 | typing replaces it | the `Text` arm, via [`take_selection`] |
| 3 | Backspace and Delete remove it and nothing else | their two arms |
| 4 | any movement without Shift drops it | [`caret::moved`], called by every movement arm |

Rule 4 is the one that looks like a detail and is not: without it a
highlight stays on screen after the caret has walked out of it, and the next
keystroke deletes text the operator is no longer looking at.

## What is NOT here, named rather than left to be discovered

**Drag-select and double-click-to-select-a-word.** The draft is drawn in an
editor box in *screen* space by [`super::paint`], and hit-testing a pointer
into it needs that laid-out galley published where the click ladder can
reach it. Real work, not a line — and until it exists, a selection is made
with the keyboard only.
