# `ui-verify/fixture`

What the harness knows about the document it opened.

## Why this module exists at all

[`crate::coords`] needs one number the application does not have to supply:
the **page height in PDF points**, for the y-flip. That number is a
property of the *document*, not of the application, so the harness can read
it itself — and doing so keeps the document-space contract real rather than
aspirational. A harness that had to be told the page size by the program
under test would be trusting the program to describe the coordinate space
the harness is checking it against.

## The MediaBox scan, and its stated limits

[`page_geometry`] scans the raw file bytes for the first `/MediaBox
[a b c d]` and reads the size from it. That is a heuristic, and here is
exactly what it does and does not handle:

**Handled.** The common case, by a wide margin: a `/MediaBox` written as a
direct array in an uncompressed object header, which is what every producer
this project cares about emits for the page tree root or the first page.

**Not handled**, each of which returns `None` rather than a wrong answer:

* a `/MediaBox` that is an indirect reference (`/MediaBox 12 0 R`);
* a page whose box lives in an object stream (compressed);
* documents whose pages differ in size — the *first* box found wins, which
  is right for a single-page fixture and wrong for a mixed one;
* a non-zero origin (`/MediaBox [10 10 622 802]`) is handled for *size* but
  the harness's document coordinates are relative to the box origin, which
  for a shifted box is not the same as the PDF origin.

Returning `None` matters more than the list. A wrong page height produces a
click that is vertically mirrored about the page centre — it lands on the
page, hit-tests something plausible, and the resulting failure looks like a
selection bug. `None` produces a SKIP that names the missing number, and
[`crate::checks`] callers can then be told the size explicitly with
`--page-size`.

This is the same discipline the rest of the crate applies to coordinates:
**refuse rather than guess**, because a confident wrong coordinate is more
expensive than no coordinate.

## Item notes

### `fn text_chunk_point`

`index` is a position in the content stream, 0 for the first `Tj` and 5 for
the last; anything higher is a caller that has out-run the fixture and is
rejected rather than clamped, because a clamp would silently turn *aim at
chunk 7* into a second measurement of chunk 5.

# The two numbers, and why neither is an estimate

`y = 704 - 16 * index`: the baselines are 16 pt apart and the glyph band at
12 pt runs about 8.4 pt above each one, so `baseline + 4` is inside the
glyphs of that line and 12 pt clear of the next one. A midpoint between two
baselines hits nothing at all — [`text_block_target`] states the trap and
this is the function that avoids it.

`x = 100`: the **shortest** line of the six ends at x = 110, so one x serves
every chunk. A larger x would be further from the edges and would also miss
the last line entirely, which is the shape of an aim that measures five
chunks while reading as six.

### `fn workspace_root`

Every fixture this harness pins is named relative to the repository root,
because that is the only place a path can be written down that survives
being run from a different working directory - and `ui-verify` is run from
the repository root by hand, from `tools/ui-verify` by `cargo run`, and
from wherever a sweep script happens to be.

# Why it is here and not in each check


⇒ The eleven are left alone on purpose. Migrating them belongs in its own
commit - a mechanical edit to eleven unrelated modules, folded into a
defect repair, makes the repair unreviewable.

### `fn grip_gesture_target`

# Why this is a function and not three literals

Three checks are the same gesture with three different verbs -
[`crate::checks`]'s `resize_scales_a_shape`, `rotate_handle_turns_a_selection`
and `shift_constrains_a_resize`. Each selects a shape, presses one of the
eight grips and drags it. All three therefore need the same thing from the
document: **a point where clicking selects a single path object whose
selection outline is large enough on screen that its eight grips do not
overlap each other.**

That is a property of the *fixture*, not of any one check, which is why it
lives here beside the other thing the harness reads out of a document.


`resize_scales_a_shape` was moved into `sweep-full.sh`'s ALONE table with
`--doc-point 0,300,500` and passed. The other two kept taking the sweep's
shared `0,2000,320` and **both failed**, each with several paragraphs
naming three application functions as the likely cause. Every one of those
functions is correct. The aim point was wrong, and the knowledge of which
aim point works was written down in a shell script, attached to one of the
three checks that needed it.

