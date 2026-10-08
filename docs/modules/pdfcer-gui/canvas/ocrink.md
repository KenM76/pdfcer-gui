# `canvas::ocrink` — the OCR layer, drawn in its own fonts

The OCR text layer is drawn in the font, size, horizontal scale and position
each word is stored in, by the same two engine calls that draw a text edit
while it is typed (`canvas::textedit::shaped`):

1. `EditSession::edit_text_preview` of an unchanged edit — each show
   operator's own text replacing that whole operator, pinned as the editor
   pins it;
2. `pdfcer_render::edit_preview::preview_outlines` of that preview, which
   answers each glyph's outline in page space.

So the layer, an edit in progress and the committed result are one drawing,
and a word does not change shape when an edit opens on it or commits
(`OPERATOR_REQUESTS.md` O289). A fitted stand-in in the interface font
(`canvas::ocrlayer`) could not promise that: a recogniser writes each word at
its own size and `Tz`, which no box-fitting reproduces.

## Cost, and why nothing waits on it

One preview per run loads the run's font and plans a rewrite, so a page is laid
out over several frames: [`step`] spends at most [`BUDGET`] per frame, keeps
what it has in egui's temporary memory keyed by (session, edit epoch, page),
and asks for another frame while runs remain. Runs not reached yet are drawn
by the stand-in, so the layer never blanks while it builds. Any edit moves the
epoch and the page is laid out again. `ocr-ink-built page= runs= laid= ms=`
traces each finished page with the wall time it took.

## Which runs it draws

A run gets outlines here only when all of these hold; otherwise
`ocrlayer` draws it with the stand-in:

- it is OCR text (`ocrlayer::is_ocr_run`);
- for each of its show operators (`textedit::pin::operators_in_run`), the
  engine plans the unchanged edit in place (no `rewritten` fallback) with one
  glyph per character, so the outlines are the run's own. A run is laid out
  operator by operator because a word typed longer can join its neighbour
  into one run while each stays its own operator;
- `preview_outlines` skips nothing (a font with no outlines is skipped).

The run an edit is open on is left out of [`paint`]; the editor's preview
draws it, at the same opacity (`shaped::paint` scales an invisible run's
preview by `ocrlayer::painted_fraction`).

## Painting

[`paint`] fills the outlines of every drawn run into one texture covering the
visible part of their extent, in the layer's colour, through the page-to-screen
affine `shaped::page_to_screen`, and draws it tinted to the layer's opacity.
The texture is kept while its tag — page, epoch, runs laid, the held run,
colour and screen rectangle — is unchanged, and recorded with the texture
census as `Surface::OcrLayer`.

## R8b

This draws text the file holds, in a view the operator turned on; with the
layer off it draws nothing. The fonts are the file's own, so the layer shows
the words as the saved file's text is laid out, not as a marking.

## When the engine can paint invisible text

`ENGINE_BACKLOG.md` G169 asks the renderer for an option to paint text in
rendering mode 3. Once it lands, the layer is a render and this module is
deleted.
