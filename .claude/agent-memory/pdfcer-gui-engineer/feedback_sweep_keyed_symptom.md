---
name: a-sweep-keyed-on-the-symptom-collects-the-healthy-too
description: Key a repair on the rule that produces the correct state, not on the state you noticed on the broken files — six correct files were "repaired" because they shared the symptom
metadata:
  type: feedback
---

**A repair sweep's predicate must come from the rule that generates the correct
state, never from the state observed on the damaged ones.** The damaged files
share their symptom with healthy files, and the sweep cannot tell them apart.

**Why:** patch scripts had silently converted source files to CRLF. The symptom
was `git ls-files --eol` reporting `i/lf w/crlf`, so the repair was keyed on
that pairing. It collected 181 files. But `core.autocrlf` is **true** on this
machine and `.gitattributes` opens with `* text=auto`, so for any file carrying
no `eol` override, `i/lf w/crlf` is the state **checkout itself produces** — the
healthy state. Six files were converted away from the form a fresh clone makes,
for having a symptom that was never a symptom for them. The correct predicate
was one command away: `git check-attr eol -- <path>` names what checkout
*should* put on disk, which is the only thing "correct" can mean here.

**How to apply:**

- Before writing the sweep, write the sentence *"a file is wrong when it differs
  from X"* and find the command that computes X. If the predicate only describes
  what the broken ones look like, it will hit the controls.
- Two commands that look interchangeable usually are not:
  `git ls-files --eol` reports **observed** state, `git check-attr` reports the
  **rule**. Same for any "what is it" / "what should it be" pair.
- A control that shares the symptom is the falsification to run first — pick one
  file you know is healthy and check the sweep declines it. See
  [[a-control-must-differ-in-exactly-one-property-and-an-absence-needs-a-witness]] and
  [[a-check-that-cannot-fail-is-not-evidence]].
- Companion finding from the same hour: [[git-status-is-not-a-content-oracle]].
