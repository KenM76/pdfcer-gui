# `ui-verify/checks/extract_pages_labels`

`extract_pages_keeps_or_drops_the_labels` — Pages ▸ Extract… on a window off
the desktop, driven only through `ScriptedPointer`, so it runs under
`--no-input`. Three launches:

- `labelled-pages.pdf` (labels i ii 1 2), range `1-2` typed, defaults:
  `pages=2 asked=2 labels=keep labels_dropped=0 label_ranges=1`, no delete.
- the same, *Keep the page labels* and *Delete afterwards* clicked:
  `labels=drop labels_dropped=1 label_ranges=0`, then
  `delete-pages page=0 n=2`.
- `pure-k-square.pdf`, which has no labels: the labels box is not drawn,
  `pages=1 label_ranges=0`, no delete.

## Steps

1. The save target is deleted, so an earlier run cannot pass for this one.
2. Clicks: `ribbon.mode.review`, `ribbon.tab.pages`, the collapsed
   `ribbon.group.pages.organise` when the band is narrow, then
   `ribbon.item.pages.extract`. The window must publish
   `extract-pages.extract`, and `extract-pages.labels` exactly when the file
   has labels.
3. A typed range is a click on `extract-pages.pages.range` and the text typed
   into that viewport; boxes are clicks on their regions.
4. `extract-pages.extract`; the picker is answered by
   `PDFCER_DIAG_SAVE_PATH`.
5. The `extract` line must carry every expected field, a `delete-pages` line
   must follow exactly when the box was ticked, and the file must start
   `%PDF-`.

`label_ranges=` and `labels_dropped=` are the engine's `AssembleReport`. What
the new file's pages read is checked by the unit test
`the_labels_choice_reaches_the_new_file`, which re-opens it.

## Falsification

- Ignoring the box (always `ExtractedPageLabels::Keep`) fails the second
  launch quoting `labels=keep`.
- Ignoring *Delete afterwards* fails the second launch: no page delete
  followed.

## What it does not cover

*All pages* and *This page* are not driven; a failed write declining the
delete is the action's contract, not driven.
