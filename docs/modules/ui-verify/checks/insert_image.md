# `ui-verify/checks/insert_image`

`insert_image_places_a_picture` — a picture reaches the page, and the
resolution the window promised is the one the document reports.

# The gap this closes

`edit.insert_image` was `P3` scaffolded with the recorded reason **"No
recorded reason for the missing arm"** — one of three such entries — while
`EditSession::add_image` had shipped the whole time.

# The assertion this check exists for, and it is the LAST one

**The resolution the window previewed and the resolution the document
reported are the same number.**


The failure it catches is specific and quiet. A re-derivation in the window
— the obvious four-liner — measures the *requested* rectangle rather than
the *placed* one, so under `ImageFit::Contain`, which is the default, it is
low by exactly the letterbox ratio. Both numbers look perfectly reasonable.
An operator sizing a logo would be told it plots at 300 dpi and get 200.

# Why the fixture is written by the check

A committed binary would be an asset to keep in step with a decoder, and
`ui_verify::png` already encodes one for the screenshot path — so the check
writes its own two-colour PNG into the scratch directory and points
`PDFCER_DIAG_IMAGE_PATH` at it. Hermetic, deterministic, and it exercises the
importer on bytes nothing else in this repository produced.

The picture is deliberately **wide and short**. A square would let a
letterbox bug pass: `Contain` on a square picture in a square box is the
identity, and the whole point of the last assertion is the case where the
placed rectangle differs from the requested one.
