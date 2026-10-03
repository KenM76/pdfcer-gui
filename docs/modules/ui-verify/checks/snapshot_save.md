# `ui-verify/checks/snapshot_save`

`a_snapshot_saves_as_a_one_page_pdf` — the snapshot box's right-click menu
offers *Save as PDF…*, and the file it writes is one page the box's size.

## Steps

1. Launch on `fixtures/blank-overhang.pdf` off the desktop, with
   `PDFCER_DIAG_SAVE_PATH` naming the target, which is deleted first.
2. View ▸ Snapshot; drag a box over part of the fixture's drawing (x 20–60 %,
   y 55–80 % of the page); require the `snapshot-box` line at the dragged
   corners.
3. Right-click inside the box; the `canvas-menu` line must name
   `canvas.snapshot`. Click `menu.item.canvas.snapshot.view.snapshot_save_pdf`.
4. Require a `snapshot-save-pdf` line and no `snapshot-save-refused`; the
   target must start `%PDF-`; its first `/MediaBox` must be the box's size
   within 0.5 pt.

The page size is the oracle that tells the box from the page: the fixture's
page is several times the box in each direction.

## Falsification

With `snapshot_pdf` passing the page's whole crop box to the engine instead
of the box's region, the saved page is the source page's size and step 4
fails, quoting both sizes.

## What it does not cover

Whether the content inside the box survives the cut is the engine's region
tests' subject, and the snapshot copy check's SVG count covers the same cut.
The over-document refusal is not driven.
