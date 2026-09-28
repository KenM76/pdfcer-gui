# `pdfcer-gui/canvas/textedit/shaped`

Typing into an existing run shows the replacement in the run's own font,
size, colour and position (O247). Everything else keeps the shell-font box
drawn by `paint`.

## Contract

- **Where the layout comes from.** `EditSession::edit_text_preview` is the
  engine's own `edit_text` plan with the splice skipped. That means the glyph
  positions shown are the ones Enter will write, and a refusal carries the
  same `EditError` text Enter would give. `pdfcer_render::edit_preview::preview_outlines`
  turns those codes into page-space paths through the loader the page
  renderer uses. It is handed the same `FontEnvironment`, so a substituted
  face previews as the substitute that will render.
- **When the editor falls back to the shell-font box.** `read` returns `None`
  and the box is drawn when any of these is true:
  - the draft is not on an existing run;
  - the engine refuses the edit;
  - `skipped` is set (Type 3 or unsupported machinery);
  - the render mode is 3 or 7 (invisible text);
  - glyph count differs from character count, so a caret stop could not be
    placed on each character.

  Falling back is never an error. The shell-font box is the older, complete
  editor.
- **Coordinate spaces.**
  - Outlines, `stops` and `up` are in page user space.
  - `page_to_screen` reads the page→screen affine off three mapped points, so
    rotation and the viewer's y-flip come along with it.
  - The texture is rasterised in physical pixels, one per `ppp`.
- **Stale for at most one frame, never flickering.** The cache key is
  (page, run, original, text). `read` matches everything except the text, so
  a frame between a keystroke and the next `refresh` draws the previous
  layout rather than dropping back to the box.

## Why `refresh` runs in the frame loop and not in the painter

The preview verb takes `&mut EditSession`, because the engine caches the
page's decoded walk in the session. The painter holds `&OpenDoc`. The
render worker holds clones of the `Arc<EditSession>`.

So `refresh` runs after actions apply, with `&mut OpenDoc`, and takes the
session with `Arc::get_mut`. If a render still holds a clone, `refresh`
tries again next frame (50 ms repaint) rather than calling
`cancel_and_wait`. A preview is not worth stalling an in-flight render.

This is a workaround and has been reported to the engine.

The first call on a page costs about 350 ms; later keystrokes cost about
10 ms. The cost is paid once when the draft opens.

## Why the run is covered in paper white

The committed page still draws the ORIGINAL run beneath the draft, so the
replacement must hide it. White is the page background for every drawing
this shell has been used on, but it is an assumption: text over a tinted
fill or an image shows a white patch while editing.

- The accent outline around it is the editor's affordance, like the
  selection handles. It is not content marking (R8b): it disappears on
  commit.
- The committed result then renders from the engine's real output.

A render-without-this-run from the engine would remove the assumption.

## Caret and selection

- **Caret stops** are glyph origins, plus an end stop. The end stop is the
  last origin, advanced along the text direction by the ink box's furthest
  projection.
- **The caret line** runs along `up` (one em, from the glyph matrix's second
  column), so rotated and skewed text gets a matching caret.
- **Selection** is a quad per selected glyph pair, from -0.25 to 0.9 em.
- **Hit-testing** is published as `hit::Caret::Stops`. `Layout::index_at`
  picks the nearest stop, so click and drag inside the draft work as they do
  in the shell-font box.

## Trace

`text-edit-shaped page= run= chars= shaped=0|1 refused=0|1`, once per
distinct draft text. `plan::plan` is run under `diag::muted`, and the
previous `last_commit` is restored, so the preview does not emit a second
`edit-text-pin` or change what a commit reports.
