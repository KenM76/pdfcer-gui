---
name: feedback-gate-scans-worktrees
description: A gate rooted at the repo scans .claude/worktrees too; a stale worktree file made engine-citation SKIP for every run
metadata:
  type: feedback
---

A gate that runs `find`/`grep -r` from the repository root also reads
`.claude/worktrees/*`, the builder agents' trees. On 2026-11-21 one worktree
held a 1,502-line `.rs` file, so `check-engine-citation` declared its premise
broken and SKIPped every run; un-skipped, it read 3,024 citations, all from
worktree evidence files.

**Why:** worktrees hold old or in-flight copies; their files are not this tree's.
**How to apply:** when a root-scanning gate SKIPs or reports a surprising
count, check whether the paths are under `.claude/worktrees` first. Prune
`-path "*/.claude/worktrees"` and `--exclude-dir=worktrees`. See
[[feedback_skip_red_check_stop]].
