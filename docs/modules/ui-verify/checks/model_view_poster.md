# `ui-verify/checks/model_view_poster`

Two checks that a 3D model's page picture can be replaced, one per route.
Both are off-screen, with a scripted pointer and no OS input, and both place
the engine corpus's `assembly.prc` from the ribbon first.

## `the_3d_viewers_view_becomes_the_page_picture`

Opens the placed model with *View…*, presses Top and then *Use this view on
the page*, and closes the viewer, because an open viewer can cover the page.
Oracles:

- a `set-3d-poster` edit line;
- at least 2,000 page pixels differ by more than 24 in some channel between
  the frame after the insert and the frame after the press. The engine's
  default picture is an isometric view, so a view from above that reached
  the page cannot leave it as it was.

Falsified twice:

- with the engine call made a no-op, it fails on 0 changed pixels;
- with the queued action dropped, it fails on the missing edit line.

## `a_picture_file_becomes_a_3d_models_page_picture`

Writes a 400 x 300 solid red PNG and sets it as the image picker's answer
(`PDFCER_DIAG_IMAGE_PATH`), then presses *Picture…* on the model's
Attachments row. The launcher is `model_view_window::launch_with_model_env`.
Oracles:

- a `set-3d-poster` edit line;
- the page gains at least 2,000 plainly red pixels (red above 200; green and
  blue below 60). The default picture of `assembly.prc` is grey on white and
  has none.

Falsified twice:

- with the picked file never handed to `set_poster`, it fails on the missing
  edit line;
- with the engine call made a no-op, it fails on "gained 0 red pixels".
