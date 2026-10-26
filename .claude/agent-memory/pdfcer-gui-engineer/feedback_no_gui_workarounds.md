---
name: feedback-no-gui-workarounds
description: When a gap is the engine's, file the request and wait — never build a GUI workaround for it
metadata:
  type: feedback
---

When a defect or missing capability belongs to the engine, file the request
(and the O-row) and stop. Do not build a GUI-side workaround.

**Why:** On 2026-10-26 Ken reported that form fields without /AP don't draw
(auto-size.pdf). I dispatched an agent to build a display-only shadow via
`regenerate_appearances`. He said: "don't spend tokens making workarounds. I
can wait for the engine to be updated."

**How to apply:** This narrows [[feedback_never_ask_add_feature]]. "Just add
the feature" still applies to GUI features the engine already supports. For
an engine gap, write the request and the row, then move on. Do not brief an
agent to build "a workaround until the engine lands". Also, there is no way
to stop a dispatched agent mid-run, so check this before dispatching.
