# `ui-verify/checks/check_mark`

`a_check_boxs_mark_can_be_chosen` and `a_radio_buttons_mark_can_be_chosen` —
the Properties panel's Mark picker turns another program's check box from a
tick, and its radio button from a dot, into a star, and the widget is redrawn.

# What it drives

A copy of `fixtures/all-field-kinds.pdf` (ignores `--pdf`), launched through
`properties_pane::launch_on_field` with `PDFCER_DIAG_SELECT_FIELD` naming the
field. Each carries an appearance another program drew:

| Check | Field | `/MK /CA` before |
|---|---|---|
| `a_check_boxs_mark_can_be_chosen` | `CheckOne` | `4`, ZapfDingbats' tick |
| `a_radio_buttons_mark_can_be_chosen` | `RadioGroup` | `l`, ZapfDingbats' dot |

1. Before anything is pressed: `widget-mark-shown field=<field> mark=<before>`.
2. `properties.widget_edit.mark` opens the picker; `properties.widget_edit.mark.2`
   (entry 2 of `CHECK_STYLES`, Star) is pressed.
3. `edit-widget-applied field=<field> … redrawn=yes`, then
   `widget-mark-shown field=<field> mark=H` — `H` is ZapfDingbats' star, read
   back from the widget's `/MK /CA`.

`redrawn=yes` is the assertion that matters: without
`ForeignAppearance::Replace` the engine records the character and keeps the
other program's artwork, so the widget would still show its old mark.

# Falsified

Removing `with_foreign_appearance(ForeignAppearance::Replace)` from the pick
fails step 3 on `redrawn`. Narrowing `checkmark::applies` back to check boxes
fails the radio check at step 1.

# Driven off-screen

Scripted pointer, window at `-4200,-4200`, `spec.place = false`; it runs under
`--no-input`.
