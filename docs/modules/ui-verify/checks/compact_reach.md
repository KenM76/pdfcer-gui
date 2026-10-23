# `ui-verify/checks/compact_reach`

`size_reduction_is_on_the_ribbon_and_the_tab_menu` — **File ▸ Save's Reduce
file size opens the compact window at the Standard preset, and a document
tab's right-click Fit to a size opens it with the size target on; each reaches
the engine.**

## What it drives

Opens `fixtures/noisy-photo.pdf` (one 300 ppi noise picture) in Edit mode.
Opens the File tab, requires `ribbon.item.file.fit_size` and
`ribbon.item.file.reduce_size` on the tab or in its overflow, presses Reduce
file size, waits for the window's first `compact-recompressed` line, presses
Cancel (`compact.cancel`), right-clicks the document tab (`doc-tab.0`),
requires the three rows `menu.item.document.tab.file.{save_compacted,
reduce_size, fit_size}`, presses Fit to a size and waits again.

## What it asserts, in order

| # | assertion | what its failure means |
|---|---|---|
| 1 | both ribbon items are drawn | a command is unplaced or unregistered |
| 2 | Reduce's line says `preset=standard`, `resized ≥ 1`, `target=none` | the window opened at the wrong starting point, or the preset never reached the engine |
| 3 | the tab menu has all three rows | the right-click route is missing |
| 4 | Fit's line says `preset=lossless`, `target=met`, `0 < bytes ≤ max` | the window opened without the target, or the search failed |

## Why this fixture

Standard scales pictures above 225 ppi, so a 300 ppi picture separates it
from Lossless (`resized=0`). Noise does not compress, so the plain copy stays
above the opening target and only a lossy step meets it.

## Falsification

Arm 2 is red against a control build whose `Start::choices` answers Lossless
with no target for every starting point (`resized=0` after Reduce). The check
stops at its first failure, so arm 4 is not reached by that control.
