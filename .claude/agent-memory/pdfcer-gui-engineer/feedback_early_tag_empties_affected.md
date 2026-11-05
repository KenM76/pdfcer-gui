---
name: early-tag-empties-affected
description: Releasing the APK before the drive moves the latest tag to HEAD, so `--affected` selects ~nothing; pass `--affected=<previous tag>`.
metadata:
  type: feedback
---

`sweep-full.sh --affected` diffs against the LATEST tag. Releasing the APK first
(Ken's "Android first") creates the new tag on HEAD before the drive, and the
sweep then picked 3 of 639 checks with "0 changed .rs files", reporting a
green TALLY over nothing.

**Why:** the selection's baseline is a moving ref; the early release moved it.

**How to apply:** whenever a tag was cut before the drive, run
`--affected=<previous release tag>`, and read the `affected-checks:` line's
count before trusting the tally. See [[release-sweep-skips-untouched]].
