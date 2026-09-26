# `app::actions::bookmarks` — the verbs whose subject is one entry in the
document's outline

A sub-enum beside `PageAction` and `DimensionAction`, filed under the rule
`super`'s declaration of `action` states: the family of variants that
**grows** is the one that becomes a sub-enum.

## What makes these a family rather than a size-driven cut

Every variant here **addresses its operand by `ObjId`** and by nothing
else, and that is a property no other family in the enum has for the same
reason. The reason is the one the engine reported from its own CLI:

> *"the indices shift after every add … I got this wrong myself while
> driving the command and nested something two levels deeper than intended,
> and the output looked entirely plausible."*

An outline is a tree that every edit to it renumbers. A position in the
walk — "the fourth row", "the second child of the first" — names a
different bookmark after any add, any delete, and any undo of either.
`OutlineItem::id` exists precisely so a GUI does not have to hold one, and
its own doc comment says so: *"identity is what a GUI needs and the tree
cannot otherwise supply."*

So the shared property is not *"they are all about bookmarks"*, which would
be a subject label. It is that **every one of them is resolvable after the
frame that raised it**, which is the one thing the action funnel requires of
an operand and the one thing a tree position cannot promise.

## `/Count` is two different quantities, and the sign carries open/closed

§12.3.3 is where implementations of this feature go wrong. The engine's
table:

| | root `/Outlines` (Table 152) | an item (Table 153) |
|---|---|---|
| counts | all visible items **including** the top level | visible **descendants**, excluding itself |
| sign | **cannot** be negative | **positive = open, negative = closed** |

A **closed** item contributes exactly **1** to its ancestors' counts,
however large its subtree is. Three consequences, and this module is built
around all three:

1. **Nothing here diffs a count to describe an edit.** Adding a bookmark
   under a collapsed ancestor leaves the document's total unchanged, so a
   surface reporting *"added N"* from a root-count diff reports **zero for
   a correct save**. [`BookmarkAction::Add`] adds one bookmark and the panel
   says one bookmark; there is no number to get wrong.
2. **A delete's count comes from the engine, not from the tree we drew.**
   See [`BookmarkAction::Delete`].
3. **`open` is the only reason a disclosure about visibility can be
   written at all.** `pdfcer_core::outline::OutlineItem::open` is the shell's
   read of that sign, and §12.3.3 defines no `/Open` key, so the sign is the
   only carrier there is.

## What is deliberately absent

A verb that deletes the whole outline. `EditError::OutlineRootIsNotAnItem`
refuses the root by name, because deleting it is *"a different act that gets
its own verb when it is wanted"*, and this shell does not want it yet.

Where a capability the engine does not have would need a control, this
module grows **no variant** and the panel draws **no handle** — R9: a
capability that does not exist renders nothing, not a greyed promise.

## The `ObjId` rule binds the destination too

[`BookmarkAction::Move`] addresses its **destination** by `ObjId` as well as
its operand, because `OutlinePlacement` is built from anchors rather than
positions, for the reason its own doc comment gives about this exact
surface: *"A shell that reads a panel, lets the operator drag a row, and
then calls with the index it read has a race with its own undo stack."*
