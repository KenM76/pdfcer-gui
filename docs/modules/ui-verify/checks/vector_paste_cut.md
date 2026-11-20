# `ui-verify/checks/vector_paste_cut`

`an_svg_or_metafile_pastes_as_a_drawing_and_cuts_as_vector` — another
program's vector formats paste as drawings, and Cut as vector removes what it
placed. Off the desktop, scripted pointer only, so it runs under `--no-input`.
The clipboard is a capture folder (`PDFCER_DIAG_CLIPBOARD_DIR`): the harness
plants the entries, the app reads and writes there, and the operator's
clipboard is never opened.

## Steps

1. The folder is emptied and given `fixtures/vector-art.svg` as
   `1-image_svg_xml.bin` and `fixtures/vector-art.emf` as
   `2-CF_ENHMETAFILE.bin`, the names a capture gives those formats.
2. The engine corpus's `four-pages.pdf` opens in Edit. A paste at (200, 500)
   on page 0 must trace `clip-pasted kind=vector format=svg as=content`,
   centred within 2 pt of the pointer, and the engine's `add-svg` line.
3. The SVG file is removed. A paste at (400, 250) must take the metafile:
   `format=emf`, and `add-emf`.
4. The paste leaves the drawing selected. Edit ▸ Clipboard ▸ Cut as vector
   must trace `clipboard-cut-out`, a `clipboard-copy-out selection=true`
   whose formats include `application/pdf` and `pdfcer-gui ObjectClip`, and a
   `delete-objects` line; the folder's PDF entry must start `%PDF-`.

## Falsification

- Reading the metafile ahead of the SVG in `clippaste` fails step 2 on
  `format=emf`.
- Dropping the `DeleteSelection` push from the cut fails step 4 on the
  missing `delete-objects`.

## What it does not cover

The real clipboard's metafile handle: under the capture folder the EMF is
read as bytes, so `native_clipboard::get_enhmetafile` is exercised only by a
paste from a real program.
