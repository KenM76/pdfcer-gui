# `a_dimensions_colour_is_stored_in_cmyk`

**Detects:** a ce dimension's colour picker that offers no CMYK, or a colour
chosen on its CMYK tab that is stored as RGB.

## Drive

1. Open `fixtures/dimension-scaled.pdf` in Review, and select the ce dimension
   by its line.
2. Open the Properties tab if it is not the one showing, and tick the colour row's *Set here*
   (`dimension-properties.colour.set`).
3. Open the swatch (`dimension-properties.colour`) and press the CMYK tab
   (`…colour.space.cmyk`).
4. Press Escape. Closing the picker is what commits the colour.

## Oracle

The `dimension-style` trace line written after the tab press must carry
`colour=cmyk:…`. The action emits that line from the `StyleOverrides` it
hands to `EditSession::set_dimension_style`, so it records the space actually
sent to the engine, not the tab the picker showed.

Switching tabs counts as a change. A tab switch is the operator choosing the
space the file stores, so it commits even when no component moved.
