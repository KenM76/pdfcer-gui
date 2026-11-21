# `ui-verify/checks/field_highlight`

`highlight_fields_tints_every_field` — **View ▸ Display ▸ Highlight fields
tints every fillable field on the canvas, a field the file stores no drawing
for included, and turning it off takes the tint away.**

# The fixture

`fixtures/need-appearances-fields.pdf` (see `undrawn_fields.md`): three `/Tx`
fields with no `/AP` and `/NeedAppearances true`. Every field is one the
shell's own wash cannot reach, so any tint seen is the engine's
`FieldHighlight`.

# The assertions, in order

1. **On the canvas.** At least three `form-box kind=text` lines.
2. **Reaches the render.** After the View tab and
   `ribbon.item.view.highlight_fields` are pressed, a `canvas-field-highlight`
   line reports `mode=1 drawn=1`: the texture on screen was drawn with the
   tint, read off its render key.
3. **Tinted.** Per field, the mean blue-minus-red over its box rises by at
   least `MIN_RISE` (8 levels) against the screenshot taken before the press.
4. **Removed.** A second press yields `mode=0 drawn=0`, and each field's
   blue-minus-red returns to within `MAX_DRIFT` (2 levels) of the first
   screenshot.

Export and print are kept out by a unit test,
`app::settings::tests::the_field_tint_reaches_the_canvas_and_no_export`:
`Settings::render_options`, the funnel those paths build from, carries no
`field_highlight`.

# Falsification

The request's `field_highlight` not copied into the render options: FAIL at 3
(the key still says tinted, so 2 holds; the pixels do not, blue-over-red
unchanged at 31.4). Pixels are the oracle for the tint; the trace only proves
which texture is on screen.
