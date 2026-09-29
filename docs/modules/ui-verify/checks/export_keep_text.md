# `ui-verify/checks/export_keep_text`

`export_keep_text` — **the Keep-text-as-text box in Export image decides
whether an SVG carries `<text>` elements or outlines, in both directions.**

## What it guards

`dialogs::export_image` carries `keep_text` into the plan, and
`app::actions::export` maps it to `SvgText::KeepText` or `SvgText::Outlines`.
A box that draws and does not bind, or a mapping that ignores the plan, writes
outlines every time. The file looks identical, so only its elements show the
difference.

## How it drives

1. It opens `fixtures/four-pages.pdf`, whose text is embedded TrueType that
   SVG can keep. It points `PDFCER_DIAG_SAVE_PATH` at a file in the output
   directory.
2. It does two rounds. In each: File tab ▸ Export image, click the SVG radio,
   click `export-image.keep-text`, press Export.
3. Each round requires a new `export-image-requested` line with `format=svg`,
   a `keep_text` value that differs from the previous round, and a written
   file. Ticked must give at least one `<text` element and
   `export-image-svg-text kept>0`. Unticked must give none.

The box is remembered, so its starting state is whatever the last export
chose. Flipping it twice covers both directions and leaves the remembered
choice where it was.

## Not covered

EMF shares the choice through `EmfText`. It is not asserted here.

## Falsification

Mapping the SVG arm to outlines whatever the plan says fails round 1 or
round 2, whichever is the ticked one, with 0 `<text` elements.
