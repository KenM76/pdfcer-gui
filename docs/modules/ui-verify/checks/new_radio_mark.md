# `ui-verify/checks/new_radio_mark`

`a_new_radio_buttons_mark_is_chosen` and `a_new_check_boxs_mark_is_chosen` —
the new-field window's Mark picker authors a radio button, or a check box,
drawn with a star, and the star is read back from the file.

# What it drives

A copy of `fixtures/layer-assign.pdf` (ignores `--pdf`; no fields, so the
first group is named `Group1` and the first check box `Check Box1`),
off-screen at `-4200,-4200`, scripted pointer,
`PDFCER_DIAG_INVOKE=mode.edit,file.properties,edit.form_radio_button` and
`PDFCER_DIAG_SELECT_FIELD=Group1`; the check-box check invokes
`edit.form_check_box` and selects `Check Box1`. The trace reader splits only
at ` key=`, so the name's space reads whole.

1. A drag at (560, 480)–(590, 450) on page 0 opens the window:
   `form-field-open kind=Radio … name=Group1`.
2. `dialog.form_field.mark`, then `dialog.form_field.mark.2` (Star, entry 2 of
   `CHECK_STYLES`), then `dialog.form_field.accept`.
3. The seam selects `Group1` once it exists; Properties reads its `/MK /CA`:
   `widget-mark-shown field=Group1 widget=0 mark=H`.

The oracle reads the written widget, not the draft, so a style chosen and
dropped before `add_radio_button` reads `l`, the dot.

# Falsified

Deleting `spec.style = draft.radio_style` in the author path reads `mark=l`.
Narrowing `checkmark::applies` to check boxes leaves no `widget-mark-shown`
line for the radio group. Deleting `spec.style = draft.check_style` reads
`mark=4`, the tick.
