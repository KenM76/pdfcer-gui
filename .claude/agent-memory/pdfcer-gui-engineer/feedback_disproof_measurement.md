---
name: a-disproof-is-a-measurement-too-and-the-dead-hypothesis-was-the-truth
description: Five explanations for a red were "disproved" one by one; the second was substantially right and was killed by a comparison against the wrong reference. Record what a disproof COMPARED AGAINST, and trace what the framework DELIVERED before theorising about what the code decided.
metadata:
  type: feedback
---

**A disproof is a measurement too. Before writing "❌ disproved", write what the number was compared against — and before any fifth hypothesis, trace what the FRAMEWORK delivered to your code, not what your code decided.**

**Why:** 2026-09-08, `resize_scales_a_shape` red. Hypothesis 2 — *"the selection is the whole page, so the grip is at the page corner"* — was recorded as disproved because *"the outline is 428 × 302 px, nowhere near the canvas."* That compared the outline to the **window**. The canvas viewport was 444 × 592 px: the outline filled its width, the SE grip sat 6 px from the viewport edge, and that edge was egui's invisible floating scrollbar band (10 px, `interact`ed after the content, press = scroll jump). Hypothesis 2 was the truth in substance and was buried under a confident disproof; three more hypotheses and most of a day followed. What ended it was not a sixth theory but a one-line trace of `response.drag_started()` per frame at the point the canvas reads egui — `started=0 … origin=1` — which said *"someone else took the press"* and pointed at the one layer none of the five had examined. Then fixing the bars exposed a second defect (7 px/frame creep) that the same instrument, plus the page-rect trace, measured in one run.

**How to apply:**
- A "disproved" row in a handoff must name the reference the disproving number was measured against (window? viewport? page?). If it cannot, the hypothesis is not dead.
- When a symptom is *"my code never ran"*, the first instrument goes at the boundary where the framework hands input to your code (`drag_started`, `clicked`, hover, focus), not inside your state machine. Five coherent stories about the machine were all downstream of a press the machine never received.
- After changing a layout constant (bar style, margin, allocation), re-read every trace that reports geometry over several frames and *walk the series* — the second defect was a monotone creep visible only across frames.
