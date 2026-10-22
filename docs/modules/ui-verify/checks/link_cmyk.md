# `a_links_border_colour_is_stored_in_cmyk`

**Detects:** a link border colour that offers no CMYK, or a colour chosen on
its CMYK tab that is not written into the link's `/C` as four components.

## Drive

1. Open a copy of `fixtures/link-edit.pdf` in Edit with Properties shown.
2. Click inside the link (page point 200, 200) and require `annot-select`
   with `subtype=Link`. Its border is visible, so the colour swatch is drawn.
3. Open the swatch (`properties.link.colour`), press its CMYK tab
   (`properties.link.colour.space.cmyk`), and press Escape: closing commits.
4. Press *Apply border*; require `set-link-border-applied` with
   `colour=cmyk:…`. Then Ctrl+S.

## Oracle

The saved copy's bytes, read as in `band_cmyk`: the fixture's only `/C` is the
link's RGB blue, confirmed before driving, so a four-component `/C` after the
save can only be the edit's. The trace's `colour=` separates "the tab did not
reach the form" from "the form did not reach the file".

The Add link window uses the same `LinkForm::border_ui` and is not driven
separately.
