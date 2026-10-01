# `pdfcer-gui-base/panelfooter`

**Rule:** a list panel draws nothing conditional above its rows. Anything that
appears in answer to a gesture — the controls a selection unlocks, the reset
row after a toggle, a report about the canvas selection — goes in the footer,
where appearing grows the footer upward, shrinks the list's viewport from the
bottom, and leaves every row where it was. The operator's request (O256) is
that selecting something never moves the thing he just clicked.

| Item | Contract |
|---|---|
| `list_height(ui, id)` | Height for the list's `ScrollArea`: available height less the footer's height measured last frame, never below three rows. The list is drawn with `auto_shrink([false, false])` so the footer sits at the bottom whatever the list's length |
| `show(ui, id, panel_height, title, always, tools)` | Separator, then `always` (unconditional), then a `CollapsingHeader` titled `title`, collapsed by default, whose body is `tools` inside a `ScrollArea` capped at half the panel. `tools = None` draws no header — Read mode, where every tool would change the document. Publishes the header as region `id`, traces `panel-footer id=… open=… height=… wants=…`, stores the height it *wants* for `list_height` and repaints when it changed |

**Why measured last frame.** egui's `Panel` would carve the space, but a
non-resizable panel expands to its previous rect every frame and so never
shrinks after the header collapses. A stored height with one repaint on change
is exact on the frame after any change and invisible to the operator.

**Why the height it wants, not the height it got.** The list takes whatever
the stored height leaves, and the open body's `ScrollArea` takes whatever the
list leaves. Storing the drawn height closes that loop on itself: the footer
records the sliver it was given, the list keeps the rest, and the open body
stays a sliver forever (66 px of a 690 px panel, with the add row and Rename
below the panel's bottom edge, before this was fixed). So the stored height is
the drawn height plus what the body's content needed beyond its granted
height, capped at half the panel; the next frame the list gives that space up.

**The open state** is egui memory keyed by `id`, so it holds for the session.
A driven check opens it with `checks::reaching::open_footer`, which reads the
`panel-footer` trace line and clicks the header only when it is closed.

Users: `panels::bookmarks` (`bookmarks.tools`: add, rename, remove, copy, cut,
paste; the drag hint stays in `always`) and `panels::layers` (`layers.tools`:
New layer, Flatten; `always` carries the reset row and the selection report).
