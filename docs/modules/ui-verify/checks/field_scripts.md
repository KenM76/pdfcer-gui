# `ui-verify/checks/field_scripts`

`a_field_is_calculated_from_others` — a text field's Properties panel makes it
the sum of two other fields, then gives it a lowest allowed value. Each is
written as the engine's helper script and read back as one.

# What it drives

A copy of `fixtures/three-text-fields.pdf` in its output folder (ignores
`--pdf`), with `PDFCER_DIAG_SELECT_FIELD=FieldThree` and
`PDFCER_DIAG_INVOKE=mode.edit,file.properties`, so the panel opens on that
field.

1. Before anything is pressed: `field-scripts-read field=FieldThree … calculate=none`.
   The scripts section drew for the field, and the field holds no calculation.
2. `properties.field_scripts.tab.calculate`, `.calculated`,
   `.operand.FieldOne`, `.operand.FieldTwo`, `.apply`:
   `field-script-set field=FieldThree trigger=calculate applied=AFSimple_Calculate
   replaced=none position=0 entries=1 created=true`.
3. The panel re-reads on the edit epoch: `field-scripts-read … calculate=helper`.
   What was written classifies as the helper it was meant to be, not `Custom`.
4. `.tab.validate`, `.lowest`, `.lowest.value`, type `5`, `.apply`:
   `field-script-set … trigger=validate applied=AFRange_Validate`.

`created=true` is the oracle that the fixture had no `/CO` array and the
engine made one; `position=0 entries=1` that the field went into it. Each
region is found by scrolling the panel body until the region is declared,
because the section sits below the field's other properties.

# Driven off-screen

Scripted pointer, window at `-4200,-4200`, `spec.place = false`; it runs under
`--no-input`.
