# `ui-verify/checks/highlight_shapes`

`highlight_shapes_draw_as_chosen`, on `blank-overhang.pdf` (1000 × 800 pt) with
`PDFCER_DIAG_INVOKE=mode.review,markup.highlight`, under `--no-input`.

1. **Alt over text.** Drag a box over *DRAWN INSIDE THE PRINTABLE AREA*
   (y 0.875–0.90) holding Alt. Exactly one `markup-commit` (a box) and no `text-markup-commit` must
   follow.
2. **The control.** Drag along the title block's *TEST SHEET* row
   (y 0.575–0.585) without Alt. One `text-markup-commit` must follow,
   otherwise the row holds no text on this build and step 1 proved nothing;
   and no `markup-commit`, which would be a second, box-shaped highlight
   authored from the same release. The drag is a slim box rather than a line
   so that such a box would have extent and commit.
3. **Ellipse.** Open the Properties tab (Review mode opens on Comments); its
   `properties.tool.highlight` block must be drawn, else FAIL. Choose shape 2 in Tool Options and drag a box on blank
   paper. The last
   `markup-author` must read `kind=Ellipse wash=Shape highlighter=true`.
4. **Polygon.** Choose shape 4, click four corners, press Backspace:
   `markup-vertex-key key=backspace vertices=3`. Press Enter: exactly one more
   `markup-author`, reading `kind=Polygon wash=Shape highlighter=true`.

Rows measured with `pdfcer.exe find-text`. The rig is
[`snapshot_box`](snapshot_box.md)'s, which always opens this fixture.
