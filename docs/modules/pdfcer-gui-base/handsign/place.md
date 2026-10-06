# `handsign::place` — where a signature's ink sits in its box

`OPERATOR_REQUESTS.md` **O287**: a signature, drawn, typed or a picture, can
be moved and resized inside its box before it is placed, and after.

All rectangles are y-down, in the box's own space: canvas units for the
placement itself, the preview's scaled space while the window draws it.

## The fit rule

`fit_rect(ink, target)` is the one rule every kind of signature starts from:

- uniform scale `min(0.95·W / ink.w, 0.95·2H / ink.h)` (a zero side drops
  out of the minimum);
- left edge at `0.03·W`;
- centred vertically when the placed height is `≤ 0.9·H`, otherwise standing
  `0.05·H` above the box's lower edge and rising above the box.

The rise limit of two box heights is Acrobat Fill & Sign's behaviour on a
short box, approved by the operator.

## The allowed region

`allowed(target)` is the box plus one more box height above it, the same
two-box-heights limit the fit rule rises to. A chosen placement is kept in it
by `clamp`: shrunk about its centre with its proportions kept when it is
larger than the region, then moved the least distance that brings it inside.
A signature can therefore not be dragged off its box onto unrelated page
content.

## Reshaping

`reshape(grip, original, delta, free, stretchable)` is one grip drag:

| Grip | Result |
|---|---|
| body (`Move`) | translated by `delta` |
| corner | proportions kept; the larger of the two reach ratios wins. With `free` (Shift) and a stretchable signature, each side follows the pointer |
| edge | that side alone, when stretchable; both sides scaled, when not |

The opposite corner or edge stays fixed, and no side falls below `MIN_SIDE`
(4 box units). A typed name is not stretchable: its proportions are the
face's, and `add_text` has no horizontal scale.

## Placement

`Placement` is a chosen rectangle **relative to the box**: `(0,0)` its
top-left, `(1,1)` its bottom-right, negative `y` the rise. The window draws
the box at its own scale; the relative form carries the operator's choice to
the canvas-space box unchanged. `ink_rect(ink, target, chosen)` is what every
placement verb calls: the chosen placement clamped, else `fit_rect`.
