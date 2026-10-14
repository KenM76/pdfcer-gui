# `ui-verify/checks/field_group_drag`

`several_form_fields_drag_as_one_undo_step` drives
`fixtures/all-field-kinds.pdf` under `--no-input`, with `mode.edit` and a
two-entry `PDFCER_DIAG_SAVE_PATH` queue. It has no SKIP arm.

1. Click TextOne; Shift-click CheckOne and ComboOne, requiring
   `form-field-toggled before=1`, `2`.
2. Press on CheckOne, which is not the primary, and drag it down. Require
   `move-mixed-applied gesture=drag widgets=3/3 one_step=true`. With
   `fieldgroup::body_at` off, or with `dragroute` not asking `fieldgroup`,
   no `move-mixed-applied` follows.
3. Save a copy; require each box's `/Rect` top lowered by the drop.
4. One `Ctrl+Z`; save a copy; require every top back. An unfolded move that
   still traces `one_step=true` fails here, on TextOne's top.
5. Escape, click TextOne alone and drag it; require `widget-drag` and no
   `move-mixed-applied`. This stands in for `dragging_a_form_field_moves_it`,
   which SKIPs under `--no-input`. A group that claims one field fails here.

Blind spot: the ghosts are not photographed, and boxes selected on another
page are not exercised.
