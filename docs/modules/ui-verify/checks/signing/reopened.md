# `ui-verify/checks/signing/reopened`

`a_signed_box_stays_signed_after_reopening` — a hand signature is read back
from the document, not remembered by the session that placed it.

## What it drives

Nothing but the launch: off the desktop, on a copy of
`fixtures/esign-one-signed.pdf`. That is `esign-three-boxes.pdf` as a session
that signed `ApplicantSig` saves it: a stroke in that box inside
`/pdfc_HandSig <</Field (ApplicantSig)>> BDC … EMC`, plus the same tag around
nothing for `CoApplicantSig`, which is what deleting a signature's objects
leaves.

## Oracles

- **Count.** `sign-strip signed=1 total=3`.
- **Tags.** The boxes declared under `form.sign-box.` are exactly `.1` and
  `.2`: `ApplicantSig` (index 0 in `Placed::unsigned`) takes no tag, and the
  empty sequence leaves `CoApplicantSig` unsigned. The tag index is the box's
  place in the full unsigned list, so a signed box leaves a gap.

The control is `next_reaches_every_box_to_sign` and the drawn check, whose
untagged fixtures open with `signed=0` and every box tagged.

## Falsified

`app::handsigned::refresh` never measuring: the strip counts `(0, 3)` and
`.0` is tagged, so both arms fire.
