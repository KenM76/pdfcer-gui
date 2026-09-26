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

## Item notes

### `const EDIT_TAB`

A pair rather than two constants, because `click_tab` takes one: a region
name and the id the shell reports for it are two spellings of one thing and
a check that let them drift would click one tab and assert about another.

### `const DECLINE_REGION`

Raised by the `edit_text` apply arm from the engine's own `EditReport`. It is
the observable half of `Pass 119.0`: the prose disclosure goes to the status
row for the operator, and this goes to the channel for a check.
The status bar's decline slot, as `app::status::decline` publishes it.

This is a SECOND COPY of a string, and nothing enforces the pair.


What makes the duplication tolerable is the DIRECTION it fails in.
Rename the region in the application and this check stops matching, so it
reports *"the operator was told nothing"* — a **false failure**, loud, on
the very check whose subject is silence. The dangerous direction would be a
false pass, and that is not reachable here: no other region carries this
name, so nothing can satisfy the assertion by accident.

### `const BECAME_ADD_EVENT`

It is raised by `canvas::textedit::place`, on the `Refusal::NoRun` arm and
nowhere else. Only that refusal falls through to an origin; an encrypted
document, a page that will not decompose, or a run the engine cannot address
are all still reported, because those say *this cannot be done here* rather
than *there is nothing here*. So its presence carries a precise claim:
**the aim was not on text, and everything else was fine.**

Quoted rather than merely detected. The conversion is a design the
operator asked for by name, and a reader meeting this skip for the first
time needs to see that the program ANNOUNCED what it did — otherwise the skip
reads as the harness excusing a silence, which is a failure mode this
project has already had to correct twice.

### `const MAX_FOLLOWERS`

**This bound is a fact about the BUILD, not about the fixture.** Reflow
shifts *the rest of the line* by the advance delta; a line is a handful of
show operators in prose and often exactly one on a drawing. A number in the
hundreds means the scan did not find the end of the line and ran on into the
rest of the stream — which is true wherever it happens and on whatever
document.


| | `followers_repositioned` | changed pixels | bounding box |
|---|---|---|---|
| before the fix | **1,676** | 34,059 | x 62–858, y 34–795 — the whole page |
| after | small | **42** | x 542–561, y 378–384 — one label |

The cause: reflow walked forward shifting every absolute `Tm` until a
`Td`/`TD`/`T*` boundary, and **a CAD stream positions everything with `Tm`
and never emits `Td`** — so there was no boundary. One four-character edit
slid the rest of the drawing sideways.

64 is deliberately generous: it is far above any real line and two orders of
magnitude below the failure. A bound tuned close to the observed-good value
would fail on the first document with a long justified line, and a check that
cries wolf gets disabled.

### `const MAX_SHIFT_PT`

**Zero is the correct answer and the tolerance is for arithmetic, not for
behaviour.** Replacing a run's glyphs does not touch the text-positioning
operand that put it there, so the left edge before and after are the same
number arrived at twice. Half a point is a fraction of the smallest
character on a title block and orders of magnitude below the defect this
catches, which he described as the whole line moving.

It is the same bound `canvas::textedit::glyphwall` holds a synthesised
document to, and the equality is deliberate: one asserts it where the engine
is called directly and this one where the operator's keystrokes arrive. A
check that allowed more slack than its unit-level twin would be saying the
shell may move text the engine may not.

### `fn decline_message`

Split out so the happy path reads as a sequence rather than as a `match`
whose arms are 20 lines apart. Its content is the diagnosis, and the three
reasons mean genuinely different things — see the call site's own comment.
