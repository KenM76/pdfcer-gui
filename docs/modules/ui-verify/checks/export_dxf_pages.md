# `ui-verify/checks/export_dxf_pages`

`export_dxf_writes_one_file_per_page` — Export to DXF with *Every page*
writes one DXF per page, each holding its own page's geometry (O284).

## Steps

1. Launch on `fixtures/cropped-sheets.pdf` off the desktop, with
   `PDFCER_DIAG_SAVE_PATH` naming `dxf-pages.dxf`; that name and its two
   per-page names are deleted first.
2. File ▸ Export to DXF; require `export-dxf-open`.
3. Click `export-dxf.pages.all`; require `export-dxf-pages pages=0,1`.
4. Click Export; require exactly two `export-dxf` lines.
5. `dxf-pages.dxf` must not exist; `dxf-pages_p1.dxf` and `dxf-pages_p2.dxf`
   must. Each file's drawing width is `$EXTMAX` x minus `$EXTMIN` x; page 1's
   rectangle is 180 pt wide and page 2's 120 pt, so the ratio must be 1.5
   within 0.01 (the stroke's half-width may sit in the extents).

The width ratio is the oracle that tells "each file holds its own page" from
"every file holds the page on screen": the latter gives 1.0. Units and scale
cancel in a ratio.

## Falsification

With `export::dxf` decomposing `doc.pages[pages[0]]` for every page, both
files hold page 1 and step 5 fails with a ratio of 1.0.

## What it does not cover

The typed range, the scale re-suggestion over several pages' groups (the
fixture has no ce dimensions), and the part-way failure sentence.
