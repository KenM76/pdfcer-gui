# `the_hide_arrow_sits_clear_of_the_tabs`

**Defect it guards.** O302: an open dock's hide arrow sits on the dock's inner
edge (the one facing the document), and no tab or tab-bar "more" button is drawn
under it. At the trailing end of the right dock the "more" button covered it.

**Fixture.** `fixtures/a1-titleblock.pdf`, launched off-screen with
`mode.edit,view.reset_layout`, so both docks are open at their defaults.

**Steps.**

1. For the left and the right dock: `dock.{side}.collapse` is drawn, and its
   gap from the inner edge of `dock.{side}` is 0 to 12 pt (the side splitter
   plus slack).
2. No live `dock.tab.*` or `*.overflow` rect overlaps either arrow.
3. Press the right arrow: `dock.right.rail` must be drawn after the press.

**Falsified** by moving the right chevron back to the dock's trailing end: step 1
reports the right arrow some 340 pt from the inner edge.

**What it does not prove.** That the arrow is clear in every arrangement;
`dock::stack`'s unit tests cover the bar giving way on each side.
