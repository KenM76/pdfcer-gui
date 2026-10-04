# `ui-verify/checks/check_mark`

`a_check_boxs_mark_can_be_chosen` — the Properties panel's Mark picker turns
another program's check box from a tick into a star, and the box is redrawn.

# What it drives

A copy of `fixtures/all-field-kinds.pdf` (ignores `--pdf`), launched through
`properties_pane::launch_on_field` with `PDFCER_DIAG_SELECT_FIELD=CheckOne`.
`CheckOne` carries `/MK /CA (4)`, ZapfDingbats' tick, and an appearance another
program drew.

1. Before anything is pressed: `widget-mark-shown field=CheckOne mark=4`.
2. `properties.widget_edit.mark` opens the picker; `properties.widget_edit.mark.2`
   (entry 2 of `CHECK_STYLES`, Star) is pressed.
3. `edit-widget-applied field=CheckOne … redrawn=yes`, then
   `widget-mark-shown field=CheckOne mark=H` — `H` is ZapfDingbats' star, read
   back from the widget's `/MK /CA`.

`redrawn=yes` is the assertion that matters: without
`ForeignAppearance::Replace` the engine records the character and keeps the
other program's artwork, so the box would still show a tick.

# Falsified

Removing `with_foreign_appearance(ForeignAppearance::Replace)` from the pick
fails step 3 on `redrawn`.

# Driven off-screen

Scripted pointer, window at `-4200,-4200`, `spec.place = false`; it runs under
`--no-input`.
