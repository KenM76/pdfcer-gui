# `ui-verify/checks/page_labels_follow`

`a_moved_or_deleted_sheet_keeps_its_label`, on `fixtures/drawing-sheets.pdf`,
whose one `/PageLabels` range (`/S /D /P (A-) /St 101`) labels the four sheets
`A-101 … A-104`. Launched with `mode.review` and default preferences, so the
*Page labels* setting is *Each page keeps its label*.

1. The opening `page-labels-now why=open` must read `A-101|A-102|A-103|A-104`.
2. Pages ▸ **Move down** with sheet 1 current.
3. Pages ▸ **Delete** on the current sheet.
4. File ▸ Save a copy; a second process opens it.

## PASS

| step | `page-labels-now` must read |
|---|---|
| 2 | `why=reorder labels=A-102\|A-101\|A-103\|A-104` |
| 3 | `why=delete`, the step-2 list with exactly one label taken out |
| 4 | the reopened file's `why=open` list equals step 3's |

Step 3 asserts the shape rather than a fixed list because which sheet is
current after a move is the page rail's decision, not this feature's; what the
feature owns is that the others keep their labels.

## What a regression reads as

Positional labels — the engine's verbs without `*_with_labels` — leave step 2
at `A-101|A-102|A-103|A-104` and step 3 at `A-101|A-102|A-103`: a moved sheet
takes its new position's number, and a deleted one renumbers every sheet after
it unless the last was deleted. Step 3 alone can pass positionally when the
last sheet goes, which is why step 2 is asserted first.
