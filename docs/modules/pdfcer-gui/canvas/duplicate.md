# `canvas::duplicate` — Ctrl+D over page content and parts of drawings

`duplicate(&mut doc, &mut actions)` lifts the selection through
`take::selected` and raises one `VectorAction::PasteObjects` of it on the same
page at `Matrix::translate(PASTE_OFFSET_PT, -PASTE_OFFSET_PT)`: right and down
by the offset a paste onto its own page uses. One paste is one undo entry,
whatever was selected.

It takes no `egui::Context`, so it cannot reach the clipboard. That is the
difference from Copy then Paste, and the reason the command exists.

Form fields keep their own duplicate (`fieldclip`), which names the copies;
dispatch sends a field selection there before this is reached.

## Why a verb and not Copy then Paste, or `edit.paste_duplicate`

Copy then Paste works and destroys whatever the operator was carrying on the
clipboard, once per duplicate. `edit.paste_duplicate` already has a second
sense (a copied form field pasted as another widget of the same field) and
falls through to the plain paste otherwise; making it duplicate the selection
would put two unrelated behaviours behind one id. So Duplicate is a sibling
command.

The lift is the copy's own (`take::selected`, `copy_selection`, `Plan::of`),
so which carrier an annotation lands on is the engine's answer, read off the
clip, never predicted here; a straight `paste_objects` of the selection would
hand back an anonymous, undated copy of a signed revision cloud.

## What a duplicate carries

- Page objects and annotations: whatever `copy_selection` lifted. The engine
  strips `/NM` from pasted annotations, so a duplicate does not share its
  original's name.
- Parts of a placed drawing: page content, where and as large as drawn,
  offset like everything else. The duplicate is not inside the drawing.
- What the lift left out comes back in the returned `Clipped` and is reported
  off-canvas by the caller ("Duplicated, but not all of it.").

The paste does not select what it pasted.

## Mode

Duplicating a markup is refused outside the mode that authors markups, and
duplicating page content outside Edit, by dispatch, before this is called.

## Trace

`selection-duplicate kind= page= objects= items= annots= thin= left_behind=
in_form_left= dx= dy=`, then the paste's own `paste-objects-applied`.
