---
name: gh-release-create-tags-remotely-so-git-describe-goes-stale
description: gh release create makes the tag on GitHub only; a local repo that never fetches tags reports a stale last-release and every count derived from git describe inflates silently — I told Ken "51 commits unreleased" when the truth was 21
metadata:
  type: feedback
---

**`git fetch --tags origin` before quoting anything derived from `git
describe`.** `gh release create <tag>` creates the tag **on GitHub**, not in
the local repository. Nothing pulls it back. `git describe --tags` then keeps
naming whichever tag was last created *locally*, and every number computed
from it is too large by however many releases have been cut since.

**Why:** measured 2026-09-11. Answering Ken's *"new release soon?"* I ran

    git describe --tags --abbrev=0     -> v0.5.0-dev.20260909.1
    git log --oneline <that>..HEAD     -> 51

and told him **"51 commits have stacked up since Tuesday's build"**. Both
commands were right; the premise was not. `gh release list` showed three
newer releases, the newest from **yesterday afternoon**. After
`git fetch --tags origin`:

    git describe --tags --abbrev=0     -> v0.5.0-dev.20260910.2
    git log --oneline <that>..HEAD     -> 21

**21, not 51, and "yesterday" not "Tuesday".** A 2.4× overstatement of how
overdue the release was, delivered as the first line of a direct answer.

★ The shape is the project's oldest one wearing new clothes: **a measurement
whose instrument reads local state while the fact lives remotely.** It cannot
fail loudly — `describe` has a perfectly good answer, it is just answering a
question about a different repository than the one that publishes.

**How to apply:** any sentence containing "commits unreleased", "since the
last release", or a release date gets a `git fetch --tags origin` immediately
in front of it. Cross-check with `gh release list --limit 5` — if its newest
tag is not what `git describe` says, the local repo is behind. Same caution
applies to `PDFCER_RELEASE_DISTANCE` and anything else the build stamps from
`describe`. Related:
[[feedback_a_tool_that_mutates_the_tree_before_stamping_it_reports_its_own_dirt]],
[[feedback_a_verbatim_quotation_of_another_files_count_goes_stale_invisibly]].
