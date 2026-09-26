# `clock` — the one place this shell reads a wall clock

A module header out of proportion to its two functions — because *"what time is it"* is a question with more wrong answers
than it looks, and because the crate below this one has **deliberately
refused to answer it**.

## ★★★ Why the engine will not do this and the shell must

`pdfcer-core`'s `MarkupNote::modified` takes a PDF date string **from the
caller** and its own note says why:

> pdfcer will not supply the timestamp, and you should not expect it to.

Two reasons, both of which land on this side of the boundary rather than
disappearing:

1. **Determinism.** A library that reads a clock cannot be tested by
   comparing bytes, and `pdfcer-core`'s whole authoring test suite is byte
   comparison. One `SystemTime::now()` inside it would make every
   annotation fixture unrepeatable.
2. **Rule 4.** A timestamp is a *claim about when something happened*. If
   pdfcer invented one, the file would assert a fact pdfcer made up. Taking
   it from the caller makes the claim the caller's.

⇒ **The shell is the caller and the shell is a program a person is sitting
in front of.** *"When did I write this comment"* has a true answer here and
does not down there. So this module exists, and the obligation it inherits
is that the answer must be **true or absent** — never plausible.

## ★★ Why UTC, when the operator is in a time zone

Because §7.9.4 permits `Z` and this crate has no way to learn the local
offset without a dependency. The three options were:

| | |
|---|---|
| **UTC with `Z`** | correct, unambiguous, reads four hours ahead of his clock in the summer |
| local time with no offset | §7.9.4 permits it and it means *"unknown zone"* — a reader in another country cannot order two comments |
| local time labelled as UTC | **a lie in the file**, and the one option that is out of the question |

⇒ The third is the tempting one and is the reason this table is written
down: it produces the string that looks right to the person who typed the
comment, and it is wrong in a way nobody would ever notice until two
reviewers in two countries compared notes.

★ The day a timezone crate is worth adding, this function is the only place
that changes, and `Z` is a correct value it will be replacing rather than a
bug it will be fixing.

## ★ The calendar arithmetic is Hinnant's, not a guess

`days_from_civil`'s inverse — the standard days-to-civil algorithm, shifted
to a March-based year so the leap day falls at the end and the leap rule
needs no special case. It is exact for every date the `i64` range holds and
is the same algorithm C++20's `<chrono>` specifies.

It is written out rather than approximated because *"good enough for a
timestamp"* is how a file ends up claiming 30 February. The engine
explicitly does **not** check the calendar — *"accepting a caller's nonsense
date is their claim about their own document"* — so nothing downstream will
catch an error made here.
