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
  - glyph count differs from character count, so a caret stop could not be
    placed on each character.

  Falling back is never an error. The shell-font box is the older, complete
  editor. Each case has a `PreviewFallback` reason, held in the cache beside
  the layout; `paint` adds `TooLarge` when the ink would exceed the texture
  limit. `fallback::publish` hands the reason to the status bar, which says
  it in a sentence from `text::previewfallback` (R8b: off-canvas, never on the
  page).
- **Invisible runs (render mode 3 or 7)** are laid out like any other, then
  drawn in the OCR layer's colour, whether or not that layer is shown, with no
  paper cover, since there is nothing
  painted beneath them to hide. `canvas::ocrlayer` leaves the run to this
  preview while it is laid out.
- **Coordinate spaces.**
  - Outlines, `stops` and `up` are in page user space.
  - `page_to_screen` reads the page→screen affine off three mapped points, so
    rotation and the viewer's y-flip come along with it.
  - The texture is rasterised in physical pixels, one per `ppp`.
- **Stale for at most one frame, never flickering.** The cache key is
  (page, run, original, text). `read` matches everything except the text, so
  a frame between a keystroke and the next `refresh` draws the previous
  layout rather than dropping back to the box.

## A preview of part of a line (`splice`)

An edit that spans several show operators, or that is narrowed to the operators
it touches, is previewed by the engine as the part it rewrites only.
`TextEditPreview::rewritten` names that part as a byte range of the request's
`find` and its replacement. On the narrowed tier the range is offset by where
the touched operators start in the run. `splice::shape` lays the part out and
puts the rest of the line's caret stops back around it, so the caret, the
selection and the hit test index the whole draft.

The commit re-lays the rest of the operator that holds the part. Those glyphs
(the tail) are drawn through the preview's font, moved by the part's change
in advance, and the original glyphs beneath them are blanked. Glyphs of other
operators stay where the page render draws them, because the commit leaves
them there. Driven by `a_key_typed_mid_line_previews_where_it_commits`.

## Why `refresh` runs in the frame loop and not in the painter

Keystrokes are applied as actions after the frame is drawn, so the painter
sees the draft's previous text. `refresh` runs after the actions apply and
lays out the text they produced; the next frame paints it. The preview verb
takes `&EditSession`, so it never waits on a render worker holding the
session.

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

`text-edit-shaped page= run= chars= shaped=0|1 refused=0|1 tier=`, once per
distinct draft text, and `text-edit-preview-fallback page= run= reason=
font_pt=` each time the reason for the open draft changes; `font_pt` is the
stand-in's size in screen points. `plan::plan` is run under `diag::muted`, and the
previous `last_commit` is restored, so the preview does not emit a second
`edit-text-pin` or change what a commit reports.
