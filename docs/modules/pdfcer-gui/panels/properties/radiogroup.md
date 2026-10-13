# `pdfcer-gui/panels/properties/radiogroup`

The Radio group row of a selected radio button: a name box seeded with the
button's group, a list of the document's other radio groups, and Move to
group. Enter or the button pushes `FieldAction::JoinRadioGroup`; a new name
makes a new group. Move is greyed, with the reason on hover, when the name is
empty or is already the button's group.

`rename_joins` makes the field rename behave as Acrobat's: renaming a radio
to another radio group's name joins that group rather than being refused as a
name collision.

Regions: `properties.form_field.radio_group.name`,
`properties.form_field.radio_group.move`.
