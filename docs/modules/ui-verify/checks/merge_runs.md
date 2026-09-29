# `ui-verify/checks/merge_runs`

`merge_runs` — **select a line written in two pieces, choose Merge text runs on
its right-click menu, and the file changes.**

## What it guards

The canvas object menu offers `format.merge_text_runs` while
`selection.text_merge_offered` holds. The page's object model is built only on
the right-click's own frame (`canvas::modelneed`), so the row's answer is
parked on that frame (`canvas::runmerge::park`) and read from there while the
menu stays open. Recomputing it on a later frame answers "no operand" and
removes the row one frame after it was drawn, before the pointer can reach it.
The press re-derives the operand, so a parked answer cannot merge anything the
preflight has not just approved.

## How it drives

1. It opens `fixtures/inherited-runs.pdf` (shared with `move_line_of_text`) and
   selects Edit mode.
2. It arms the Points tool (`A`) and clicks the first line at (87, 704). That
   line is written in two runs. It requires the ladder at `Part`.
3. It right-clicks the same point and waits 35 frames. Then it requires the row
   `menu.item.canvas.object.format.merge_text_runs` to be declared and not
   retired.
4. It clicks the row.

It passes when a `merge-text-runs-applied` line follows with `merged` ≥ 2. On a
failure it quotes the first `canvas-decline-recorded` line after the pick,
which is where the merge's refusals are narrated.

## Falsification

The row is made to recompute its operand on every frame instead of reading the
parked one. The check then fails: the row is drawn on the click frame and
retired on the next.
