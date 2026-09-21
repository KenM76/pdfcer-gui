---
name: git-status-is-not-a-content-oracle
description: A byte-identical rewrite still shows ` M` because the index caches size and mtime — status went 11 to 184 entries with nothing changed, and the obvious reflex would have discarded the commit
metadata:
  type: feedback
---

**`git status` reports stat, not content. `git diff` reports content.** A
script that rewrites a file with byte-identical bytes leaves it listed as
modified, and `git update-index --refresh` does not settle it — it prints
`needs update` for every one and exits 1.

**Why:** a bulk line-ending repair rewrote 180 files whose contents were
already what git stores. `git status --short` went from 11 entries to 184,
which reads exactly like a bulk corruption of the working tree. It was not:
`git diff --stat` never moved off the eleven real files, and
`git hash-object --path <f> <f>` matched `git ls-files -s <f>` on every one of
the rest. The index had simply cached each file's old size and mtime, and the
rewrite invalidated both.

**How to apply:**

- Never verify a mechanical sweep by asserting `git status` is unchanged before
  and after. It will fail on a perfectly correct run. The invariant to assert is
  that **`git diff --name-only` is unchanged**.
- When the hashes match, staging clears the noise and stages nothing. That is
  the repair, and it is free.
- ★ The danger is not the noise, it is the reflex the noise provokes. 173
  unexplained modified files invite `git checkout -- .` to "undo the mess" —
  which in this case would have thrown away an uncommitted file split. See
  [[never-git-checkout-to-undo-an-experiment]].
- Generalises past git: any instrument that answers a *cheaper* question than
  the one asked will answer confidently and wrongly. Before quoting one as an
  oracle, ask what it actually compares. Related:
  [[a-proxy-condition-survives-one-correction]], [[a-measurement-of-the-wrong-surface-looks-exactly-like-a-broken-one]].
