# `ui-verify/checks/redact_menu`

The canvas object menu's **Redact selection** row, and the bounds it asks for.

# Why this is a module and not a function inside one check

Two checks press this row, and they differ in exactly one thing: the
selection standing when they press it.

- `redacting_a_clicked_chunk_marks_only_that_chunk` presses it twice, on a
  whole block and then on one line, and compares the two boxes.
- `marking_two_chunks_makes_one_mark_and_two_regions` presses it on one line
  and then on a set of two, and compares the counts.

The gesture between those presses — right-click, confirm the OBJECT menu
resolved, find the row's published rect, click its centre, read the verb's
own trace line — is a hundred lines of refusal prose, every sentence of which
names a specific way the route can be broken and what to suspect first. A
second copy would be a second place for the route to drift, and the drift
would be invisible: both copies would keep passing, against different
programs.

# What a caller gets, and what it must still decide

[`mark_through_the_menu`] returns [`Marked`] — the **number of quads** the
verb built and their **union**. It asserts nothing about either. Whether one
quad is right or two are is the caller's subject, and the two callers want
opposite answers, so an assertion here would have to be satisfied by both
and would therefore measure neither.

⚠ **The union is not the quads.** The trace publishes one `bbox=` for the
whole request, so a check reading it can prove *how many* regions were built
and *what they span*, never that each one is the box of the thing that was
selected. The end-to-end proof is the apply report, which lists the text it
will destroy region by region.

# The three-way return, and why it is not two

`Err(..)` is a finding about the **run** — input disabled, a window that
never came up, a trace that cannot be read — and every caller reports it as
SKIPPED. `Ok(Err(..))` is a finding about the **program** and is a failure.
`Ok(Ok(..))` is a measurement. Collapsing the first two would make a broken
harness indistinguishable from a broken build, which is the reading this
project has been wrong about most often.

## Item notes

### `const CONTAINMENT_SLACK_PT`

The trace publishes one decimal place, and the boxes compared by callers are
computed from the same outlines, so this absorbs rounding and nothing else.
A real containment failure is tens of points, not tenths.
