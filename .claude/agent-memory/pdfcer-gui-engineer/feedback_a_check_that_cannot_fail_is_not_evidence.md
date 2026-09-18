---
name: a-check-that-cannot-fail-is-not-evidence
description: Before quoting a green driven check, prove it can go red — make it SKIP when it did not observe the mechanism it names, and take any baseline at the last instant before the action, not before the setup.
metadata:
  type: feedback
---

Every driven check must **refuse to PASS on a run in which it did not observe
the mechanism it is named after**, and must be falsified by re-introducing the
defect before its green is quoted to Ken.

**Why:** 2026-08-22, `panning_at_deep_zoom_stays_where_it_was_put` reported
PASS **twice** against a binary with the O24c placement defect deliberately put
back in. It zoomed to 1,867 %, the region tier engages at ~2,070 % on Letter, so
every trace line said `region=none` and the assertion had nothing to compare.
The check was being quoted as proof of a fix while incapable of failing.

Three ways a check silently stops measuring, all seen in that one hour:

- **It never reaches the tier / branch / state.** → assert the precondition and
  SKIP loudly if absent (`REGION_TIER_REQUIRED`).
- **The gesture is too small to trigger the mechanism.** A pan smaller than the
  raster grid step requests nothing. → repeat the gesture, sample between.
- **The transient has already healed.** Sample ~2 frames after the gesture, not
  after settling — and use a *separate later* reading for anything needing the
  gesture applied. One reading cannot do both; two frames reported a stuck view
  on a correct build on two runs of four.

**How to apply:** before writing "verified by driving" anywhere, run the
falsification: revert the fix, confirm RED, restore, confirm GREEN three times.
Restore the source immediately — a defective binary left in `target/release` is
one `package-portable.py` away from shipping. See
[[a-backlog-row-is-a-record-not-evidence]] and
[[a-measurement-of-the-wrong-surface-looks-exactly-like-a-broken-one]].

★★ **A test can also fail for the wrong reason and look like proof.** 2026-08-26:
a test asserting *"an empty models directory is refused"* passed against the very
resolver it was written to condemn — because the path it built was wrong, so
resolution failed for a reason unrelated to emptiness. Falsifying showed nothing,
because the test failed under both. **The fix is a positive control**: assert
that the same directory *with* the files in it succeeds. That second assertion is
what makes the first one mean "because it was empty" rather than "because the
path was wrong".

Generalised: whenever a test asserts *"X is rejected"*, ask what else would also
be rejected, and add the case that must be **accepted**.

## ★★★ A FALSIFICATION THAT READS REDNESS FROM THE EXIT CODE CAN BE SATISFIED BY SOMEONE ELSE'S COMPILE ERROR — 2026-09-04

With several tracks writing one tree, `cargo test` exits non-zero for reasons
that have nothing to do with your plant. A harness that plants a defect, runs
the suite, and concludes *"exit code 1, therefore my test caught it"* is
measuring the tree's health, not its own assertion.

It happened, and it cost two false confirmations before being noticed:

> Two plants first reported RED on exit code alone; the raw output showed the
> non-zero exit was a **concurrent track's compile error**, with no test having
> run at all.

⇒ **The verdict must be the test's own line**, not the process's status:

```bash
cargo test -p pdfcer-gui --lib the_test_name 2>&1 | grep -q "test result: FAILED"
```

★★ And the payoff is the reason to bother: re-run properly, **both plants came
back GREEN** — the two tests were genuinely weak and neither would have caught
its defect. One read a heading and body joined, so the heading could stop
carrying the fact; the other could have split on "exactly four spaces" and
passed everything. Both fixed. **The exit-code shortcut had been hiding two
real holes behind two false confirmations.**

★ Same family as the existing entries here: assert the *mechanism*, never a
proxy for it. And it composes with the plant-landed check —
[[feedback_a_backlog_row_is_a_record_not_evidence]] — so a falsification now
needs three things, all of them: **the plant matched, the test's own line said
FAILED, and the file was restored.**

## ★★★ FALSIFY ONCE PER CLAUSE, NOT ONCE PER CHECK — 2026-09-09

A check with two independent claims needs **two** falsifications, one per
claim, each changing exactly one constant — and the payoff is not that it goes
red, it is that the **two complaints are different sentences**.

`load_anomalies_reach_the_status_bar` asserts (1) the disclosure region is
present on a contradicting file and (2) absent on a clean one. Aimed the
presence constant at the clean fixture → *"…declares no
`status-group:load-anomalies` region, so the census line never drew"*. Aimed
the control at the contradicting fixture → *"The census line is on for files
that have nothing to disclose."* Two clauses, two messages, neither carrying
the other.

**If both falsifications produce the same sentence, one clause is wearing a
costume** and a future regression in either half will point at the wrong half.

**Why this is the moment to distrust it:** the check passed the **first time it
was ever run**. That is not reassurance — a check that has only ever been green
is indistinguishable from a check that asserts nothing. The trace-oracle green
failure modes are all first-run-green: a misspelled region so the "present"
branch was never taken, `.last()` on a change log returning a fossil, a clause
that is trivially true, a fixture already in the end state.

★★ **The control fixture is a claim too.** Assert it through the real engine on
every `cargo test`, and spell the filename **literally** if the constant you
would reuse resolves against a different corpus — `app::state::FOUR_PAGES`
points into the ENGINE's read-only tree, same basename, different bytes. A
constant that quietly names other bytes is how a control stops being a control
with every gate green.

