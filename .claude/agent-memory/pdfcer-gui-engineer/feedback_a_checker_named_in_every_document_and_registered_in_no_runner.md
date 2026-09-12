---
name: a-checker-named-in-every-document-and-registered-in-no-runner
description: A tool cited by RESUME, by the file it guards, and by its own siblings can still be in no runner — grep the runner, not the prose, before believing a rule is enforced
metadata:
  type: feedback
---

**Before believing a rule is enforced, grep the RUNNER for the tool's name — not
the documents that cite the tool.**

**Why:** 2026-09-11. `ENGINE_BACKLOG.md` is governed by two rules that both have
an executable checker: *re-walk the section headings, never increment them* (they
had gone wrong on seven separate occasions in four days, every one a hand edit)
and *a register row is capped at 1,200 characters*. `tools/walk-engine-backlog.py
--check` implements both. `RESUME.md` named it. All five of the register's own
heading comments named it. **`tools/gates/run-all.sh` named it nowhere**, so it
ran only when a human remembered to type it — and nobody did for a week.

The citation density is what made it invisible: reading any of those documents
leaves you certain the rule is mechanised. And the sibling file
`check-engine-backlog.sh`, in the same directory, already carried the sentence
*"A gate nobody runs is a gate that does not exist"* — written about itself,
acted on for itself, and its twin left out.

What it was hiding, found on the first registered run: **25 rows filed under
`wanted` whose own verdict cells opened `✅ WIRED`** — the section whose heading
says *"these are the rows to read if you are choosing what to build next"* read
**70** where the real gap was **48** — plus one row three times over the cap,
written that same morning.

**The mechanism is worth keeping separately from the incident:** the register's
verdict is *the section a row sits in*, which is the right design. But nothing
guarded the reverse direction, so a row could be marked wired **by the session
that wired it** and still sit under `wanted` forever, because moving it is a
separate act no instrument asked for. A count can be right about what it
measures and wrong about what a reader takes it for.

**★ It happened AGAIN the same day, eight hours later, and the second one is
the more instructive of the two.** `tools/package-portable.py` had carried a
`--self-test` since the day it was written. It asserts three things nothing else
can see: that a build folder name cannot be swallowed by pdfcer's own packager's
`pdfcer-*` glob, that the source digest is deterministic and moves on a renamed
file, and that the asset-copy loop works even though `PAYLOAD_ASSET_DIRS` is
empty so it never executes in a real package. **It was in no runner.** It was
reachable only by a session that already suspected something and typed the flag.

⇒ The shape is not "somebody forgot". A `--self-test` is written by the person
fixing a bug, in the file they are already in, at the moment the bug is fresh —
and registering it is a **different file's edit**, made after the satisfaction of
the fix has been collected. Two files or neither. That is the rule this
repository now holds, and both halves of it were violated by the same author on
the same day.

**How to apply:**
- When a doc says "checked by X", run `grep -rn X tools/gates/run-all.sh`.
  Absent ⇒ the rule is a convention, and say so in those words.
- A one-way check invites its own hole. Having built one direction, ask what the
  other direction would catch, and make that a *report* if legitimate
  exceptions exist — a gate that forces exceptions to move teaches re-baselining.
- **Writing a `--self-test` is half the work.** The other half is the line in
  `run-all.sh`, and it is not done until both are in the same commit.
- Related: [[a-check-that-cannot-fail-is-not-evidence]],
  [[a-runners-sentinel-is-a-claim-about-the-runner]],
  [[a-long-green-check-can-be-aiming-at-nothing]].
