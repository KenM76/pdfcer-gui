# `one_paragraph_opens_as_lines_while_its_neighbour_opens_whole`

**Defect it guards.** O295 item 5: Paragraphs can be set for one paragraph. A
paragraph answer that the click ignores, that reaches every paragraph on the
page, or that cannot be switched back, fails.

**Fixture.** `fixtures/paragraph-two.pdf`: the six-line paragraph of
`paragraph.pdf` twice on one page, the second 200 pt lower, each in its own
text object (`tools/gen-reflow-fixture.py`; the engine will not rewrite a block
that shares its `BT … ET` with other text).

**Steps.**

1. With Edit text armed, click inside line 2 of the first paragraph: the
   `text-edit-caret` line names `block=`.
2. Right-click there and press `menu.item.canvas.text.edit.paragraph_here`. The
   trace must hold `open-paragraph-here enabled=false … now=true` (the
   unedited paragraph went back to its line).
3. Escape; the same click opens `run=`, and the same click in the second
   paragraph opens `block=`.
4. Click the first paragraph's line again, press the row: `enabled=true …
   now=true`; Escape and click: `block=`.

**Falsified** by making `promote::here::opens` ignore the stored answer: the
first paragraph opens `block=` and the check fails at step 3.

**What it does not prove.**

- That an edited paragraph keeps its draft when switched off (the answer holds
  from the next click; only the unedited case is driven).
- That the answer follows the paragraph after an edit moves its box.
