# `canvas::forms::sigadjust`

Moving or resizing a hand signature already in its box, on the page (O287
part 3).

## Contract

`overlay` runs where the red signature tags do, after them, over every
unsigned signature box that `HandSigned` reads a mark in. For each mark:

- **Hover** outlines it and shows a pointing hand.
- **Click** selects it: an accent outline and the four corner grips of
  `GripSet::scale_only()`. Escape, or a click on the page that no signature
  took, drops the selection. The selection lives in egui's temp data, keyed by
  field and page; a selection whose mark has gone (undo, reopen) is dropped.
- **Drag** on the body moves it; on a grip (selected only) resizes it, aspect
  locked, Shift free (`handsign::place::reshape`). The rectangle being dragged
  is drawn as an outline, the cursor's, and is clamped with
  `handsign::place::clamp` to the box plus the room a short box allows above
  it — the same room Place uses.
- **Release** raises `FieldAction::AdjustHandSign{field, page, from, to}` in
  PDF user space. Nothing is applied before release, so the page shows the
  saved state throughout (rule 8b): the outline is the cursor, not the
  content.

Returns whether the frame's click was taken, so the fill click below does not
also act on it.

## Why on the page and not in a dialog

The signature is content on the page; the operator sees it where it landed
and adjusts it there, as Acrobat Fill & Sign does. A second window would make
him judge the result through a preview again.

## Trace

`hand-sig-selected page= grip=` on a select or drag start;
`hand-sig-adjust page= grip= x0= y0= x1= y1=` on release (PDF space). Regions
`form.hand-sig` (the first mark) and `form.hand-sig.selected`.
