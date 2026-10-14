# `ui-verify/checks/field_delete_many`

`several_form_fields_delete_as_one_undo_step` drives
`fixtures/all-field-kinds.pdf` (see that fixture's `PROVENANCE.py` for its ten
widgets) under `--no-input`, with `mode.edit` and a two-entry
`PDFCER_DIAG_SAVE_PATH` queue. It has no SKIP arm.

1. Click TextOne; Shift-click CheckOne and both buttons of `RadioGroup`,
   requiring `form-field-toggled before=1`, `2`, `3`. The two radio buttons are
   one field, so the second is found only if the first delete's renumbering is
   accounted for.
2. Press Delete; require `delete-widgets-applied deleted=4/4 one_step=true`.
   Deleting lowest index first traces `deleted=2/4`; a Delete reading the
   primary only traces no `delete-widgets-applied`.
3. Save a copy; require the four boxes absent from the page's `/Annots`, the
   combo box present, six boxes listed. Boxes are found by their `/Rect`'s
   lower-left corner, because a radio button carries no `/T` of its own.
4. One `Ctrl+Z`; save a copy; require all four back and ten listed. Unfolded
   deletes bring back only the last.

Blind spot: the status-line sentence is not asserted, and nor is a refusal
part-way.
