# `ui-verify/checks/model_view_picture`

`the_3d_viewer_saves_its_view_as_a_picture` — **Save picture… in the 3D
viewer writes the view on screen to a PNG file.**

Off-screen, scripted pointer, no OS input. Places the engine corpus's
`assembly.prc` from the ribbon, with a path in the run's output folder as the
picture picker's answer (`PDFCER_DIAG_PICTURE_SAVE_PATH`; any file left there
by an earlier run is deleted first). Opens *View…*, presses Top, then *Save
picture…*, then Close.

Oracles:

- a `model-picture-saved` line;
- the file decodes as PNG and its long side is 1,200 pixels
  (`dialogs::model3d::POSTER_SIDE`);
- at least 5,000 of its pixels are more than 40 below white in some channel,
  so the model is drawn in it, not only the background.

Falsified twice:

- with the write skipped but the success line still traced, it fails on the
  missing file;
- with the rendered picture blanked to white before encoding, it fails on the
  drawn-pixel count.
