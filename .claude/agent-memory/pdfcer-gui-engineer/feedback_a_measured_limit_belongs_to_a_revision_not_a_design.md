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

## ★★★ THE COMMIT THAT FALSIFIES THE SENTENCE IS USUALLY THE ONE THAT MOVES THE CONSTANT — AND IT UPDATES THE TEST INSTEAD — 2026-09-12

Our own limit, not the engine's. `MAX_MAX_ZOOM_PERCENT` is `1e12` (a trillion
percent). Four sentences of its doc comment describe `1e11`, including
*"a hundred billion percent, which is the deepest zoom the page has been
confirmed to actually DRAW at"* and *"a trillion … does not put a page on
screen … drawn at 8.6×10^9×, not drawn at 1×10^10×"*.

**All four were falsified by the very commit that raised the constant**
(`390fcc40`, 2026-08-22). That commit removed the ceiling the comment's last
sentence asks for, measured a page drawn at 10^12 % on Ken's own file, deleted
the clamp — and **renamed and rewrote the unit test in the same diff while
leaving the prose eleven lines above the constant untouched.** Twenty-one days.

★★ **Why it survived: the wrong prose is the most authoritative-looking text in
the module.** A driven measurement to two significant figures, a unit
conversion, and a paragraph on what would have to change for the limit to move.
Everything that makes a comment trustworthy was present, and all of it was
evidence for a number that had already been superseded.

★ **The direction is the part to remember.** It was found while auditing a
public `README.md` capability claim, on the assumption that product copy
overstates and source comments are conservative. **The product copy was right
and the source comment was wrong.** Nobody audits in that direction, which is
why it lasted.

**How to apply:**

- A test assertion and a doc comment are **two records of one claim** and only
  one can go red. When a numeric constant changes, the comment above it is part
  of the change — treat an unchanged comment in a constant-changing diff the way
  you would treat an unchanged test.
- **It is machine-checkable and the gate is cheap:** a `pub const NAME: f32 =
  <literal>;` whose doc comment contains a numeric literal or a magnitude word
  (*thousand, million, billion, trillion*) must agree with the literal. One line
  of disagreement a script sees instantly and a reader never does, because the
  reader believes the prose.
- Run the README-audit question in **both** directions: *cite the row* for every
  product claim, and *cite the commit* for every source sentence stating a
  measured number. See
  [[a-capability-claim-in-product-copy-needs-the-same-citation-as-a-limitation-claim]]
  and [[a-limitation-sentence-is-a-citation-with-an-hours-long-shelf-life]].
