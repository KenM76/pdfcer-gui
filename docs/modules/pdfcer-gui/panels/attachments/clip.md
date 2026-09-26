# `panels::attachments::clip` — **copy, cut and paste an embedded file**

The three controls that let an attachment move from one open document to
another. `tools/gates/check-verb-coverage.sh` asserts that this shell names
`copy_attachment`, `cut_attachment` and `paste_attachment`, because an
engine verb no surface reaches is a capability the operator does not have.

## Why the controls are in this panel and not on the ribbon

The same argument `panels::bookmarks::clip` makes, and it is stronger here.
Every other attachment verb is in this panel — Attach, Save a copy, Remove —
because an attachment is only ever *seen* here. It is not on a page, it has
no `/Rect`, and no canvas gesture can select one.

⇒ So `Ctrl+C` and `Ctrl+V` are **not** wired to it. Those chords belong to
the canvas, and a chord whose meaning depends on which panel has focus is
the kind of thing this project's standing rule about conventional
interactions forbids inventing.

## The question that MUST be asked before the press

**Does the destination already have a file of that name?**

`attach_file` builds its name-tree patch with
`entries.retain(|(k, _)| k != &name_bytes)` before pushing the new entry —
so a same-named attachment is **replaced**. Not refused. Not suffixed. The
existing entry is dropped from the tree and the new one takes its key.


A **statement**, not a confirmation. One paste is one `EditSession` command
and therefore one `Ctrl+Z`, which is enough for the rule that a destructive
verb be *confirmed or clearly undoable*. What it must not be is silent.

## What CANNOT be asked in advance, and is filed

A document whose `/EmbeddedFiles` root holds `/Kids` rather than `/Names`
refuses the attach entirely (`AttachmentTreeUnsupported`), and rightly:
inserting into a multi-node tree means repairing every `/Limits` range up
the chain, and getting that subtly wrong stops the document's *existing*
attachments resolving.

`AttachmentNotes` reports six conditions and the tree's shape is not among
them. So that refusal arrives **after** the press, in words, through the
ordinary decline path. Honest, and one press worse than R9 wants; reported
to the engine rather than worked around.

## Where the Paste control lives

At the **top of the panel**, above the list, and drawn **only when the
clipboard holds an attachment** — R9's rule that an unavailable capability
renders nothing rather than a greyed stub. Above rather than below because
the list can be long and a control at the bottom of a scrolled list is one
the operator has to hunt for.
