# `a_turned_page_draws_the_right_place_past_the_region_wall`

**Defect.** On a page with `/Rotate`, past the zoom at which a whole-page
raster would exceed the pixmap limit, the canvas rasterizes only the visible
window. If that window is handed to the renderer in canvas space (y down,
turned) instead of PDF user space (y up, unturned), the picture jumps to
another part of the sheet and its width and height swap.

**Fixture.** `fixtures/turned-sheet-mark.pdf`, written by its
`PROVENANCE.py`: a letter sheet `/Rotate 270`, blank except a black square
`(60, 600)`–`(200, 740)` in user space. A blank sheet with one asymmetric mark
makes any misplaced window land on paper.

**Drive.** Off-screen, scripted pointer. Single page, actual size. Ctrl+wheel
at the square's right-edge midpoint `(200, 670)` until `canvas-halo` reports
`tier=region` (around 2000–2450 %; a letter sheet reaches the pixmap limit
near the top of the zoom range, so the tier is read before the zoom's rise is
judged).

**Assertions, in order.**

1. The window raster lands: the last `canvas-pos` shows `region=` equal to
   `want=`.
2. The landed window, in user space, covers both probe patches around the
   edge. This is the arm that catches the defect: the stale whole-page picture
   kept underneath (the canvas never blanks) still shows the square, so pixels
   alone pass on a misplaced window.
3. Pixels from the window's own frame: a patch 40 pt inside the edge is at
   least 0.90 ink, and the control 40 pt outside it at most 0.05. Inside and
   outside are found by mapping a point 1 pt either side of the edge through
   `CanvasMapping`, so the check writes no assumption about the turn.

# Falsified

Planting the upright formula in the `/Rotate 270` arm of
`PageFrame::canvas_to_user` ordered the window `(94, 347)`–`(157, 411)` with
the probe at `(200, 670)`: FAIL on arm 2. Pixels read 1.000 / 0.000 on that
planted build — arm 3 alone would have passed it. Restored: PASS.
