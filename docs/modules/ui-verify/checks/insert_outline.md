# `ui-verify/checks/insert_outline`

`inserted_pages_bring_their_bookmarks` — **pages inserted from a file bring
the bookmarks and named destinations that reach them, after this document's
own, and one undo takes all of it away.**

Driven off the desktop through the scripted pointer, so it runs under
`--no-input`. Edit mode is rung at launch (`PDFCER_DIAG_INVOKE=mode.edit`);
the Pages tab, Insert from file and the Insert button are clicked, so their
reachability is part of what passes. The picker is answered by
`PDFCER_DIAG_INSERT_PATH`.

## Fixtures

Both built by their `PROVENANCE.py`, and both read by the engine
(`pdfcer list-outline`, and `pdfcer add-named-dest --name sheet` refusing the
key as taken on each).

| file | pages | bookmarks | `/Names /Dests` |
|---|---|---|---|
| `fixtures/outlined-target.pdf` (opened, as a copy) | 1 | `Target cover` → page 1 | `sheet` → page 1 |
| `fixtures/outlined-source.pdf` (inserted) | 2 | `Source sheet A` → page 1, `Source sheet B` → page 2, explicit destinations | `sheet` → page 1 |

The shared key `sheet` is what exercises the rename. The source's bookmarks use
explicit destinations, so the carried-item count does not depend on the
named-destination half.

## Steps and oracles

| step | requires |
|---|---|
| raise the Bookmarks dock tab | the last complete frame's `bookmark-row` titles are `[Target cover]`; anything else is a SKIP, the panel or the fixture is not what the check needs |
| Pages ▸ Insert from file, press `insert-pages.insert` | an `insert-pages` line, then `insert-pages-carried outline_items=2 dests=1 dests_renamed=1 outline_dropped=0` |
| ditto | the `insert-pages` disclosure contains *"2 bookmarks came across"* (`text::insertoutline::carried`) |
| ditto | the rows read `[Target cover, Source sheet A, Source sheet B]` — `InsertedOutline::Append` puts the carried items after the target's last, in source order |
| Ctrl+Z once | one more `undo-applied`; the `canvas` line after it reads `pages=1`; the rows read `[Target cover]` |

A frame is the `bookmark-row` lines between the last two `bookmarks-panel`
census lines: the panel writes the census and then every row each frame,
whether or not the row is scrolled into view, so rows after the last census may
belong to a frame still being written.

What the wrong outcome cannot produce: an insert that drops the outline traces
`outline_items=0 dests=0`, and the rows stay `[Target cover]`; an insert that
overwrote the target's destination traces `dests_renamed=0`.

## Falsification

`pages::carry()` returning `InsertOptions::default()` (`InsertedOutline::Drop`,
no destinations) fails at the counts: `outline_items=0 dests=0 dests_renamed=0
outline_dropped=1`.
