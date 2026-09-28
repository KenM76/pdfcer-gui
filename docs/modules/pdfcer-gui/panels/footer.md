# `panels::footer` — a panel's controls pinned under its list

**Rule:** a list panel draws nothing conditional above its rows. Anything that
appears in answer to a gesture — the controls a selection unlocks, the reset
row after a toggle, a report about the canvas selection — goes in the footer,
where appearing grows the footer upward, shrinks the list's viewport from the
bottom, and leaves every row where it was. The operator's request (O256) is
that selecting something never moves the thing he just clicked.

| Item | Contract |
|---|---|
| `list_height(ui, id)` | Height for the list's `ScrollArea`: available height less the footer's height measured last frame, never below three rows. The list is drawn with `auto_shrink([false, false])` so the footer sits at the bottom whatever the list's length |
| `show(ui, id, panel_height, title, always, tools)` | Separator, then `always` (unconditional), then a `CollapsingHeader` titled `title`, collapsed by default, whose body is `tools` inside a `ScrollArea` capped at half the panel. `tools = None` draws no header — Read mode, where every tool would change the document. Publishes the header as region `id`, traces `panel-footer id=… open=…`, stores its height for `list_height` and repaints when it changed |

**Why measured last frame.** egui's `Panel` would carve the space, but a
non-resizable panel expands to its previous rect every frame and so never
shrinks after the header collapses. A stored height with one repaint on change
is exact on the frame after any change and invisible to the operator.

**The open state** is egui memory keyed by `id`, so it holds for the session.
A driven check opens it with `checks::reaching::open_footer`, which reads the
`panel-footer` trace line and clicks the header only when it is closed.

Users: `panels::bookmarks` (`bookmarks.tools`: add, rename, remove, copy, cut,
paste; the drag hint stays in `always`) and `panels::layers` (`layers.tools`:
New layer, Flatten; `always` carries the reset row and the selection report).
