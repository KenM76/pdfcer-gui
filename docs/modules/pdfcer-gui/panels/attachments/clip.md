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

## Item notes

### `const REGION_REPLACES`

**Paired with [`REGION_FRESH`], and the pairing is not decoration.**

`crate::diag::ui_rect` is a **change log**: a region that stops being drawn
does not un-declare itself, so a harness cannot learn "this warning is not
showing" from the absence of the name. That is written up in
`D:/dev/rag/egui/a_change_log_ui_rect_trace_cannot_report_that_a_widget_stopped_being_drawn.md`.
It bites this control in particular: a check that asserts the warning is
absent in one document will read the declaration this panel legitimately
made frames earlier against **another** document that really did hold a file
of that name.

⇒ So the control declares **one of two names**, always exactly one, and a
reader takes whichever came last. An absence assertion becomes a presence
assertion, which a change log can answer.

### `const REGION_FRESH`

It has **no visible text** — there is nothing to say, and a line reading
*"this will not replace anything"* on every paste is the noise that trains
an operator to stop reading the one that matters. It publishes the button's
own rectangle under a second name, which costs a trace line and no pixels.

### `fn take`

# Why this is here and not an `Action`, unlike almost everything else

`copy_attachment` is `&self` and commits nothing, and the panel already
holds `&OpenDoc`. Routing it through the queue would gain nothing and cost
the thing `canvas::clipboard::cut` is careful about: the copy has to be able
to **fail before the delete is raised**, and an action queued behind another
action cannot report back to the code that decides whether to queue the
second one.

⇒ So Copy and Cut are widget-layer, exactly as `canvas::clipboard::copy` and
`canvas::fieldclip::copy` are, and only the delete crosses into the queue.

**And that is why `EditSession::cut_attachment` is never called.** It
exists, it works, and it folds its two commands into one undo entry with a
private method — which a shell cannot reach. Here it does not matter: the
copy half commits nothing, so copy-then-`Detach` is already **one** command
and therefore one `Ctrl+Z`. The same argument `canvas::clipboard::cut`
records for `cut_objects`. Recorded in `EDITABLE_SURFACES.md`.

### `fn the_regions_are_named_apart`

`attachments.paste` and `attachments.paste.replaces` deliberately share
a stem — the harness's `declared_names(.., "attachments.paste")` lists
both, which is wanted — but `declared(.., "attachments.paste")` is an
exact match and resolves only the button.
