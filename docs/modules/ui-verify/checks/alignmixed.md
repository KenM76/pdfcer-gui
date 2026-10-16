# `ui-verify/checks/alignmixed`

Align, drag and delete with a selection that mixes parts of placed drawings with page
objects, on `fixtures/parts-beside-objects.pdf` (built by its `.PROVENANCE.py`): a green
page rectangle at (300,220)..(360,260), drawing Fm0 holding a red part
(60,60)..(120,100) and a blue part (60,140)..(100,170), and drawing Fm1
holding an orange part.

The two mixed checks press Ctrl+Alt+7, Align centres left to right, because
it moves every entry not already on the selection's centre line. Align right
would leave whichever entry already holds the right edge where it is, and
that entry could be the very kind the check is about.

`a_part_and_a_page_object_align_together`: the red part is entered at (90,80)
as `leafplace` does and green is shift-clicked at (330,240). PASS needs, in
order:

1. A `canvas-selection` with `sel=2` and `level=Object`. Entries are kept
   sorted with page objects first, so `first=` names green; the part's
   presence is read in step 3.
2. No `align-declined` after the key.
3. A `move-mixed-applied` holding `objects=1/1`, `parts=1/1`,
   `drawings=1/1`, `one_step=true` and `notes=0`: both kinds moved, through
   `move_objects_each` and `move_objects_each_in_form`, folded into one undo
   step.

`a_part_inside_a_selected_drawing_rides_with_it`: as above, then the blue part
is shift-clicked at (80,155). The earlier click on green left the drawing, so
this click resolves to the whole of Fm0. PASS needs:

1. `sel=3` after that click.
2. A `move-mixed-applied` holding `objects=2/2`, `parts=0/0`,
   `drawings=0/0` and `notes=1`: the red part is dropped from the moves,
   because moving Fm0 moves it, and one sentence says so.
3. The funnel's `move-mixed` line has `disclosures=` other than `none`. This
   is what reached the status line, not what the action asked for.

`align_at_the_part_rung_says_why`: the red part is entered, then
double-clicked again, which stands at the Part rung (`level=Part`). Ctrl+Alt+6
must produce no `move-mixed-applied`, and must produce an `align-declined`
with `reason=part-rung`.

`a_part_and_a_page_object_drag_together`: the red part and green, selected as
above, then green is dragged by (20,-15) page units in 8 steps. PASS needs a
`move-mixed-applied` after the drag holding `gesture=drag`, `objects=1/1`,
`parts=1/1`, `drawings=1/1` and `one_step=true`. Without it the check quotes
the drag's `canvas-move` line, which names the verb that ran instead.

`a_part_inside_a_dragged_drawing_rides_with_it`: as above, then the blue part
is shift-clicked (`sel=3`, the whole of Fm0 added) and green is dragged. PASS
needs no `move-mixed-applied` and a `canvas-move` whose action names
`objects: [0, 1]`: the drawing carries its part, so the part must not be moved
a second time. The verb is `TransformObjects`, since a placed drawing has no
coordinate operands; the check reads the operands, not the verb.

`a_part_and_a_page_object_delete_together`: red part and green selected as
above, then Delete. PASS needs `delete-parts-applied` with `objects=1/1
parts=1/1 forms=1/1 one_step=true`, and the next Ctrl+Z tracing
`undo_depth=1`: `undo` traces the depth before it steps, so 1 is the whole
delete as one command.

`a_part_inside_a_deleted_drawing_goes_with_it`: as above plus a Shift+click
on blue (`sel=3`, Fm0 whole), then Delete. PASS needs `objects=2/2 parts=0/0
forms=0/0 carried=1`: the drawing takes its part, which is not deleted again.

## Falsified

- `align::moves_action` planted to drop every part move: the first two checks
  FAIL with no `move-mixed-applied` (the parts are gone, so the action goes
  down the plain page route).
- `leftout::uncarried` planted to keep every part: the carried check FAILs on
  `parts=0/0`, reading `parts=1/1 drawings=1/1 notes=0`. This is the
  double-move the check exists for.
- `canvas::deleting::object_rung` planted to return the page objects alone:
  both delete checks FAIL with no `delete-parts-applied`.
- `uncarried` planted to keep every part in `deleteparts::commit`: the
  carried delete FAILs reading `parts=0/1 forms=0/1 carried=0`, the part's
  delete refused because its drawing was already gone.
- `leftout::unmeasured`'s Part-rung arm planted off: the third check FAILs,
  reading `reason=nothing-selected`.

## What it does not prove

- Where anything lands. `objects=1/1` says the engine committed, not the
  distance moved.
- Parts of two different drawings in one Align. The canvas offers no
  gesture that selects them together: a click inside a second drawing
  resolves to the whole drawing. `leftout`'s unit test covers the grouping
  instead.
- Entries on other pages. They need the multi-page selection that the
  Objects panel builds; this is not driven.
- Rendered pixels, and the status line's visible text.
