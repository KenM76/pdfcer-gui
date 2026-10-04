# `ui-verify/checks/field_extras`

`a_text_fields_extras_reach_the_file` — a text field's Properties panel turns
off scrolling and spell-checking, gives the field an export name, and makes it
a file-select field; each change is read back from the document.

# What it drives

A copy of `fixtures/three-text-fields.pdf` in its output folder (ignores
`--pdf`), with `PDFCER_DIAG_SELECT_FIELD=FieldOne` and
`PDFCER_DIAG_INVOKE=mode.edit,file.properties`, so the panel opens on that
field. The fixture's fields carry no `/Ff` and no `/TM`.

1. Before anything is pressed:
   `field-extras-read field=FieldOne scroll=1 spell=1 file=0 sent=1 export=-`.
2. `properties.field_edit.scroll` → `scroll=0`.
3. `properties.field_edit.spell_check` → `spell=0`.
4. `properties.field_edit.export_name`, type `qty_total`, Enter →
   `export=qty_total`.
5. `properties.field_edit.file_select` → `file=1`, the three earlier values
   unchanged.
6. `properties.field_edit.sent` → `sent=0`, with `file=1` and the export name
   unchanged, and `properties.field_edit.file_select.note` declared.

The read line is drawn from the field as the document holds it after the edit
epoch, not from the panel's draft, so each step proves the change reached
`edit_field` and the file. Steps 5 and 6's unchanged values prove the flag edits do
not clobber one another. Each region is found by scrolling the panel body
until it is declared (`properties_pane::press`).

# Falsified

Inverting the NoExport bit the Sent with the form checkbox writes fails
step 6 with `sent=1`.

# Driven off-screen

Scripted pointer, window at `-4200,-4200`, `spec.place = false`; it runs under
`--no-input`.
