# `ui-verify/checks/restack`

`an_object_can_be_brought_forward_and_sent_to_back`, on
`fixtures/overlapping-boxes.pdf`: a 400×300 page painting a red box at
(60,60) 160×120, then a blue box at (160,100) 160×120, then a green box clear
of both. The point (190,140) lies inside red and blue; what a click there
selects is the topmost, read from the `geometry-draft` origin Properties
traces every frame.

PASS needs, in order:

1. Control: the overlap selects blue (`x=160.00 y=100.00`).
2. Red clicked at (80,80), then Ctrl+] (Bring forward): `restack-applied
   moved=0 indices=1` (`moved` names red's index before the press), and red is still selected, now as object 1.
3. The overlap now selects red.
4. Edit tab, Send to back: `moved=1 indices=0` (`moved` names red's index
   before the press), red selected as object 0.
5. The overlap selects blue again.

## Falsified

- No reselect after the restack → FAIL at step 2 (the selection named blue).
- Send to back mapped to `StackMove::Forward` → FAIL at step 4 (`moved=-`).

Send to back mapped to `StackMove::Backward` gives the same result on two
overlapping objects and is not distinguished here.

## What it does not prove

- Rendered pixels; the oracle is the engine's hit test via Properties.
- The `Scope` and `Entangled` disclosures.
- Undo and redo clearing the selection.
