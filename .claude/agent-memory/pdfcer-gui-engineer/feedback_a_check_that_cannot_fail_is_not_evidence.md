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
