# `dialogs::handsign` — the *Sign here* window

Opens on a click on an unsigned signature box. Draw in the pad, press
**Place signature**; the window closes and the signature is in the box.

- The pad keeps strokes in pad-local points. The first point of a drag comes
  from `press_origin`, because egui reports a drag only after the pointer has
  moved past its threshold, and without it every stroke loses its first few
  points. Points closer than 0.75 to the last are dropped; a click adds a dot.
- Strokes draw in the palette's text colour on its surface colour: the pad is
  a widget, not the document. The document ink colour is chosen at placement.
- **Use my last signature** restores this session's last mark, else the
  remembered one, scaled into the pad.
- *Remember my signature on this computer* starts ticked iff a remembered copy
  exists, so the operator who opted in once is not asked again.
- **Use a digital ID (certificate) instead…** is drawn only when `file.sign`
  is registered (R8); it hands the field to the existing certificate flow.
- **Place signature** is greyed until the mark has extent, explained on hover.

The window writes nothing: it pushes `FieldAction::HandSign` or
`FieldAction::SignWithId`.
