# `panels::properties::markup::anyannot` — what every annotation takes

Rows drawn under the markup section for every annotation outside
`set_markup_style`'s family (`Reach::TextAnnot` and `Reach::Neither`), plus the
redraw offer, which is drawn for any selected annotation.

## Opacity

`opacity_row` shows `/CA` as a percentage and raises
`AnnotAction::SetOpacity`, which calls `EditSession::set_annot_opacity`. The
verb writes the dictionary only and leaves the appearance bytes alone, so it
takes any subtype — a sticky note, a stamp, a text box, a caret. Clear removes
`/CA` (fully opaque). `set_markup_style`'s own opacity row serves the markup
family; the two never draw together.

## Marker colour

`marker_row` is drawn when the subtype is in `pdfcer_core::edit::MARKER_SUBTYPES`
(caret, file attachment, sound, screen). It raises
`AnnotAction::SetMarkerColour`, which calls `set_marker_style`. The current
colour is read by `markup::read_colour` off `/C`, because the engine's own
reader (`annot_author::read_color`) is crate-private; that workaround is filed
in the request channel.

## The redraw offer

`offer_row` reads `OpenDoc::redraw_offer`. When the last restyle of the
selected annotation was refused for a look pdfcer did not draw, it shows why
and one button; the button raises `RedrawOffer::retry()`, the refused call
with `redraw_as_plain` set. The offer belongs to one annotation: it is drawn
only while that annotation is selected, and the next successful restyle of it
clears it.

## Regions

`properties.markup.any.opacity`, `.marker` (plus `.marker.picker` while
open), `.redraw`.
