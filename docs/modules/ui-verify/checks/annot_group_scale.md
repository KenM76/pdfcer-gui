# `ui-verify/checks/annot_group_scale`

`several_annotations_resize_and_turn_as_one_step` drives
`fixtures/annots-align.pdf` (see `annot_multiselect.md` for its geometry)
under `--no-input`, with `mode.edit` and a two-entry `PDFCER_DIAG_SAVE_PATH`
queue. It has no SKIP arm.

1. Click `properties.tool.scale.distort`; require `resize-modifiers
   distort=true`. The fixture's markups were not drawn by pdfcer and a drag
   is never exactly proportional, so without it the engine refuses both.
2. `annot_group_drag::select_members`: click the square, Shift-click the
   circle. Both are markups, so neither is left out.
3. Drag the `canvas.selection-outline` south-east corner 40 points outward;
   require `resize-annotations-applied changed=2/2 left_out=0 one_step=true`.
   A drag routed to the primary alone traces `resize-annotation-applied`
   instead, and the step fails on the missing line.
4. Save a copy; one `Ctrl+Z`; save a copy. Require both tops back after the
   Undo (unfolded resizes bring back only the last) and both page-1 heights
   at least 5% taller in the first copy than in the second.
5. Select the two again; swing `canvas.rotate-handle` from due north of the
   outline's centre to due east; require `rotate-annotations-applied
   changed=2/2 left_out=0 one_step=true`.

Blind spots: the turn is judged by trace only, not in a saved copy; the
per-member resize ghosts are not asserted; a locked or ce-dimension member
(`left_out`) is not driven.
