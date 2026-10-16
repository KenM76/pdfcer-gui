# `ui-verify/checks/field_delete_panel`

`the_properties_delete_buttons_take_every_selected_field` drives
`fixtures/all-field-kinds.pdf` (geometry in `field_delete_many.rs`) under
`--no-input`, with `mode.edit,file.properties` and a four-entry
`PDFCER_DIAG_SAVE_PATH` queue. It has no SKIP arm.

1. Click `TextOne`, Shift-click `RadioGroup #0`; require `form-field-toggled
   before=1`. `RadioGroup` has a second box that stays unselected, so both
   panel buttons are drawn.
2. Wheel the `dock.body.file.properties` body until
   `properties.form_field.delete_boxes` is inside it, click it, and require
   `delete-widgets-applied deleted=2/2 one_step=true`. Save: `TextOne` and
   `RadioGroup #0` gone, `RadioGroup #1` and `ComboOne` kept, eight boxes.
3. One `Ctrl+Z`, save: all ten boxes back.
4. Select the same two again, press `properties.form_field.delete_fields`,
   require `delete-fields-applied deleted=2/2 one_step=true`. Save: both
   radio boxes gone with `TextOne`, `ComboOne` kept, seven boxes.
5. One `Ctrl+Z`, save: all ten back.

Falsified by sending only the first selected field from the fields button:
step 4 fails on `deleted=1/1`.

Blind spots: the status-line sentences are not read; a certified document's
refusal is `field_delete_gate`'s; fields on two pages are not driven.
