# `ui-verify/checks/bookmark_clipboard`

`a_bookmark_subtree_can_be_copied_and_pasted` — the driven proof of
`OPERATOR_REQUESTS.md` **O59**'s third and last item.

# The one operation in this program Acrobat cannot do

`pdfcer-core`, 2026-08-29: *"Acrobat cannot do this between two files at all;
Adobe's own documentation says so by name."*

That is worth stating in a check file, because it changes what a failure
here means. For most of this suite a red result says *pdfcer is behind a
reference implementation*. Here it says *pdfcer has lost something nothing
else offers*, and there is no workaround to fall back on — an operator who
wants a chapter's bookmarks in another drawing does it by hand, one at a
time, or not at all.

# The oracle, and the two readings that would not have been enough

| reading | what green would prove |
|---|---|
| `bookmark-copy` | the panel asked the engine for a clip |
| `bookmark-paste` | the panel raised an action |
| `bookmark-paste-applied items=N` | `paste_outline_item` returned `Ok` with a count |
| **the panel's own census grew** | **the operator got more bookmarks** |

The third is very nearly enough and is still not, for one specific
reason: `paste_outline_item` returns `Ok(default())` — `items_pasted: 0` —
on an **empty clip**, without touching the document. So a build whose copy
produced an empty clip would emit every line above, report success, and add
nothing. Only the census distinguishes it.

⇒ So this asserts on `bookmarks-panel items=`, which is the panel counting
what it is actually drawing.

# The sequence

1. open the bookmarks panel;
2. author a bookmark, so there is something to copy — reusing the authoring
   row `bookmark_can_be_written` owns;
3. read the census;
4. press **Copy**, then **Paste**;
5. assert the census grew.

Step 2 exists because the fixture corpus is **not uniform** —
`bookmark_move`'s header records the same discovery — and a check that
assumed a bookmark was already there would SKIP on half the corpus while
reporting the fixture as the fault. Authoring one first makes the check
independent of what the document arrived with.

# What this does not prove, said out loud

**Cross-document paste**, which is the whole point of the feature. This
pastes back into the same document, because driving two documents and moving
a clip between them is a harness capability that does not exist yet. The
clip is application-scoped by construction — it lives in `egui::Memory`, not
on the document — so the mechanism is the same one either way, but *the same
mechanism* is an argument and not a measurement. Recorded as a gap.

And the **dropped-destination warning**, which needs a clip whose deepest
page exceeds the destination's page count. Same reason: two documents.

## Item notes

### `const MODE`

**Review**, because Review is the mode that offers the authoring row (see
[`INVOKE`] point 3) and the mode `bookmark_add` uses for the same reason.
Its default dock mounts Bookmarks, so the panel needs raising to the front
of its stack and not toggling into existence.

### `const ROW`

A trace **event**, not a `ui_rect` region — which is what the first
version of this check got wrong and what its own failure message could not
tell it. The panel writes one `bookmark-row` line per row per frame carrying
`row=[[x y] - [x y]]`, and publishes `ui_rect` only for its two authoring
controls. Asking `declared_names(.., "bookmark")` therefore returned the
authoring row and nothing else, and the check concluded the Copy control was
missing when in fact the selecting click had never been aimed anywhere.

⇒ Ask what a check SAMPLED before asking what is broken. Fifth instance on
this project, and the first where the wrong sample was a *region* where the
truth was an *event*.

### `const TITLE_KEYS`

Two letters, because every character is a synthesised keystroke through
the OS and a longer title buys nothing. Letters rather than digits so the
row is unmistakable in a trace beside page numbers.

### `fn wait_for_region`

This replaced three fixed `settle` calls and is the difference between a
check that passes most times and one that passes.

`declared` is not a snapshot. The application's `ui-rect` channel is a
**change log** — it emits when a rect moves and a `ui-rect-gone` when a
control stops being drawn — so `declared` answers *"was this region alive at
the moment the trace was read?"* Reading it once, after a fixed wait, asks
that question at an arbitrary point in a layout that is still moving.

And this panel's layout moves a great deal: the invoke chain changes MODE
(which reconfigures the whole dock, tearing the panel down and rebuilding
it) and then opens the panel, and selecting a bookmark inserts an edit block
that pushes the list two hundred points down. A fixed settle that is nearly
long enough produces a check that fails **intermittently and differently
each time** — which is exactly what happened: alternating between *"the
authoring row is not on screen"* and *"the Copy control is not on screen"*,
on a build where both were fine.

⇒ Polling asks the question repeatedly and stops at the first *yes*, which
is what a person watching the screen does. It is not a widened tolerance:
the caller still fails if the answer is never yes.
