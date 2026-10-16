# `word_gap_opens_from_the_text_being_edited`

**Defect it guards.** O295 item 5: the word-gap setting decides how much of a
line one click opens, so it must be reachable where text is edited, not only
from Settings. A *Word gap…* row that is missing, or that opens Settings on
another page, fails.

**Fixture.** `fixtures/paragraph.pdf` (`tools/gen-reflow-fixture.py`).

**Steps.**

1. With Edit text armed, click inside line 2 of the paragraph: a
   `text-edit-caret` line.
2. Right-click there and press `menu.item.canvas.text.edit.word_gap`.
3. The first `settings-page` line after the right-click must say `key=text`;
   the slider is the first setting on that page.

**Falsified** by routing `edit.word_gap` to no page in
`app::dispatch::settings::focus`: Settings opens at `key=general` and the check
fails at step 3.

**What it does not prove.**

- That a changed gap, once saved, changes which run the next click opens
  (Save hands every open document the new settings through
  `adopt_settings`; that route is not driven here).
