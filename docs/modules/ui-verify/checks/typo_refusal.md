# `typo_refusal` — **his spelling mistake, corrected**, driven on the file he
reported it on


> *"on page 2 there is a spelling mistake — clien instead of client. if I try
> to edit the edit is not accepted. **the lines I added below `price)` are
> editable, but everything else that existed when I got the pdf is not.**"*

Two sentences, and the second is a complete diagnosis he made himself. This
check drives both halves of it in **one launch**.



### What was actually wrong, which is not what it looked like

His producer writes **one glyph per show operator** — a thirty-six character
line is thirty-six `Tj`s on one row, stepped by x-only `Td`s. `Pass 256.0`
taught `edit_text` to match a `find` across exactly that shape and the engine
measured `"clien"->"client"` on his own file, `operators_spanned=5`.
**That capability was in this shell's pin and his typo still failed.**

The standing diagnosis was that the shell sends the run's whole text as
`find` and that extraction synthesises the spaces inside it, so the string
named characters no operator wrote. ⇒ **Measured on his file, that is false.**
One `EditSession` per shape, page 2, the run he reported:

| request | result |
|---|---|
| whole-run `find` **+ pin** — what this shell sent | `NotFound` |
| whole-run `find`, **no pin** | **OK**, `operators_spanned=36` |
| `"clien"` **+ pin** | `NotFound` |
| `"clien"`, **no pin** | **OK**, `operators_spanned=5` |

Thirty-six characters, thirty-six operators: **the spaces are in the
operators**, and the whole-run `find` matches perfectly once the pin is off.
The pin was the defect. `Pass 256.0`'s contract says *"a pinned request never
spans"*, so a `find` sent beside a pin is confined to the one operator the
pin names — which on his line holds a single character. The engine was
answering the question it was asked, correctly, every time.

## ★★★ What is asserted, and why it is TWO gestures and not one

| gesture | what must happen |
|---|---|
| correct the typo in text the document arrived with | the commit **lands**, the plan's own line names one of the three legitimate request shapes with the pin still ON, and **no `⊗` slot draws** |
| commit text pdfcer itself wrote | the edit lands **and no `⊗` slot draws after it** |

The second row is the whole reason this file is long. The oracle for the
decline half is *"a region was published"*, and a probe whose baseline has no
dynamic range **cannot produce a verdict**. It is the contrast the operator
noticed — text pdfcer authored commits, text that arrived does not — so the
check's two rows and his two sentences are the same two facts. ★ Since the
inversion **both** rows now succeed, which is the point: his complaint was
that they differed.

## ★★★ THE ASSERTION THAT CARRIES THE VERDICT IS NOT "THE EDIT LANDED"

It is `edit-text-pin … pinned=true`, and the distinction is the difference
between a working program and a dangerous one.

The pin is the **only** disambiguator `EditRequest` carries — there is no
occurrence index on it — so dropping it hands the choice of *which*
occurrence to edit to the engine's left-to-right scan. On a page holding the
same words twice that silently corrects whichever it reaches first. **The
document this was reported against is a signed quotation**: a wrong edit
there is not a defect he reports, it is one he finds later in a file he has
already sent.

⇒ So a build that dropped the pin **unconditionally** would land this edit,
satisfy a naive assertion for ever, and be exactly the build that must never
ship. The plan's own line is what tells the two apart, and this check reads
it.

The shipped mechanism keeps the pin and narrows the search instead:
[`EditRequest::spanning_from`](pdfcer_core::text_edit::EditRequest::spanning_from)
starts the span search **at the pinned operator** rather than at the first
operator on the page, with every other guard unchanged. `find` says *what*;
the pin says *which one*.

⚠ Counting the occurrences and dropping the pin when the text is unique is
**not** a weaker version of the same thing and must not be reintroduced as a
fallback — `find_anchor` tries a single-operator match across the whole page
before the spanning search runs at all, so an unpinned request cannot reach a
spanning run whenever a single-operator twin exists anywhere on that page.
`canvas::textedit::Plan::occurrences` carries the engine's own ruling;
`canvas::textedit::glyphwall` holds it as unit tests over two authored
fixtures — one where the run is unique and the edit must land, one where it
appears twice and the **clicked** occurrence must be the one that changes.

## The oracle, and its one honest weakness

