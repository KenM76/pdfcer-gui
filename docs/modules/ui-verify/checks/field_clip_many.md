# `ui-verify/checks/field_clip_many`

`several_form_fields_copy_cut_duplicate_as_one_undo_step` drives
`fixtures/all-field-kinds.pdf` under `--no-input`, with `mode.edit`, a
four-entry `PDFCER_DIAG_SAVE_PATH` queue and its own
`PDFCER_DIAG_CLIPBOARD_DIR`. It has no SKIP arm.

1. Click TextOne, Shift-click CheckOne (`form-field-toggled before=1`).
2. `Ctrl+D`; require `fieldclip-duplicate-many fields=2` and
   `paste-fields-applied pasted=2/2 one_step=true`. Save; require both
   originals and both duplicates, offset `(+10, -10)`, twelve boxes listed.
3. One `Ctrl+Z`; save; require both duplicates gone, ten listed.
4. Reselect both; `Ctrl+C` must trace `fieldclip-copy … fields=2`; `Ctrl+X`
   must trace `delete-widgets-applied deleted=2/2 one_step=true`. Save;
   require both boxes gone, eight listed.
5. Hover `(210, 580)`; `Ctrl+V`; require `pasted=2/2 one_step=true`. Save;
   require TextOne at `(110, 586)` and CheckOne at `(110, 552)`: the union
   of the two boxes centred on the pointer, so their relative layout is
   kept. Ten listed.
6. Click ComboOne alone; `Ctrl+C` must trace `fields=1`; `Ctrl+V` and
   `Ctrl+D` must each trace `paste-field-applied … widgets=1`, the one-field
   route.

Boxes are found by their saved `/Rect`'s lower-left corner.

Falsified by: duplicating the primary only (step 2), cutting the primary
only (step 4), centring each pasted field on the pointer separately (step
5), and sending a one-field `Ctrl+D` past the field rung (step 6).

Blind spot: the Read-mode refusal of `Ctrl+D` and the status-line
sentences are not asserted; a radio group in the batch is covered by unit
tests only.
