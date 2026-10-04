# `ui-verify/checks/split_pages`

`split_writes_the_files_the_window_listed` — Pages ▸ Split… on a window off
the desktop, driven only through `ScriptedPointer`, so it runs under
`--no-input`. Each launch writes into its own emptied folder, handed to
Browse… by `PDFCER_DIAG_SPLIT_FOLDER`.

| Launch | Fixture | Input | Files |
|---|---|---|---|
| every | `labelled-pages.pdf` | `2` in every-N, labels box cleared | `labelled-pages_1.pdf` 1-2, `_2.pdf` 3-4, each `labels_dropped=1 label_ranges=0` |
| after | `four-pages.pdf` | `1, 3` in after-pages, pattern `part-{start}-{end}.pdf` | `part-1-1.pdf`, `part-2-3.pdf`, `part-4-4.pdf` |
| bookmarks | `four-pages.pdf` (one top-level bookmark per page) | the bookmarks radio | `four-pages_1.pdf` … `_4.pdf` |

## Steps

1. Clicks `ribbon.mode.review`, `ribbon.tab.pages`, the collapsed organise
   group when the band is narrow, then `ribbon.item.pages.split`.
2. `split-pages-open bookmarks=` must say whether the rule is offered:
   `0` for the labelled fixture, which has no outline.
3. The rule, pattern and labels box are set; Browse… fills the folder.
4. Before Split, the last `split-pages-preview` must list exactly the
   launch's names, ranges and folder.
5. After Split: `split-written`, one `split-part` per file with its pages
   and the launch's fields, every file on disk starting `%PDF-`, and no
   other file in the folder.

What each file's pages read under Keep and Drop is the unit test
`the_split_writes_every_part_with_or_without_labels`.

## Falsification

Mapping every rule to `EveryN(1)` fails the first launch at the preview
(`files=4`).

## What it does not cover

The refusals (source overwrite, bad pattern, missing folder) are unit tests,
not driven. A failed write is not driven.
