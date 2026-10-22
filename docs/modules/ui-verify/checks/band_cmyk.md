# `a_marks_outline_from_the_format_tab_is_stored_in_cmyk`

**Detects:** a Format ▸ Markup outline colour that offers no CMYK, or a
colour chosen on its CMYK tab that is not written into the annotation's `/C`
as four components.

## Drive

1. Open a copy of `fixtures/square-no-appearance.pdf` in Review.
2. Click inside the square (page point 270, 430) and require `annot-select`.
3. Open the Format tab if the swatch (`markup_band.stroke`) is not drawn,
   press it, then the picker's CMYK tab (`markup_band.stroke.space.cmyk`).
4. Press Escape: closing the picker commits. Then Ctrl+S.

## Oracle

The saved copy's bytes. The fixture's only `/C` is the square's RGB red,
which the check confirms before driving, so a four-component `/C` after the
save can only be the edit's. A `/C` array's length is its colour space
(§12.5.2): one is grey, three RGB, four CMYK. The tab switch alone is the
operator choosing the space the file stores, so it commits the CMYK that
draws the same red.

The fill swatch (`markup_band.fill`) is the same `swatch` helper and is not
driven separately.
