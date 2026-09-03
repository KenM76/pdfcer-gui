---
name: kens-sentences-are-reports-not-measurements
description: Ken's descriptions locate a defect precisely — but a phrase of his must never be promoted to evidence for a mechanism.
metadata:
  type: feedback
---

Ken's bug reports are unusually precise and should always be believed and
chased. But **a sentence of his is a symptom, not a measurement**, and must
never be cited as confirmation that a particular mechanism is the cause.

**Why:** 2026-08-22 he wrote *"Up to 800% things work perfect. Over that…"*. It
was read as proof that 800 % is where the whole-page raster gives out and the
region tier takes over — a tidy confirmation of the diagnosis in hand. The
trace said the region tier actually engages at ~2,070 % on a Letter page. 800 %
was simply the application's **previous maximum zoom**: he was saying *"the
range I had already tested is fine; the new range is not."*

The diagnosis happened to be right. The evidence was worthless, and worse, it
stopped the search — the driven check was then tuned to a zoom where the defect
could not occur and reported PASS on a binary that had it.

**How to apply:** when a phrase of his appears to confirm a mechanism, go and
measure the boundary it names. If a number in his report matches a constant in
the code, check *which* constant — a round number is far more likely to be a
setting he remembers than a threshold he derived. His words are excellent at
saying **where to look**; the trace is the only thing that says **why**.
