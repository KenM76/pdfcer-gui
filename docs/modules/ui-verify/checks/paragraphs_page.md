# `paragraphs_off_on_one_page_leaves_the_next`

**Defect it guards.** O295 item 5: Paragraphs can be set for one page. A
per-page answer that the click ignores, or that reaches every page, fails.

**Fixture.** `fixtures/paragraph-2p.pdf`: two pages sharing one content stream,
the six-line paragraph of `paragraph.pdf` on each (`tools/gen-reflow-fixture.py`).

**Steps.**

1. With Edit text armed on page 1, press `ribbon.item.edit.paragraphs_page`
   with the pointer. The trace must hold `open-paragraphs-pages enabled=false
   pages=0`.
2. Click inside line 2 of page 1: the `text-edit-caret` line names `run=`.
3. Escape, PageDown, and the same click on page 2: the caret names `block=`.

**Falsified** by making the click read the global switch only: page 1 opens
`block=` and the check fails at step 2.

**What it does not prove.**

- The thumbnails' pick as the operand; only the page on screen is driven.
- The text right-click menu's row.
- That the answer is forgotten when the document closes.
