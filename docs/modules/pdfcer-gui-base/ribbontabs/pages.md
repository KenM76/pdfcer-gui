# `pdfcer-gui/shell/manifest/pages`

The **Pages** tab — *what am I doing to the set of pages?*

`RIBBON_IA.md` §5.3. Three groups: Insert, Organise, Transform.

# Why this tab exists at all

It is the largest structural change in the new layout, and it is a
discoverability fix rather than a feature:

> **Page operations are hidden.** Insert, delete, extract, reorder,
> split and merge exist and work, but live in the thumbnail rail's
> selection action bar and in a `Tools ▸ Batch` pane. Nothing on any
> ribbon tab says "pages". A user who wants to delete page 7 has no
> path that starts at the ribbon.

Every command here already worked. What was missing was a name for the
place they live.

# The organising rule, and the line it draws against Tools

Every command on this tab operates on **the current document's page
set**, and every one of them respects the thumbnail rail's current
selection when there is one. That is what distinguishes it from Tools:
**Pages changes *this* document; Tools produces *new* files.**

The rule matters because two pairs of commands would otherwise look
like duplicates and are not:

| Here | On Tools | Difference |
|---|---|---|
| `pages.split` | `tools.split_files` | this document, at boundaries you choose · one or more files on disk, originals untouched |
| `pages.merge_into` | `tools.merge_files` | adds pages to this document · combines files into a new one |

They are four distinct commands with four distinct ids, and their
tooltips point at each other so an operator who reached for the wrong
one is told where the other lives. This is not a P1 violation: P1 is
about one command with two homes, and these are two commands.

# The thumbnail rail keeps its action bar

Also not a P1 violation — the rail is a panel, not a tab, and a
selection-scoped action bar next to the selection is correct. The
ribbon becomes the *discoverable* path and the rail stays the *fast*
path. The same relationship the QAT has to the File tab.

# What is absent

The whole **Stamp** group — watermark, header & footer, Bates numbering
— is **N**, so the group is not here at all rather than here and empty.
`Insert blank` is **C**: `pdfcer-core` can do it and no GUI reaches it.
Crop, Resize, Replace and Insert scan are **N**. All are in
[`super::PLANNED`].
