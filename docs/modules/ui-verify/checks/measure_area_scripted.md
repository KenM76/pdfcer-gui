# `ui-verify/checks/measure_area_scripted`

`an_area_measures_the_region_it_encloses` — the Area tool beside Perimeter
traces a closed outline, shows the area it encloses while it is traced,
commits a closed perimeter ce dimension labelled with that area, and the
dimension's right-click menu switches the label to its perimeter and back
(O280). The window is off the desktop and driven only through
`ScriptedPointer`, so the check runs under `--no-input`.

## Steps

1. Open `fixtures/blank-overhang.pdf`; click `ribbon.mode.review`,
   `ribbon.tab.measure`, then `ribbon.item.measure.area`, opening collapsed
   `ribbon.group.measure.*` buttons until it shows.
2. On the blank right half, a square of half side 0.08 page widths centred at
   (0.70, 0.35) of the page: click its four corners. The last
   `measure-perimeter-vertex` line's `area_pt2` must be the square's area
   within 2 %.
3. Hover back over the first corner. The last `measure-area-readout` line must
   carry a non-empty `text` — the engine's formatted area of the closed
   outline. The in-app screenshot `measure-area-scripted-readout.png` shows it
   beside the pointer.
4. Click the first corner: `measure-finish` must carry `via=close-ring
   kind=area`. Click inside the square to place the label; `add-dimension`
   must increase.
5. Arm Select from `ribbon.item.view.tool_select` and click the bottom edge.
   Right-click it: the menu must offer
   `menu.item.canvas.dimension.format.dimension_perimeter` and not
   `…dimension_area`. Click it: `dimension-area … area=0`,
   `set-dimension-area` and `dimension-area-applied area=0` must follow, and
   the applied `text` must differ from the readout's.
6. Right-click again: the menu must offer `…dimension_area` and not
   `…dimension_perimeter`. Click it: the applied `text` must equal the
   readout's from step 3 — the area shown while tracing is the area committed.

## Falsification

Making `dimdisplay::switch` raise `SetArea { area: true }` for both commands
fails step 5 with `area=1`. Dropping `MeasureKind::Area` from the click arm of
`canvas::measure` fails step 2 with no vertex line.

## What it does not cover

The Properties panel's Perimeter / Area radio raises the same action and is
not driven. A double-click ending and Finish both close the ring; only the
click on the first corner is driven. That an open perimeter ce dimension is not
offered the switch is not driven; the engine refuses it
(`AreaNeedsClosedOutline`) if it were.
