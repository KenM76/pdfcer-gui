---
name: a-measurement-of-the-wrong-surface-looks-exactly-like-a-broken-one
description: Before believing a check's verdict, prove it sampled the surface it names — a wrong-surface reading, or a blocked one, is indistinguishable from a defect.
metadata:
  type: feedback
---

A measurement that landed on the wrong surface — or never landed at all —
**reads exactly like a measurement of a broken one**. Before acting on a
contrast, colour, layout or reachability verdict, establish that the sampler
was pointed at the thing it names and that it got there.

**Why:** on 2026-08-21 the same mistake was made twice within an hour, and on
2026-08-25 a third variant cost forty minutes.

- **The wrong WINDOW.** After thirteen dialogs became real OS windows, a
  contrast check went on capturing the *application's* window and measured the
  drawing where the dialog used to be — reporting a confident **1.51:1** about
  two headings that actually render at **15.07:1**.
- **The wrong PART of the right one.** `diag::ui_rect_visible` published any
  region that *intersected* the clip, on the stated argument that a
  half-scrolled heading is still worth measuring. A heading two points inside a
  scroll area's bottom edge measured **1.53:1**, read off the anti-aliased top
  rows of glyphs whose bodies had been clipped away, at 5.3 % coverage.
- **★ No surface at all, reported as a property of the application.** Nine
  driven checks skipped with *"Windows refuses SetForegroundWindow to a process
  without foreground rights"* — a **true** sentence that is also printed when
  something entirely different is wrong. The cause was a stray `OpenWith.exe`
  dialog holding the desktop. Three raise strategies were probed against a live
  build and all three failed, which felt like confirmation the harness was
  broken. The question that settled it was not *"why can't we raise our
  window"* but **"what is holding the foreground?"** — two Win32 calls.

All of these verdicts were specific, quantitative, and about working code.

**How to apply:** when a check fails or skips, ask *"what did it sample, and
did it get there?"* before *"what is broken?"* — read the artefact PNG, which
every pixel check writes. Capture the window a frame describes rather than the
application's, and raise it by matching **client origins**, never by z-order
(the raise is about to change z-order). A diagnostic channel that publishes a
region nobody can read is manufacturing false failures. And **make the harness
report what it observed, not merely that it failed** — a refusal that does not
name the refuser has withheld the one fact that separates *wait* from *act*.

★ Corollary on falsifying such a message: *always on top* and *will not yield
the foreground* are **different properties**, and only the second breaks a
harness. A .NET `TopMost` form does not reproduce it; `rundll32
shell32.dll,OpenAs_RunDLL` on an unassociated file does.

Related: [[ui-verify-competes-for-the-machine]],
[[a-check-that-cannot-fail-is-not-evidence]].
