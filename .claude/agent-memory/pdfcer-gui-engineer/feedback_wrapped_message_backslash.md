---
name: a-wrapped-message-needs-its-backslash-as-you-type-it
description: A Rust string literal wrapped across source lines without a trailing backslash keeps the next line's indentation as interior spaces; it compiles, it survives every driven run because it lives in a FAILURE branch, and writing the lesson into a commit message did not stop it recurring 90 minutes later
metadata:
  type: feedback
---

Write the trailing `\` at the moment you wrap a string literal, not afterwards.
Without it Rust keeps the newline **and the next line's indentation** as
interior spaces, so the sentence reaches the reader as a word followed by a
dozen blanks.

**Why:** it compiles, and it cannot be caught by running the program. These
literals are the text a check prints when it **fails**, and a green run never
formats them — [[a-falsification-can-lie-in-both-directions]] covers why a
falsification does not reach them either: a plant fires one branch, and every
other failure message ships unexercised. `tools/gates/check-string-gaps.sh` is
the only instrument for the class, because it reads the source rather than the
behaviour.

**How to apply:** when the sentence is long, type it wrapped with `\` from the
start. Do **not** write it as one long line intending to let `cargo fmt` wrap it
— `fmt` does not touch string contents, so the long line survives, and the
padding arrives the first time anyone edits the wrap by hand. If a run of
spaces is genuinely wanted, the gate takes a same-line
`string-gap-exempt:` comment with a reason.

⚠ **It recurred within 90 minutes, which is the point.** The defect was found,
fixed and committed on 2026-09-20 at 10:42 with the mechanism spelled out in the
commit message — and the same defect was authored again before 12:00 in the
sibling check beside it. **A commit message is not an instrument.** The habit
has to change at the keystroke, and the gate has to run before the batch is
quoted clean; expecting the next session to have read the history is what
failed. [[a-register-row-outranks-memory-so-correcting-the-row-is-the-work]] is the same shape: the record is
where a claim is settled, never where a behaviour is enforced.
