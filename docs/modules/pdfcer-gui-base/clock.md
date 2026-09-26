# `clock` — the one place this shell reads a wall clock

A module header out of proportion to its two functions — because *"what time is it"* is a question with more wrong answers
than it looks, and because the crate below this one has **deliberately
refused to answer it**.

## Why the engine will not do this and the shell must

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

## Why UTC, when the operator is in a time zone

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

The day a timezone crate is worth adding, this function is the only place
that changes, and `Z` is a correct value it will be replacing rather than a
bug it will be fixing.

## The calendar arithmetic is Hinnant's, not a guess

`days_from_civil`'s inverse — the standard days-to-civil algorithm, shifted
to a March-based year so the leap day falls at the end and the leap rule
needs no special case. It is exact for every date the `i64` range holds and
is the same algorithm C++20's `<chrono>` specifies.

It is written out rather than approximated because *"good enough for a
timestamp"* is how a file ends up claiming 30 February. The engine
explicitly does **not** check the calendar — *"accepting a caller's nonsense
date is their claim about their own document"* — so nothing downstream will
catch an error made here.

## Item notes

### `fn format_pdf_date`

Split out for exactly that reason and for no other. A function that reads
the clock and formats it in one body is a function whose formatting can only
be tested by asserting on today's date — which is a test that passes for a
year and then starts failing at a month boundary for reasons nobody
remembers.

### `fn civil_from_days`

Howard Hinnant's `civil_from_days`, the algorithm C++20 `<chrono>`
specifies. The shift to a **March-based** year is the whole trick: with
March as month 1, the leap day lands on the last day of the year, so the
day-of-year formula is a single linear expression and the leap rule needs
no branch at all.

The magic numbers are the algorithm's own and are not tunable:
`719_468` is the day count from 0000-03-01 to 1970-01-01, `146_097` is the
days in a 400-year Gregorian cycle, and `153` and `2` are the coefficients
of the linear month-length pattern March..February. Changing any of them
does not make it approximate — it makes it wrong.

### `fn known_instants_format_exactly`

Four dates chosen for what each one would break: the epoch itself, a
leap day, the day after a leap day, and a century year that is **not** a
leap year. The last is the one a hand-rolled calendar gets wrong — 1900
and 2100 are divisible by four and are common years — and it is why the
algorithm is Hinnant's rather than `days / 365`.

### `fn the_engine_accepts_what_this_module_writes`

Not a re-implementation of `MarkupNote::validate` — the real one, called
on the real output. A format that drifted from what the engine parses
would be refused at author time with `MarkupDateMalformed`, and the
operator would meet it as *"my comment did not save"*.

⇒ This is the assertion that makes the two sides of the boundary agree
by test rather than by both files claiming to follow §7.9.4.

### `fn the_live_clock_is_shaped_like_a_pdf_date`

Deliberately weak — it asserts the SHAPE and a lower bound on the year,
never the value. A test that asserted today's date would be a test that
starts failing tomorrow, and the formatting itself is pinned above by
instants that do not move.
