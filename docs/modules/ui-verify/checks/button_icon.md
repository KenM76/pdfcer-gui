# `a_push_button_takes_a_picture`

**Defect it guards:** a push button cannot be given a picture, have its
caption placed beside one, or lose it, from anywhere in the shell.

## What it drives

Off-screen, scripted pointer, `PDFCER_DIAG_INVOKE=mode.edit,file.properties`
and `PDFCER_DIAG_SELECT_FIELD=PushOne` on a copy of `all-field-kinds.pdf`,
whose push button carries another program's artwork. Two launches:

1. The picker seam answers a 16×16 PNG the harness encodes. Press *Choose
   picture…*, open the caption combo, press its `/TP 1` entry, press *Remove
   picture*. Each region is reached by wheel-scrolling the Properties panel
   until it is declared.
2. The picker seam answers `fixtures/vector-art.svg`. Press *Choose
   picture…*.

## Oracles

| step | requires |
|---|---|
| before | `button-icon-shown field=PushOne icon=absent` |
| Choose | a new `edit-widget-applied`, then `icon=present`, and `redrawn=yes` |
| Picture only | `position=1` and `redrawn=yes` |
| Remove | `icon=absent` and `redrawn=yes` |
| SVG | no new `edit-widget-applied`, and `button-icon-declined reason=svg` |

`redrawn=yes` is required because the fixture's artwork is foreign: kept, it
hides the change while every other line says it applied. The oracle is the
trace only; a window capture reads the desktop, which an off-screen drive
must not.

## Falsification

Setting `with_foreign_appearance(ForeignAppearance::Keep)` on the Choose
edit fails the Choose row on `redrawn`.
