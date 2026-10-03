# `dialogs::extract_pages` — Extract Pages

`pages.extract`, Pages ▸ Organise ▸ Extract… and the page tile's menu.
Opened by `app::dispatch::pages` with the page operands (the rail pick, else
the page on screen). Raises `PageAction::ExtractPages`, applied in
`app::actions::pages` through `app::actions::extract::extract`.

## Contract

| Item | Contract |
|---|---|
| Pages | All, this page, or a typed range (`pageselection::parse_page_range`). A single operand that is the page on screen opens as *This page*; anything else opens as the typed range `format_page_range` writes. A range naming no page disables Extract. |
| Keep the page labels | Drawn only when the document has page labels; ticked by default (`ExtractedPageLabels::Keep`). Unticked, the new file numbers its pages from 1. |
| Delete afterwards | Off by default. One undo step, and nothing is deleted unless the new file was written. Follows Acrobat's *Delete pages after extracting* (R11). |
| Frozen state | The page on screen and the page count at open, so paging behind the window does not change what *This page* means. |

## Regions

`dialog:extract-pages`, `extract-pages.extract`, `extract-pages.pages.range`,
`extract-pages.pages.{all,current,typed}`, `extract-pages.labels`,
`extract-pages.delete_after`. Driven by
`extract_pages_keeps_or_drops_the_labels`.
