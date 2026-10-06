# `dialogs::handsign` — the *Sign here* window

Opens on a click on an unsigned signature box. Three tabs, **Draw**, **Type**
and **Picture**, above a preview of the box showing where the signature will
land; press **Place signature** and the window closes with the signature in
the box. It opens on the tab last placed from this run, else on the first
kind with a remembered copy: drawn, typed, picture.

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

## Picture

- Greyed, with the reason on hover, on a page shown turned: `add_image`
  places along the page's own axes.
- **Choose picture…** opens the image picker the Insert Image command uses
  (`pick_image_source`), so the same formats and the same scripted-path hook
  apply. A file that does not import is reported in the danger role with the
  importer's sentence.
- *Make white see-through* is ticked for a new picture when it can apply, and
  greyed with the reason when it cannot (`handsign::picture`).
- The picture in the preview is the engine's own drawing of it
  (`handsign::picture::preview`), redrawn when the clear-white choice
  changes, so the preview and the page agree.

## Where it lands

- The preview draws the allowed region (the box and the rise above it) in the
  panel colour, the box in the surface colour and outlined, and the signature
  where it would land, outlined in the accent with corner and edge grips.
- Drag the body to move it; drag a corner to resize it with its proportions
  kept; Shift frees them, and an edge stretches one side, for a drawn mark or
  a picture. A typed name keeps the face's proportions.
- Untouched, the signature lands by the fit rule. A change of shape (another
  stroke, another name, another picture) or of tab returns it to the fit,
  because a rectangle chosen for one shape would distort another.
- **Reset to fit** returns it to the fit rule.
- The choice travels as `handsign::place::Placement`, relative to the box.

## All three

- *Remember my signature on this computer* keeps a copy of whichever was
  placed; unticked, placing deletes all three copies.

The window writes nothing: it pushes `FieldAction::HandSign` (carrying a
`Signature`, drawn, typed or a picture, and the chosen `Placement` or `None`) or
`FieldAction::SignWithId`.
