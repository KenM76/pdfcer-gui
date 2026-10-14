# `ui-verify/checks/annot_group_drag`

`several_annotations_drag_as_one_undo_step` drives `fixtures/annots-align.pdf`
(see `annot_multiselect.md` for its geometry and `annot_delete_many.md` for
the locked square `/NM (LO)`) under `--no-input`, with `mode.edit` and a
two-entry `PDFCER_DIAG_SAVE_PATH` queue. It has no SKIP arm.

1. Require `status-group:edit-disclosure` not yet declared.
2. Click the square; Shift-click the circle, the link and the locked square,
   requiring `annot-toggled after=2`, `3`, `4`. The locked square is the
   primary, so a press on the circle can start a move only through the group
   route.
3. Drag from the circle 30 pt down; require `move-mixed-applied gesture=drag
   annots=3/4 one_step=true`, then the disclosure declared (the locked one is
   said). A drag routed to the primary alone traces no `move-mixed-applied`.
4. Save a copy; require the square, circle and link tops 30 pt lower and the
   locked square's unchanged.
5. One `Ctrl+Z`; save a copy; require every top back. Unfolded moves bring
   back only the last.

Blind spot: the per-member ghosts drawn during the drag are not asserted.
