# `dialogs::handsign` — the *Sign here* window

Opens on a click on an unsigned signature box. Two tabs, **Draw** and
**Type**; press **Place signature** and the window closes with the signature
in the box. It opens on the tab last placed from.

## Draw

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

## Type

- Drawn only when `handsign::typed::faces()` is non-empty (R9).
- A name field, a **Style** choice when more than one face is installed, and a
  preview in the chosen face. The preview registers the face with egui under
  its own family name; a family is used only once egui reports it defined,
  because egui applies `add_font` on the next frame and panics on a family it
  does not know.
- The name's coverage is checked against the face whenever the name or face
  changes; a missing character greys **Place** with the reason, in the danger
  role, rather than leaving the refusal to the placement.
- Greyed, with the reason on hover, on a page shown turned: the engine's text
  verb writes along the page's own x axis only.

## Both

- *Remember my signature on this computer* keeps a copy of whichever was
  placed; unticked, placing deletes both copies.

The window writes nothing: it pushes `FieldAction::HandSign` (carrying a `Signature`, drawn or typed) or
`FieldAction::SignWithId`.
