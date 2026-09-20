---
name: a-capability-claim-in-product-copy-needs-the-same-citation-as-a-limitation-claim
description: Verify every claim in operator- or public-facing copy against its source row by row — capability AND history. "Measure" does not imply area and angle; and a remembered release list had 3 of 6 facts wrong, one of them another release's headline. A correct SHAPE is what makes the invented specifics feel safe.
metadata:
  type: feedback
---

**Every feature claim in operator- or public-facing copy gets checked against
`FEATURES.md` row by row before it ships — including the ones you are sure of.**

**Why:** on 2026-09-12, rewriting the repository landing page for humans at
Ken's request, a bullet read *"Lengths, areas and angles in real units."*
`FEATURES.md`'s **Area and Angular** row marks both ⬜ — not shipped. Nothing
prompted the error: the claim was generated from the *shape* of the measure
feature ("a measuring tool measures these things") rather than from its rows.
`DimensionKind::Angular` existing in the engine makes the invention more
plausible, not more true. This is the inverse of the already-recorded failure
mode about **limitation** sentences going stale within hours — that one is a
claim that something *cannot* be done, this one is a claim that it *can*, and
the second is the one a reader acts on and then reports as a bug.

**How to apply:**

- Write the copy, then walk each bullet against `FEATURES.md` and a grep of the
  source. A capability you have personally driven this week still gets the walk,
  because the bullet usually names **more than you drove**.
- The dangerous bullets are the ones naming a **family**: "lengths, areas and
  angles", "PNG, JPEG, SVG and PDF", "text fields, checkboxes and signatures".
  A family is a list of claims wearing one sentence, and the partial ones are
  invisible.
- When part of a family is missing, say so in the copy — *"(Area and angle
  aren't there yet.)"* A stated gap costs a parenthesis; a discovered one costs
  a retraction and his trust in the rest of the page.
- Applies to `README.md`, release notes, the `FEATURES.md` revision header, the
  `MANUAL.md`, and anything `package-portable.py` ships in `PAYLOAD_DOCS`.

Related: [[a-limitation-sentence-is-a-citation-with-an-hours-long-shelf-life]]
(the same discipline, opposite direction) and the global "Claim-bearing copy —
verify the source, don't invent" rule, which lists READMEs explicitly and was
written after exactly this shape of invention in a pitch deck.

## ★★★ MEASURED, SAME DAY: the full audit came back 15 wrong out of ~33 bullets

The area/angle bullet above was caught by eye. A separate adversarial pass was
then run over **every** bullet of the new landing page — *"for each member of
each list, cite the row"* — and it returned **2 outright false and 13 partly
false**, against about twenty correct and cited.

**The 13 partials were all one shape: a family with one absent member.**
"Export to PDF, DXF, PNG, JPEG, SVG, EMF, plain text or form data" (no
export-to-PDF command exists at all). "position, size, colour, line weight"
(object line weight is a read-only fact row). "text fields, checkboxes, radio
buttons, dropdowns, buttons" (an authored push button writes no `/A` action, so
it can never do anything). "dock, undock, tear off" (there is no drag-to-tear —
floating is a command). "a preview that can pop out into its own window" (never
rendered). "copy and paste **any** markup" (`/Widget`, `/Popup`, `/Redact`
refused by name). "pastes into Word, Inkscape and LibreOffice" (LibreOffice
gets EMF; Inkscape verified nowhere).

The two outright false ones were both **two true facts joined by a connective
that manufactured a third**: *"edit text in place and let it reflow"* (reflow is
a command; live re-layout is engine-blocked and recorded as never becoming
automatic) and *"you never wait for detail after moving"* (the module under the
claim has a "what it does NOT fix" section that refutes it).

**How to apply, strengthened:**

- **The author cannot run this pass.** The draft was written by the engineer who
  built most of the program, which is exactly why every family read as obviously
  true. Dispatch a subagent whose only instruction is *cite the row for each
  member*, and treat its PARTIALs as correct until disproved.
- **Budget for it.** Expect roughly **half** the bullets to need an edit. That is
  not a bad draft; it is what one-verification-per-sentence produces when a
  sentence carries three promises.
- **`⬜ BUILT AND UNDRIVEN` is a third category and it is NOT a defect in the
  copy.** Keep the capability on the page and name the undriven families in the
  status section. Withdrawing a shipped feature is its own inaccuracy, and the
  tick discipline only protects a reader if the public page repeats it.

## ★★★ AN ILLUSTRATION IS A CLAIM TOO — 2026-09-13, and this one was published with BOTH ends wrong

The units enumeration found a real defect: the program holds six private
points-to-millimetres constants and two different rounding rules, so two surfaces
can print different whole numbers for one sheet. That analysis was correct, it
named the two rounding rules correctly, and the remedy it proposed is still the
right one.

Then it was **illustrated**, in one sentence, and the sentence was invented:

> *a sheet of exactly 210.5 mm renders **211** in the page thumbnail's tooltip
> and **210** in the print dialogue*

