# `ui-verify/checks/table_nav`

`arrow_keys_walk_a_table_by_its_cells`, on `fixtures/cell-walk-table.pdf`: a
3×3 ruled table whose bottom-left word "Nut" is right-aligned in its cell, so
it sits under the middle column of the row above. Off-screen, scripted
pointer and keys, no OS input.

Edit mode, Edit tab, the text tool; a click inside "Nut" must open a caret in
run 12. One `ArrowUp`. PASS needs `text-caret-step … to_run=6`: "Bolt", the
cell above. `to_run=8` ("12", the word under the caret in the row above) is
the cell-blind model's answer and fails naming it.

A caret in any other run is a harness error, not a verdict: the run
constants are the fixture's page-text order.

Falsified: `canvas::textedit::blocks::model_of` built with
`EditableTextModel::recognize` and default options → FAIL, `to_run=8`.

## What it does not prove

- Down, or a multi-line cell. The engine's own tests cover the order inside
  `vertical_from_cell`.
- Home and End, which read lines, not cells.
