# `app::dispatch::format` — the contextual Format tab's command arms

## The seam

The same one `dispatch::pages` took, one tab over. [`super`]'s subject is
*"a command id becomes an intent"* across the whole ribbon; this file's is
the **Format tab's** share of it. They change for different reasons: a new
tab or a new dispatch convention touches the parent, a new verb on the
thing-you-just-clicked touches this.

Format is a small tab and this file grows with it: every `format.*` entry
in `manifest::PLANNED` lands here when it ships.

## What the selection arms have in common, and it is not the tab

They act on **the selection**, and each has to answer the same question
first: *which of the two index spaces is this?* A selection can name a page
object — an index into the page's own paint order, which every
`EditSession` verb accepts — or a **leaf**, an object painted from inside a
form XObject, whose token range indexes the form's content stream and which
no paint-order verb can address.

Keeping them together is what makes their answers reviewable side by
side:

| arm | what it does about a leaf |
|---|---|
| `format.delete` | raises nothing, and **says why** — an outline with an unexplained dead Delete reads as a broken program |
| `format.select_form` | selects the leaf's outermost enclosing form, which *is* an operand |
| `format.properties` | opens the panel, which describes either kind |

## Why the arms re-ask what `enabled_when` already asked

Because `enabled_when` greys a ribbon item and **enforces nothing**. Every
non-ribbon route — the context menu, a chord, a future script — reaches the
dispatcher without consulting it. That was settled on this project after a
blanket guard at the top of `dispatch_command` was written and two tests
refused it, for making `Ctrl+Z` on an empty stack do nothing *and say
nothing*: greying is a hint, the worded decline is the answer, and only the
arms that would otherwise act unconditionally need the check — and they must
say why.

## Item notes

### `fn delete_the_selection`

# Why this is a function and not four lines inside the arm

Because the rule is destructive and there must be exactly one of it. Both
claimants — this command and the Delete **key** in `canvas::keys` — ask
`crate::canvas::deleting::subject`, which answers for whichever rung of the
ladder the operator is on.

Two hand-written copies of one destructive rule is exactly what
`deletable_objects_on`'s own header refuses, and the divergence is not
hypothetical: it has happened over form fields, where the key reached a
selected widget and this command did not, so Delete-the-key and
Delete-the-command acted on different things — which `app::keyboard`'s
header calls the defect the single dispatcher exists to make impossible.

**Both** callers therefore ask [`crate::canvas::deleting::subject`], and
the Part and Node rungs come with it: the ribbon's Delete removes one line,
one label or one corner point exactly as the key does, because there is
only one answer to ask for.

# The provider, and why it is read here rather than passed in

The dispatcher is not inside the canvas's frame — it runs from the command
funnel, after the ribbon or a menu has already closed — so it cannot inherit
the canvas's borrow the way `canvas::keys` does. (It takes an
`egui::Context` for a different arm's parked operand; a context is not a
frame and buys this one nothing.) `doc.page_objects()` is keyed on
`(page, edit_epoch)` and the canvas built it on the frame that drew the
selection outline the operator is looking at, so this is a cache read rather
than a second `decompose_page` — the same key, the same epoch, the same
`Ref`.

The `Ref` is taken and dropped inside the `map_or_else` below, before
anything is pushed, because `doc` is borrowed immutably for the whole arm and
holding a `RefCell` guard across an `actions.push` is how a re-entrant read
becomes a panic nobody can reproduce.

# What it deliberately does NOT do

It does not clear the selection, it does not take the erase preview, and it
does not word anything. The first two belong to `actions::vector::apply`
(which owns the four-step protocol and the O63 preview) and the third to
[`crate::text::deleting`]. This function's whole job is *ask, then raise*.
