---
name: a-stages-trace-records-what-the-stage-decided
description: A start-up trace line reports what that stage adopted, not what the frame settled on — a later stage can overrule it and the trace still reads green
metadata:
  type: feedback
---

**A stage's trace records what the stage decided, not what the frame settled
on.** Those are different claims wherever a later stage may overrule an earlier
one, and a harness grepping only the first will report a working feature.

**Why:** *"Open in the mode you were last in"* was dead from the day it shipped
and nobody saw it for ten days. `PdfcerApp::new` builds the `RibbonState` before
it calls `modes::start`, seeding it with the manifest's first mode (Read)
because an unset mode makes the shell show every tab. `start` then restored the
operator's remembered mode into `Modes`, the dock and the layout — but not into
the ribbon, which was already built. `docks` reconciles the pair once a frame
and treats the **ribbon** as authoritative, so the restoration lived exactly one
frame.

The trace said it was working throughout:

```text
mode-restore stored=Some("review") using=Some("review")     ← true of what assemble adopted
…forty lines later…
mode-changed from=Some("review") to=read remembered=true    ← and then it was undone
```

Neither line is a lie. The first answers a narrower question than the reader is
asking.

**How to apply:**

- When a trace asserts a start-up outcome, **read to the end of the trace before
  believing it.** The contradiction is usually dozens of lines further down and
  in a differently-named event.
- Prefer an oracle that reads the **settled** state over one that reads a
  stage's announcement. Here it was *which ribbon tabs published a rect*: before
  the fix `file, view`; after, `file, view, pages, markup, measure`.
- It bit markup hardest — markup is authored in **Review** and the program
  reopened in Read every time — so a whole day's work was one click from being
  invisible on every launch. **A defect in start-up state is a multiplier on
  everything built above it.**
- ⚠ The first regression test written for it asserted
  `ribbon.mode() == modes.active()` after `PdfcerApp::new()` and **stayed green
  with the fix deleted**, because `new` reads the real layout file and on a
  machine with no stored mode both sides are Read for a trivial reason. It was
  deleted rather than left to imply coverage — see
  [[a-check-that-cannot-fail-is-not-evidence]].

Sibling shape, same session: **an ordering argument written about statements is
not a claim about frames.** `destination::actions_for` orders zoom before scroll
for exactly the right reason, and a one-frame lag between deciding a zoom and
applying it defeats it anyway (`DEFECTS.md` D23).
