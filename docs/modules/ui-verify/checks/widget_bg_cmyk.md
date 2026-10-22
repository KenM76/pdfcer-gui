# `a_widgets_background_is_stored_in_cmyk`

**Detects:** a form box's background picker that offers no CMYK, or a colour
chosen on its CMYK tab that is not written into the widget's `/MK /BG` as
four components.

## Drive

1. Open a copy of `fixtures/three-text-fields.pdf` with `FieldOne` selected,
   in Edit, with Properties open (`properties_pane::launch_on_field`).
2. Press the Background swatch (`properties.widget_edit.background`), then
   the picker's CMYK tab (`…background.space.cmyk`).
3. Press Escape: closing the picker commits. Then Ctrl+S.

## Oracle

The saved copy's bytes. The fixture's widgets carry no `/MK`, which the check
confirms before driving, so a four-component `/BG` array after the save can
only be the edit's. A `/BG` array's length is its colour space (Table 189):
one is grey, three RGB, four CMYK.

An absent background opens the picker on black; the tab switch is the
operator choosing the space the file stores, so it commits the CMYK that
draws that black.

The same `mkcolour` row serves the border colour and the placement dialog's
two colours, so this one drive covers the shared path; the border's own
region is `properties.widget_edit.border_color`.
