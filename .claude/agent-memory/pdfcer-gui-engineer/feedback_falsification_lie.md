---
name: a-falsification-can-lie-in-both-directions
description: A red falsification proves the check as a whole, not each assertion in it; a green one may mean the planting script's anchor rotted and it mutated nothing; and a script that scores the run by scanning cargo output can call every working plant a broken build; and a plant goes green when the fixture cannot produce the input its two spellings differ on; and a plant on a branch no fixture executes goes green too, which reads as a weak assertion and is a missing fixture; and a red plant proves a TWO-FIELD oracle only when it leaves the other field's assertion green; and a plant fires ONE failure branch, so every other failure message the check can emit ships unexercised
metadata:
  type: feedback
---

Two ways a falsification run misleads, and they point opposite directions.

## Red, but it proves less than it looks

**A falsification proves a check AS A WHOLE discriminates. It says nothing
about whether each assertion inside it can be reached.** After planting a
defect and watching the check go red, ask of every `if` in it: *what input
reaches this line?*

**Why:** O185's driven check had three assertions. The one its own module
header called load-bearing — the cross-run comparison that is the only thing
ruling out "Cancel and Keep are the same button" — **could never fire**, because
a per-run guard earlier in the file already implied it. Two falsification runs
had both gone red and neither exposed it: the other two assertions caught both
planted builds and reported them well. Being red is evidence about the check,
not about its parts.

**How to apply:** when a check has more than one assertion, falsify it once per
assertion, each plant aimed at only that one — or read the guards above them
and prove by hand which inputs survive to each line. And when an assertion turns
out to be unreachable, the two cases are not the same defect:

- **Unreachable because of ORDERING** — an earlier guard or an earlier
  assertion already implies it. That is a defect. Reorder so the strongest
  claim is tested first, where it can still fail.
- **Unreachable because the DOMAIN is currently too small** — the value it
  guards against has only two possible tokens today. That is a legitimate guard
  against the domain widening, and it must be **labelled in-comment as one**, or
  the next reader counts it as evidence the check does not actually provide.

Related: [[a-check-that-cannot-fail-is-not-evidence]],
[[an-assertion-both-outcomes-satisfy-is-not-a-measurement-of-which-one-shipped]],
[[a-measurement-of-the-wrong-surface-looks-exactly-like-a-broken-one]].


## Green, because nothing was actually planted

**A falsification script is code with a dependency on the shape of the thing it
mutates. When that shape changes, the script does not fail — it matches
nothing, plants nothing, and reports that the test is still green.**

**Why:** 2026-09-13. `f_o196.py` planted its defect by deleting the line
`options.units = remembered.units;`. Clippy then forced `seeded_options` into
functional-update syntax, so that line became `units: remembered.units,` inside
a struct literal. The script's anchor no longer existed. Re-running it would
have printed a green result for a test that was never actually challenged —
and a green falsification run is the strongest evidence this project accepts.

Two adjacent failures of the same family, same day:

- **An em dash in the prose makes a patch-script anchor fail.** The crate's
  comments use `—` (U+2014) freely; typing `-` looks identical in a terminal.
  Verify with `grep … | cat -A` (`M-bM-^@M-^T`) and then **choose an anchor that
  contains no em dash at all**, rather than trying to reproduce one.
- **`cd` inside a Bash tool call persists into later calls.** A walker run from
  the scratchpad reported "0 declared" and looked like a clean tree.

**How to apply:**

- Every patch and falsification script asserts its anchor count is exactly 1
  and exits non-zero otherwise. That single line converts all three failures
  above from silence into a message.
- Re-falsify after any refactor that touches the mutated function, not only
  after a change to the test.
- The one-line check that the falsification actually bit: the planted run must
  exit **101** (a Rust test panic), not 0 and not 1. A script that exits 0 on
  the "should be red" leg has not run the test.
- Restore from the `.bak` copy the script wrote, never with `git checkout`.

Related: [[the-write-python-to-a-file-workaround-does-not-protect-an-escape]],
[[a-check-that-cannot-fail-is-not-evidence]],
[[never-git-checkout-to-undo-an-experiment]],
[[a-measurement-of-the-wrong-surface-looks-exactly-like-a-broken-one]].

---

## Red, and scored by a script that read the wrong word

When a plant harness runs `cargo test` and decides the outcome by scanning
output, key on **`could not compile`**, never bare `error:` — and print the
**names** of the failing tests, not only how many.

**Why:** a red suite ends with `error: test failed, to rerun pass …`. That is
the test failing, not the build. Scoring seven plants with
`if "error:" in out: DID NOT COMPILE` reported six broken builds and one green
— when the truth was six precise reds and one genuine blind spot in the test
set. The misclassification is almost invisible in this particular instrument,
because a plant harness *expects* some deliberate defects not to type-check, so
"DID NOT COMPILE" reads as an ordinary result rather than a broken tool. The
count-only variant hides the neighbouring defect: a plant that reddens the
*wrong* test is itself a finding.

