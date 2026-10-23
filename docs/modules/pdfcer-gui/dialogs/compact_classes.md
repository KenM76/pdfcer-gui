# `pdfcer-gui/dialogs/compact_classes`

Separate picture settings for colour, grey and black-and-white pictures in
Save a compacted copy, the split Acrobat's PDF Optimizer makes in its Images
panel.

## Contract

- `Classes { separate, colour, grey, mono }`, each kind a `compact_images::Pictures`.
- `Classes::rules()` is `None` while not split. Split, it is one engine
  `optimize::Rule` per kind switched on, selected by `Selector::image_classes`.
  `recompress::settings_for` then sets `OptimizeSettings::images` to an empty
  patch and `rules` to these, so nothing outside the kinds is touched.
- A kind switched off gets no rule; its pictures are left as stored.
- The split applies only while the Pictures switch is on, so the one
  "this changes how the page looks" switch still governs everything.

## Decisions

- **Palette pictures count as colour**, as Acrobat counts them. Under JPEG they
  get `ImageEncode::Auto` and no quality, as the engine's presets do, because
  the engine skips palette pictures as JPEG (`ImageSkip::PaletteAsJpeg`).
- **Black and white has no colour row and greys JPEG** (`ImageSkip::OneBitAsJpeg`).
  It opens at 300 ppi stored the smallest exact way, the engine's Standard
  preset's mono rule; colour and grey open at 150 ppi.
- **600 ppi** is offered in every resolution list, for black-and-white scans.

## Regions

`compact.pictures.split`, then for each kind `compact.pictures.<colour|grey|mono>`
as the stem: `.on`, `.format`, `.format.<key>`, `.colour`, `.colour.<key>`.

Driven by `a_compacted_copy_treats_each_kind_of_picture_separately`.
