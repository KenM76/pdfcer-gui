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
