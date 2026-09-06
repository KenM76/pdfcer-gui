---
name: a-citation-that-supports-my-theory-may-be-about-a-different-document
description: Two wrong diagnoses of one defect in one day; the second had a real citation behind it that was true of a different file, and the design I proposed would have made things worse.
metadata:
  type: feedback
---

Before proposing a fix, **measure the mechanism on the file in front of you** —
not on the file the supporting comment was written about. A citation in this
codebase is evidence that something was once true somewhere, not that it is true
here.

**Why:** 2026-09-05/06, the operator's `clien` → `client` typo, diagnosed wrong
**twice in one day**.

1. *"`Identity-H` fonts cannot be written to."* Retracted after editing one of
   the very faces I had called unwritable.
2. *"Text extraction synthesises the spaces on his line, so the string holds
   characters no show operator wrote and no matcher can reach it."* This one had
   a **real citation** — a comment in `canvas/textedit/mod.rs` recording a
   title-block cell whose extraction produced twenty-one synthesised spaces. It
   is false on **his** file: 36 characters, 36 operators, spaces included, and
   the whole-run find matches perfectly.

The actual cause was one clause of the engine's new capability — **a pinned
request never spans show operators** — and the shell was sending a `find` *and*
a `pinned_span`, switching off the very matcher it needed. The fix was to drop
the pin.

★★ **And the design I proposed would have made it worse.** I suggested sending
only the changed span. On that page the changed span (`"n"` → `"nt"`) occurs
**33 times**; the whole run occurs **once**. Narrowing the find *widens* the
ambiguity — on a signed quotation, where the failure is silently correcting the
wrong word.

⇒ **A brief that names a cause invites the agent to confirm it.** Both times,
what saved the outcome was an agent measuring instead of building what I asked
for. Write briefs that name the *symptom and the instruments*, mark any
hypothesis as a hypothesis to falsify first, and say plainly that overturning it
is the most valuable thing they can report. Related:
[[when-two-things-differ-in-two-ways-the-measured-one-is-not-the-cause]],
[[a-limitation-sentence-is-a-citation-with-an-hours-long-shelf-life]].
