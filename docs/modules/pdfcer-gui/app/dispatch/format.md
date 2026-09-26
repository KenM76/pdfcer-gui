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
