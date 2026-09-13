---
name: a-capability-claim-in-product-copy-needs-the-same-citation-as-a-limitation-claim
description: Verify every feature bullet in README/release copy against FEATURES.md row by row; "measure" does not imply area and angle, and the shape of a feature is not evidence of its parts.
metadata:
  type: feedback
---

**Every feature claim in operator- or public-facing copy gets checked against
`FEATURES.md` row by row before it ships — including the ones you are sure of.**

**Why:** on 2026-09-12, rewriting the repository landing page for humans at
Ken's request, a bullet read *"Lengths, areas and angles in real units."*
`FEATURES.md:908-932` marks **Area and Angular as ⬜ — not shipped.** Nothing
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

Related: [[feedback_a_limitation_sentence_is_a_citation_with_an_hours_long_shelf_life]]
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
