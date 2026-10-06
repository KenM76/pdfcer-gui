---
name: push-only-with-release
description: Commits stay local; the main (coordinator) session decides releases and does every push, tag and GitHub publish
metadata:
  type: feedback
---

Commit locally; never push, tag or publish. Since 2026-10-05 Ken's standing rule is that releases go out whenever the main session judges them appropriate, and the main session does the push and the publish. This role gates, commits, packages and mirrors to OneDrive when asked, then hands over the exact commands (push main FIRST, then `gh release create`, see [[feedback_gh_release_create]]).

**Why:** a per-commit "push main" step once sent O272 S5 to origin unauthorized (2026-10-03); and on 2026-10-05 a release authorization relayed by the coordinator was correctly held, after which Ken moved push/publish to the main session.

**How to apply:** the routine ends at `git commit` (plus packaging if asked). After a release, update RESUME's "Start here" paragraph and the O-rows it ships.
