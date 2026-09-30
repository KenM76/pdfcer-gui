# `dialogs::settings::nav` — the page list, the search box and the page pane

The Settings window shows **one page at a time**, the way SolidWorks' System
Options does: a list of pages on the left under section labels, the selected
page on the right in its own scroll area, and a search box above both. The
operator asked for it because one long column of collapsible groups had
grown past the point where anything could be found in it.

| Item | Contract |
|---|---|
| `PAGES` | Every page as `(section, key)`, in list order. Start (General, Presets), Program, Document, Authoring, Output. The order is the contract; `super::page_body` draws a key's settings |
| `has_page(key)` | Whether a route may select `key`. Tools ▸ Font folders selects `fonts` through `Draft::focus`; a focus naming no page is ignored |
| `show(ui, height, draft, focus, viewer)` | Search box, then list and pane side by side, filling `height` and no more: the row sets its own maximum height, because the vertical separator between list and pane spans the row's *available* height, and uncapped that is the whole window. The row then pushes Save and Cancel below the window, the host's fit-to-content grows the window after it, and the fit's budget runs out with the footer off-screen (`dialog-fit-runaway` in the trace). The selected page and the query are egui temp data, so they survive the frame and reset with the process |
| `REGION_SEARCH`, `REGION_PAGE` | The search box and the pane, for driven checks |
| list entries | Each publishes `settings.heading.<key>` (see `super::REGION_HEADING_PREFIX`). A click selects; it never toggles |
| trace | `settings-page key=…` when the selection changes |

## The search

A page is listed when its heading, or any option name on it, contains the
query (case-insensitive). The names come from an index built on the first
search of the process by drawing every page once into an invisible child
`Ui` clipped to nothing, inside `diag::muted` so it publishes no regions or
trace lines, while `widgets::collect` records every title and option label
the page hands to `widgets`. Drawing the real pages is what keeps the index
complete: a hand-written keyword list would miss the next setting added.
`every_page_offers_the_search_a_name` fails if a page draws no names.

When a search hides the selected page, the first page it kept is selected, so
the pane never shows a page the list does not. On the page, matching names
are underlined in the `notice` colour by `widgets`.

The General page carries text drawn outside `widgets` (the default-program
act), so it is found by its heading only.
