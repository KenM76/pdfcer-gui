---
name: never-mark-a-release-prerelease-it-hides-it-from-the-front-page
description: GitHub excludes pre-releases from "Latest", so a --prerelease dev build leaves the repo front page advertising an older zip; publish with --latest instead
metadata:
  type: feedback
---

**Do not publish a build with `--prerelease`.** Publish it as a normal release
and pass `--latest`.

**Why:** Ken, 2026-09-06: *"why is our portable release 3 days old on GitHub?
the portable should always be released with the latest version."*

He was right, and the cause was a flag I had been setting on every build.
GitHub **excludes pre-releases from `releases/latest`**, which is what the
repository's front page, the *Latest* badge, and any "download the latest
release" link resolve to. So five builds went out on one day, each with its
portable zip attached, and the front page kept advertising `v0.5.0` from
**three days earlier**.

⇒ **A release nobody can find from the front page has not been released.** The
zip existed, the tag existed, `gh release list` showed it — and the one surface
an operator actually looks at showed something else. Every check I had been
making was of the *list*, never of `releases/latest`.

**How to apply:**

- `gh release create <tag> <zip> --title … --notes-file … --latest`
  — **not** `--prerelease`.
- **Verify the thing he sees**, not the thing you published:
  ```
  gh api repos/KenM76/pdfcer-gui/releases/latest -q '.tag_name, .published_at, (.assets[]|.name)'
  ```
  `gh release list` is not that check — it shows pre-releases happily, which is
  exactly why this went unnoticed.
- To repair one already published: `gh release edit <tag> --prerelease=false --latest`.
- These builds are what he **runs**. "Pre-release" was a label borrowed from a
  project shape this is not: there is no separate stable channel, and the dev
  build *is* the product.

See [[publish-the-portable-zip-to-github-every-release]] — the same act, and
this is the second failure mode found in it in one day (the first was packaging
from a dirty tree). Both were invisible in the step that did the publishing and
visible only from the operator's side.
