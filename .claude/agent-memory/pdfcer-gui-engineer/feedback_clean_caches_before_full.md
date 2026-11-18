---
name: clean-caches-before-the-disk-is-full
description: Ken wants Rust build caches cleaned automatically BEFORE the drive gets low, not after a failure; run the free-space guard before every build
metadata:
  type: feedback
---

Keep the drive from getting near full: prune dead Rust caches automatically,
before builds, without being asked.

**Why:** 2026-11-17 a release packaging run failed at 2.6 GB free on D:; the only
pruner ran at the END of packaging, after the space was already gone. Ken: "You
should be always cleaning up your old rust cache files automatically before the
drive gets nearly full."

**How to apply:** `python tools/cache_prune.py --if-below 30` before any build,
test run or gate run started by hand; the packager and `run-all.sh` call it at
their start. Report a low disk only when the guard could not free enough
(something outside our caches). See [[disk-is-tight-and-target-grows-unbounded]].
