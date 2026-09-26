# `pdfcer-gui/app/actions/forms/widget`

## Item notes

### `fn rotate`

# What the engine may not be able to do, and why it says so

`WidgetRotation::appearance_stale` carries a reason when the widget's baked
`/AP` could not be regenerated at the new angle. That is not a failure — the
rotation is written and the file is correct — but the box will draw at its
old orientation until something regenerates it, and an operator watching a
box refuse to turn deserves the sentence rather than a mystery.

`siblings_untouched` is surfaced for the same reason `edit_widget`'s is: a
field with three boxes has three orientations, and turning one is a
statement about one placement. Saying how many were left alone is what stops
*"I rotated the field"* meaning two different things.
