# `ui-verify/checks/clipboard_annotation`

`copying_a_sticky_note_carries_the_whole_comment` — **the annotation
clipboard, driven end to end.**


Said here, in its own header, rather than left for an absent result to
imply. The session that wrote it was told the operator might be at his
keyboard, and this harness drives the real cursor and the real keyboard and
takes the whole desktop while it does. **No line below has been observed
against a running binary**, and nothing in the report that ships with it
claims otherwise. It is registered so the next sweep picks it up.

★ That is the honest state and it is worth naming what it costs: every
failure message here is a *prediction* of what a wrong build would print,
and this project has three recorded cases of an articulate, plausible
failure message being about nothing at all. Treat the first run as
calibration, not as a verdict.

## What this is for


The repair routes the copy through `EditSession::copy_selection`, which
carries an annotation pdfcer does **not** model as its own dictionary plus
the object closure it reaches — including its baked `/AP`.

## ★★ Why this cannot be a unit test

The unit tests in `canvas::clipboard::tests` already assert the clip's
contents and the round trip through `ObjectClip::to_bytes`. What they cannot
reach is the **chord**, and the chord is where this family's defects have
actually lived:


The second row is the one this check exists for. `RESUME.md` records the
form-field copy shipping **without** the marker, one function away from the
comment explaining why it was needed, and the symptom was `Ctrl+V` working
or not depending on what the operator had last copied in another program.
The annotation copy is a new copy site and inherits exactly that hazard.

## ★★★ It pins its own fixture and IGNORES `--pdf`

Same posture as `ocr` and as `three_clicks_round_a_hole_measure_the_hole`,
and for a stronger reason than either: this check's subject is *"a
`/Text` annotation, which pdfcer does not model, copies whole"*, and on a
document whose only annotations are squares and clouds **the defect cannot
occur** — every one of those took the shell's spec route (deleted 2026-09-08
once the engine's own carrier matched it), which worked before this
change and works after it. An arbitrary drawing would make this check unable
to fail, which is this suite's own stated worst outcome.

`fixtures/annots-with-everything.pdf` is built by
`tools/gen-annots-with-everything-fixture.py`, whose header argues for every
key on every annotation in it. The two facts this check depends on:

* `/Annots` position **1** is a `/Text` sticky note at `/Rect [360 660 380
  680]`, carrying `/CA 0.4`, `/T`, `/M`, `/Contents` and an `/AP`;
* `/Annots` position **0** is a `/Square`, which is what a *wrong* build
  would copy if the click missed — and it would look like a pass, which is
  why the copy's own trace line is read for `annots=` rather than for mere
  presence.

## What it does NOT assert, said rather than implied

**The pixels.** A pasted sticky note is a 20 × 20 pt icon; at fit-page zoom
on a 595 × 842 pt sheet that is a handful of screen pixels, below the noise
floor of a window capture. What is asserted instead is the engine's own
count of what it planted — `paste-objects-applied … annots=1` — which is a
number a wrong build gets wrong and a capture cannot resolve.

**That the author, date, note text and opacity survived.** They do, on this
route, because the raw carrier copies the dictionary — and that is asserted
key by key in `canvas::clipboard::tests`, where the dictionary is readable.
From out here the trace carries no dictionary and inventing an oracle for
one would be a proxy.
