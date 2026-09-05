---
name: a-driven-failure-is-a-claim-about-the-check-too
description: A [FAIL] or [SKIP] from ui-verify is not yet an application defect. Triage which side is wrong before writing a fix — a stale check's failures are confident and about the wrong subject.
metadata:
  type: feedback
---

**Before treating a driven result as an application defect, establish which side
is wrong.** A stale or blind check fails *confidently* and *about the wrong
subject*, which is the most expensive kind of wrong.

## The evidence, 2026-09-05

The first full run of all 175 driven checks produced **7 harness defects** — each
of which had made a check permanently unrunnable — and a list of six apparent
application defects. When the first of those six was investigated properly,
**all three of its symptoms were the check**:

- **The check was not hermetic.** Its four sections each relaunched the binary,
  and the application persists its dock layout to `userdata/layout.ron`. Section
  B inherited what section A had saved; the failure even crossed **whole runs**,
  so a fresh sweep began in the state a previous sweep had left. The headline
  number `docked=0` was **honest** — nothing was floating, because an earlier
  section had closed it.
- **The oracle was blind.** It asserted that some region carried a `viewport=`
  tag. Nothing published one and nothing could. **It could not have passed
  against a working build.**
- **The fixture could not exercise the subject.** The panel under test draws its
  measurable control only with ≥2 optional-content groups, and **no fixture in
  the repository has any** — all 26 grepped.

★★ Three independent reasons, and any one alone would have produced the same
red line.

## The tells

| tell | what it usually means |
|---|---|
| a check that **relaunches** the binary more than once | it inherits its own previous launch's persisted state — assert a reset, and **assert the reset landed** |
| a number that reads as failure (`0`, `false`, `moved=false`) | ask whether the precondition ever happened. `docked=0` after nothing floated is a correct answer |
| a retry that does not help | **the aim, not the wait.** One check called an 800-px miss "a harness timing question" for days; it was clicking a child-viewport dialog against the main window's coordinates |
| an assertion nothing in the codebase emits | grep for the key. If no site produces it, the check has never passed and never could |
| a check that fails on a subject the fixture lacks | grep the corpus for the feature before believing the verdict |

## How to apply

- **Read the trace, in launch order, before reading the source.** The 2026-09-05
  diagnosis came entirely from four traces read in sequence; the answer was in
  them plainly.
- **Ask what the check SAMPLED** before asking what is broken. That question is
  already recorded here twice for gates; it applies identically to driven checks.
- When you fix a check, **falsify the repair**: plant the defect it now claims to
  detect and require its own `[FAIL]`.
- ⚠ And do not over-correct: the same sweep found **real** application defects
  too — `edit.paste` withheld in Review while `edit.copy` is offered, found by
  two independent checks agreeing. *Triage* means deciding, not disbelieving.

★ One more from the same investigation, small and vicious: `egui-shell` pins egui
with `default-features = false` *"so this crate does not silently acquire
fonts"*, so **every galley in its tests measures 0 × 0** and a test written with
`ui.label("x").rect` fails against a perfectly working dock. Use
`allocate_space` there; keep label-based measurement in `pdfcer-gui`, which has
fonts.

Related: [[feedback_a_measurement_of_the_wrong_surface_looks_exactly_like_a_broken_one]],
[[feedback_a_harness_with_a_bad_input_produces_defects_that_do_not_exist]] —
this is the same lesson in the driven harness rather than in a gate, and the
2026-09-05 sweep is its largest single instance.
