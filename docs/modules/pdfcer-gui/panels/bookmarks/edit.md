# `panels::bookmarks::edit` — renaming a bookmark, and removing one with
everything under it

## What this closes

Without these two controls a bookmark can be **created and never changed**,
and renaming is the commonest bookmark edit there is — a heading typed with
a typo would otherwise have to be deleted and re-authored, losing its
destination and everything filed under it. `EditSession::set_outline_title`
and `EditSession::delete_outline_item` are the engine verbs these controls
drive.

## The gesture is already there: the selected row

No new selection mechanic is invented. Clicking a bookmark row **already**
meant two things — *"take me there"* and *"this is the parent for the next
add"* — and this makes it mean a third: *"this is the one I am editing."*
All three are true of the row the operator pointed at, which is what makes
the overload honest rather than a shortcut, and the alternative is a second
selection gesture (a checkbox column, a right-click menu) that would have to
be taught for two verbs.

The consequence is stated rather than left to be discovered: pressing **Move
to top level** in the add row clears the selection, so these controls
disappear with it. That is correct — nothing is selected — and it is why
this block names the bookmark it acts on in words
(`bookmark_edit_selected`) rather than relying on a highlight in a list that
may be scrolled out of view.

## Delete is UNDOABLE, not confirmed — and that is the choice made here

A destructive verb must be **confirmed or clearly undoable**. This surface
takes the second, for three reasons, in order of weight:

1. **One press is one `EditSession` command.** The engine plans every relink
   — the previous sibling's `/Next`, the next one's `/Prev`, the parent's
   `/First`/`/Last`, every open ancestor's `/Count` — *inside* that single
   command. So `Ctrl+Z` restores the entire subtree and there is no
   half-undone state to land in. An undo that is total is a better safety
   net than a question, because it also covers the operator who *meant* to
   press it and changed their mind afterwards, which a confirmation does
   not.
2. **The question a confirmation would ask is the wrong question.** *"Are
   you sure?"* carries no information. What the operator actually needs is
   *"this takes the eleven bookmarks filed underneath as well"* — and a
   modal is a bad place to put that, because it arrives after the decision
   has been made and is read by nobody by the fourth time. So it is on the
   panel, beside the button, **before** the press.
3. **Modals train themselves away.** An operator who has dismissed the same
   dialog four times dismisses the fifth without reading it, and the fifth
   is the one that mattered. This is the same reasoning the ce-dimension
   group window applies to its own Delete, which asks a *substantive*
   question (where do the members go?) rather than a ceremonial one.

⇒ The obligation that replaces the modal: **the blast radius must be
visible before the press and reported after it.** Both are done, and the
two numbers are allowed to differ — see [`delete_row`].

## Why the encryption and certification refusals are not pre-empted here

Both verbs refuse an encrypted document and refuse one whose certification
signature forbids the change. Neither is checked before drawing the buttons,
deliberately and consistently with every other authoring surface in this
shell — including [`super::add`], which has the same two refusals and the
same absence of a guard.

R9 says an *unavailable capability* renders nothing, and the distinction it
turns on is whether the shell can know. Here it cannot know honestly: the
certification gate is a property of a signature the panel does not read, and
duplicating the engine's test at the widget would produce a second
implementation of a permission rule that can drift from the first. A refusal
is traced and the document is left alone, which is this shell's whole
response to any engine refusal today; wording declines is `FEATURES.md`'s
"Worded decline" row and belongs to `super::super::super::app::actions::apply`'s
`Err` arm, not to this file.

## Reorder and re-parent are NOT in this block

They exist — `move_outline_item` and `set_outline_open` — and
[`super::reorder`] is the surface for both. The move is a **drag on the
row**, which is the conventional gesture every outline panel uses and which
answers the operator's standing tie-breaker: *"make it work the way other
programs do."* A pair of *Move up* / *Promote* buttons is the easy thing to
add here and would be a second, worse idiom beside the one
[`crate::panels::pages`] already established for reordering.

**Absent:** a verb that deletes the whole outline.
`EditError::OutlineRootIsNotAnItem` refuses the root by name; emptying an
outline is a different act and gets its own verb when it is wanted.
