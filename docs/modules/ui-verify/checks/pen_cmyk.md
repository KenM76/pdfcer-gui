# `a_mark_drawn_with_a_cmyk_pen_is_stored_in_cmyk`

**Detects:** a Markup ▸ Style colour whose *More colours* picker offers no
CMYK, or a mark drawn after choosing CMYK that is not written into the
annotation's `/C` as four components.

## Drive

1. Open a copy of `fixtures/square-no-appearance.pdf` in Review.
2. Open the Markup tab if the pen swatch (`markup.style.ink`) is not drawn,
   press it, expand *More colours* (`markup.style.ink.more_colours`), and
   press its CMYK tab (`markup.style.ink.more.space.cmyk`).
3. Require the last `markup-pen` line to read `ink=cmyk:…`, then Escape.
4. Choose the rectangle tool, drag from page point (100, 600) to (220, 680),
   require `add-markup`, then Ctrl+S.

## Oracle

The saved copy's bytes, read as in `band_cmyk`: the fixture's only `/C` is the
square's RGB red, confirmed before driving, so a four-component `/C` after the
save can only be the new rectangle's. The `markup-pen` line separates "the tab
did not reach the pen" from "the pen did not reach the file".

The highlighter swatch is the same `chip` helper and is not driven separately.
