---
name: a-measurement-of-the-wrong-surface-looks-exactly-like-a-broken-one
description: Before believing a check's verdict, prove it sampled the surface it names, got there, and was handed the right input — a wrong-surface reading, a blocked one, a long-green wrong AIM, and a harness given a bad coordinate are all indistinguishable from a defect.
metadata:
  type: feedback
---
<!-- old-name-exempt-file: quotes a trace argument that names the pre-rename path. -->

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

---

## The input variant — what the harness was ASKED to do


**Before believing a driven failure, check what the harness was ASKED to do.**

### The incident, 2026-09-04

A 153-check sweep was run with `--doc-point 1,300,400` against a **one-page**
fixture. `PAGE` is **0-based**, so `1` names a second page that does not exist.

That single wrong digit produced **six failure reports** — canvas loses the
pointer after a long scroll, resize commits nothing, rotate commits nothing,
shift does not constrain, multi-node move moves nothing, wheel-paging does not
page. Each named a real function, a real trace event and a real line number.

**I filed four of them as defects**, and wrote a paragraph claiming one of them
disproved an earlier theory about the pasteboard.

Isolated properly — *same fixture, same zoom*, page index `1` → `0` — every one
passes.

★ I also proposed a *second* wrong explanation on the way (the A1 fixture's 20 %
zoom making grips too small) and wrote it into the ledger as the likely cause.
It was plausible, it was consistent with every observation, and it was wrong.
**Two wrong diagnoses before the right one**, both written down confidently.

### Why this is worse than a harness that will not start

A harness that cannot start costs nothing. One given a bad coordinate produces
output that is detailed, plausible, **perfectly reproducible** — which reads as
*reliable* rather than as *broken* — and indistinguishable from a real finding
without re-deriving it from scratch.

⇒ **The cost of a false driven failure is a whole investigation, plus whatever
is built on top of it before anyone notices.**

### How it was caught, and how nearly it was not

Only by **shape**: four failures at once, all one gesture family. That finally
prompted the standing question — *what did the check SAMPLE?*

**Had it produced one failure instead of four, it would still be filed as a
defect today.**

### The root cause was a guard that could never fire

`doc_to_window` already refused a point on the wrong page, with the right
reasoning written above it. But every caller did:

```rust
CanvasMapping::from_trace(&trace, vocab, page, target.page)
//                                             ^^^^^^^^^^^
```

The mapping was told its page index **by the point it was about to check**.
`p.page != self.page_index` compared a number with itself.

Fixed by comparing against what the **application publishes** — the page it says
it is showing, on the same trace line as the rect. Two independent quantities.

★ A first fix — parsing the page count out of the PDF — was written, measured,
and **deleted**: it could not read either fixture confidently, so it was a guard
that never fired. Two weak mechanisms are worse than one that works.

### How to apply

- **Validate a harness's INPUTS as hard as its outputs.** Coordinates, page
  indices, region names, fixture paths: all can be wrong in ways that produce
  *output* rather than errors.
- **Several failures in one family are a question about the instrument first.**
  Real regressions cluster too, but the instrument is cheaper to check and is
  wrong more often.
- **Preconditions belong where preconditions are checked** — once, before
  anything is driven — not inside a conversion where each caller's error
  handling can turn a refusal into some other verdict.
- When retracting, **retract loudly and keep the wrong reasoning visible.** The
  useful record is not that it was fixed; it is which plausible explanations
  were believed on the way.

Related: [[feedback_a_proxy_condition_survives_one_correction]],
[[feedback_a_check_that_cannot_fail_is_not_evidence]],
[[feedback_a_trace_grepping_check_passes_on_a_build_that_crashed]].
Full write-up:
`D:\dev\rag\egui\a_harness_given_a_bad_coordinate_does_not_fail_it_lies_fluently.md`

### ★★★ FOURTH INSTANCE — and this time the fabricated diagnosis named a mechanism that had ALREADY been repaired — 2026-09-13

`the_wheel_turns_pages_when_the_operator_asks_it_to` FAILed with:

> ★★★ THE WHEEL DID NOT TURN A PAGE. … The most likely cause is the one this
> check was written for: `OpenDoc::prefs` is a SNAPSHOT adopted when the
> Settings window is applied…

The document it was driving had **one page**. `open ok pages=1
path="fixtures/a1-titleblock.pdf"`. There was no page 2 to turn to. And the
preference it accused of being inert reached the status line on the very next
frame after the press — `wheel=scroll` before, `wheel=flip` after — so the live
push was working and the snapshot defect had been fixed some time ago.

★★ **A wrong-fixture SKIP announces itself; a wrong-fixture FAIL writes a
diagnosis.** The inherited fixture *opened fine*, so every precondition the
check thought to assert held. What it never asserted was that the document it
had been handed could exhibit the behaviour in the check's own name.

★ **The diagnosis was the most convincing thing in the report**, because it was
a real mechanism, correctly described, with the right module named — just
already repaired. A check that hypothesises a cause in its failure text will
keep publishing that hypothesis long after the cause is gone, and every reader
believes it, because the alternative is believing the check is wrong.

**How to apply:** a check's `const FIXTURE` is part of its assertion, not part
of its plumbing. Before trusting a driven FAIL, grep its trace for the one fact
that makes the behaviour *possible at all* — `pages=`, object count, selection
count, whether the panel is the active tab. And when a failure message names a
mechanism, **date the mechanism** before repeating it.

### ★ FIFTH INSTANCE — the check sampled the right window, the right gesture, and the wrong REGION KIND — 2026-09-18

Three assertions in one new file failed at once, each with a precise sentence,
all about a drag-and-drop that worked perfectly.

- Two asked *"is this affordance on screen?"* about a **pre-commit affordance**
  — a drop compass, a tear outline. Every such thing is retired by the release,
  which is what makes it an affordance rather than a mark on the document. The
  present-tense reader can only ever answer "absent".
- One asked *"does the landed panel have a tab?"*. **This dock draws no tab
  strip while its rail is showing.** The panel was docked exactly where the
  offer promised.

★★ **The tell was in the failure text itself and needed no debugging**: each
sentence listed its own subject in its *"regions drawn"* list, because the
assertion used the live reader while the helpful what-I-found list used the
ever-seen one. **If a check's error names its own subject as found, the check
is asking a present-tense question about a past-tense subject.**

**How to apply.** Two questions before believing any region assertion:

1. *Is the subject retired by the gesture that proves it?* If yes, it needs an
   anchored `declared_since(.., gesture_start)`, never `declared`. The anchor is
   required — without it the helper is fossil-reading renamed.
2. *Is the subject chrome the container is free not to draw?* Tab strips,
   scrollbars, splitter grips, resize handles and title bars are all optional in
   some configuration, and each has been somebody's oracle. Assert on content:
   **every docked panel has a body; only some have a tab.**

Full write-ups: `D:\dev\rag\egui\a_ui_rect_change_log_produces_confident_wrong_failures_in_BOTH_directions.md`
(§ the tell) and
`D:\dev\rag\egui\a_panels_presence_in_a_dock_must_be_asserted_on_its_body_because_a_tab_strip_is_a_property_of_the_compartment.md`.
