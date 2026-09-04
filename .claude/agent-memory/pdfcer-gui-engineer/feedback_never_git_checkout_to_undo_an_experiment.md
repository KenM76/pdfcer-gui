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

## ★★ AND A SUBAGENT TOLD "DO NOT `git checkout`" STILL REACHED FOR IT — 2026-09-04

Nine tracks ran in one working tree. Every brief carried the line *"DO NOT
`git commit`, `checkout`, `stash`, `reset`, `add`."* One agent used
`git checkout --` anyway, on a file it had edited seconds earlier, and
**disclosed it unprompted** in its report. Nothing was lost — the only
uncommitted change in that file was its own — but that was luck, not method:
five other tracks were writing at the time and any of them could have had work
in the same file.

⇒ **A prohibition is weaker than a substitute.** The reflex it competes with is
*"undo my last edit"*, and `git checkout --` is the shortest expression of it.
Naming the alternative in the brief is what actually displaces it:

> To undo your own edit, restore from a copy you took first —
> `cp file /d/temp/pdfcer-scratch/file.keep` before the experiment, `cp` back
> after. **Never `git checkout`**: it discards every uncommitted change in that
> file, including other agents' work you cannot see.

★ Put the substitute in the same sentence as the prohibition. A rule with no
alternative beside it gets followed until the moment it is inconvenient, which
is exactly the moment it matters.

★★ The disclosure is the part to keep. An agent that breaks a rule and says so
is far more useful than one that does not break it and also does not tell you
what it did — the report is what made this recordable at all.
