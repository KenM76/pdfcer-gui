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

## ★ PUSH BEFORE `gh release create` — 2026-09-09

`--target main` is resolved on the SERVER at creation time. With three
unpushed commits, the tag landed on the previous session's last commit while
the zip was built from the newest; `releases/latest` looked perfect
(`prerelease: false`, asset attached) and pointed at the wrong source. Order:
commit → `git push` → `gh release create`. If it has already happened:
`git tag -f <tag> <built commit>` and `git push -f origin refs/tags/<tag>`,
then read the tag's sha back from the API.

## ★★ A RELATIVE LINK IS ONLY SAFE IF ITS TARGET IS ALSO IN `PAYLOAD_DOCS` — 2026-09-13

`tools/package-portable.py` ships a fixed list into the zip:

```python
PAYLOAD_DOCS = ["MANUAL.md", "LICENSE", "THIRD_PARTY_LICENSES.md",
                "README.md", "FEATURES.md"]
```

So the **human** landing page travels with the download and the engineering
page does not. That is the right split — someone who unzips a portable build
wants the manual, not build instructions — but it means a relative
`[DEVELOPING.md](DEVELOPING.md)` in `README.md` is a **dead link for everyone
reading the README out of the download**, which is the larger audience, and it
is dead in the one situation where they cannot work around it: no internet, no
repo, just the folder.

**The rule:** a link inside a `PAYLOAD_DOCS` file may be relative **only** if
its target is also in `PAYLOAD_DOCS`. Otherwise spell it absolutely
(`https://github.com/KenM76/pdfcer-gui/blob/main/<file>`).

**Why it is easy to miss:** on GitHub both forms render and both resolve. The
defect exists only in the artifact, and nothing in the build looks at it. ⇒ The
check is *"read the README from inside the extracted zip"*, not *"read the
README"*. The same applies to any image or asset path.

This is machine-checkable and would be a cheap gate: for each `PAYLOAD_DOCS`
file, fail any markdown link whose target is a repo-relative path not itself in
`PAYLOAD_DOCS`.
