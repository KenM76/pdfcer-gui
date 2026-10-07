# `ui-verify/checks/dimension_text_select`

`clicking_a_dimensions_text_selects_it`, for O288 item 7.

## What it drives

`fixtures/dimension-scaled.pdf`, Review mode, the Select tool at rest. It
selects the ce dimension by clicking its line (trying both sides of the 30 pt
standoff), reads the `canvas.dimension-label` region the selected dimension
publishes (the engine's `label_quad`), clicks blank paper to clear the
selection, then clicks the text 2 pt inside its edge farthest from the line.

## Verdict

PASS when that click is followed by `annot-select kind=CeDimension`. FAIL
when nothing is selected.

## Falsified

With the label claim disabled in `selection::annot::under_pointer`, the click
10.3 pt from the line selects nothing and the check FAILs; restored, it
PASSes.

## What it does not prove

The non-linear kinds' text (same claim, through the same `label_quad`), and
a press-drag that starts on an unselected dimension's text.

## Shared driving

`checks/dimdrive.rs` holds the launch, region press and page click the
`dimension-scaled.pdf` checks share.
