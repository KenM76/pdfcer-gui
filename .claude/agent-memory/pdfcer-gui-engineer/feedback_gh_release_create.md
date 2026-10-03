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
[[a-tool-that-mutates-the-tree-before-stamping-it-reports-its-own-dirt]],
[[a-verbatim-quotation-of-another-files-count-goes-stale-invisibly]].

**Second trap, same command:** `--target` takes a branch or a **full** 40-char
SHA. An abbreviated SHA (`d0642d43`) fails with *HTTP 422 Release.
target_commitish is invalid* (measured 2026-09-28). Pass
`--target $(git rev-parse <short>)` to tag the commit the package was
actually built from, rather than whatever `main` is by then.

**`--target` takes a full SHA.** A short hash fails with HTTP 422 `target_commitish is invalid`; pass `$(git rev-parse HEAD)`.

**Third trap: `--target main` names the REMOTE main.** If the local commits
were never pushed, the tag lands on GitHub's older main while the zip holds the
newer build. Measured 2026-10-03: v0.5.0-dev.20261003.1 was tagged at 2620b10f
with 7 unpushed commits, so the zip was a5423376. Push main first, or pass the
full SHA (which also fails if that SHA is unpushed). Afterwards, check
`git rev-parse <tag>^{commit}` against the SHA in the zip name. To repair:
`git push origin main`, then `git tag -f` and `git push -f` the tag. The release
stays attached to the tag.
