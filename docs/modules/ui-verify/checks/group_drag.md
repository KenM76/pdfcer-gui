# `ui-verify/checks/group_drag`

`a_group_drags_as_one`, on `fixtures/annots-align.pdf`: the square and circle
markups of `group_markups`.

PASS needs, in order:

1. Square clicked, circle Shift-clicked, Ctrl+G: `markup-group-applied`.
2. Escape clears the selection.
3. One press on the square's outline, dragged 30 pt down, with nothing
   selected before the press.
4. Saved: both markups' tops are 30 pt lower.

The press lands on an unselected member, which is the gesture an operator
makes on a group: `several_annotations_drag_as_one_undo_step` selects every
member before its drag, so it cannot see a press that picks up only one.
