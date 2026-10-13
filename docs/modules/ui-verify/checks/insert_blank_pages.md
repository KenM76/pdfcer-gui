# `ui-verify/checks/insert_blank_pages`

`blank_pages_are_inserted_as_one_step` — Pages ▸ Insert blank…, driven
off-screen with the scripted pointer on a copy of `fixtures/four-pages.pdf`
in Edit (`PDFCER_DIAG_INVOKE=mode.edit`).

## Steps

1. Press `ribbon.item.pages.insert_blank` on the Pages tab; the dialog must
   declare `insert-blank.body`.
2. Open the size list (up to three presses, as a combo popup can take a
   second) and pick `insert-blank.size.item.4`, A3.
3. Press Landscape and *At the end* (`insert-blank.where.3`).
4. Click the count, Ctrl+A, type `3`, Enter, in the dialog's viewport.
5. Press `insert-blank.insert`.
6. Ctrl+Z once in the main viewport.

## Oracles

| after | requires |
|---|---|
| 5 | `insert-blank-dialog n=3 landscape=true position=End`; anything else is a harness error (a control not reached), not a feature failure |
| 5 | no `pages-resync-failed` — three copies of one blank page are three page objects |
| 5 | `insert-pages-landed at=4 pages=7 was=4` |
| 5 | the last `canvas` line has `page=4` and `crop=0,0,1190.55,841.89` (A3 landscape) — the view landed on the first new sheet, in a continuous display |
| 6 | the last `canvas` line has `pages=4` — one undo removes all three |

## Falsification

Each break was built, driven, and failed at its own oracle:

- one `insert_pages` call with the whole list (`for run in [pages]`):
  `pages-resync-failed detail=page tree cycle at object 64 0 kept=4`;
- the fold skipped (`|| false && session.coalesce_last(…)`): one undo left
  `pages=6`;
- Landscape ignored in `sheet_pt`: crop `0,0,841.89,1190.55`;
- `tracked_page` set beside the landing in `pages::insert_from_view`:
  `page=0` — the view stayed where it was.
