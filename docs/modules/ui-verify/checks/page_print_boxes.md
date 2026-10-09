# `resizing_a_sheet_says_which_print_boxes_reach_past_it`

**Defect it guards.** A resize writes only `/MediaBox`. A page's own
`/BleedBox`, `/TrimBox` and `/ArtBox` stay as written, and ISO 32000-2
§14.11.2.1 has every reader intersect each with the new media box. Shrinking a
sheet under its trim box therefore changes the finished size a print shop
cuts to, and nothing on the canvas shows it. The engine reports each box the
new sheet does not contain (`MediaBoxChange::bleed_box_outside`,
`trim_box_outside`, `art_box_outside`); unread, the operator is told nothing.

**Fixture.** Written by the check: one 600 × 800 sheet with
`/BleedBox [0 0 600 800]`, `/TrimBox [10 10 590 790]` and
`/ArtBox [20 20 120 120]`. A6 portrait holds the art box and neither of the
others, so the art box is the control.

**Steps.** Launch in Edit, open Pages ▸ Sheet size, choose A6 portrait, press
the commit button. `page-size-applied` must say `bleed_outside=1
trim_outside=1 art_outside=0`, and the `page-size-changed` line's sentences
must name the bleed box and the trim box and not the art box.

Falsified: the trim sentence keyed on `art_box_outside` instead of
`trim_box_outside` fails it (the trim box goes unnamed).

**What it does not prove.** That the boxes are left byte-identical in a saved
copy (the engine's own test); several sheets at once, which goes through the
same count.
