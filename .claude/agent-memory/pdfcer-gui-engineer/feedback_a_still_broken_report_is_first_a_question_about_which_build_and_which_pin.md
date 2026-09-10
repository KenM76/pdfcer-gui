---
name: a-still-broken-report-is-first-a-question-about-which-build-and-which-pin
description: Four "still broken" reports in one sentence on 2026-09-09 — three were engine deliveries of the previous twelve hours this shell had not pinned or wired, one was a real engine gap; read the request channel and check the slot/pin BEFORE diagnosing, and measure on HIS files headlessly while he is at the PC.
metadata:
  type: feedback
---

**When Ken says something is "still" broken, the first three checks are: which
OneDrive slot is he running, what does the pin carry, and what has the engine
shipped since — in that order, before any code is read.**

**Why:** 2026-09-09, mid-morning, at the machine: *"still no way to edit the
size of a placed stamp, some text in sw41177 still isn't editable, we're back
to the apply redactions box that just tells me we can't do it, and I can't edit
or delete nodes on the freehand shape … why don't you just get rid of the code
that doesn't work instead of compiling with 2 day old bugs that were resolved."*
Measured on his own three sheets headlessly (no pointer taken):
- freehand nodes: the engine had shipped the verbs **overnight** in answer to
  our request; we had not pinned or wired them;
- SW41177 text: the shell's remedy panel existed; the engine's first-named
  face was a no-op — **fixed by the engine at 08:05**, after our own note;
- redaction: **one** of his sheets is an Excel-365 hybrid-reference PDF the
  writer refuses to rewrite in full — a real gap, filed; our sentence had said
  "cannot be rewritten in full", which he read as "can't do it";
- stamp resize: a real engine gap (third authoring family), filed; our
  sentence had been FALSE ("pdfcer did not draw it").

Three of four were answered on the engine side before he wrote. The engine now
ships within the hour of a request, so a report that reads as "you never fixed
it" is usually "the fix exists and is not in the build I run".

**How to apply:**
- Read `…/open/` for `reply_*` files newer than the pin's commit before
  investigating anything. `cargo update` first if the reply says SHIPPED.
- Say which slot has what, in the row and the reply: he cannot tell
  `pdfcer-gui1` from `pdfcer-gui2` by looking.
- Measure on HIS files with a `#[ignore]` probe under `target/scratch/` —
  `tests/ken_sw41177_probe.rs` is the template — rather than reasoning from
  fixtures. Two of the four answers came from that probe in three minutes.
- Where a sentence was false, fix the sentence the same session even if the
  capability must wait; his "get rid of the code that doesn't work" is R9,
  and a true refusal is the minimum it demands.

## ★★★ THERE IS A THIRD LOCATION, AND IT IS THE ONE HE ACTUALLY RUNS — 2026-09-09

`C:\Users\Ken\OneDrive\pdfcer\pdfcer-gui.exe` is his install. The rotation
script writes `pdfcer-gui1` and `pdfcer-gui2` and **never touches it**. So
"published to OneDrive" and "he has it" are different claims, and the gap is
not small: measured that evening after five releases in one day, his exe was
the **10:20** build — two releases behind. Everything from 10:57 and 22:51 was
absent from the program he was using while reporting on it.

**How to apply:** `ls -l` that exact path as part of triaging any report, and
put the measured date in `RESUME.md` so a cold session does not re-derive it.
Say it in the reply too — he cannot see it.

⚠ **Do not copy over it.** Three instances were live; the folder is his install
rather than a rotation slot, and overwriting a running exe is either a failed
write or a broken session. Tell him it is stale and let him take the update.
