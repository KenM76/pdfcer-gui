---
name: a-gate-hit-inside-the-repo-is-not-a-mandate-to-sweep-outside-it
description: A gate names one line in this repo; grepping the same token across a shared external folder rewrote 118 occurrences in 62 archived exchanges, including an old-name→new-name table's left column.
metadata:
  type: feedback
---

**When a gate flags an old name, fix the line it names. Do not grep the same
token across the shared request channel, `D:\Dev\pdfcer`, or anything else
outside this repo — and if you already have, check whether the file still
carries the OTHER old names before deciding the edit was an improvement.**

**Why:** 2026-09-11. `check-old-name-absent` flagged exactly one line —
`pdfce-cli` in a `HANDOFF.md` table I had just written. Correct hit: the crate
is `pdfcer-cli`. I then swept the same substitution across the whole request
channel and changed **129 occurrences in 70 files**, 118 of them in 62
**archived** exchanges the engine team wrote.

Two measurements said it was wrong, and both were available before the edit:

- **37 of the 62 still carried `pdfce-core` / `pdfce-gui`.** So the sweep did
  not rename those files — it left each one **half** renamed, which is worse
  than either consistent state. An old-named document is dated; a half-renamed
  one is just wrong.
- **The rename note's own table lost its left column.** The archived
  `2026-09-03-note_the_engine_is_becoming_pdfcer_…` exists to *date* the
  rename, and its row read ``| `pdfce-cli` binary | `pdfcer` (no dash) |`` — an
  **old-name → new-name mapping**. The sweep rewrote the left side into the
  right side, so the row mapped a name to itself. That is this project's
  standing defect: a corrected claim whose history is erased cannot be audited.

**How to apply:**

- **A dated archive is a record, not a codebase.** `pdfce-cli` inside a
  2026-08-18 exchange is *true about 2026-08-18*. Live documents (`INDEX.md`,
  `README.md`, anything in `open/`) describe the tool as it is now and do get
  the current name. That split is the whole rule.
- **The gate's scope is the gate's opinion about scope.** It stops at the repo
  boundary deliberately — the channel is shared, and the engine session reads
  and writes it. Extending a gate's reach by hand is a decision, not
  housekeeping.
- **The channel is not a git repo, so there is no undo.** The reversal was only
  possible because `mtime` recorded exactly which files I had touched — use
  that, never a hand-transcribed list, and `assert` the reversal count matches
  the original count before trusting it. Mine was 118 both ways.
- **Windows `find.exe` shadows POSIX `find` when Python shells out.** The
  reversal script died on `FIND: Parameter format not correct`; `os.stat` did
  the job. Bash-tool `find` is fine; `subprocess` `find` is not.
- Related: [[a-rename-can-blind-an-instrument-silently]] is the other half —
  that one is a rename making a check stop seeing; this one is a rename sweep
  seeing too much. And [[a-backlog-row-is-a-record-not-evidence]] is the same
  instinct in a different place: a record's job is to be old.
