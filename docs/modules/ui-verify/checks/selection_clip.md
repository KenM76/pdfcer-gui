# `ui-verify/checks/selection_clip`

Copy, Cut and Duplicate over the whole selection, driven under `--no-input`
in `mode.edit`. The chords are sent as scripted keys.

`several_annotations_copy_cut_duplicate_as_one_step`, on
`fixtures/annots-align.pdf`, launched with a two-entry
`PDFCER_DIAG_SAVE_PATH` queue (`selection-clip.cut.pdf`,
`selection-clip.undone.pdf`) and `PDFCER_DIAG_CLIPBOARD_DIR` set, so the
system clipboard is never written:

1. `annot_group_drag::select_members`: the square by a click, the circle and
   the link by Shift-click.
2. Ctrl+C: `clipboard-copy annots=3`.
3. Ctrl+D: `selection-duplicate annots=3`, then `paste-objects-applied
   annots=3`.
4. Ctrl+X: `delete-annotations-applied deleted=3/3 one_step=true`. The
   selection stays on the originals after Ctrl+D, so this cuts them.
5. Save a copy: page 1 lists no `/NM (SQ|CI|LK)`, and an annotation whose
   top is each original's top less the 10 pt paste offset (the duplicates
   carry no `/NM`). Only page 1 is read: the fixture's page 2 carries the
   same three annotations, `/NM` and all.
6. One Ctrl+Z; save a copy: all three originals listed on page 1 at their
   own tops.

`ctrl_d_duplicates_a_page_object`, on `nudgecontent::BOXES`: a click on the
red square only, then Ctrl+D: `selection-duplicate kind=selection items=1`,
`paste-objects-applied page=0 pasted=1`, and no `clipboard-copy`.

`ctrl_d_duplicates_a_part_of_a_placed_drawing`, on
`fixtures/half-scale-form.pdf`: `form_node_move::enter_leaf_at` on the line
inside the drawing, then Ctrl+D: `kind=form-leaves items=1`, the same paste,
and no `clipboard-copy`.

## Falsified

Against the binary before whole-selection Duplicate:

- The annotation check FAILs at Ctrl+C with `annots=1`: only the primary was
  copied.
- The page-object and drawing-part checks FAIL with no `selection-duplicate`:
  Ctrl+D refused with "nothing selected".

## What it does not prove

- Where the duplicated page object or part lands; the trace's `dx dy` and the
  engine's paste are trusted for that.
- A mixed page-content-and-annotation selection, which clicking cannot build
  (selecting an annotation clears page content).
- A locked annotation inside the cut, which refuses the whole cut by
  `annotdelete::gate`; `cutting_a_redaction_mark_is_refused` drives a refusal.
