# `ui-verify/checks/signing/next_box`

`next_reaches_every_box_to_sign` — O269's signing strip, end to end.

## What it drives

Off the desktop, scripted pointer only, on a copy of
`fixtures/esign-three-pages.pdf` (one box at the foot of each of three tall
pages, so they cannot all be on screen at once).

1. Read `signstrip`, `canvas-viewport` and the `sign-strip` count.
2. Press `signstrip.next` three times.
3. Press it twice more (wrapping to box 0, then box 1), click box 1's tag,
   draw one stroke and press `handsign.place`.
4. Press Next three more times.

A photograph is kept after every press.

## Oracles

- **Layout.** The strip's bottom edge is at or above the canvas's top.
- **Count.** `(0, 3)` at opening, `(1, 3)` after signing box 1.
- **Fixture sanity.** Box 2's tag is not inside the canvas at opening;
  otherwise the walk proves no scrolling.
- **Walk.** Press *k* traces `sign-next index=k` and `form.sign-box.k` is
  declared, live and wholly inside `canvas-viewport`.
- **Skip.** After box 1 is signed, Next goes to 2, 0, 2.