⇒ **Knowledge a check cannot run without belongs beside the check, not in
the runner's arguments** - the runner's arguments are a compromise chosen
for the majority, and a check whose subject cannot exist under that
compromise does not report *my input is wrong*. It reports something
specific and believable about the program.

# The numbers

`fixtures/a1-titleblock.pdf`, page 0, **300, 500** in PDF user space -
a shape near the lower-left of the 2383.9 x 1683.8 pt sheet, clear of the
title block's own dense line work. Verified by `resize_scales_a_shape`
committing `resize-commit grip=SouthEast sx=1.1449 sy=1.2052` from it.

### `fn a1_text_target`

# What the document actually has there

The sheet's only ink is its title block. `pdfcer extract-text --json` reports
fourteen runs with a box, and this point is inside the one reading
`PROJECT NO`, whose box is `1831.2, 181.3 → 1872.2, 187.3`. It is a 6 pt
label on a 2383.9 × 1683.8 pt sheet, which is the property that makes it
worth aiming at: a click that lands on it is a click the operator could make
and the harness only just can.

# Two unrelated checks want this point for two unrelated reasons

`the_font_controls_are_live_on_the_drawing_you_open` wants **text under the
cursor**, because a click on blank paper is symptom-identical to a hit test
that does not work.

`text_annot_takes_the_keyboard_unclicked` does not care about text at all. It
wants **room to the right**: it drags a box 440 pt wide from the point, and
the sweep's shared `0,2000,320` puts the far corner at 2440, which is off a
2383.9 pt sheet. Every assertion in that check passed at the shared point and
the run was then reported SKIPPED on the geometry — a correct skip that reads
like a broken check.

⇒ The second requirement is the one a reader would not guess, so it is
written here rather than left in whichever check happens to be read first.
**Any replacement point must satisfy both**: on a glyph, and at least 440 pt
clear of the right edge and 190 pt clear of the top.

### `fn heavy_stroke_target`

# The property being asked of the document

`preview_width_ignores_zoom` (`OPERATOR_REQUESTS.md` **O184**) measures how
wide the drag preview is painted, and it asks two questions of that number:
that zoom does not change it, and that the one-pixel line-weight view does.
Both need **a stroke wider than one point**, because
`canvas::shapes::StrokeRule::preview_px` floors at one device pixel: on a
hairline the correct build answers `1.00` under every setting, and an
assertion that two settings differ would be asserting something true builds
do not do.

⇒ The requirement is not "a path" but **"a path with a heavy pen"**, and
that is a property of the fixture rather than of the check.


Driven first on the sweep's shared `a1-titleblock.pdf --doc-point
0,2000,320`, the check **SKIPPED**: nothing at that coordinate has stroked
geometry, so `canvas::shapes::for_move_subject` answers with an erase and no
shapes, and there was no preview stroke to measure at all.

⚠ The shared point is bare paper. `extract-text --json` puts the nearest
text run 132.9 pt away and the sheet's only ink is the title block in the
bottom right. A `marquee-mode` line read at that point counts what a BAND
returned, not what is under the cursor, so it cannot be quoted as evidence
about the coordinate.

Moved to [`grip_gesture_target`]'s `0,300,500` it **passed, at 1.00 px at
both zooms** - and that pass is half a measurement. Ruling 1 was real there
(the defect multiplies before the floor, so a 0.5 pt line would have read
4.74 px at 948 %), but ruling 2 could not be measured at all, because a
number already at the floor cannot be lowered to it.

⇒ On this fixture the same check reports **3.00 px at both zooms, and 1.00
px with line weights off**. Three separate numbers, a 13.5x magnification
between two of them, and every assertion has somewhere to fail.

# The numbers

`fixtures/polyline-nodes.pdf` is 535 bytes and one open path - a zigzag and
two Beziers, drawn `3.0 w`, which is the widest single-path pen in this
fixture set. `deeper_rung_delete` and `bezier_handle` already pin it, for
the unrelated reason that its tail has enough anchors to delete one from.
