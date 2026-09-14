---
name: an-injected-file-is-a-dated-snapshot
description: MEMORY.md as shown in my system prompt was 25.4KB and half its hooks did not exist on disk, where it is 23,958 — never patch a file from the copy in context, always re-read it first
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