It reached `UNIT_SURFACES.md`, `OPERATOR_REQUESTS.md`, `RESUME.md` and the
**published GitHub release note** for `v0.5.0-dev.20260913.2`. Compiling the two
expressions took four minutes and gave the opposite:

```text
210.5 mm into points      f64 596.6929133858    f32 596.6929321289
tooltip  {:.0} on f32  ->  210.5000000000  ->  "210"
print    .round() on f64 -> 210.5000000000  ->  "211"
```

★★ **And the attributed CAUSE was wrong too, which is the half that would have
misdirected the fix.** The release note said *"two of them in single
precision"*. Precision has nothing to do with it: **both** paths land on exactly
`210.5000000000`. The whole disagreement is that `.round()` is
half-away-from-zero and `{:.0}` is half-to-**even**. The f32/f64 split is a real
separate defect — about three decimal digits on a 14,400 pt sheet — and it
contributed nothing here. ⇒ See
[[when-two-things-differ-in-two-ways-the-measured-one-is-not-the-cause]]: the
difference that already had a bullet in my own analysis got the blame.

⇒ **The consequence for the remedy, which is why this is worth more than the
correction.** "Share one conversion constant" would have fixed nothing in this
example — both sites already convert to the same value. The fix has to settle
**the rounding rule for an operator-facing length**, in one place, with the
choice argued in the source. An unmeasured illustration had been quietly
steering the fix at the wrong target.

**How to apply:**

- **A worked example is a measurement, not an explanation.** The surrounding
  analysis being right is exactly what makes the example feel safe to write from
  the same reasoning — and the example is the part a reader will quote back,
  because it is the concrete one.
- **Anything of the form "X shows A while Y shows B" must be RUN before it is
  written.** Forty lines of `rustc` is the whole cost. If it cannot be run
  cheaply, write the property (*"the two rounding rules disagree on a tie"*)
  and no numbers.
- **Numbers in a sentence are the first thing to doubt when correcting it, and
  the attributed cause is the second.** I corrected the direction first and
  nearly stopped there; the cause was wrong independently.
- ★ **The published artifact is part of the correction.** `gh release edit
  --notes-file` on a shipped release, with a dated parenthetical saying what it
  used to say. A document corrected in the repo while the release note still
  carries the old claim is worse than either alone, because the two now disagree
  in public.

---

## ★★★ Second instance, 2026-09-14 — **history is product copy too, and a
correct SHAPE is what makes the invented specifics feel safe**

Restating `RESUME.md`'s release narrative after cutting the fourth release of
the day, I added a one-line roll-up of the three earlier ones — times and
headlines — **from recall**. Then I ran the query. Three of six facts were
wrong:

| | I wrote | The API says |
|---|---|---|
| `.1` | 01:24 UTC, *“rotate grip + the two O16x fixes”* | 04:39 UTC, *“The export windows finally remember what you last used”* |
| `.2` | 07:22 UTC | 07:18 UTC |
| `.3` | 10:56 UTC, *“redaction override + stamps”* | 11:14 UTC, *“Print: Cancel really cancels, and Keep and close really keeps”* |

`.1`'s was not a mistimed fact, it was **a different release's headline**.

**Three things made it feel safe, and each is the thing to distrust:**

1. **It was inside a patch script whose own docstring listed the commands it
   had re-measured.** The discipline covered the rows the script was written
   for and not the paragraph added three functions below them. ⇒ *A
   re-measurement note is about the lines it names, not about the file.*
2. **The invented facts were HISTORY, not capability**, and history feels like
   recall rather than like a claim. It is the same thing: a reader acts on
   *“`.1` shipped the rotate grip”* exactly as they act on *“it measures
   angles”*.
3. **The SHAPE was remembered correctly** — four releases, ascending through
   the day, each with a real headline — and every specific hung off that
   correct shape. A wrong shape gets caught; a right shape with wrong
   specifics does not.

**How to apply:**

- **Any list of past releases, commits, dates or headlines is a measurement.**
  `gh api repos/<owner>/<repo>/releases --jq '.[]|[.tag_name,.published_at,.name]|@tsv'`
  is two seconds and answers all three columns at once.
- ⚠ **A release time is two different numbers and the word “cut” means
  either.** The binary's `PDFCER_BUILD_TIME` and GitHub's `published_at` sit
  twelve to twenty minutes apart on every release this project has made,
  because packaging and the pre-flight re-drive are in between. `10:56` and
  `11:14` were both true of the same release, of different events. **Label
  which clock.** Half of that roll-up's “wrong” times were this, not recall.
- Write the roll-up as a **table with the source named in its caption**. A
  table invites the column-by-column check a sentence does not.

Related: [[a-limitation-sentence-is-a-citation-with-an-hours-long-shelf-life]]
— its mirror image, and the two together are the whole rule: **a claim about
what the program can do, cannot do, or once did, all need the same citation.**
