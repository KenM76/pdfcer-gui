# `view_page_boxes_outlines_trim_and_bleed`

**Defect it guards.** View ▸ Display ▸ Page boxes is missing, outlines a box
the page does not write, puts an outline in the wrong place, or does not turn
off.

**Fixture.** `fixtures/page-box-sizes.pdf` (see `box_sizes.md`). Page 1 is
600 × 800 with a written bleed `[10 10 590 790]`, a written trim
`[20 20 580 780]` and no art box.

**Steps.** Launch with no invocation. The control: `page-boxes` is absent or
reads `on=0`. Press `ribbon.tab.view`, then `ribbon.item.view.page_boxes`. The
next line must read `on=1` with `1:bleed+trim` among its pages, and
`canvas.page-box.1.art` must not be declared. The `canvas.page-box.1.trim` and
`.bleed` regions must agree with the file: trim width ÷ bleed width = 560/580,
and the left, right and top insets each 10/580 of the bleed width, within
0.01. The ratios come from the fixture's numbers, so the check does not take
the program's own transform as its oracle. An in-app screenshot,
`page_boxes_view.on.png`, is kept as the pixel record. Press the item again; the next line must read
`on=0`.

**Falsified** by drawing the crop box as the trim (trim/bleed ratio 1.0343,
FAIL) and by a toggle that never turns off (no `on=0`, FAIL).

**What it does not prove.** Rotated pages, and the colours and stroke patterns
— the outlines' roles are kept distinct by `overlayroles`' unit test.
