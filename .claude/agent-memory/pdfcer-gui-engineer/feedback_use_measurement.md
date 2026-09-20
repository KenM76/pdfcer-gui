---
name: a-use-measurement-must-be-scoped-the-way-the-language-scopes-it
description: "Is this name used?" answered by a bare-identifier search is discharged by any namesake anywhere — three successive cuts gave 73, then 9, then 0, where the truth was 3
metadata:
  type: feedback
---

**A declaration-and-use measurement has to be scoped the way the LANGUAGE
scopes it — by import — or it is measuring a different relationship and looks
clean while doing it.**

**Why:** 2026-09-13, finding which `REGION_*` constants nothing publishes.
A `pub` constant that no code uses is invisible to `dead_code`, to clippy at
`-D warnings`, and to every gate in `tools/gates/`. The symptom reaches a
reader as *a driven check reporting that a control plainly on screen does not
exist.* Three instruments were built and each was wrong in its own way:

| cut | answer | why it was wrong |
|---|---|---|
| first argument of a `ui_rect(` call in its own file | 73 of 159 | did not know `ui_rect_visible` exists, and a name is routinely handed to a local helper that publishes it |
| mentioned anywhere in its own file | 9 of 159 | declaration and publisher are frequently in **sibling modules** |
| mentioned anywhere in the workspace | 0 | ★ a **different module's healthy namesake** discharged three dead twins |

The honest number was **three, all in one file**. The third cut is the
dangerous one: it returns zero, which reads as a clean bill of health, and it
returns zero *for the real defects too*.

**How to apply:**

- A "is it used?" gate must resolve each candidate against the routes the
  compiler would: the declaring file itself, `module::NAME`, `super::`/`self::`,
  and a `use` statement naming both the name and its module. Flatten `use`
  statements first — `rustfmt` wraps them and a line-oriented regex misses the
  wrapped ones.
- Strip doc comments and the declaration line before searching, or the
  declaration discharges itself and a `///` mention discharges a dead name.
- Falsify against **the real pre-fix file**, not only a synthetic fixture. A
  self-test proves the parser; the real file proves the scoping.
- The tell that a cut is too wide: the count collapses toward zero when you
  loosen it. A correct widening removes false positives one at a time.

`tools/gates/check-region-names.py` is the instrument; its self-test case 5 is
"a healthy namesake does not discharge a dead twin" and case 6 is "a doc
mention is not a use", because those were the two failures.

Related: [[an-absence-claim-is-a-claim-about-every-route]],
[[a-substring-match-on-another-crates-prose-survives-a-narrowing]],
[[a-gate-keyed-on-a-name-is-discharged-by-prose]],
[[a-count-command-can-be-wrong-not-just-its-quoted-answer]].
