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

# The typed unit

Before pressing Insert the check types `2 in` into the width box, which
holds millimetres, and requires the request to carry a box 144 pt wide. A
build that ignores the unit sends 2 mm, 5.67 pt.

## Item notes

### `const FIXTURE_W`

**Wide and short, deliberately.** `Contain` on a square picture in a
square box is the identity, so a square fixture would let a letterbox defect
pass the last assertion — which is the assertion this check exists for.

### `fn fixture_pixels`

The colours are irrelevant to every assertion — nothing here reads a pixel
back — and they are two rather than one so the saved artifact is legible as
a picture rather than as a swatch.

### `fn dpi_in`

The sentence is *"At this size the picture is 300 dpi."* — so this looks for
the token before `dpi` and parses it. Reading the operator's own words
rather than a field carried beside them is the point: it proves the number
**they see**, and a build that traced one figure and displayed another would
pass a field comparison.
