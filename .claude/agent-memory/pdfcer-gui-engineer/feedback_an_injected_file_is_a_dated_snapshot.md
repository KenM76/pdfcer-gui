---
name: an-injected-file-is-a-dated-snapshot
description: Anything the harness injects — MEMORY.md, gitStatus, a size WARNING — was sampled when the session started: 13 of 26 in-context anchors were not on disk, and a 36 h stale warning nearly bought a 136-file rename
metadata:
  type: feedback
---

**Every file the harness puts in front of me — `MEMORY.md` in the system
prompt, the `gitStatus` block, a CLAUDE.md excerpt, a system-reminder — is a
snapshot with a timestamp I cannot see. Before editing or quoting any of them,
read the file on disk.**

**Why:** on 2026-09-13 I wrote a patch script for `MEMORY.md` whose twenty-six
find-and-replace pairs were lifted from the copy in my own system prompt. It
exited on the first pair with zero matches. Re-run tolerantly, **13 of 26 did
not exist on disk at all.**

The two copies disagreed on the file's *size* as well: the injected one carried
`WARNING: MEMORY.md is 25.4KB (limit: 24.4KB) … Only part of it was loaded`,
while the file on disk was **23,958 bytes** with
`tools/gates/check-memory-index.sh` reporting clean. The reason is simple once
seen — an earlier session had trimmed the hooks, and the injected copy predates
that trim. It is not corrupt, not a competing file, just **old**. (I checked:
`find /d/Dev/pdfcer-gui /c/Users/Ken/.claude -maxdepth 6 -name MEMORY.md`
returns exactly one file for this agent, plus unrelated per-project ones.)

The same thing bit me twice more the same day: the `gitStatus` block in the
session context showed HEAD at `b6d7cfd` and a **clean** tree when the real
state was HEAD `a1b5752` with eight dirty paths.

**How to apply:**

- Never build a patch, a `sed`, or a replacement pair out of text that came
  from context. `cat` the file first and copy the anchor from *that* output.
  This is the same discipline as
  [[a-verbatim-quotation-of-another-files-count-goes-stale-invisibly]], applied
  to my own input rather than to a document.
- Every patch script asserts a **unique** anchor and fails loudly on zero
  matches — but for a bulk pass, collect the misses and print them instead of
  exiting on the first, or you learn about one of thirteen.
- Never quote a byte count, a commit hash, a branch state or a "clean tree"
  from an injected block. `git status --porcelain`, `git rev-parse HEAD`,
  `wc -c` — they cost nothing.
- A size warning in an injected copy is about the injected copy. Measure the
  real one against the real cap (24,000 bytes for `MEMORY.md`, enforced by the
  gate, not by the loader's warning).

See also [[a-backlog-row-is-a-record-not-evidence]] — same shape, different
source: a written record standing in for a measurement.

## The size warning is a snapshot too, and it nearly bought a 136-file rename

**The `MEMORY.md` block injected into a session — and the size WARNING printed
under it — is a snapshot, not a measurement. Re-measure before acting on it.**

**Why:** on **2026-09-14** the injected block ended with

> `WARNING: MEMORY.md is 25.4KB (limit: 24.4KB) — index entries are too long.
>  Only part of it was loaded.`

read as a live fact, that says memories written in good faith are **not reaching
cold sessions right now**, which is urgent and structural. The plan it produced
was to rename all **136** memory files to short slugs — filenames are **8,942 of
the index's 23,987 bytes, 37 %**, by far the biggest lever — rewriting 309
wiki-links and the citations in `DOC_DRIFT.md` and `HANDOFF.md` to match.
That is an hour of work, an unreviewable diff, and it would have broken
greppability of the folder for good.

**It was false.** `wc -c` said **23,987 bytes = 24.0 KB**, under the 24.4 KB
limit, not truncated. `25,411` bytes is *exactly* the file at commit `dc5c0c3`,
**2026-09-13 03:57 — thirty-six hours earlier**. The injected copy proved its own
age without needing git: its row 3 still read *"means sweep the whole suite,
reversibly"*, wording that had been shortened to *"means sweep it all"* hours
before the session started.

★ **And the same prompt carried a second stale block, from a different day.**
Its `gitStatus` named `b6d7cfd` as `HEAD` with a list of "recent commits" —
**2026-09-09, five days and ~90 commits old.** Two independent snapshots, two
different ages, both presented in the present tense. ⇒ **Nothing in the system
prompt that describes the repository is a measurement of the repository.** The
prompt is assembled once and carried; the repo moves every commit.

**How to apply:** any instruction-block number about this project's own state —
memory size, `HEAD`, a commit list, a file count — is a *dated citation with no
date printed*. Measure it before it becomes a premise:

```bash
wc -c .claude/agent-memory/<agent>/MEMORY.md     # the live size
bash tools/gates/check-memory-index.sh           # the live instrument
git log --oneline -3                             # the live HEAD
```

☆ **The calibration won on the way, which makes the gate's constant exact.**
`25,411` bytes → `"25.4KB"` proves the harness's KB is **bytes ÷ 1000** and that
it measures *this file*, nothing more. So the ceiling is **24,400 bytes**, and
`check-memory-index.sh`'s `MAX_INDEX = 24000` is correct with 400 bytes of
deliberate margin. That upgrades a cross-system citation the gate itself flagged
as *"unknown shelf life"* into an arithmetic conversion — which is the only kind
of borrowed constant that does not rot.

⚠ **The other half: hook-shortening is now measurably spent.** The same session
rewrote the **36 fattest rows** with intent and freed **506 bytes — 14 bytes per
row**. The index is titles 6,914 / filenames 8,942 / hooks 6,900 / markup 1,080,
so the hook is the *smallest* third and the least compressible. The gate's advice
*"shorten hooks; the detail lives in the topic files"* has no room left in it;
the next lever is the filename, and the one after that is consolidating two
entries of the same shape. Write that down before spending an hour rediscovering
it — and **do not spend the hour before re-measuring whether the ceiling is
actually being hit.**

Related: [[a-verbatim-quotation-of-another-files-count-goes-stale-invisibly]],
[[a-limitation-sentence-is-a-citation-with-an-hours-long-shelf-life]],
[[a-backlog-row-is-a-record-not-evidence]],
[[a-measured-limit-belongs-to-a-revision-not-a-design]],
[[kens-sentences-are-reports-not-measurements]].