**How to apply:** any time falsification is scored by a script rather than by
eye. Same genus as [[a-command-judged-through-a-pipe-reports-the-pipes-exit-code]]
— an outer layer's success word standing in for the inner one's — and the
third direction this file is about: a misread harness is a falsification lying
in the direction that looks like diligence. Full recipe in `D:/dev/rag/rust/cargo_test_prints_error_on_a_red_suite_so_a_harness_keyed_on_error_reports_a_build_failure.md`.

---

## Green, because the mechanism you aimed at is IMPLIED by another — 2026-09-18

The third direction, and the only one where the green is a finding about the
**production code** rather than about the harness.

Seven plants against the dock's tear-out tests; **three green**, all three
aimed at the mechanisms that appear to enforce *only one affordance answers a
drag* — two stand-down guards and the stated branch order in the settlement.
All three are implied by the tear's own geometric predicate, so no input
reaches them and no test can redden them.

★ **Do not delete the implied mechanism.** The predicate is expected to move,
and a release build where the implication breaks should do the safe thing. Keep
it, label it in-comment as unreachable-under-today's-predicate, and move the
claim to an assertion where the decision is consumed — every driven test runs
through that.

★★ **Falsifying the replacement takes two plants and the first is expected
green**: widen the predicate and the guard absorbs it — *that* green is the
evidence the guard is live under the one input that reaches it — then widen it
and remove the guard, and the assertion fires by name.

★★★ **A tripwire behind an assertion that fires earlier is an unfalsified
tripwire.** Every test asserting the absence *before* the release failed there
instead, so the assert was never reached; it had to be falsified by deleting
that intermediate line in one test.

Related: [[a-check-that-cannot-fail-is-not-evidence]],
[[a-temporary-shim-needs-a-tripwire-that-names-its-own-deletion]].

---

## The scorer itself lies three more ways, and all three read CAUGHT-or-clean — 2026-09-18

Thirteen plants against two driven dock checks, scored by a shell runner that
greps the report for the defect sentence. Three of the thirteen were scored
wrongly before the runner was fixed.

- **The probe matched the check's own advertisement.** A `ui-verify` report
  prints a `detects:` line describing what the check is *for*, in the same
  words as the defect sentence. The probe `"release put"` matched that line, so
  it read **CAUGHT** while a completely different branch ran.
  ⇒ **Filter the report's self-description out before grepping it.** Anything a
  check prints unconditionally is not evidence about this run.
- **The probe was defeated by line wrapping.** A sentence long enough to wrap
  never matches a multi-word probe. Read **NOT CAUGHT** over a perfect catch.
  ⇒ Flatten newlines and squeeze whitespace before matching.
- **★ The plant silently did not apply, and the run reported PASS.** A python
  edit raised on a failed `assert`, the tree stayed clean, the check passed —
  and a *green that means nothing was planted* is indistinguishable from *green
  because the mechanism holds*. Caught only because the runner prints its own
  verdict per plant rather than relying on the suite's.
  ⇒ **A falsification runner must fail loudly when its own edit did not land** —
  `git diff --stat` after planting, or a non-zero exit from the editor. Same
  genus as [[a-commit-message-can-describe-work-that-never-landed]].

**How to apply:** a falsification score has three inputs — the plant applied,
the check ran, the probe matched — and all three can fail silently in the
direction that looks like success. Print all three per plant.


---

## Green, because the FIXTURE cannot produce the input the plant turns on — 2026-09-20

The fourth direction. The plant is real, the check is sound, and the two
spellings it separates **agree on every input this fixture can generate**.

Ask 5's travelling-copy gate reads *does the shape preview CONTAIN anything*.
The documented plant was the plausible wrong spelling, `is_none()`. It went
**green**, and the recipe in the check's own header had asserted it would go
red quoting a named trace field. The reason: `shapes::for_move_subject` answers
`None` for every text-line subject, so a chunk drag carries no preview at all
— and `None` satisfies both spellings. The empty-preview trap the plant was
written from is real one layer down, at a *different* function, on a subject
this fixture does not contain.

★ **A falsification recipe written from the design rather than run is a
claim, and it reads exactly like a measurement.** It had been written into the
check's doc comment, in the imperative, with the expected red output quoted.

**How to apply:**

- Run every plant in the list before shipping the list. A recipe nobody
  executed belongs in the same class as a quoted count nobody re-measured.
- When a plant goes green, the first question is *which input separates these
  two spellings, and can this fixture produce it?* If it cannot, say so in the
  doc, name the unit test that hands the predicate the input directly, and say
  what would have to become draggable before a driven row could reach it.
- Replace it with plants the fixture CAN separate. Here: delete the call site
  (absence), and blit only the first held chunk (count).

Related: [[an-unevidenced-excuse-is-worse-than-silence]],
[[a-check-that-cannot-fail-is-not-evidence]],
[[an-oracle-built-from-the-system-under-test-needs-an-independent-calibration]].

