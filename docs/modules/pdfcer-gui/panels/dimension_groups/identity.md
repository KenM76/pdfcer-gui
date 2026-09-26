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

## Item notes

### `fn raise_delete`

The selection move is not tidiness. `body` falls back to the default
group when the selected one has gone, which is correct and arrives **one
frame late** — for that frame the lower half of the panel would draw
against a group the document no longer has. Moving it here means the
operator never sees the flicker, and the fallback stays as the guard it
is for every path that is not this one.

### `fn rename_draft_for`

Stale means *"held for a different group"* — see the module header. It
is also what makes the field follow an **undo**: a rename undone bumps
the epoch and changes `group.name`, and the next frame's draft is
re-seeded because... it is not, and this is the honest limitation.

**The draft does NOT follow the document while it is being typed**,
deliberately, and that differs from `panels::docprops`'s
epoch-reseed. The difference is what the two fields are: a metadata box
commits on focus loss and is otherwise idle, so re-seeding it costs
nothing; a rename box is typed into and then committed by a button, and
an epoch bump from an unrelated edit — placing a dimension, moving a
page — would wipe a half-typed name mid-keystroke.

The narrow cost is that undoing a rename leaves the old name in the box
until the operator selects another group and comes back. The button
re-appears, because the draft now differs from the document, so the
state is legible rather than wrong.
