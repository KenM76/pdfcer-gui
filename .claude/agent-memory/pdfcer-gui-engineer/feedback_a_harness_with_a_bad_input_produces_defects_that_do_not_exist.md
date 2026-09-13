---
name: a-harness-with-a-bad-input-produces-defects-that-do-not-exist
description: A driven check given a wrong coordinate does not error — it emits detailed, plausible, reproducible failure reports. Validate the harness's inputs as hard as its outputs.
metadata:
  type: feedback
---
<!-- old-name-exempt-file: quotes a trace argument that names the pre-rename path. -->

**Before believing a driven failure, check what the harness was ASKED to do.**

## The incident, 2026-09-04

A 153-check sweep was run with `--doc-point 1,300,400` against a **one-page**
fixture. `PAGE` is **0-based**, so `1` names a second page that does not exist.

That single wrong digit produced **six failure reports** — canvas loses the
pointer after a long scroll, resize commits nothing, rotate commits nothing,
shift does not constrain, multi-node move moves nothing, wheel-paging does not
page. Each named a real function, a real trace event and a real line number.

**I filed four of them as defects**, and wrote a paragraph claiming one of them
disproved an earlier theory about the pasteboard.

Isolated properly — *same fixture, same zoom*, page index `1` → `0` — every one
passes.

★ I also proposed a *second* wrong explanation on the way (the A1 fixture's 20 %
zoom making grips too small) and wrote it into the ledger as the likely cause.
It was plausible, it was consistent with every observation, and it was wrong.
**Two wrong diagnoses before the right one**, both written down confidently.

## Why this is worse than a harness that will not start

A harness that cannot start costs nothing. One given a bad coordinate produces
output that is detailed, plausible, **perfectly reproducible** — which reads as
*reliable* rather than as *broken* — and indistinguishable from a real finding
without re-deriving it from scratch.

⇒ **The cost of a false driven failure is a whole investigation, plus whatever
is built on top of it before anyone notices.**

## How it was caught, and how nearly it was not

Only by **shape**: four failures at once, all one gesture family. That finally
prompted the standing question — *what did the check SAMPLE?*

**Had it produced one failure instead of four, it would still be filed as a
defect today.**

## The root cause was a guard that could never fire

`doc_to_window` already refused a point on the wrong page, with the right
reasoning written above it. But every caller did:

```rust
CanvasMapping::from_trace(&trace, vocab, page, target.page)
//                                             ^^^^^^^^^^^
```

The mapping was told its page index **by the point it was about to check**.
`p.page != self.page_index` compared a number with itself.

Fixed by comparing against what the **application publishes** — the page it says
it is showing, on the same trace line as the rect. Two independent quantities.

★ A first fix — parsing the page count out of the PDF — was written, measured,
and **deleted**: it could not read either fixture confidently, so it was a guard
that never fired. Two weak mechanisms are worse than one that works.

## How to apply

- **Validate a harness's INPUTS as hard as its outputs.** Coordinates, page
  indices, region names, fixture paths: all can be wrong in ways that produce
  *output* rather than errors.
- **Several failures in one family are a question about the instrument first.**
  Real regressions cluster too, but the instrument is cheaper to check and is
  wrong more often.
- **Preconditions belong where preconditions are checked** — once, before
  anything is driven — not inside a conversion where each caller's error
  handling can turn a refusal into some other verdict.
- When retracting, **retract loudly and keep the wrong reasoning visible.** The
  useful record is not that it was fixed; it is which plausible explanations
  were believed on the way.

Related: [[feedback_a_proxy_condition_survives_one_correction]],
[[feedback_a_measurement_of_the_wrong_surface_looks_exactly_like_a_broken_one]],
[[feedback_a_check_that_cannot_fail_is_not_evidence]],
[[feedback_a_trace_grepping_check_passes_on_a_build_that_crashed]].
Full write-up:
`D:\dev\rag\egui\a_harness_given_a_bad_coordinate_does_not_fail_it_lies_fluently.md`

## ★★★ FOURTH INSTANCE — and this time the fabricated diagnosis named a mechanism that had ALREADY been repaired — 2026-09-13

`the_wheel_turns_pages_when_the_operator_asks_it_to` FAILed with:

> ★★★ THE WHEEL DID NOT TURN A PAGE. … The most likely cause is the one this
> check was written for: `OpenDoc::prefs` is a SNAPSHOT adopted when the
> Settings window is applied…

The document it was driving had **one page**. `open ok pages=1
path="fixtures/a1-titleblock.pdf"`. There was no page 2 to turn to. And the
preference it accused of being inert reached the status line on the very next
frame after the press — `wheel=scroll` before, `wheel=flip` after — so the live
push was working and the snapshot defect had been fixed some time ago.

★★ **A wrong-fixture SKIP announces itself; a wrong-fixture FAIL writes a
diagnosis.** The inherited fixture *opened fine*, so every precondition the
check thought to assert held. What it never asserted was that the document it
had been handed could exhibit the behaviour in the check's own name.

★ **The diagnosis was the most convincing thing in the report**, because it was
a real mechanism, correctly described, with the right module named — just
already repaired. A check that hypothesises a cause in its failure text will
keep publishing that hypothesis long after the cause is gone, and every reader
believes it, because the alternative is believing the check is wrong.

**How to apply:** a check's `const FIXTURE` is part of its assertion, not part
of its plumbing. Before trusting a driven FAIL, grep its trace for the one fact
that makes the behaviour *possible at all* — `pages=`, object count, selection
count, whether the panel is the active tab. And when a failure message names a
mechanism, **date the mechanism** before repeating it.
