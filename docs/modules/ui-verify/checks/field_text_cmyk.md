# `a_fields_text_colour_is_stored_in_cmyk`

**Detects:** a form field's text colour picker that offers no CMYK, or a
colour chosen on its CMYK tab that is not written into the field's `/DA` as
CMYK.

## Drive

1. Open a copy of `fixtures/three-text-fields.pdf` with `FieldOne` selected,
   in Edit, with Properties open (`properties_pane::launch_on_field`).
2. Press the Text colour swatch (`properties.field_edit.text_colour`), then
   the picker's CMYK tab (`…text_colour.space.cmyk`).
3. Press Escape: closing the picker commits. Then Ctrl+S.

## Oracle

The saved copy's bytes. Every `/DA` in the fixture sets grey (`g`), which the
check confirms before driving, so a `/DA` ending in `k` after the save can
only be the edit's. Reading the file rather than a trace line means the
assertion is on what the engine wrote, not on what the picker showed.

A tab switch counts as a change: it is the operator choosing the space the
file stores. Black on the grey tab is written as the CMYK that draws the
same colour (a rich black, not `0 0 0 1`), ending in `k`. The engine writes
a rewritten `/DA` as a hex string `< … >`, so the reader decodes both string
forms and collapses whitespace before testing the last operator.
