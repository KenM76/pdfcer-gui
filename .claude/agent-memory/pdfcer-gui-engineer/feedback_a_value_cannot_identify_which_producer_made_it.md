---
name: a-value-cannot-identify-which-producer-made-it
description: A trace field is evidence only about the code that COMPUTED it — trace the decider, not the outcome, and take a count from inside the callee, never off the caller's own input
metadata:
  type: feedback
---

**When more than one producer can emit a value, the value cannot tell you which
one did. Put the producer's name in the trace and read that instead.**

**Why:** 2026-09-13, the canvas placement regression. `canvas::offset::decide`
is an eight-arm ranked chain and the trace printed only `want=0.0,0.0`. At
least three of those arms produce exactly `(0,0)` — `deep` writes the literal,
`fit` reaches it through `strip_offset`'s lower clamp, `open-seed` lands on it
whenever the content is narrower than the viewport. I formed three hypotheses
and killed two of them by *source reading* before measuring anything; the real
culprit was a fourth arm nobody had suspected. Adding
`Decision { offset, source: &'static str }` and printing it as `src=` named the
arm on the first launch after it existed. **`(0.0, 0.0)` is the most
over-subscribed value in any offset subsystem: the origin, the default, the
clamp floor, and the never-assigned value all at once.**

**How to apply:** before writing or trusting a diagnostic line about a ranked
`if / else if` chain, count how many arms could emit the same bytes. If the
answer is more than one, the line is an *outcome* and not a measurement. Carry
the arm identity as a field on the returned type (so the compiler asks for it)
rather than a log inside each arm (which an arm can forget), give the
nobody-won case its own token, and publish it **before** any frame counter
advances. ★ The tell that you are already in this defect: **the number in the
trace is correct and the behaviour is wrong** — you are reading downstream of a
decision that has no name in the record.

★ A **control binary** that neutralises exactly one constant and re-launches is
the cheapest way to isolate: if only the `src=` field changes between the two
runs, the cause is the ranking and the whole of the geometry is excluded.

★★★ **The same rule, one step earlier: a count taken from the argument you
handed a function is true whether or not that function ran.** 2026-09-19, the
O215 chunk outlines: `paint(&boxes); trace!("drawn={}", boxes.len())` would
have stayed green over a blank canvas, and deleting the paint call was the plant
in my own falsification runbook — written as a step I expected to go red. Fix is
structural, not a better assertion: make the painter return a `#[must_use]`
tally taken **inside its own loop** and trace that. A loop that never runs then
reports `0`, and deleting the call becomes a compile error. Ask of every field
in a trace line: *which line of code computed this, and is that line on the path
the check claims to exercise?* Full write-up in
`D:/dev/rag/egui/a_traced_count_taken_from_the_callers_own_vector_is_satisfied_by_a_painter_that_drew_nothing.md`,
which also states the boundary a painter-side tally still cannot see — a
transparent stroke, a page-coloured one, an off-screen mapping.

See [[feedback_a_stages_trace_records_what_the_stage_decided]] and
[[feedback_when_two_things_differ_in_two_ways_the_measured_one_is_not_the_cause]].
Written up in full at `D:/dev/rag/egui/several_arms_of_a_ranked_chain_produce_the_same_value_so_the_trace_must_carry_the_SOURCE.md`.
