# `ui-verify/checks/field_multiselect`

`several_form_fields_align_as_one_undo_step` drives
`fixtures/all-field-kinds.pdf` (one 420 × 620 page) under `--no-input`, with
`mode.edit` and a two-entry `PDFCER_DIAG_SAVE_PATH` queue. It has no SKIP arm.

Fixture geometry it relies on: `TextOne [170 520 370 542]`,
`CheckOne [170 486 186 502]`, `ComboOne [170 412 370 434]`, radio members
`[170 452 186 468]` and `[260 452 276 468]`. Every widget's left edge is 170
except radio member B, and the strip x 160–169 is empty page.

1. Click TextOne; require `form-field-selected field=TextOne`.
2. Shift-click CheckOne, then ComboOne; require `form-field-toggled` naming
   each, with `before=1` then `before=2`. A Shift-click that replaces instead
   of adding traces `before=1` twice.
3. `Ctrl+Alt+8` (Align top); require `move-mixed-applied widgets=2/2
   one_step=true`. Save a copy; require all three saved tops at 542.
4. One `Ctrl+Z`; save a copy; require CheckOne's top back at 502 and
   ComboOne's at 434. This is the undo-step claim: unfolded moves leave the
   first-moved field at 542 after one Undo, whatever the trace says.
5. Drag a window marquee from (160, 510) to (290, 444); require
   `form-field-band caught=3 merge=replace`.
6. `Ctrl+Alt+4` (Align left); require `move-mixed-applied widgets=1/1`. Only
   radio member B is off the shared edge, so this proves the marquee's fields,
   not the earlier Shift-click set, are what Align reads.

Saved files are read with `checks::savedannots::listed_annots`, matching a
widget by `/T (name)` in its last body.
