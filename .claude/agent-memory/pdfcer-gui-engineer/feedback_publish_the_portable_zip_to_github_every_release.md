---
name: publish-the-portable-zip-to-github-every-release
description: A release is not done until the portable zip is attached to a GitHub release AND mirrored to OneDrive — both, every time, without being asked
metadata:
  type: feedback
---

**Every release publishes the portable build to GitHub as a release asset, not
just to OneDrive.** Ken, 2026-09-06: *"You also need to update the portable
release on git. You should always try to do this."*

**Why:** the two destinations are not redundant, they fail differently. OneDrive
is what he runs — two alternating slots, `pdfcer-gui1` / `pdfcer-gui2`, so the
previous build survives beside the new one. GitHub is the **durable, addressable**
copy: it survives a OneDrive resync, it can be handed to someone else, and its
tag ties the binary to a commit. A build that exists only in a synced folder is
one sync conflict away from not existing.

He said it as a correction, which means at least one release had gone out
without it — so the failure mode is real and it is silent: OneDrive gets
updated, the work feels finished, and nothing red appears.

**How to apply:**

- The GitHub half is a `gh release create` with the packaged zip attached. The
  practice was already established when he said this (every `v0.5.0-dev.*`
  prerelease carries a `pdfcergui-<date>-<time>-<engine>-<shell>.zip`), so the
  rule is **do not skip it**, not *start doing it*.
- Both halves, same act. Do not treat "released" as done after the OneDrive
  mirror. See [[always-publish-the-latest-build-to-onedrive]] — the trigger is
  the same one: **finishing work is itself the trigger**; do not weigh the cost,
  he has.
- ⚠ **Package from a clean tree.** `v0.5.0-dev.20260906.3`'s asset is named
  `…-788dbb0-dirty-8cc9bc2859d2.zip`. The `dirty` token means the zip was built
  over uncommitted changes, so the tag names a commit the binary is not made of.
  Commit first, then package.
- Also refresh `FEATURES.md` against the build before packaging — see
  [[refresh-features-md-before-every-release]] — and smoke-launch the exe
  off-screen first, see [[smoke-launch-before-every-release-it-is-ninety-seconds]].
