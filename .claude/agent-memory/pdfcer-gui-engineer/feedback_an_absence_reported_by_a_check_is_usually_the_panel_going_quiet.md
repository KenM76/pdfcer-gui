---
name: an-absence-reported-by-a-check-is-usually-the-panel-going-quiet
description: Before believing "the surface did not update", ask whether it was still DRAWING. A dock renders only its active tab, so a panel behind a sibling stops tracing — and a whole-capture search then returns a fossil from hundreds of frames earlier.
metadata:
  type: feedback
---

**When a driven check reports that a surface did not change, first ask whether
that surface was still on screen at all.**

## The evidence, 2026-09-05

Two checks reported, in the same words:

> *"THE COMMENTS PANEL DOES NOT SEE THE ANNOTATION THAT WAS JUST AUTHORED: it
> listed 12 before the drag and 12 after it. The engine traced `add-markup`, so
> the annotation IS on the session."*

It read as an application defect with **two independent witnesses**. It was
neither.

- A **persisted `userdata/layout.ron`** put another panel in front of Comments.
  A dock draws **only its active tab**, so the panel stopped emitting its census
  entirely — the last one in the whole capture was **three hundred frames before
  the drag.**
- Both checks read `Trace::last("comments-panel")`, which searches the **whole
  capture**. Both found the same fossil and subtracted it from itself.
- The canvas pop-up, reading the same session on the same frames, saw **13**.
  The session was never wrong.

★★★ **And the two witnesses were one witness copied.** Each check carried its own
*copy* of the same eight-line `fn listed(trace)`. A third check carried a third
copy and passed only because that run's layout happened to be favourable — it
was one dock arrangement away from printing the same false report.

## The rules

- **A whole-capture `last(...)` is a fossil finder.** Anchor every reading to a
  named cause that must precede it — the mode change, or the engine's own
  `add-markup` line. `Trace::last_after` / `Trace::mark` exist for this; it is
  the **third** recurrence of reading a fossil in that crate.
- **Distinguish "said nothing" from "said the wrong number."** Silence is a
  **SKIP** — the surface was not being drawn, so nothing was measured. Only a
  wrong number is a **FAIL**. Collapsing them turns a layout accident into a
  bug report.
- **If the surface has gone quiet, bring it back and re-read** — press its dock
  tab, or invoke its *show* command (a *show*, never a toggle, or you close what
  you were trying to open).
- ★★ **Two checks agreeing is not two witnesses if they share a helper — and a
  copied helper is a shared helper.** Before treating agreement as
  corroboration, check whether the agreement is one function pasted twice.
- **Assert a shape, not a magnitude.** "A number went up" passes on the wrong
  number going up. For a canvas-drawn rectangle: exactly one more row, **and**
  no more notes, **and** no more authors — which is the shape of that gesture
  and of nothing else.

## Related, and it is the same family

[[feedback_a_driven_failure_is_a_claim_about_the_check_too]] — this is its
sharpest instance: **three checks wrong, zero application defects.** Also
[[feedback_a_measurement_of_the_wrong_surface_looks_exactly_like_a_broken_one]].

⚠ And the enabling condition is worth its own line, because it has now caused
three separate false reports in one day: **the application persists its dock
layout, and the harness does not clear it.** A driven check that does not fire
`view.reset_layout` — *and assert the reset landed* — is measuring whatever the
previous run left behind.