---

## Green, because the plant landed on a branch NO FIXTURE EXECUTES — 2026-09-20

The fifth direction, and the one that costs the most, because the green result
reads as a finding about the *assertion* and sends you to rewrite a correct one.

`check-ui-strings.sh`'s test-item skip has two entry branches — the attribute
and the item on one line, or on two. Falsifying "the skip actually starts" by
patching the same-line branch to `in_test = 0` left the self-test **green**.
That is the exact signature of six assertions that do not depend on the skip.
They do: the dirty fixture only ever contained the two-line shape, so the
patched line never ran. Patching the other branch reddened it immediately.

★ **A green falsification has two explanations demanding OPPOSITE responses —
the assertion is weak (strengthen it) or the fixture never arrives (extend it)
— and nothing in the output distinguishes them.** The first is the one that
comes to mind, because it is the reason falsification exists at all.

**How to apply:**

- Before concluding anything from a green plant, **prove the patched line
  executed**: `print "REACHED" > "/dev/stderr"` in awk, `panic!("reached")` in
  Rust. A panic cannot be swallowed by the same gap that swallowed the
  behaviour change, which a changed return value can.
- When the fixture turns out to be the gap, that is worth more than a red
  result would have been — it names a branch **no test has ever executed**.
  Here it produced a third fixture shape (`#[cfg(test)] mod x {` on one line)
  and two more assertions.
- A falsification measures the pair **(assertion, fixture)**, never the
  assertion alone. Say which one a green result indicted, in the commit.

Related: [[a-check-that-cannot-fail-is-not-evidence]],
[[a-check-whose-input-is-chosen-for-convenience-tests-the-assertion]].
Full write-up: `D:/dev/rag/rust/a_falsification_that_patches_an_unexercised_branch_passes_and_reads_as_a_weak_assertion.md`.

---

## Red, and it proves a TWO-FIELD oracle — only if each plant leaves the other half GREEN — 2026-09-20

The constructive case, and the one worth copying. A check whose oracle is two
fields is only two oracles if a plant exists that reddens one of them **while
the other stays satisfied**. If every plant reddens both, the second field is a
restatement of the first and the check is one assertion wearing two hats.

O217's plural-redaction row asserts a panel census rising by exactly one AND
the verb reporting two regions. Two plants, each aimed at one field:

- fold the outlines into an enclosing rectangle → regions read **1**, census
  still **+1**. The census half stayed green, and it had to.
- issue one annotation per outline → census reads **+2**, regions still **2**.
  The region half stayed green, and it had to.

★ **The green half is the evidence, not a gap.** Each plant is a build that a
one-field check would have passed, so the surviving assertion is the proof that
the other field is load-bearing. Design the plants to prove that, and say so in
the check's header — a recipe predicting BOTH halves red is describing a check
with one oracle in it.

★★ **Predict each plant's reading in the header and then correct it from the
run.** Mine predicted the split plant would report one region; it reported two,
because the count is taken from the outlines and the grouping happens later. A
wrong prediction in a falsification recipe is worse than none: the next reader
runs the plant, sees a number the header did not name, and concludes the plant
did not land.

**How to apply:** any check whose PASS rests on more than one field — and
prefer that shape whenever a single field is
[[an-assertion-both-outcomes-satisfy-is-not-a-measurement-of-which-one-shipped]].
Measure a CONTROL for each field in the same launch, or `quads=2` cannot be
distinguished from a build that always writes two.

## Two plants, four failure branches — the other two shipped unread — 2026-09-20

The plural-redaction check above was driven green twice and falsified twice, and
a source gate then found that one of its failure messages was mangled: two runs
of padding baked mid-sentence, because the branch was written late as one long
line and never had its continuation backslashes. It would have reached the
operator exactly when they most needed a readable sentence.

**Why neither falsification saw it.** A plant fires **one** branch. The union
plant tripped the region count, the split plant tripped the census, and the
`ratio > MAX_PLURAL_RATIO` branch — a *third* way for the same check to fail —
was executed by nothing. A passing run never reaches a failure message by
definition, and a falsifying run reaches only the one it aimed at. **The more
failure modes a check discriminates, the more of its own output is unexercised
by the exercise that proved it works.**

**How to apply.** After falsifying a check, **read every failure branch a plant
did not fire** — they are the branches with no evidence at all behind them. And
keep the source gates in the loop for this class: `check-string-gaps` caught it
by reading the literal, which is the only instrument that looks at a branch
nothing executes. ⚠ Do not read "the check was falsified" as "the check was
exercised" — [[a-check-that-cannot-fail-is-not-evidence]] is about the check
having a red path at all; this is about the paths it has and never took.

Related: [[an-assertion-both-outcomes-satisfy-is-not-a-measurement-of-which-one-shipped]],
[[an-oracle-built-from-the-system-under-test-needs-an-independent-calibration]], [[a-check-that-cannot-fail-is-not-evidence]].
