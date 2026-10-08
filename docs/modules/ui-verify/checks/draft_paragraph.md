# `ui-verify/checks/draft_paragraph`

`a_paragraph_reopens_as_one_draft`, O288 item 9, on
`fixtures/four-pages-unrotated.pdf` (612×792), page 1 below the band, where
the paper is blank.

## Drive

1. Edit mode, *Edit text* armed (`PDFCER_DIAG_INVOKE`).
2. Click (100, 400): blank paper, so the click becomes an add. Type `Notes`,
   Enter, `second line here`, Ctrl+Enter.
3. Click (110, 403), on `Notes`.
4. Type `Z`, Ctrl+Enter.
5. Click (110, 403) again, then Escape to close that draft unchanged.
6. Drag a box from (100, 300) to (220, 220), type
   `alpha beta gamma delta epsilon zeta eta theta`, Ctrl+Enter, click
   (110, 290).

## Oracle

- Step 2 and step 6 each trace `add-text`; step 4 traces
  `edit-block-text-applied`. A missing one is an error, not a verdict.
- Step 3: `text-edit-widened lines=2 breaks=1 len=22` — one draft for the
  two lines, the Enter kept as a break. `Notes` is short enough for `second`
  to fit after it, so only a kept break holds them apart on rewrite.
- Step 5: `lines=2 breaks=1` again. An edit that re-wrapped the paragraph
  would merge the lines, or split the widest (G162).
- Step 6: `lines>=2 breaks=0 len=45` — the wraps the engine made read back
  as spaces.

## Falsified

Each mutation run alone against the passing build:

- `widen` not called → FAIL at step 3: the click opened a one-line draft.
- `joint` always a space → FAIL at step 3: `breaks=0`.
- The commit given no wrap width → FAIL at step 5: the edit split the
  paragraph's widest line, and it re-opened as `lines=3 breaks=0`.
- `begin_box` back to kind `Add` → FAIL at step 6: the box closed the next
  frame and Ctrl+Enter wrote nothing.

## What it does not prove

A paragraph from a foreign producer, where the break inference (G163) meets
lines it did not write; a paragraph in a table cell; mixed looks, which
decline as before. The cost of `widen` on a dense sheet is unmeasured; it is
1–2 ms here.
