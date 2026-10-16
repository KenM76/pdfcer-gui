# `one_click_selects_a_paragraph_drawn_from_several_objects`

**Defect it guards.** O295 item 1: a paragraph behaves as one however its text
is chunked in the file. Many producers write each line as its own text object,
so a Select-tool click that takes one object selects one line, and the
paragraph cannot be moved or styled whole.

**Fixture.** `fixtures/paragraph-lines.pdf` (`tools/gen-reflow-fixture.py`):
the six-line paragraph of `paragraph.pdf`, one BT…ET per line.

**Steps.**

1. With Edit armed (Select tool), click inside line 3. The last
   `canvas-selection` line after the click must say `sel=6`, and a
   `paragraph-pick` line must appear.
2. Click the same point again. The selection must be `sel=1` (the line
   clicked) with no `paragraph-pick` line: a click on a member already held
   narrows, it does not re-take the paragraph.

**Falsified** twice: `promote::objects::of` returning `Err` fails step 1
(`sel=1`); `paragraph_pick` running even when the hit was already held fails
step 2 (the second click re-picks six).

**What it does not prove.**

- Shift-click and double-click skip the pick by construction; neither is
  driven.
- A paragraph whose switch answers no (*Paragraphs* off, the page or the
  paragraph sent to lines) is declined with `reason=switched-off`; that path
  is not driven here.
