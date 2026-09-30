# `dialogs::labels` — Number pages

`pages.labels`, Pages ▸ Stamp ▸ Number pages…. Opened by
`app::dispatch::pages` with the page rail's pick, the page count and
`OpenDoc::label_ranges`. Commits `PageAction::SetLabels` or
`PageAction::ClearLabels`, applied in `app::actions::pages` through the edit
funnel as `EditSession::set_page_labels` / `clear_page_labels` (one
`CommandKind::SetPageLabels` / `ClearPageLabels` undo entry each).

## Contract

| Item | Contract |
|---|---|
| Range | "Pages from *n* to *m*", 1-based. Defaults to the rail pick when it spans two or more pages, else every page — the Bates scope rule, not the shared current-sheet operand rule. A back-to-front or out-of-range range replaces the preview with `text::labels::bad_range` and disables Apply. |
| Style | One of `STYLES`: 1 2 3, i ii iii, I II III, a b c, A B C, or the prefix alone (`LabelStyle::PrefixOnly`, which disables Start). |
| Seed | The form opens on the stored range covering the first page of the default range, else decimal from 1, so reopening shows what is there. |
| Preview | The run's first labels and its last, through `LabelFormat::label` — the engine's own numeral code, so the preview cannot disagree with the result. |
| Current labels | The stored ranges, one line each (`text::labels::range_line`), or `none_yet`. *Remove all labels* is drawn only when there are some. |
| Receipt | `text::labels::applied` names the pages and the number of ranges the document now stores; `cleared` / `nothing_to_clear` for the other button. |
| Lifetime | Holds a snapshot of the ranges, so it closes with its document. |

Labels change only how pages are named in the page box and thumbnails
(ISO 32000-2 §12.4.2); nothing is drawn on the page. That is what separates
this command from Bates numbering, and the window's first line says so.

## Where the labels are read

`OpenDoc::ensure_labels` rebuilds `LabelCache` once per edit epoch, so an
undo or redo that moves the epoch refreshes the page box and captions.
`page_label(i)` is `None` when the label is just the page's number, which keeps
an unlabelled or decimal-from-1 document's captions unchanged.

## Verification

`ui-verify` check `page_labels_without_the_mouse`
(`docs/modules/ui-verify/checks/labels_scripted.md`).
