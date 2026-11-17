# `ui-verify/checks/group_objects`

`page_objects_group_into_one_and_ungroup`, on `fixtures/overlapping-boxes.pdf`:
a 400×300 page painting a red box (object 0), a blue box over part of it
(object 1) and a green box clear of both (object 2). Red and green are not
neighbours in paint order, so grouping them carries red past blue.

PASS needs, in order:

1. Control: the overlap (190,140) selects blue (`x=160.00 y=100.00`).
2. Red clicked, green Shift-clicked, Ctrl+G: `group-applied members=2 index=1`.
3. A click on green and a click on the overlap each select object 1, a form
   XObject (`properties-panel object=1`), and neither box alone.
4. Edit tab, Ungroup: `ungroup-applied first=1 count=2`.
5. A click on green selects green alone (`x=20.00 y=230.00`).

## Falsified

- A build without the command → FAIL at step 2 (no `group-applied`).

## What it does not prove

- Rendered pixels; the oracle is the engine's hit test via Properties.
- The refusal sentences, and the depth sentence a non-adjacent group owes.
- Undo and redo of either verb.
- Ungroup on a placed drawing that pdfcer did not group.
