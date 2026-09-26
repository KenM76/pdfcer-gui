# `text::ocr` — every word the Recognise-text surface says

Consumed by [`crate::dialogs::ocr`] (the dialog that runs recognition and
reports what it inferred) and by [`crate::find::bar`] (the offer that
appears when a search found nothing on a page that has no text to find).

## Why this catalog is unusually careful, and it is not house style

**OCR is the single largest inference pdfcer makes.** `pdfcer-core`'s own
`ocr::layer` header says it in those words — *"every word here is a
guess"* — and project rule 4 (*"fuzzy, never sneaky"*) therefore binds this
surface harder than any other in the program. Two of its clauses bite here
and they pull in different directions, which is why the copy is written the
way it is:

1. **The result must look normal.** The operator asked for exactly that:
   *"I want OCRed stuff to look normal when the command is executed too."*
   Mode 3 is not a compromise, it is the whole mechanism — nothing visible
   is added, the page renders pixel-identically, and there is **no**
   highlighting of doubtful words baked into the document. So none of the
   copy below promises a visible mark, and none of it should ever grow one.
2. **The uncertainty must be stated anyway**, off-canvas, before the
   recognition becomes a file. That is what this dialog is for.

## The one fact this surface exists to carry

**`ocrs` reports no confidence at all.** Not "low confidence", not
"confidence pending" — its output type is a character and a rectangle, and
there is no score on a character, a word, a line or the page.
`OcrsEngine::reports_confidence()` returns `false` and
`pdfcer_core::ocr::engine_ocrs`'s header is explicit that this is *"a fact
about the world"* rather than a stub awaiting improvement.

The consequence for copy is sharp, and it is the reason [`no_confidence`]
is worded as a negation of a specific wrong reading rather than as a
neutral note: **an absent score and a high score must never look the
same.** A dialog that reported "0 words need review" would be true of an
engine that scores nothing and would read as a clean bill of health. So
this surface never says that, and `OcrPage::words_needing_review` already
encodes the same principle on the other side by counting an unscored word
as needing review.

## Where the *engine's* sentences come from, and why they are not here

[`crate::dialogs::ocr`] renders `OcrLayerReport::disclosures()` — a
`Vec<String>` built inside `pdfcer-core` — as a list, verbatim, beneath the
headings below. That is deliberate and it is the engine's own instruction:
the disclosures are built *"here rather than at each call site so the GUI
and the CLI cannot disagree about what was disclosed."*

They are therefore **data at run time**, not literals in this crate, and
`tools/gates/check-ui-strings.sh` is untouched by them. What lives here is
the shell's own framing — the headings, the buttons, the refusals — which
is exactly the split rule R1 is about: a catalog owns the words this
program chose, not the words another crate reported.

## Conventions

[`crate::text`]'s, unchanged: sentence case and no trailing period on a
label, full sentences with punctuation for prose, an ellipsis on a control
that asks a question before acting. One addition — **no sentence here
makes a claim about accuracy.** pdfcer has never measured this engine
against a real scan (`FEATURES.md` records that its only test documents are
vector PDFs that already contain text), so "accurate", "reliable" and
"high quality" are words this surface is not entitled to.
