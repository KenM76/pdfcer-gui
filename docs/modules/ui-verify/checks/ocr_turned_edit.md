# `a_recognised_line_on_a_turned_page_edits_upright`

**Defect it guards.** O294: on a page stored sideways with `/Rotate`, a click
on a recognised line flashes an oversized draft, then turns it across the words
and squeezes it into the line's box.

**Fixture.** A copy of `fixtures/scan-rotate-90.pdf`, recognised with `ocrs`
through the window and saved (`ocr_scripted::recognise`). The check reads the
first `Tm` of the saved layer and the `Tf` size before it. It then reopens the
copy in Edit mode with the OCR layer shown and the Text tool armed.

**Steps.**

1. The first line's matrix must be the quarter turn `0 b c 0`, with b > 0 and
   c < 0. A layer written across the words fails here.
2. The band runs from 1 size before the line's origin to 16 sizes along it, and
   from 0.6 below the baseline to 1.6 above it, clipped to the page. Before the
   click, the layer-coloured ink in it must be at least `MIN_INK` pixels and
   more than three times as wide as it is tall.
3. Click 2 sizes along and 0.35 up, and take a screenshot. Type `XQ` and take
   another. In each, the layer-coloured ink must be at least 0.8 of the
   before-width and at most 1.5 of the before-height.
4. After the click, the trace must hold `text-edit-shaped` and no
   `text-edit-preview-fallback` other than `reason=none`.

**Falsified** against the v0.82.0 build: step 1 fails, with the layer at
`[1 0 0 1 …]`. The step 3 arm is falsified only by its unit test: a draft
20 px wide by 200 px tall where the layer drew 200 by 20 fails.

**What it does not prove.**

- That a draft too small is caught. The band also holds the line's
  neighbouring words, so their ink keeps the width up.
- That the inserted letters are where the caret was.
- `/Rotate 180` and `/Rotate 270`.
- Any engine other than `ocrs`.
