---
name: new-file-overwrote-tracked
description: A "new" module file written by heredoc/Write can silently replace a tracked file of the same name; check `git ls-files` for the path first
metadata:
  type: feedback
---

Before creating a "new" source file, run `git ls-files <path>` (or `test -e`).
A shell heredoc or a Write to an existing path overwrites it with no warning.

**Why:** 2026-10-23, the overprint-default text went to
`text/settings/overprint.rs`, which already held the zero-tint wording (123
lines). Only `git status` showing ` M` instead of `??`, plus clippy's
`duplicate_mod` / missing `zero_tint_*`, exposed it. Restored with
`git show HEAD:<path>`, the new text moved to `overprintdefault.rs`.

**How to apply:** a name that sounds like the feature (`overprint`, `colour`,
`print`) is the likeliest to be taken already. Read `git status --short` after
writing new files: any of them showing ` M` is an overwrite.
