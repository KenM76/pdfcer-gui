# `ui-verify/checks/bookmark_edit`

`a_bookmark_can_be_renamed_and_removed` — **the panel that could only ever
create.**

# What this proves, and why the two verbs are one check

`pdfcer-core` `Pass 156.0` shipped `set_outline_title` and
`delete_outline_item` together, with the engine's own note saying *"bookmarks
could be created and not changed — renaming is the commonest bookmark edit
there is"*. Both reach the operator through the **same block**: click a row,
and a *Selected bookmark* section appears above the list carrying a name
field and a Remove.

⇒ So one check drives one gesture (the row click) and then both verbs. Two
checks would each pay for a launch, a mode click, a panel open and a
bookmark authored — about four seconds apiece on this machine — to assert
two halves of one surface that cannot appear separately.

# ★★★ The fixture has NO outline, and that is the point rather than a
limitation

`SW41177.pdf` and every other CAD export in this project's fixture set are
exported without bookmarks. So phase A **authors one**, through the same
authoring row `bookmark_can_be_written` drives, and phases B and C then act
on it.

That is a real dependency and it is stated rather than hidden: if
`bookmark_can_be_written` fails, this check SKIPS rather than reporting a
rename defect, because there is nothing to rename and *"could not set up"* is
a different fact from *"the feature is broken"*. A harness that cannot tell
those apart reports the wrong module, which is what seven of ten failures in
the 2026-08-28 sweep turned out to be.

# ★★ The rename oracle is the PANEL's census, not the trace alone

`rename-bookmark …` says the engine accepted the call.
`bookmarks-panel items=N` unchanged says the outline still holds one item
rather than two — which is the assertion that a rename did not silently
become an *add*, and that is not a hypothetical failure: both verbs take a
title, both go through `vector_edit`, and a dispatch arm routed to the wrong
one produces a document that looks right until somebody counts.

The title itself is deliberately **not** asserted from the trace: the panel
traces the LENGTH of a bookmark name and not its text, because a bookmark's
name is the operator's own words about their drawing and the trace is a file
a harness keeps. `chars=` moving from 5 to 6 is the evidence available, and
it is enough to distinguish the two builds that matter.

# ★★★ The delete oracle is the COUNT, and the reason is the engine's

`delete_outline_item` removes the subtree. This fixture's outline is one
top-level item, so `descendants=0` and the count goes 1 → 0. A check that
asserted only *"shorter than before"* would pass on every defect the engine
itself injected on that Pass — its own words:

> The delete test asserted the bookmark list was *"shorter than before"*, and
> every defect we injected leaves a shorter list. One leaves it **empty**,
> which is also shorter.

So this asserts the count **exactly**, and the panel's promised subtree size
against the engine's reported one.

# Phases

| Phase | Does | Expected |
|---|---|---|
| A | open the panel, type a title, press Add | `add-bookmark`, and `items=1` |
| B | click the row, retype the name, press **Enter** | `bookmark-rename chars=6`, `rename-bookmark`, and `items` **unchanged** |
| C | press Remove | `bookmark-delete descendants=0`, `delete-bookmark`, and `items=0` |

★★★ **Two of those three gestures were missing until 2026-08-29, and without
them this check could not pass on any build.** It went from the Add press
straight to reading `bookmarks.rename` — on a comment saying *"fall through
to the row click below"*, and there was no row click below — and then typed
six letters and read the trace without committing them. `BookmarksUi`'s
selection is set only by a row click, and `edit::rename_row` raises its
action only on the button or on Enter, so both halves reported a working
panel as broken.

★ Enter, not the Rename button: that button publishes no `ui_rect` region,
so there is no coordinate for a harness to aim at. `edit::rename_row` commits
on either, and its own header commits to the keystroke.

★ The row publishes no region either — it is a frameless `Button` in a
`ScrollArea` — so the aim comes from the `bookmark-row … rect=` line the
panel traces per row. See [`ROW`].
