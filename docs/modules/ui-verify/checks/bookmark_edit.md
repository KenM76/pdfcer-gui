# `ui-verify/checks/bookmark_edit`

`a_bookmark_can_be_renamed_and_removed` — **the panel that could only ever
create.**

# Driven off-screen

The check drives the scripted pointer in a window placed off the desktop, so it
runs under `--no-input` while the operator uses the machine. It needs `--pdf`;
`fixtures/four-pages.pdf` (four bookmarks already) is the one it was driven on.
Every footer control is first scrolled wholly inside the panel with
`bring_into_body`, because on a 900-high window the open footer's controls start
below the panel's bottom edge.

Falsified both ways: with the footer storing its drawn height it fails at the
rename (the field is below the panel and the typing reaches nothing); with a
label drawn above the list on selection it fails on the row moving.

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

# The fixture has NO outline, and that is the point rather than a
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

# The rename oracle is the PANEL's census, not the trace alone

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

# The delete oracle is the COUNT, and the reason is the engine's

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

**Two of those three gestures were missing until 2026-08-29, and without
them this check could not pass on any build.** It went from the Add press
straight to reading `bookmarks.rename` — on a comment saying *"fall through
to the row click below"*, and there was no row click below — and then typed
six letters and read the trace without committing them. `BookmarksUi`'s
selection is set only by a row click, and `edit::rename_row` raises its
action only on the button or on Enter, so both halves reported a working
panel as broken.

Enter, not the Rename button: that button publishes no `ui_rect` region,
so there is no coordinate for a harness to aim at. `edit::rename_row` commits
on either, and its own header commits to the keystroke.

The row publishes no region either — it is a frameless `Button` in a
`ScrollArea` — so the aim comes from the `bookmark-row … rect=` line the
panel traces per row. See [`ROW`].

## Item notes

### `const MODE`

Read's dock does carry Bookmarks (see `app::modes::defaults` — *Read: Pages,
Bookmarks*), so the panel is on screen and `dock.body.view.panel_bookmarks`
is published. What Read does **not** carry is the *authoring row*: the
application deliberately withholds `bookmarks.new_title` and `bookmarks.add`
there, and `bookmark_add`'s second half —
`read_mode_offers_no_bookmark_authoring` — asserts that absence and passes.

So this check's phase A, which types a title into that box to give itself a
bookmark to rename, could never begin. Its SKIP reason was accurate and
unhelpful: *"no `bookmarks.new_title` region … Regions beginning
`bookmarks`: none."* Nothing was broken; the check was asking Read for a
control Read is specified not to offer.

⇒ **Review**, which is the mode `bookmark_add` authors in and the one whose
whole posture is marking up somebody else's drawing. Its default dock also
carries Bookmarks, so no extra toggle is needed — which is why [`INVOKE`]
stopped toggling the panel at the same time.


It is kept rather than deleted because it is the inference that produced a
defect, and a reader who never sees it will draw it again.

*Carried in the default dock* means the panel is **mounted**. It does not
mean the panel is **raised**, and those came apart on the very day that
sentence was written. Ken asked on 2026-09-05 for *"no tabs in the left
side bar when the left rail is visible"*, and the dock now draws no tab
strip whenever a rail can raise every panel in the stack — which, in this
application, it always can. `dock.tab.view.panel_bookmarks` has not been
published since.

This check calls [`driving::raise_dock_tab`] and **discards the bool**. It
got `false`, correctly, and carried on; the panel it then read was whatever
happened to be in front. Three checks in this family did the same and all
three skipped for a week with a green suite, because a SKIP is not red.

The repair is in `raise_dock_tab` — it now falls back to the left rail,
which publishes `rail.tabs.<id>` in every mode and cannot be folded away —
so nothing here changed and this check passes again. The rule it earned is
on that function: **a helper that can decline must not hand back a `no` a
caller is free to ignore.**

The general shape, and it is this project's commonest: **a check that
SKIPs is not red, so a check aimed at a surface the application has since
been specified not to have can sit there for ever looking like an ordinary
wrong-fixture skip.** The tell here was two checks disagreeing — one
asserting the row is absent in Read and passing, three others requiring it
in Read and skipping.

### `const ROW`

This is how the row is aimed at, and it is not a `ui_rect` region because a
row is not one: `panels::bookmarks::rows` draws a frameless `Button` per
item inside a `ScrollArea` and traces
`bookmark-row level=… title=… page=… enabled=… rect=…` from the same
`Response`. `rect` is `egui::Rect`'s `Debug`, so `TraceLine::get_rect` reads
it and `WindowFrame::declared_center` converts it exactly as it converts a
declared region — same space, same origin, same frame.

`.last()` is the most recently drawn row of the most recently drawn
frame. This check authors exactly one bookmark before it aims, so there is
one row and the choice does not arise; a check that authored several would
have to filter on `title=` instead.

### `const RENAME_KEYS`

Six letters against five is the whole oracle: the panel traces the length
of a bookmark name and not its text, so a rename to a same-length word would
be indistinguishable from no rename at all in the only evidence available.
