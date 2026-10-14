# `ui-verify/checks/annot_multiselect`

`several_annotations_align_as_one_undo_step` drives `fixtures/annots-align.pdf`
(one 400 × 300 page, built by `fixtures/annots-align.PROVENANCE.py`) under
`--no-input`, with `mode.edit` and a two-entry `PDFCER_DIAG_SAVE_PATH` queue.
It has no SKIP arm.

Fixture geometry it relies on: Square `/NM (SQ)` `[40 200 100 240]`, Circle
`/NM (CI)` `[140 180 200 220]`, Link `/NM (LK)` `[240 160 300 180]`, a ce
dimension (the only `/LineDimension`) with `/Rect` top 108.5, and one patch
of page content `[330 20 370 60]`.

1. Require `status-group:edit-disclosure` not yet declared, so the band's own
   sentence can be told apart. Drag a crossing band from (380, 10) to
   (230, 190) over the link and the content patch; require `annot-band
   caught=1 kept=content`, then the disclosure declared. A build that silently
   drops the link passes the first half and fails the second.
2. Click the square; require `annot-select subtype=Square`.
3. Shift-click the circle, the link and the dimension; require
   `annot-toggled before=n after=n+1` for n = 1, 2, 3. A Shift-click that
   replaces traces `before=1 after=1`.
4. `Ctrl+Alt+8` (Align top); require `move-mixed-applied annots=3/3
   one_step=true`. Save a copy; require all four saved tops at 240. The
   dimension reaches 240 by its placement (line and value), not by
   translation, which is why its top is the one the check reads.
5. One `Ctrl+Z`; save a copy; require the tops back at 240, 220, 180 and
   108.5. Unfolded moves leave the first-moved annotation at 240.
6. Drag a window band from (20, 250) to (210, 170); require `annot-band
   caught=2 kept=annots selected=2`.
7. `Ctrl+Alt+4` (Align left); require `move-mixed-applied annots=1/1`: only
   the circle is off the square's left edge, so this proves the band's pair,
   not the earlier four, is what Align reads.

Steps, the saved-edge reader and the save are in `checks::multiselect`.
