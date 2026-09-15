---
name: a-measurement-of-the-wrong-surface-looks-exactly-like-a-broken-one
description: Before believing a check's verdict, prove it sampled the surface it names and got there — a wrong-surface reading, a blocked one, or a long-green wrong AIM is indistinguishable from a defect.
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

---

## ★★ The long-green variant — **a wrong aim that happens to HIT**

*(Merged in 2026-09-15 from its own entry, at `check-memory-index.sh`'s asking:
two index rows for one rule. Same question — what did it sample — reached from
the other end, where the check has been GREEN rather than red.)*

A `ui-verify` check that has been green for days is **not** evidence that
its aim is correct. It is evidence that its aim has been landing on
something.

**Why:** on 2026-08-27 `checks::ocr::click_region` was found converting a
dialog's `ui-rect` against `session.frame()` — the *application's* window
— where that dialog has been its own OS window since 2026-08-21. It was
missed in the bulk conversion to `driving::frame_of` and **passed for six
days**, because the button happened to sit where the stray click landed.
It failed the moment the new page-scope group pushed the button ~100 pt
down. My first reading of that failure was *"the Recognise button is
broken"*; the application was fine. The trace settled it —
`canvas-pointer screen=(34.0, 225.0)` was the **main window's** canvas
receiving a pointer event at the dialog's own coordinates.

**How to apply:** when a long-green driven check fails right after a
layout change that had nothing to do with it, ask *what did this check
sample* before *what is broken*. Two specific instruments:

- `grep -n "session.frame()?.declared_center" tools/ui-verify/` — any hit
  in a check that drives a dialog is a latent version of this defect.
  `frame_of` is safe on a main-window region, so converting pre-emptively
  costs nothing.
- Read the trace for a pointer event landing in the **wrong window**. A
  `canvas-pointer` line whose coordinates match a dialog rect is the
  signature.

The sibling finding from the same run: a check pointed at
`--pdf SW41177.pdf` failed with `NothingRecognised` and the application
was **right** — every page of that CAD sheet already has text, so the
doubling guard skipped all of it. A check whose subject is *"did X read
this input"* must pin its own fixture and ignore a suite-wide `--pdf`.
