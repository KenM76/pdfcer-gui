---
name: gate-reads-moving-upstream
description: A gate that reads a sibling repo's working tree loses the race against a 90-minute verified release; read it at the pin.
metadata:
  type: feedback
---

A gate comparing against another repository must read it **at the revision this build links**, not its working tree.

**Why:** on 2026-10-07 `check-engine-backlog` failed three `package-portable --verify` runs in a row. Each run takes about 90 minutes, and while it ran the engine session landed another answered G request, adding a `[x] core / [ ] gui` row. Every failure named a capability past the v0.80.0 pin, which the shell could not wire until a tag. The gate now reads `git show <Cargo.lock pin>:docs/FEATURES.md` and lists rows past the pin as a note. `ENGINE_BACKLOG_AT=worktree` gives the old behaviour.

**How to apply:** when a verified run fails on a gate that reads upstream, check `git -C /d/Dev/pdfcer log` for commits made during the run before blaming the tree. Any new gate that reads the engine gets the same pinned read. See [[engine-session-runs-parallel]].
