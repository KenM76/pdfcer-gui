# `clicked_text_takes_an_opacity`

**Detects:** clicked page text with no opacity field, or an opacity typed
there that the page and the saved file do not then carry.

## Drive

1. Launch on a copy of `fixtures/paragraph.pdf` in Edit, window off the
   desktop, scripted pointer; raise Properties if needed.
2. Click inside the first line's second word (`word_styles::IN_WORD`), which
   selects that text object.
3. Press `properties.textobject.opacity` (a press opens a drag field with its
   text selected), type `40`, Enter. Then Ctrl+S.

## Oracle

- Before the edit, `text-opacity-shown` must read `alpha=1.000`: the fixture
  writes no `gs`, so its text is opaque and readable. Any other value is the
  reader, not the fixture.
- After it, the next `text-opacity-shown` must read `alpha=0.400`. The line is
  re-traced from the engine's own re-read of the edited page, so it proves the
  write reached the content and reads back, not only that the field moved.
- The saved file must hold a numeric `/ca 0.4`.
