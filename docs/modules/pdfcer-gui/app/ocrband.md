# `app::ocrband` — the View ▸ Display blend slider

One control, drawn for one custom-item kind
([`crate::shell::manifest::OCR_BLEND`]), reporting a new position for
`ViewState::ocr_overlay`.

## Why it reports rather than writes

The ribbon's custom-item closure holds the open document through a shared
borrow — it has to, because every other control in the band reads it to
draw itself. So this returns the new position and
[`crate::app::PdfcerApp::ribbon_band`] applies it after the band is drawn,
in the shape `file.recent`'s parked choice already uses.

## Why there is no command behind it

It edits what is on screen: no document, no undo entry, nothing that
reaches the file. That is [`crate::shell::manifest::COLOUR_SWATCH`]'s case
exactly, and that constant's own note carries the ruling. R8 is met by the
item's `shown_when`, which is the *toggle's* selected-condition — see
[`crate::shell::manifest::OCR_BLEND`].

## It commits continuously, and that is the point

No draft, no commit-on-release, unlike the Format band's number fields.
Those commit a document edit and an intermediate value would be an undo
entry nobody asked for; this one changes a blend the operator is *looking
at* while dragging, and a slider that only took effect on release would be
a slider he could not aim. Nothing is re-rasterized — the veil is a tint on
the cached texture and the text is vector — so the cost of a frame mid-drag
is a repaint, not a render.