`status-group:decline` is a `ui-rect` region published on the frame it
draws. The harness cannot read rendered text — there is no accessibility
reader and no OCR — so this check asserts that the slot **did not draw**, not
what it would have said. The wording is held by unit tests in
`app::status::decline` and `text::textedit`, and by `check-ui-strings.sh`.

★ Ordering is load-bearing and is asserted by `lineno`. A whole-capture
`last(...)` is a fossil finder; every region read here is anchored to a cause
that must precede it — the successful commit for both arms.

## Aim

`--doc-point PAGE,X,Y` in PDF user space, on a run the document arrived
with. For the operator's own file:

```text
--pdf "…/apartment work - signed.pdf" --doc-point 1,200.4,537.1
```

— the centre of *"Final quality walkthrough with clien"*, whose box
`extract-text --pages 2 --json` reports as `[33.47, 526.22, 367.26, 547.90]`.
★ `PAGE` is **0-based**; his page 2 is `1`.

⚠ **Copy his file to scratch and drive the copy.** The edit under test writes
to the document; never point this at OneDrive.

### Reaching the spanning decision without his file

His document is not in this repository, and on any ordinary whole-operator
run the plan takes the other branch — `pinned=true span_from_pin=0` — which is
correct but exercises nothing this check was written for. The repository owns
a fixture of exactly his shape:

```text
--pdf fixtures/per-glyph-twice.pdf --doc-point 0,84.3,703.8
```

— the centre of the **first** of two identical per-glyph `ABC` runs,
`find-text` box `[72.00, 697.36, 96.67, 710.20]`. Two occurrences is the point:
it is the page on which dropping the pin would edit the wrong one.

★ The driven check can only read the trace, so it asserts the *decision*
(`pinned=true`, in one of the three shapes below). Which occurrence actually
changed is asserted
by `canvas::textedit::glyphwall` against the same fixture, by position.

## ★★ Why the caret is expected to be OFFERED, not withheld

R9 argues that a control which fails on press is worse than one that is not
there, and the obvious reading of this defect is *"do not offer a caret on
text pdfcer cannot edit"*. **That reading was tested and is false**, and the
falsification is worth carrying here because it is the reason this check
asserts a caret rather than the absence of one.

The tempting forecast is *"the run's font is `Identity-H`, so refuse"*.
`pdfcer-core`'s own fixture `fixtures/synthetic/text/composite-editable.pdf`
is `Type0/CIDFontType2`, `Identity-H`, `verdict=blocked-identity` — the
**same** `list-fonts` verdict as every uneditable face in the operator's
document — and it edits end to end, asserted by
`composite_refusal_reachable.rs::an_invertible_composite_run_is_editable_end_to_end`.
`Identity-H` is *necessary* for this refusal and nowhere near *sufficient*:
`Pass 29.0` made composite runs editable whenever their `/ToUnicode`
inverts.

⇒ A shell that withheld the caret on `Identity-H` would refuse editing on
text pdfcer can edit, silently, on every document — the exact failure
`Editability` was made an enum rather than a `bool` to prevent, and one this
project has already committed once (`Refusal::InsideForm`, whose whole
episode is written at `canvas::textedit::Refusal`). **The caret stays. The
silence goes.**

The negative control needs no aim of its own: it arms **Add text** and
clicks the very same coordinate, which `place::click`'s `Add` arm turns into
an origin draft without consulting the hit test at all. `add_text` then
writes the engine's bundled Helvetica — which is exactly what the operator's
own editable lines are made of, so the control is his contrast rather than a
stand-in for it.

## ✅ A defect this check found while looking for somewhere to put that
control — FILED, FIXED, AND CLOSED


⚠ **The design decision this finding produced is NOT superseded and must not
be undone.** The negative control still arms **Add text** at the subject's
own coordinate rather than hunting for bare paper, and the reason given below
— *same point, same page, same process, same instrument* — never depended on
the defect. It was the better control before the fix and it is the better
control after it. What expired is the justification's tense, not the choice.

> `pdfcer-core`'s `EditableTextModel::hit_test` ends with *"fall back to the
> nearest line by baseline distance"* and applies **no distance bound**. So on
> a page carrying any text at all, every point resolves to a run:
> `canvas::textedit::Refusal::NoRun` is unreachable, and with it
> `place::click`'s *"a click that names no run starts a new one"* — the
> 2026-08-19 answer to the operator's *"How do I make new text when I click on
> the canvas and expect to edit there?"* Driven here: clicks 215 pt to the
> right of a run's box and 99 pt below it both resolved that same run. Filed
> rather than worked around.
