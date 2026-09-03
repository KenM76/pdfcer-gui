---
name: a-long-green-check-can-be-aiming-at-nothing
description: When a driven check fails after an unrelated layout change, suspect the harness's aim before the application — a wrong aim that happens to hit reports green for as long as the coincidence holds
metadata:
  type: feedback
---

A `ui-verify` check that has been green for days is **not** evidence that
its aim is correct. It is evidence that its aim has been landing on
something.

**Why:** on 2026-08-27 `checks::ocr::click_region` was found converting a
dialog's `ui-rect` against `session.frame()` — the *application's* window
— where that dialog has been its own OS window since 2026-08-21. It was
missed in the bulk conversion to `driving::frame_of` and **passed for six
days**, because the button happened to sit where the stray click landed.
It failed the moment the new page-scope group pushed the button ~100 pt
down. My first reading of that failure was *"the Recognise button is
broken"*; the application was fine. The trace settled it —
`canvas-pointer screen=(34.0, 225.0)` was the **main window's** canvas
receiving a pointer event at the dialog's own coordinates.

**How to apply:** when a long-green driven check fails right after a
layout change that had nothing to do with it, ask *what did this check
sample* before *what is broken*. Two specific instruments:

- `grep -n "session.frame()?.declared_center" tools/ui-verify/` — any hit
  in a check that drives a dialog is a latent version of this defect.
  `frame_of` is safe on a main-window region, so converting pre-emptively
  costs nothing.
- Read the trace for a pointer event landing in the **wrong window**. A
  `canvas-pointer` line whose coordinates match a dialog rect is the
  signature.

The sibling finding from the same run: a check pointed at
`--pdf SW41177.pdf` failed with `NothingRecognised` and the application
was **right** — every page of that CAD sheet already has text, so the
doubling guard skipped all of it. A check whose subject is *"did X read
this input"* must pin its own fixture and ignore a suite-wide `--pdf`.

Related: [[a-measurement-of-the-wrong-surface-looks-exactly-like-a-broken-one]],
[[a-check-that-cannot-fail-is-not-evidence]].
