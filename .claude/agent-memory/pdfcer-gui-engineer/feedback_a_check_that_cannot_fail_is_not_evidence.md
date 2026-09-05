---
name: a-check-that-cannot-fail-is-not-evidence
description: Before quoting a green driven check, prove it can go red — and make it SKIP when it did not observe the mechanism it names.
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
