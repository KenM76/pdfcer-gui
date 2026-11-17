# `ui-verify/checks/group_markups`

`grouped_markups_select_together`, on `fixtures/annots-align.pdf`: a square
and a circle markup on one page, each picked by a point on its outline.

PASS needs, in order:

1. Square clicked, circle Shift-clicked, Ctrl+G: `markup-group-applied
   members=1` (the circle, selected last, is the primary).
2. A plain click on the square, then on the circle: each traces
   `annot-group-select mates=1 selected=2`.
3. Ctrl+Shift+G: `markup-ungroup-applied`.
4. A plain click on the square traces `annot-select` and no
   `annot-group-select`.

## Falsified

- A build without the command → FAIL at step 1 (no `markup-group-applied`).

## What it does not prove

- That the saved file carries `/IRT` and `/RT /Group`; that is the engine's.
- That a drag moves both; `annot_group_drag` covers a multi-markup drag.
- The refusal sentences (a ce dimension, a link, a locked markup, two pages).
- Markup ▸ Arrange's two buttons; only the chords are pressed.
