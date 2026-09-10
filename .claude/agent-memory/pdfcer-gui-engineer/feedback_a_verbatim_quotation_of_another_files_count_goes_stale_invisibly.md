---
name: a-verbatim-quotation-of-another-files-count-goes-stale-invisibly
description: when a doc comment quotes another module's header verbatim, adding a finding there silently falsifies the quote here — the file that changed does not contain the number that went wrong
metadata:
  type: feedback
---

**A verbatim quotation of another file's prose carries that file's counts, and
goes stale the moment the other file changes.** This is the hardest kind of
count-drift to notice, because **the file that changed does not contain the
number that went wrong**, so nothing about the edit points at it.

**Why:** 2026-09-10. `app/status/anomalies.rs` quotes `app/status/notes.rs`'s
header verbatim, including *"the same nine decisions"* and *"a tenth counter"*.
Adding the tenth Render-notes finding to `notes.rs` corrected those numbers
*there* and silently falsified the copy in `anomalies.rs`. Every gate stayed
green. This project has now spent **seven** corrections on prose drifting from
a count, including one in the gate runner's own header.

**How to apply:**
- After changing any count in a module header, `grep -rn` a distinctive phrase
  from the sentence you edited across the whole tree — not just the file.
- When you *write* a verbatim quotation, mark it as one and say it will do this:
  the corrected block now carries a ⚠ paragraph naming itself a quotation that
  carries another file's count. That is the cheapest available tripwire, since
  no gate can key on it.
- Prefer a pointer (*"see `notes.rs`'s header for the ordering argument"*) over
  a copy whenever the quoted text contains a number.

Related: [[a-limitation-sentence-is-a-citation-with-an-hours-long-shelf-life]],
[[a-backlog-row-is-a-record-not-evidence]], [[cheap-to-read-is-not-no-need-to-reread]].
