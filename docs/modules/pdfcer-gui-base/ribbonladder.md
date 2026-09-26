# `ribbonladder` — which groups give up their rows first

**The editorial half of S3.** `egui-shell`'s
[`egui_shell::ribbon::plan::collapse`] knows *how* to collapse a group; it
deliberately does not know *which*, because that answer is a judgment about
this application's commands and the shell is forbidden to hold one (R7).
This file is where pdfcer answers.

## Why one table and not a `.collapses_at(n)` on each group

Because a collapse priority is not a property of a group. It is a
**ranking of groups against each other**, and the only way to review a
ranking is to see all of it at once. Scattered across the eight tab
modules, the question *"is Export really less important than Document?"*
could not be answered without opening two files and holding a third in your
head; here it is two adjacent lines.

The cost is that the priority sits away from the group's definition. That
is a real cost and it is paid deliberately, with a guard: every id in this
table is checked against the built manifest by
[`tests::every_ladder_entry_names_a_real_group`], so a group that is
renamed or removed fails the build rather than silently losing its rung.

## The rule the ranking follows

Word's, read off the series `tools/word-ribbon-study.ps1` photographs:
**the group that never collapses is the one carrying the verb the operator
came to the tab for**, not the smallest one. Clipboard is wider than Editing and outlives it at
every width from 1900 down to 460, because Paste is why you are on the Home
tab.

So each tab here keeps one or two groups off the ladder entirely, and ranks
the rest by how far they are from that tab's reason for existing:

| tab | never collapses | why |
|---|---|---|
| File | File, Save, Print | open it, keep it, print it — the whole tab |
| View | Navigate, Zoom | moving around the document IS the View tab |
| Pages | Organise | reordering sheets is the reason to be here |
| Edit | Content | the edit verbs themselves |
| Markup | Shapes | the pen — everything else configures it |
| Measure | Dimension | the measuring tools |
| Tools | — | every group here is a utility; none outranks the others |
| Format | Selection | one group; collapsing it would gain nothing |

**Lower collapses first**, and the numbers are deliberately sparse (1, 2,
3, 4) rather than dense, so a group can be inserted between two existing
rungs later without renumbering the tab.

## Item notes

### `const LADDER`

A group absent from this table **never collapses**, which is the safe
default and the reason absence rather than a sentinel means "never": a tab
added later without a ladder entry behaves exactly as it did before this
feature existed.

### `fn every_ladder_entry_names_a_real_group`

The guard that pays for keeping the ranking away from the definitions.
A renamed group would otherwise lose its rung silently: the ribbon
would still work, still collapse, and simply never collapse *that*
group — a defect with no symptom until an operator's band overflows at
a width where it used not to.

### `fn the_built_manifest_carries_the_priorities`

Named separately from the test above because they fail for opposite
reasons: that one catches a stale table, this one catches an `apply`
that stopped being called — which would leave every group unrankable
and every band collapsing nothing, a state that looks exactly like the
feature having never been built.

### `fn every_tab_keeps_something_expanded`

This is the invariant that stops the ladder from being tuned into
uselessness. A tab whose every group may collapse can reach a width at
which it is a row of identical chevron buttons and nothing else: the
operator can still reach every command, and the band has stopped
telling them anything. Word never does this — Clipboard is expanded at
460 pt, the narrowest width measured.

### `fn apply`

Called once, at the end of `built_in`, so the tab modules stay lists of
commands and this file stays the only place a ranking is stated.

Silently ignores an entry naming a group that does not exist — the test
below is what makes that safe, and it is the right split: a typo should
fail the build, not the running application, and a *layer* that removed a
group at runtime should not panic the ribbon.
