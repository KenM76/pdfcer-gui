# `ui-verify/checks/widget_dash`

`a_widget_borders_dash_can_be_chosen` — a dashed field border's pattern is
offered in the Properties panel, and a pick reaches the widget's `/BS /D`.

# What it drives

A copy of `fixtures/all-field-kinds.pdf` (ignores `--pdf`), launched through
`properties_pane::launch_on_field` with `PDFCER_DIAG_SELECT_FIELD=TextOne`,
whose border is solid.

1. Before anything is pressed: no `widget-dash-shown field=TextOne` line — a
   solid border draws no dash row (R9).
2. `properties.widget_edit.border` opens the style picker;
   `properties.widget_edit.border.1` (Dashed) is pressed. Then
   `widget-dash-shown field=TextOne dash=dashed`: `/S /D` with no `/D` reads as
   the default dash.
3. `properties.widget_edit.dash` opens the dash picker;
   `properties.widget_edit.dash.1` (Long dash) is pressed. Then
   `edit-widget-applied field=TextOne … redrawn=yes` and
   `widget-dash-shown field=TextOne dash=long-dash`, read back off `/BS /D` on
   the next frame.

# Falsified

Dropping `with_border_dash` from the pick fails step 3 on `dash`; dropping the
`Dashed` gate in `widgetedit::border_rows` fails step 1.

# Driven off-screen

Scripted pointer, window at `-4200,-4200`, `spec.place = false`; it runs under
`--no-input`.
