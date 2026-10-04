# `picture` — a file chosen for the page

`Picture` is what Insert image, a dropped file and an OS paste carry to the
apply arm: a raster (`ImportedImage`, placed by `EditSession::add_image`) or a
vector drawing that stays vector (`ImportedSvg` by `add_svg`, `ImportedEmf`
by `add_emf`).

## Contract

- `EXTENSIONS` is the one list the picker filter and the drop handler both
  read. A format added here is offered everywhere at once; there is no second
  list to forget.
- `Picture::import(path, bytes)` picks the importer by extension: `.svg` and
  `.emf` are drawings, anything else goes to the raster importer, which
  identifies its formats by signature. A `.png` holding SVG is therefore
  refused by the raster importer rather than guessed at.
- `natural_size_pt` is in PDF points: a raster at its declared resolution, an
  SVG at 96 px per inch (0.75 pt per px), an EMF at its own frame.
- `kind()` is the token in the `image-imported` and `insert-image-requested`
  trace lines and in the apply label (`add-image`, `add-svg`, `add-emf`).
- `drawing_notes()` is the engine's own one-line summary of what the import
  skipped or approximated (an SVG's `<text>` is not carried), or `None`.

## Why a drawing has no fit choice

The engine's drawing verbs stretch the drawing to the rectangle and report
`distorted` when the two axes scale differently. The window offers the box
and says the drawing fills it; a stretch is disclosed after placement. The
raster fit (contain or stretch) is the raster verb's option and is not
imitated for drawings.
