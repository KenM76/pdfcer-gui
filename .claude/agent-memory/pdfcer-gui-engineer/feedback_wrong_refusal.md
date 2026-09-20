---
name: a-wrong-refusal-sentence-hides-a-defect-in-whoever-believes-it
description: A refusal or staleness sentence is a claim about which code paths reach it - measure the paths, never the prose. Twice it hid a defect in whoever believed it, once in the shell and once in my own evidence doc
metadata:
  type: feedback
---

When a refusal's *stated reason* is wrong, it does not merely mislead — it
**suppresses the defect in the consumer that believes it**. Read every refusal
you route as a claim to check, not just as text to display.

**Why:** on 2026-09-08 the shell drew eight resize handles on a sticky note.
Dragging a corner produced a decline saying *"pdfcer did not draw it, so pdfcer
will not redraw it"*.

- That sentence is a fact about **the file** — foreign artwork — so it read as
  *"this particular document's sticky came from elsewhere"*, an unremarkable
  edge case.
- The truth is a fact about **the kind**: a `/Text` marker is fixed-size
  (`NoZoom/NoRotate`), so its `/Rect` says *where* and not *how big*, and a
  corner drag has no meaning for any sticky ever.

Had it said the second, the handles would have come off the moment anybody read
it. Instead they shipped for the life of the feature and Ken found them.

**How to apply:**

- A refusal you pass through is a **claim**. If it names a cause, ask whether
  that cause is true of *this* case — the engine fixed a sibling of this the
  same day, where the same sentence was flatly false.
- ★ **A refusal about the KIND should change what the surface offers; a refusal
  about the FILE should not.** Conflating them is how a control that can never
  work survives: it looks conditional.
- When a decline surprises you, probe it **with a control** — one thing that
  should refuse and one that should not. Here, a `/Square` accepting the same
  operation is what turned "this kind is limited" into "the engine does not
  recognise its own work".
**Second instance, 2026-09-16, and this time the believer was me.** The engine
writes a check box's rotation and reports success while never turning it. I
built `evidence/forms-parity/engine-form-verbs.md`'s verdict column from the
engine's own `appearance_stale` sentence - *"the stream is a push button's
caption artwork, a signature, or a form built elsewhere"* - and recorded
*"works for text/choice, discloses the rest"*. False in both directions: a push
button pdfcer drew never reaches that sentence, and a foreign check box that
does reach it is told it is a push button. The measurement that settled it was
the fork, not the prose: which `/FT` arm runs, what it reads, and what its
callee's signature can even accept.

★ **The sharpened rule: a sentence that enumerates cases is a claim about which
code paths reach it.** A staleness/refusal string sits behind a gate. Read the
gate and every arm that satisfies it. The sentence was written when the gate had
a different shape and nothing fails when they diverge - the string still
compiles, the tests still pass, and the enumeration is now fiction.

- Corollary: **a gate that cannot express the actual outcome will lie.** Here it
  is `if !appearance_regenerated` - which has no way to say *"rebuilt, but not
  rotated"*, so it says "fine".

- Sibling of [[a-backlog-row-is-a-record-not-evidence]] and
  [[a-limitation-sentence-is-a-citation-with-an-hours-long-shelf-life]].
  The family rule: **a sentence explaining an absence is evidence about its
  author, not about the world.**
