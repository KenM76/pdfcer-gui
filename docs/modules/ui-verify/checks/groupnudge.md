# `ui-verify/checks/groupnudge`

Arrow keys over a multi-item selection, driven under `--no-input` with
`mode.edit`. Each press must raise one `move-mixed-applied gesture=nudge` with
`one_step=true`. The two group checks save copies through
`multiselect::run_saving`, whose two-entry `PDFCER_DIAG_SAVE_PATH` queue names
`<stem>.moved.pdf` and `<stem>.undone.pdf`. The bare arrow's step is 1 pt;
`multiselect::tops` allows 0.5 pt, so a top that did not move fails.

`several_annotations_nudge_as_one_step`, on `fixtures/annots-align.pdf`:

1. `annot_group_drag::select_four`: the square by a click, then the circle,
   the link and the locked square by Shift-click. The locked one is the
   primary, so a nudge that reads only the primary refuses with the lock
   sentence and nothing moves.
2. ArrowDown. Require `annots=3/4`, then `status-group:edit-disclosure` drawn
   for the locked one.
3. Save a copy: the three unlocked tops 1 pt lower, the locked one unchanged.
4. One `Ctrl+Z`; save a copy: every top back.

`several_form_fields_nudge_as_one_step`, on `fixtures/all-field-kinds.pdf`:
`field_group_drag::select_three` (TextOne, CheckOne, ComboOne), ArrowDown,
`widgets=3/3`, then the same save, Undo and save as above.

`a_ce_dimension_is_nudged`, on `fixtures/dimension-scaled.pdf`: the linear ce
dimension selected by its line (`dimension_text_select::select_by_line`),
ArrowRight, `annots=1/1`, and no `annot-nudge-declined` after the press.

## Falsified

Against the binary before group nudge, each FAILs with no `move-mixed-applied`
after the press: the annotation check traces `annot-nudge-declined … reason=Locked`
(only the locked primary was read), the field check `annot-nudge-declined
reason=nothing-selected` (no field arm), and the dimension check
`annot-nudge-declined … reason=Dimension`.

## What it does not prove

- A single unlocked markup's nudge; `a_markup_is_nudged_duplicated_restacked_and_typed_to_size`
  drives that path (`AnnotAction::Move`).
- Where the ce dimension lands, or that it still measures what it measured.
- Fields selected on a page other than the primary's, which do not move.
- Ctrl's quarter-point step and a held key's repeats.
