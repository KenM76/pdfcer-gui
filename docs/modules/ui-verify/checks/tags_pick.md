# `the_tags_panel_outlines_the_picked_element`, `the_tags_panel_says_a_file_is_untagged`, `the_tags_panel_outlines_a_figure_with_no_text`

**Owns:** the Tags panel (O410): the tree, picking, and the outline.

**Drive.** `tags-split-table.pdf`, launched with `view.panel_tags`: one 2×2
table, row 1 on page 1 and row 2 on page 2, with a `/K` cycle from the last
cell back to the table. Press the first `TD`, `TR` and `Table` rows.

**Oracle.**

- `tags-panel state=read`: the cyclic file opens and reads.
- The cell: one box on page 1 holding `A1` at (72, 700) and ending before
  the second column at x = 300.
- The row: one box on page 1 holding the cell and reaching x = 300.
- The table: boxes on pages 1 and 2, and `canvas.tag-outline.1` drawn after.
- Untagged (`ruled-table.pdf`): `state=untagged` and no row regions.
- Figure (`tags-figure.pdf`: a caption, then a `Figure` whose content is
  a grey square at (100, 400)–(300, 600) with no text): one box on page 1
  within 1 pt of the square, and `canvas.tag-outline.1` drawn after.

**Not covered.** The outline's pixels; its region is read instead. A deep
tree's depth cap is covered by reading, not by a fixture.
