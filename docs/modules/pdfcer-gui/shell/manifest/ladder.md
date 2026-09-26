# `shell::manifest::ladder` — which groups give up their rows first

**The editorial half of S3.** `egui-shell`'s
[`egui_shell::ribbon::plan::collapse`] knows *how* to collapse a group; it
deliberately does not know *which*, because that answer is a judgment about
this application's commands and the shell is forbidden to hold one (R7).
This file is where pdfcer answers.

## ★★★ Why one table and not a `.collapses_at(n)` on each group

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

★ **Lower collapses first**, and the numbers are deliberately sparse (1, 2,
3, 4) rather than dense, so a group can be inserted between two existing
rungs later without renumbering the tab.
