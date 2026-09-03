---
name: refresh-features-md-before-every-release
description: Ken expects FEATURES.md re-measured against the build before each portable package, not just when convenient.
metadata:
  type: feedback
---

Refresh `FEATURES.md` against the **actual build** before running
`tools/package-portable.py`. Re-run the commands that produce every number in
it; do not edit a figure by hand.

**Why:** on 2026-08-19 Ken said, mid-session, *"i think the features.md file is
stale. please update for your next release."* He was right — it was four days
and a whole phase behind. It claimed 1,367 tests against 1,452, ten gates
against fourteen, ~128,000 lines against ~215,000, no panel count at all, and it
described Phase 6 as *"the substrate does not exist"* five days after eight
markup kinds started placing. Worse, five rows were **false claims** rather than
merely out of date — a blocker that had been removed, a control that had been
built, a dialog that had shipped.

The file is the thing he reads to know what he has. A stale one is not an
inconvenience; it is a report he cannot act on, and this project has now
corrected the same class of drift seven times.

**How to apply:** treat it as part of the release step, in this order —
re-measure, correct, commit, *then* package. Numbers come from
`cargo test`, `run-all.sh`, the registry tests and `ui-verify --list`, never
from the previous revision. When a row turns out to have been a *claim* rather
than a stale figure, correct it **in place with the date** and keep what it got
wrong — that record is what stops the next one. See
[[report-the-slot-after-packaging]] and [[always-publish-the-latest-build-to-onedrive]].
