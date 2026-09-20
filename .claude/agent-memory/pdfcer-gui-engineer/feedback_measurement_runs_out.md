---
name: when-a-measurement-runs-out-read-a-better-instrument
description: A failing threshold at the extreme end of a range is usually the harness's resolution, not the application — never widen the tolerance to fix it.
metadata:
  type: feedback
---

When a driven check fails only at the extreme end of a range, ask first whether
the **measurement** still resolves the quantity. Do not widen the tolerance.

**Why:** 2026-08-22, climbing to 10¹² % zoom on Ken's request, a check failed
with *"moved 0.0000 pt, where 0.0000 is the tolerance"* against a build that was
holding the point perfectly. It read the page rect from the `f32` trace line; at
41,000,000 % that rect's magnitude is 2.5 × 10⁸, where `f32` spacing is 32, so
the derived page point resolved to 8 × 10⁻⁵ pt against a 3 × 10⁻⁵ tolerance. The
instrument had run out before the application did.

Widening the tolerance would have "fixed" it and hidden every real defect below
that zoom. The right move was to read the `f64` trace line that already carried
the same quantity.

Two distinct changes that look identical in a diff:

- **A floor on the tolerance**, applied only where the proportional one would be
  finer than any instrument can resolve — honest, and worth a documented
  constant.
- **A widening of the tolerance** at the depths where the proportional one is
  meaningful — that is loosening a threshold to fit an observation, and it is
  how a check stops being able to see the defect it was written for.

Same shape as the guard phrased "every notch advances" that fired on a perfect
saturating climb: the assertion described an idealisation, not the property.

**How to apply:** at the extremes, before believing a failure — what precision
does the reading have here, and is the tolerance below it? If a `PDFCER_DIAG`
line carries the value in `f64`, use that one. If none does, add it; see
[[a-check-that-cannot-fail-is-not-evidence]] and
[[a-measurement-of-the-wrong-surface-looks-exactly-like-a-broken-one]].
