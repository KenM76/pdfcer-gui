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

---

## ★★★ Eighth instance, 2026-09-14 — **the first where the carried-over figure
licensed a CAPABILITY claim rather than merely counting something**

`FEATURES.md`'s thirty-third revision header, stamped **14:40 EDT** and shipped
to the operator six hours later, said the engine pin was

> one documentation commit behind the tip of its `main` **and level with it in
> every line of code**.

`42b47f30` had landed on the engine's `main` at **14:31** — nine minutes
earlier — and it touches `reflow_apply.rs`. The pin was two commits behind, not
one, and not level in any sense. The clause was never measured for that header:
it was carried over verbatim from `RESUME.md`'s `Engine HEAD` row, which had
been measured earlier the same afternoon and was correct when IT was written.

**What makes this one worse than the seven before it.** The previous instances
were counts — a test total, a gate total, a check roster — where a wrong number
is embarrassing and inert. This clause is the **licence** for every sentence in
the document of the form *“the engine cannot do X”*. It is the one line a reader
is entitled to lean on before believing a limitation, and it is the one that was
wrong. ⇒ **Rank a carried-over figure by what rests on it, not by how wrong it
is.** *“Level in every line of code”* was off by one commit of rustdoc and
nothing the build does differed — and it is still the most dangerous instance
so far, because of its position in the argument.

**How to apply:**

- **A clause that licenses other claims gets measured at the moment it is
  written, never carried.** The command is `git -C <engine> log --oneline -6
  main` and it takes two seconds; the temptation is that the row you are
  copying from was itself measured *today*.
- ⚠ **“Today” is not a freshness guarantee when the other repository is being
  worked in parallel.** The engine session commits several times an hour. A
  figure from ninety minutes ago is as stale as one from a week ago and feels
  fresh, which is worse.
- **Correct a published dated stamp by ANNOTATION, not by edit.** The header
  was in his hands; the correction is bracketed and separately dated beside it.
  A silently-right number teaches the next reader nothing about the decay.
