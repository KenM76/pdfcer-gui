# `ui-verify/checks/text_edit_real`

`text_edit_on_a_real_drawing` — **the operator's own report, driven**: arm
Edit text on a dense CAD sheet, click on text the engine says is there, and
find out what actually happens.

# Why this exists when two text checks already pass

`text_edit_pins_an_aligned_tail` and `add_text_takes_real_keystrokes` both
pass, and the operator's report on 2026-08-19 was *"text editing on canvas
still doesn't work"* — twice, weeks apart.

**Both of those checks drive a fixture this repository generated.**
`tail-alignment.pdf` is 924 bytes with three lines of text placed by a
script, and `add_text` writes onto blank paper. Neither has ever touched the
documents the operator actually opens: SolidWorks-exported drawing sheets,
1584 × 1224 pt, dense with vector geometry, where the text is small and
sparse and lives inside title blocks and tables.

> **A feature verified only against the fixture that was written to verify
> it is verified against the author's model of the problem, not against the
> problem.**

# What this check is really asking

Not *"does the caret work"* — the other two settle that. This asks the two
questions that separate a working feature from an operator's *"it doesn't
work"*:

1. **Does a click on real drawing text resolve a run at all?** The hit test
   runs against extracted page text on a page with tens of thousands of
   objects. `pdfcer find-text` gives the ground truth: aim at a rectangle
   the engine itself reports, and a miss is the application's, not the aim's.
2. **When it declines, does the operator get told?** Every decline path here
   ends in `crate::text::textedit::refusal`, whose sentences are good, are
   tested, and were aimed at a status row that `R128` forbids growing.
   `Refusal::SpansRuns` is 47 words. **A decline nobody can read is
   indistinguishable from a feature that does nothing** — which is the
   operator's sentence, exactly.

# The aim comes from the ENGINE, not from a guess

`--doc-point` is required and there is deliberately no default, for the
reason `CheckContext::target` already records: *a click on empty page is
symptom-identical to a broken hit test*, and this project has already filed
and retracted one defect over that confusion.

So the point is supplied by whoever runs it, and the honest way to obtain
one is to ask the engine where text is:

```text
# ⚠ Drive a COPY. A run opens the document, types into it and may save;
# his drawing is not a fixture. `target/scratch/docs/` is where the copy goes.
cp D:/dev/pdfTests/SW41177/SW41177.pdf target/scratch/docs/SW41177-ken.pdf

pdfcer find-text --needle PART target/scratch/docs/SW41177-ken.pdf
  match page=1 text="PART" rect=1187.45,1178.37,1215.82,1191.21

ui-verify … --doc-point 0,1201,1185 --check text_edit_on_a_real_drawing
```


A failure at a point sourced that way is the application's. A failure at a
guessed point is nobody's.


Driven against a copy of the operator's `SW41177.pdf` at
`--doc-point 0,272.7,724.2` — the centre of `FAR SLOT`, whose rect came from
`pdfcer find-text` and not from a guess:

```text
text-edit-caret kind=Edit page=0 run=24 len=9
edit-text-target … disposition=Pin
```

The click resolved a run, the caret took keystrokes, the edit reached the
engine, and **0 following operators were repositioned** — against **1,676**
on the same drawing before the engine's `Pass 121.1`. So O198's
*"still can't edit when the text has been reflowed"* **does not reproduce
on this build**.

That is not a claim the operator was wrong. It is a claim about WHICH
BUILD: he is running a published one that predates the fixes, and this
check being green and his report being accurate are the same state of the
world one release apart. **The release is the answer to O198**, not more
investigation — which is why `RESUME.md` orders it that way.

⚠ AND THE SAME SWEEP SHOWED WHAT AN UNGUARDED AIM COSTS. Run with the
sweep's shared `--pdf fixtures/a1-titleblock.pdf --doc-point 0,2000,320`,
this check FAILED with *"the shell built no plan"* — false in every clause.
The point is 151 pt of blank title-block paper; the shell converted the
Edit draft to an Add draft, said so on its own trace line, and committed
it. The `--- 4b` guard in `drive` is the repair, and the rule it enforces
is written out IN THIS FILE, at step 6, where the identical mistake was
corrected once before: **a check that asserts on the absence of a line
must first ask whether a DIFFERENT line explains the absence.** That
correction was about `edit-text-refused`; this one is about
`text-edit-became-add`. A rule written down beside one instance of
itself does not generalise on its own — the next instance arrives wearing
a different event name and reads as a new problem.
