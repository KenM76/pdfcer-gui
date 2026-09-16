---
name: a-stages-trace-records-what-the-stage-decided
description: Two ways a trace lies about a frame: a stage line reports what that stage adopted rather than what the frame settled on, and a frame boundary you infer from the log shape rather than the draw order invents lag that is not there
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

## ★ SECOND, 2026-09-15 — the trace has no frame boundary, so I invented one and it was wrong

A trace is a flat stream. To count frames I picked a recurring line — the
canvas's — and treated the line after it as the end of a frame. It is the end of
the **canvas**, not of the frame, and everything drawn later in the same
`update` (dialogs, overlays, the status bar) then appears to belong to the next
frame. That cut put a ribbon command's line at the end of frame N and the dialog
it opens at the start of frame N+1, in **every** trace, consistently — which is
exactly what a real one-frame-late defect looks like. I filed it as D64, wrote
`RESUME` item 16 on it, prescribed a fix, and published a cross-project RAG
entry. All four were wrong. The ribbon is the *first* surface the update draws,
so its line OPENS the frame and the dialog draws in the same one.

⇒ **A consistent, clean, repeatable number derived from a log's SHAPE is not a
measurement of the program — it is a measurement of my cut.** The three greps
that disproved it (`fn ` boundaries in the file, the ribbon's step number, the
`dialogs.show` line number) were available before the first sentence was
written, and reading the draw order first would have made the wrong cut
impossible.

**How to apply:** before counting anything per-frame in a trace, **read the
update function's draw order** and use the first surface's first line as the
boundary — or emit a real marker at the top of `update` and stop inferring. And
when a derived number comes out suspiciously uniform across every sample,
suspect the derivation before the program: a constant is what an artefact looks
like. Related: [[a-value-cannot-identify-which-producer-made-it]],
[[an-oracle-built-from-the-system-under-test-needs-an-independent-calibration]],
[[a-count-command-can-be-wrong-not-just-its-quoted-answer]].

## ★ THIRD, 2026-09-15 — the harness filtered its input on the very property it was asserting

Same failure as the first section, but this time in the *instrument* rather than
the program. A check wanting *"what did this drag select?"* collected the
`canvas-text-selection` lines, **filtered them to the ones with `chars > 0`**,
and took `.last()`. A sweep whose pointer ran off the end of the text emits a
final `chars=0` line — which the filter discards — so the check read the last
state the gesture was **in** and scored it as the state the gesture **left**. It
then clicked a control that was correctly greyed and reported the application
broken.

⇒ **An instrument that filters its input on the property it is asserting cannot
observe the property being false.** The filter is the assertion, moved earlier,
where nothing can see it fail. Read the settled line first — the last line of
the event, unfiltered — and *then* test it.

⚠ **The corollary, and it is why this is not a blanket "always read settled":
presence and absence assertions want opposite readings.** A phase asserting
*"this gesture must select nothing"* keeps the filtered form deliberately, because
*"no non-empty line appeared at any point"* is a stronger claim than *"the last
line is empty"*. Written into `DEFECTS.md` D67 so the asymmetry is not tidied
into consistency later.
