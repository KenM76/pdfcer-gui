---
name: an-unevidenced-excuse-is-worse-than-silence
description: A check or doc that explains an absence without measuring it reads as an answered question and stops anyone investigating — worse than saying nothing
metadata:
  type: feedback
---

When a check finds nothing, it must either **prove why** or **report the absence
bare**. Never write a plausible reason it did not measure.

**Why:** 2026-09-07. A driven check clicked blank paper every run and, on
absence, printed *"the point named an existing run, which is a fact about this
fixture rather than about the feature"* — having checked nothing. Neither branch
could fail. A completely dead feature and a badly-aimed click produced the
identical green line, and a fix that had shipped **two days earlier** was
believed broken the whole time, in four documents including Ken's own
`MANUAL.md`.

A missing check is a known gap someone will eventually close. An unevidenced
excuse is an **answered question** — an auditor reads the note, accepts it, and
moves on. That is why it is the worse of the two.

**How to apply:**
- Before writing *"which is a fact about the fixture/environment rather than the
  program"*, ask what trace line or measurement would prove it. Read that, or
  say only *"the event was absent"*.
- The evidenced version is usually cheap and already available: the same trace
  line that says `run=N` on one outcome says `origin=X,Y` on the other, so
  PASS / SKIP-quoting-N / FAIL are all readable.
- Applies to prose too. *"pdfcer-core cannot do X, so this shell cannot ask the
  question"* is the same shape and rots the same way — see
  [[a-backlog-row-is-a-record-not-evidence]].

Related: [[a-check-that-cannot-fail-is-not-evidence]],
[[a-skip-is-not-red-so-a-check-can-stop-running-unnoticed]],
[[an-absence-reported-by-a-check-is-usually-the-panel-going-quiet]].
