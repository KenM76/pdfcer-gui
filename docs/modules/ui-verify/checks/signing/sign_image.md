# `ui-verify/checks/signing/sign_image`

`a_visible_signature_carries_its_picture`: the Sign window's *Picture in the
box* reaches the engine and is drawn in the signature's appearance.

## What it drives

Off the desktop, scripted pointer only, on a copy of `fixtures/four-pages.pdf`.

1. Write a 60 x 24 RGB PNG to the run folder. The picture is wide on
   purpose: it fails unless the box widens by the picture's share
   (`sign::picture_widening`). Without that, the test certificate's name does
   not fit beside the picture and the engine refuses with
   `AppearanceOverflow`.
2. Open the Sign window through `PDFCER_DIAG_INVOKE=file.sign`. The certificate
   path is the engine fixture `signing/rsa2048-modern.pfx`. The picker answers
   are `PDFCER_DIAG_CERTIFICATE_PATH` and `PDFCER_DIAG_IMAGE_PATH`, and the
   destination is `PDFCER_DIAG_SAVE_PATH`.
3. Click `sign-choose-certificate`, type the passphrase into `sign-passphrase`,
   then click `sign-open-certificate`, `sign-place-box` and `sign-picture`, and
   finally `sign-confirm`. A region outside `sign-body` is wheeled into view
   before it is clicked.

## Oracles

- **Named.** `sign-picture-name` is declared after the pick: the window says
  which picture the box will carry.
- **Drawn.** `sign-picture-drawn asked=1 drawn=1`: the request carried the
  picture and the engine's `SignReport::appearance_image_drawn` says it drew it.
- **Written.** The bytes the signing appended after the source's length hold at
  least one `/Image` XObject.

## Falsified by

Not passing `appearance_image` in `sign::prepare` gives `asked=1 drawn=0`.
Placing the picture in the unwidened `default_rect` gives no
`sign-picture-drawn` line, because the signing is refused.
