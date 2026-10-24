# `ui-verify/checks/scaled_text_size`

`scaled_text_shows_and_takes_its_drawn_size` — O309 and O310, driven.

CAD exporters write every label `1 Tf` and carry its size in the text matrix.
By default the check writes such a page itself (one Helvetica line, `1 Tf`
under an 18-point `Tm`). `--pdf FILE --doc-point PAGE,X,Y` drives any other
file instead; the check copies it to `target/ui-verify/scaled-text-size.pdf`
and never touches the original.

The route is the operator's: Edit tab, Text tool, click in the line (the
I-beam), type a size into Properties, Enter. Like a word processor, that
restyles the **word at the caret**, not the line. The check requires:

1. **the read** — Properties' `text-style-synced … size=` is the drawn size
   (18 on the fixture), not the `Tf` operand 1;
2. **the size** — the restyle applies, and the size it wrote, read from the
   engine's disclosure `Tf size was changed from A to B` as
   `read × B / A`, is the typed size. The panel's re-read is the fallback when
   no such note is traced. It cannot be the primary oracle: it reads the
   run's first glyph, which on a multi-word line is not the restyled word;
3. **the characters** — the re-read's `chars=` and `text_hash=` are unchanged
   (re-encoded codes draw as squares);
4. **the redraw** — a page render finishes after the change and its
   `notdef=` is no higher than before it (a blank or squared-out redraw);
5. **the re-open** — a Text-tool click on the line opens the same number of
   characters (`text-edit-caret len=`), and the opened line draws no squares:
   laid out in its own font, `text-edit-shaped unread=0`; refused a layout
   (`shaped=0`, drawn in the stand-in font over the line), no page render so
   far above the original `notdef=`.

It then presses Escape and Ctrl+S, so the restyled copy is on disk for the
engine CLI to read back.

A size of 432 (fixture) or 318 (SW41177) at step 2 is the typed size written
as the `Tf` operand under the matrix — the text redrawn off the page, which is
what the operator saw as "the text chunk goes blank".

Driven off the desktop through the scripted pointer, under `--no-input`.

## Driven

| input | doc-point | verdict |
|---|---|---|
| fixture | default | PASS: 18.00 read, 24.00 written, 17 chars, notdef 0→0, re-open len 17, stand-in font |
| `D:/Dev/pdfTests/SW41177/SW41177.pdf` (private, never checked in) | `0,1215,595` ("TRUCK", run 108, 41 chars, CenturyGothic Type0) | PASS: 13.23 read, 24.00 written, notdef 0→0, re-open len 41, stand-in font |

On both, the re-opened line is refused an own-font layout
(`text-edit-preview-fallback reason=refused`) once the restyle has split it
into differently sized pieces, so it is shown in the stand-in font. The saved
SW41177 copy read back with the engine CLI: `find-text` puts "TRUCK" 24 pt
tall, `render-page` reports 0 `.notdef` and 0 substituted glyphs.

## Falsification

| control build | expected verdict |
|---|---|
| `RunStyle.size` = `tf_size` instead of `GlyphProvenance::drawn_size` | FAIL at step 1, reads `1.00` |
| `StyleChange::stamp` calls `FormatRequest::size` instead of `drawn_size` | FAIL at step 2 — watched red on SW41177, `317.52` pt |
