---
name: a-measured-limit-belongs-to-a-revision-not-a-design
description: Never state a measured engine limit as a permanent property — date it, name the revision, and check it still holds before quoting it.
metadata:
  type: feedback
---

A limit you measured in `pdfcer` is a fact about **the revision you measured**.
Write it with the date and the commit, never as a property of the design — and
re-measure before repeating it.

**Why:** 2026-08-22 the shell measured that a molecule drawn at true scale
rendered near the page origin and was blank at page (540, 560): the renderer's
path coordinates were `f32`, whose step there is 21.5 nm against an atom's
0.34 nm. That went into `README.md` — **which ships inside every build** — as
*"Path coordinates are `f32`, whose step near the middle of a letter sheet is
about 11 nanometres."* The engine removed it the next day under an unrelated
request (`1d6db9e`), and the same probe rendered perfectly. A published claim
had to be retracted.

**The pair to watch for**, because this project has now made both in two days:

- a *sentence of Ken's* promoted to a measurement — see
  [[kens-sentences-are-reports-not-measurements]];
- a *measurement* promoted to a permanent property — this one.

Both stop the search early, and both feel like rigour at the time.

**How to apply:** when writing a limit into any artefact that outlives the
session — README, FEATURES, a RAG entry, a feature request — state it as
*"measured against `<commit>` on `<date>`"*. Before quoting an existing one,
re-run the probe; the engine session moves fast and answers within the hour, so
"I measured this yesterday" is not current. And keep the probe: the request that
found this was closed by re-running its own command unchanged, which took
thirty seconds and settled it beyond argument.
