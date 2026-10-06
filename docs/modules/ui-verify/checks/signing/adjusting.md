# `ui-verify/checks/signing/adjusting`

`a_placed_signature_moves_and_resizes` — O287 part 3: a hand signature
already on the page is selected, resized and moved there.

## What it drives

The same opening as `picture_sign` (a white PNG with one dark bar on the
Picture tab), Place, then on the page:

1. Click `form.hand-sig`.
2. Drag its lower-right corner to its centre.
3. Drag its body three box-widths to the right.
4. Photograph; Ctrl+Z; Ctrl+Shift+Z; undo and save a copy.

## Oracles

- **Selected.** The click yields `form.hand-sig.selected`.
- **Resize.** `hand-sign-adjusted tagged=1` at the placed undo depth + 1; the
  mark is half as wide, proportions kept to within 15%.
- **Move.** `hand-sign-adjusted tagged=1` at depth + 2; the mark's right edge
  is at the box's (clamped), within 3 points.
- **Pixels.** At least 10 ink pixels in the signing area, none in the box's
  left 40%: the content moved where the outline did.
- **Undo.** After Ctrl+Z the mark is back at its halved place: the move was
  one undo step on its own.
- **File.** The saved copy keeps a `/pdfc_HandSig` tag.

An absent mark after a drag or after Ctrl+Z is a finding, not a skip.

## Falsification

- The drag's rectangle left unclamped (`place::clamp` dropped in
  `sigadjust`) fails **Move**: the mark ends at x 1,607–1,685, past the box's
  right edge at 930.6.
- `adjust` refusing before the engine call fails **Resize**, **Move**,
  **Pixels** and **Undo**: no `hand-sign-adjusted` line, the mark unmoved.
