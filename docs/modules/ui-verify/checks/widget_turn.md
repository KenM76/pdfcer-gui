# `ui-verify/checks/widget_turn`

Two checks on `fixtures/widget-turn.pdf`, driven with
`PDFCER_DIAG_INVOKE=mode.edit,file.properties`:

- `a_fields_box_tilts_by_its_rotate_grip`
- `a_fields_opacity_row_writes_it`

The fixture is one 400 x 400 page with one text field, `Name`, at
`/Rect [100 180 300 220]`. Its normal appearance has a `/BBox` and no
`/Matrix`, so it is upright and `turn_widget` can tilt it.

# Tilt

1. A click at the field's centre (200, 200) requires `form-field-selected`
   naming `Name`, and the `canvas.rotate-handle` region to be published.
2. Drag the handle's centre about (200, 200) by 30 degrees anticlockwise as
   the operator sees it (window y points down), in 12 steps.
3. Require `turn-widget-applied` with `changed=true` and `now` within 3
   degrees of 30. A sign error lands near −30 and fails.
4. Scroll `properties.widget_edit.upright` into view and press it. Require a
   second `turn-widget-applied` with `now=0.00`.

A screenshot is taken after the drag.

# Opacity

1. Select the field as above.
2. Type `40` into `properties.widget_edit.opacity` and press Enter.
3. Require `edit-widget-applied` with `opacity=0.4`.

# Driven off-screen

The checks use the scripted pointer, with the window off every monitor and
`spec.place = false`. They run under `--no-input`.