**How to apply:** restore **from a copy in `$TMP`**, never by reverting through
git — a new check is usually untracked and a revert deletes rather than
restores. See [[never-git-checkout-to-undo-an-experiment]] and
[[an-or-between-two-required-conditions-asserts-neither]].

## ★★★ A CORRECT, WELL-EVIDENCED SKIP CAN MARK A PERMANENTLY INERT CHECK — 2026-09-12

The tell is a SKIP message that **concedes its own skip is the normal outcome.**

`print_clip_claim_follows_the_preview` skipped with
`clipped=Some(0) claim=none:0 overhang=fits` and the sentence *"that is the
expected result on most machines: the scale mode defaults to Fit, which does not
clip."* Everything about that message is honest — it measured, it named the
mechanism, it printed the numbers. And it means the check **has almost certainly
never run its assertion on any sweep since it was written**, because the
condition it needs is one the operator would have to have changed a setting to
produce.

This is the opposite failure from [[an-unevidenced-excuse-is-worse-than-silence]]:
there the absence was explained without being measured, so nobody investigated.
Here the absence **was** measured, the explanation is true, and still nobody
investigates — because a precise SKIP reads as diligence.

**How to apply: triage a SKIP by asking what would have to be true for the
assertion to run, and whether the check is in a position to MAKE it true.**

- *"the document lacks the feature"* → fixture defect, repair the input.
- *"the default setting does not produce the condition"* → **the check is inert**;
  it must drive the setting itself, not wait for it. Same family as
  [[a-fixture-that-defeats-a-default-does-not-defeat-a-starting-state]].
- *"expected on most machines"*, *"usually"*, *"on a typical setup"* — any
  hedge about the environment in a SKIP message is the mechanism announcing
  that it was written to be skipped.

⇒ A SKIP count is not a measure of unluckiness. Every SKIP is a check that did
not run, and some fraction of them are checks that **cannot** run. The two are
indistinguishable from the tally and distinguishable from the message, so the
messages have to be read one at a time — which is the work a clean
`passed=N failed=0` invites you to skip.

---

## ★★★ When the check asserts a CHAIN, one falsification proves ONE link

2026-09-15, O188(B). The new driven check reads three stages of one refusal:
the canvas traces the cause, the apply phase traces the sentence's stable
token on the far side of the `Action` boundary, the status bar publishes its
region a frame later. Falsifying it once — deleting the sentence — turns it
red, and proves only that stage two is wired.

⇒ **Plant a defect per link, and require a DIFFERENT failure message each
time, with the neighbouring notes still green.** That is what distinguishes a
check that discriminates from one that merely reddens. Restore from a byte
copy between plants, never through git.

★★ **And a presence assertion needs a pre-state control.** *The region drew*
means nothing unless the region was empty before the gesture — the shared
`status-group:decline` is satisfied equally by the right sentence, the wrong
one, and a stale one. **Assert the control, do not assume it**: SKIP if the
slot is already full, and say why (a decline that survived a command is its
own defect). This is the presence-shaped twin of the absence-baseline lesson
below.

Related: [[an-assertion-both-outcomes-satisfy-is-not-a-measurement-of-which-one-shipped]].

---

## A BASELINE TAKEN BEFORE THE SETUP ASSERTS ONLY THAT THE TOTAL GREW

**A check that counts events before an action and again after it is asserting
that the TOTAL grew.** Everything the setup emitted in between satisfies it —
and the setup is usually the richest source of the very event the check wants
to see absent.

### The evidence, 2026-09-05

`scrolling_far_keeps_the_canvas_its_pointer_input` moved the pointer (1
`canvas-pointer` line), turned the wheel, moved the pointer again, and required
`after > before`.

A defect was **planted** in `canvas::trace::pointer` — an early `return` past a
scroll offset of 700 pt, exactly the failure the check claims to detect. The
check **passed**: sixteen pointer lines had accumulated *while the wheel was
turning and the offset was still small*, so `16 > 1` held and the final move —
which produced nothing at all — was never isolated.

Corrected to compare against the count taken **immediately before the final
move**, it fails against the plant with the intended message, and passes on two
fixtures once the plant is reverted.

★★ **The plant is the only thing that found this.** The check had been repaired
that same hour and re-run and looked healthy; it was green for the wrong
reason.

### The same check's other fault, for the shape of it

Its original failure message asserted *"the page is still drawn and its rect is
still published — only the input is gone."* The trace of that very run ended:

```text
canvas-unavailable reason=nothing-visible
ui-rect-gone name=canvas-viewport
ui-rect-gone name=page
```

Forty wheel notches had scrolled a one-page document clean off the viewport.
**The message stated a precondition the run never measured.**

### The rules

- Take the baseline **at the last possible instant before the action under
  test**, after all setup has settled. Keep the pre-setup control as well and
  say what each is for — one separates *"the feature broke"* from *"this build
  never emits that line"*, the other makes the assertion about the action.
- **Never state a precondition in a failure message that the run did not
  measure.** If the message says "the page is still drawn", the check must have
  asked.
- **Plant the defect.** A check repaired and re-run is not a check verified; it
  is a check that agrees with the current build. This applies to every
  monotonic counter — event counts, file sizes, undo depths, render counts.
