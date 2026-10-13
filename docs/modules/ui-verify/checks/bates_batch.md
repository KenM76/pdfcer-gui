# `ui-verify/checks/bates_batch`

`bates_numbering_across_files`: Tools ▸ Batch ▸ Bates number files… with
nothing open, over four staged copies of fixtures, one of them signed. The
window is off the desktop and driven only through `ScriptedPointer`, so the
check runs under `--no-input`. The picker is answered by
`PDFCER_DIAG_BATES_SOURCES`.

## Steps

1. Stage `labelled-pages.pdf` (4 pages), `signed-two-pages.pdf` (signed),
   `small-page-first.pdf` (2) and `four-pages-unrotated.pdf` (4) into a fresh
   folder under the run's output, in that order.
2. Click `ribbon.mode.edit` when Tools is not drawn (Tools is an Edit-mode
   tab), `ribbon.tab.tools`, the collapsed `ribbon.group.tools.batch` if the
   band is narrow, then `ribbon.item.tools.bates_files`.
3. Wait for four `bates-batch-probed` lines. The signed file's must carry
   `refused=1`; every other file's `refused=0` and its page count.
4. `bates_batch.run` must not be declared: Number files is not offered over a
   list holding a refused file.
5. Click `bates_batch.row.1.remove` (the signed file), then
   `bates_batch.row.0.down`, then `bates_batch.run`.
6. `bates-batch-done` must carry `files=3 pages=10 next=11`, and the three
   `bates-batch-wrote` lines, in order, must number `small-page-first.pdf`
   000001–000002, `labelled-pages.pdf` 000003–000006 and
   `four-pages-unrotated.pdf` 000007–000010. The order proves the reorder
   reached the run; the labels prove the count carries from file to file.
7. Each original is byte-identical to its fixture, each `<stem>_Bates.pdf`
   begins with its original's bytes and is longer (an appended update), and
   no `.pdfcer-part` file is left in the folder.

## Falsification

Each break was built into the release binary and driven:

- The count not carried (`BatesForm::batch` never advances `first`): step 6
  fails on `bates-batch-done … last_label=000004 next=5`.
- Down a no-op: step 6 fails on the first `bates-batch-wrote` line, which
  names `labelled-pages.pdf`.
- Number files drawn whatever the plan says: step 4 fails.
- Each part renamed over its source instead of its target: step 7 fails on the
  first original.

## What it does not cover

- Pixels. The window is an immediate viewport, which never answers a
  screenshot; its layout is read from the declared rects.
- The one-folder output, the range naming modes and Replace, which are
  `naming::Output::plan`'s unit tests.
- A password-protected or unreadable file's row.
