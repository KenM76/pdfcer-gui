# `dialogs::split_pages` — Split Document

`pages.split`, Pages ▸ Organise ▸ Split…. Opened by `app::dispatch::pages`
with the rail's picked pages. Raises `PageAction::SplitDocument`, applied in
`app::actions::pages` through `app::actions::split::split`. The open document
is not changed.

## Contract

| Item | Contract |
|---|---|
| Rule | Every N pages (`SplitCriterion::EveryN`), after typed pages (`AfterPages`, 1-based in the box, `parse_page_range`), or at each top-level bookmark (`TopLevelBookmarks`). No rule is chosen at open unless pages were picked: then *After pages* opens with the picks, the last page left out. |
| Bookmarks rule | Disabled, with the reason on hover, when `plan_split` refuses `TopLevelBookmarks` for this document. |
| File names | The engine's pattern, `pageops::split::DEFAULT_NAME_TEMPLATE` at open. A path separator, a character Windows refuses, or no `.pdf` ending is refused before the engine is asked. |
| Folder | A text box plus Browse… (`files::pick_split_folder`, seam `PDFCER_DIAG_SPLIT_FOLDER`). Defaults to the document's own folder; empty for a created document. |
| Keep the page labels | Drawn only when the document has labels; ticked by default. |
| Preview | `actions::split::plan` — the engine's `plan_split`, so the list shown is the list written. It names every file and its pages, how many already exist and will be replaced, or the one sentence saying why nothing would be written. Split is enabled only on a list. |
| Document | Bound to the document it opened on by path; on another tab the preview says so and Split is off. |

The preview is recomputed only when an input or the document's `edit_epoch`
changes.

## Regions

`dialog:split-pages`, `split-pages.split`, `split-pages.rule.every`,
`split-pages.rule.every.n`, `split-pages.rule.after`,
`split-pages.rule.after.pages`, `split-pages.rule.bookmarks`,
`split-pages.template`, `split-pages.folder`, `split-pages.browse`,
`split-pages.labels`. Driven by `split_writes_the_files_the_window_listed`.

## Trace

- `split-pages-open pages= picked= bookmarks= labelled=`
- `split-pages-preview files= existing= names= ranges= folder=`, or
  `files=0 refused=` with the sentence.
