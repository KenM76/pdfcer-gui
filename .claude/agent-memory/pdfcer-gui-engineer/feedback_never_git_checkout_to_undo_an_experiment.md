---
name: never-git-checkout-to-undo-an-experiment
description: Undo a deliberate sabotage-and-restore with a file copy, never `git checkout --` — it discards every uncommitted edit in that file.
metadata:
  type: feedback
---

When temporarily sabotaging code to falsify a test, **keep a copy and restore
from the copy**. Never reach for `git checkout -- <file>`.

```bash
cp src/thing.rs /tmp/keep.rs      # before
# ...sabotage, run the test, see it fail...
cp /tmp/keep.rs src/thing.rs      # after
touch src/thing.rs                # mtime, or cargo won't rebuild
```

**Why:** this has now cost twice. 2026-08-24, `git checkout -- <4 files>` used
as *inspection* discarded about an hour of uncommitted work — `FitMode::Height`,
`fit_placement_offset` and two text catalogues, all retyped from scratch.
2026-08-26, the same reflex mid-falsification discarded a resolver change and
the test written to prove it; recovered only because it was one file and minutes
old.

The trap is that `git checkout --` **feels** scoped to the experiment. It is
scoped to *the file*, and the file usually also contains the work being proved.

**How to apply:** the moment you edit code you intend to revert, make the copy
first. And after restoring, `touch` — cargo compares mtimes, so a restored file
with an older timestamp leaves the sabotaged binary in place and produces a
confident failure against correct code (`D:/dev/rag/rust/`).

★ Commit before falsifying where you can. A falsification against committed work
has nothing to lose, which is the version of this rule that needs no discipline.

Related: [[a-check-that-cannot-fail-is-not-evidence]].
