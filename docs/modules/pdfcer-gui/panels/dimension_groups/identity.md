# `panels::dimension_groups::identity` — renaming a group, and removing one

## What this closes


> *"A group cannot yet be renamed or removed — pdfcer's editing engine has no
> command for either … Both are requested."*

They were requested that day and shipped the next
(`EditSession::rename_dimension_group`, `delete_dimension_group_with`), so
the sentence is gone and these are the controls that replace it.

## Deleting a populated group is the ORPHAN question, and it is asked

`pdfcer-core` refuses a populated group by default and puts the **count** in
the refusal — `EditError::DimensionGroupNotEmpty { id, members }` — and its
reply says exactly why the count is there:

> *"this group is not empty"* and *"this group holds forty dimensions"*
> prompt different decisions from an operator, and only you can put that
> question in front of them.

So the panel does. Pressing Delete on a populated group does not delete it
and does not refuse it: it **asks where the members should go**, and only
then offers a button that will succeed.

Two facts travel with that question, and both are stated before the press
rather than discovered afterwards:

- **The members are re-measured.** A ce dimension's label is derived from
  its group's scale, unit and format, so moving them changes the numbers
  they print. The engine's measured example is `70.6 mm` becoming `2.00 m`
  for the same geometry.
- **pdfcer will not delete the dimensions with the group**, and that is the
  engine's decision with a reason rather than a gap: doing it inside the
  group verb would be a second implementation of `delete_dimension`'s
  `/Annots` removal, and looping the existing verb would make undo take one
  press per member and be able to stop halfway.

## Why the rename draft carries its own `GroupId`

A half-typed name must not follow the operator to a different row. Holding
the id **with** the text makes a stale pair detectable, so selecting another
group re-seeds the field from the group actually on screen rather than
offering to rename *it* to a name meant for the last one.

That is the same hazard `dialogs::scale` names for its own captured group —
*"a group picker that moved underneath an open dialog would let them type a
number for one group and commit it to another"* — one control smaller.
