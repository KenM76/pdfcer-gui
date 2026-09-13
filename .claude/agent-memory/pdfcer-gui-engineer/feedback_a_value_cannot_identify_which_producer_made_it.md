---
name: a-value-cannot-identify-which-producer-made-it
description: When several arms of a ranked chain can emit the same number, the trace of the number supports whichever theory you brought — trace the DECIDER, not only the value
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

See [[feedback_a_stages_trace_records_what_the_stage_decided]] and
[[feedback_when_two_things_differ_in_two_ways_the_measured_one_is_not_the_cause]].
Written up in full at `D:/dev/rag/egui/several_arms_of_a_ranked_chain_produce_the_same_value_so_the_trace_must_carry_the_SOURCE.md`.
