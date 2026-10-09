# `ui-verify/checks/replaceimage`

`replace_image_swaps_the_picture_in_place`, on `fixtures/image-box.pdf`: one
image XObject, 2 × 1 pixels, drawn into x 100..300, y 100..200 of a 400 × 300
page. The check writes a 30 × 10 red PNG to its scratch folder and hands it to
the picker through `PDFCER_DIAG_IMAGE_PATH` (`dimdrive::run_on_with`).

PASS needs, in order:

1. A click at the image's centre selects it at the Object rung.
2. Format ▸ Replace image: `image-replaced page=0 object=0`, `old` non-zero,
   `new` different, `model=` the new id.
3. A right-click on the image and the canvas menu's `format.replace_image`
   row: the same, with `old` equal to step 2's `new`.
4. The Properties panel's `properties.stroke.replace-image` button: the same,
   with `old` equal to step 3's `new`.

Each route is a separate press, so a route that is missing or reaches nothing
fails at its own step.

## Falsified

- The dispatcher handing the engine object 1: FAIL at step 2, no
  `image-replaced` (the engine refused the index).
- The trace's `model` read before the edit: FAIL at step 2, `model=5`.

## What it does not prove

- Rendered pixels; the oracle is the page model re-read after the edit.
- The disclosures' wording (they appear in the trace's `replace-image` line).
- The refusal of an SVG or EMF file, and of a form XObject.
- Undo.
