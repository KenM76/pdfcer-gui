# `ui-verify/checks/ribbon_order`

`ribbon_puts_clipboard_first_and_save_beside_file` — O313, driven off the
desktop through the scripted pointer.

The operator asked for the ribbon in a word processor's order: Clipboard at
the very left of Edit, Save next to File, and the highlighter's band filled
yellow. The check clicks each tab and requires:

1. Edit's groups, read left to right from their declared rects, start with
   `clipboard`;
2. File's start with `file`, then `save`;
3. the Markup tab's highlight button, in the window's own screenshot, holds
   at least `MIN_YELLOW` pixels of the wash yellow.

A group the band collapsed to fit the narrow off-desktop window is declared
as `….collapsed` and still counts, under its own id, at its own position.

Coloured icons are the default. With them off the band is hollow by design,
because a filled band in the plain set reads as the redaction bar; a run on a
settings file that turned them off fails step 3 for that reason, not for a
defect.

## Falsification

| control | expected verdict |
|---|---|
| `HIGHLIGHT` pointed at `ribbon.item.markup.underline` | FAIL at step 3, 0 yellow pixels (the highlight button reads 16) |
| the expected first Edit group renamed | FAIL at step 1 |
