---
name: push-only-with-release
description: Commits stay local; pushing main is pre-authorized only as part of an engine-triggered release, any other push needs Ken's OK
metadata:
  type: feedback
---

Commit locally; do not push main between releases. The next release pushes main as one of its steps (main before `gh release create`, see [[feedback_gh_release_create]]).

**Why:** Ken pre-authorized pushes only as part of an engine-triggered release ([[release-after-new-engine]]). A per-commit "push main" step had crept into the commit routine and O272 S5 went to origin without that authorization (coordinator correction, 2026-10-03).

**How to apply:** the routine ends at `git commit`. Push only inside a release, or when Ken says so directly.
