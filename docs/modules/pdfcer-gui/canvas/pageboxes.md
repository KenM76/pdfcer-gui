# `canvas::pageboxes` — the trim, bleed and art outlines

`view.page_boxes`, Acrobat's *Show Art, Trim & Bleed Boxes*. A display toggle
in View ▸ Display beside Overprint preview, off by default and not
remembered.

**What is drawn.** For every page on screen, each of bleed, trim and art that
the file writes (`BoxResolution::AsWritten` or `Clipped`). A defaulted box
equals the crop box, and the crop box is the edge of the displayed page, so
outlining it would say nothing. Corners go through
`viewer::pdf_space_to_canvas`, which applies `/Rotate` and the crop origin,
then the frame's `PageMapping`.

**Why it is not content marking.** The outlines describe the page's geometry,
not any content's state, and they go away with one press. A screenshot of the
saved file differs only when the operator has asked to see the boxes. This is
the same class as the grid.

**Telling them apart.** Each box has its own overlay colour role
(`page_box.trim` = accent, `page_box.bleed` = danger, `page_box.art` =
notice), held distinct on every preset by a unit test. Each also has its own
stroke: trim solid, bleed dashed, art dotted. Colour is therefore never the
only cue.

**Order.** Drawn over the grid and under the OCR text and the find wash.

**Trace.** `page-boxes on=1 pages=1:bleed+trim,…` while on, and `page-boxes
on=0` while off. Each is emitted through `trace_changed`, so only on change. A
`canvas.page-box.<page>.<which>` region is declared per outline.
