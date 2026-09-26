# `app::actions::merge` — **Combine several PDFs into one new file**

`OPERATOR_REQUESTS.md` row **O68**, in the operator's words:

> *"Also the Merge files and Split files buttons don't do anything."*

This module is the body of `tools.merge_files`. Without it the command is
registered, drawn on the Tools tab, given an icon and a tooltip promising
*"Combine several PDFs into one new file"* — and every press falls to
`PdfcerApp::dispatch_command`'s catch-all, traces `command-unimplemented`,
and does nothing an operator can see.

## The engine is not the blocker

`pdfcer_core::pageops::merge` implements Acrobat's Combine Files behaviour
**whole** — per-source bookmark generation, `Doc0_`/`Doc1_` duplicate-field
auto-renaming, the first source's `/Info`, no inherited page-label scheme.
What was missing here was a caller, never a capability, and the rule that
catches that class of entry in the reachability register is worth keeping
next to the verb it unblocked:

> *"A blocker naming a missing HOST is weaker than one naming a missing
> capability, and it goes stale the moment any other host will do."*

A dialog is a host. So is a picker. Nothing about combining files on disk
needs a batch pane.

## Why this is not `vector_edit`

Because a merge **produces new file bytes and touches neither the session
nor the undo log**. That is `app::actions::extract`'s argument, and this is
the same shape one step further out: extract reads the open document, this
reads several documents none of which need be open.

Nothing here bumps `edit_epoch`, drops a raster, invalidates a cache or
writes an undo entry, because nothing about the open document changed.

## Rule 4, and it decides what this may draw

A merge writes somewhere else. **Nothing about it may appear in the page
view of the open document** — no badge on the source pages, no provisional
tint, no preview overlay. The disclosures the engine returns are real and
are owed to the operator, and they go to the status row like every other
off-canvas disclosure in this shell.

Two of them matter enough to name here, because they are pdfcer policy that
an operator would otherwise discover by comparing files afterwards:

* the combined document takes the **first** source's `/Info` and no other's;
* it carries **no page-label scheme**, because there is no single source to
  inherit one from and Acrobat does not generate one either.
