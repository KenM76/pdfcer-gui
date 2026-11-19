---
name: parallel-pin-duplicates-row
description: A worktree feature consuming an engine reply can collide with a pin commit landed on main meanwhile — it files the same capability as a `wanted` row under the engine's key; grep the row title after rebasing.
metadata:
  type: feedback
---

After rebasing a worktree branch onto a pin commit that landed on main in parallel, grep ENGINE_BACKLOG.md for the capability's title. The pin commit may have filed the same engine row as `wanted`, keyed in the engine's own words. A shipped row written in the worktree under a different first cell then duplicates it. Neither git (a clean rebase) nor the gates (both keys look valid) report the duplicate.

**Why:** G303 consumed on branch `multi` (2026-11-19): the rebase onto 7e021d12 was clean, and two rows named "a multiSelect list reads the same everywhere", one wanted and one shipped. The worktree also failed 5 gates purely because main's pin move was not yet in the branch.

**How to apply:** when a worktree's gates fail on pin debt (api-drift, pin-citation, licences, engine-backlog), first check `git log main` for a pin commit and rebase onto it before redoing the pin work. Then keep one row under the engine's key, and rewrite the headings with `walk-engine-backlog.py --write`. See [[backlog-key-matches-engine]].
