# `ui-verify/checks/signing/placement`

`a_signature_lands_where_it_was_put` — O287's pre-commit placement in the
*Sign here* window.

## What it drives

The same opening as `picture_sign` (a white PNG with one dark bar, chosen on
the Picture tab), then, in the preview `handsign.placement`:

1. Drag the lower-right corner of `handsign.ink` to its centre.
2. Press `handsign.reset-fit`.
3. Halve it again, then drag its body to the preview's right end.
4. Press `handsign.place`; photograph; Ctrl+Z; save a copy.

## Oracles

- **Corner.** The halving drag leaves the signature half as wide, its
  proportions kept to within 10%.
- **Reset.** After Reset to fit the ink rectangle is the fit again, and
  `hand-sign-placement reset=1` is traced.
- **Move.** The body drag moves it right by more than half its width.
- **Traced.** At least three `hand-sign-placement grip=` lines.
- **Requested.** `hand-sign-requested placement=chosen`.
- **Pixels.** At least 10 ink pixels in the signing area, none in the box's
  left 40%: it landed where it was put, not by the fit rule, which centres
  it. Zero after Ctrl+Z.

## Falsification

Placing with the fit rule whatever the operator chose (the `HandSign`
action's `placement` dropped to `None` in `app::actions::apply`) fails the
**Pixels** oracle: 1,770 of 2,160 ink pixels fall in the box's left 40%.
