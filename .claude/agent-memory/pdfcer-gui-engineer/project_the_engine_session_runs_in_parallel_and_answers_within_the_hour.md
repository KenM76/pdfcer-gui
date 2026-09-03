---
name: the-engine-session-runs-in-parallel-and-answers-within-the-hour
description: A pdfcer session works D:\Dev\pdfcer concurrently, reads the request channel and answers within minutes — so file precisely, and expect the read-only tree to show uncommitted changes that are not yours
metadata:
  type: project
---

**`D:\Dev\pdfcer\` has a live session working it at the same time as this one.**
It reads `D:\Dev\FeatureRequests\pdfce_FeatureRequests\open/` and starts acting
on a request within minutes of it being filed.

**Why this matters, in three concrete ways:**

1. **`git status` in the read-only tree will show modifications that are not
   yours.** Observed 2026-08-18: two requests were filed at ~19:30 and by ~19:35
   `docs/{FEATURES,ROADMAP,ARCHITECTURE,SESSION_LOG}.md` were dirty in
   `D:\Dev\pdfcer`, documenting the very gaps just reported. **Do not read that as
   a violation of the read-only rule.** Check `git log` — the engine's HEAD moves
   too. On that day it went `6af5655` → `860ddcc` → `bea3cb1` → `0eff831` inside
   about two hours.

2. **File requests as if they will be read immediately, because they will be.**
   The channel README's *"what you tried to call, what you expected, what
   happened, with `file:line`"* is not a formality here — a vague request gets a
   round trip inside the same working session, which costs both sides. The two
   requests filed on 2026-08-18 named the exact `edit.rs` line numbers of every
   verb that *does* exist, which is why the reply could be about the decision
   rather than about the facts.

3. **Their reading of your request is worth checking against your own.** The
   engine session found `Group::unit` was the same hole as the name — fixed at
   `Group::new`, read-only afterwards — **and the request had not named it.** It
   was appended as a same-day addendum. Expect that: they read the model you are
   describing, and they read it more closely.

**How to apply:**

- Run `cargo update -p pdfcer-core -p pdfcer-render -p pdfcer-print` at the *start*
  of a session as well as before a build (see
  [[update-engine-before-every-build]]) — the pin can be hours stale by the time
  you get to packaging.
- Before assuming an engine verb is missing, check `git log` in the engine tree
  as well as grepping `edit.rs`. A verb requested this session may have shipped
  since you looked.
- `INDEX.md` in the channel is the memory and `open/` is the working set:
  **empty `open/` means nothing is owed.** Read it at session start.
