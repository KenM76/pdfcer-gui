# `ui-verify/checks/group_of_groups`

`a_group_groups_with_a_third_markup_and_comes_apart_one_level`, on
`fixtures/annots-nest.pdf`: three markups on one page, a square (`/NM (SQ)`),
a circle (`/NM (CI)`) and a second square (`/NM (TR)`), each picked by a point
on its outline.

PASS needs, in order:

1. Square clicked, circle Shift-clicked, Ctrl+G: `markup-group-applied
   members=1`, without `recorded=true`.
2. A click on the square (selects the group of two), the third markup
   Shift-clicked: `annot-toggled after=3`.
3. Ctrl+G: `markup-group-applied recorded=true` (the engine recorded the
   earlier group, `GroupNesting::Recorded`).
4. A click on the circle traces `annot-group-select mates=2 selected=3`.
5. The first saved copy: the square and the circle carry `/IRT` to the third
   markup and a `/pdfcer_Groups` record; the third markup carries no `/IRT`.
6. Ctrl+Shift+G: `markup-ungroup-applied restored=1`.
7. A click on the square traces `annot-group-select mates=1 selected=2`; a
   click on the third markup traces `annot-select` and no
   `annot-group-select`.
8. The second saved copy: the square carries `/IRT` to the circle again; the
   circle and the third markup carry none.

The saved copies are what make the "first group is back" claim independent of
the trace: the trace is the GUI's report of the engine's answer, the file is
the answer.

## Falsified

- The build before groups of groups existed → FAIL at step 3: after the third
  markup is Shift-clicked (`annot-toggled after=3`) Ctrl+G traces no
  `markup-group-applied` at all.

## What it does not prove

- The status-line sentences; they are unit-tested in `text::grouping`.
- The refusals (a reply, a ce dimension, two pages); the engine's
  `AlreadyLinked` and the others reach the status line through
  `actions::group::refusal_for`, unit-tested there.
- More than two levels of nesting.
