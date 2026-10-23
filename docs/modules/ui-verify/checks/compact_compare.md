# `ui-verify/checks/compact_compare`

`a_compacted_copy_lists_each_picture_before_and_after` — **once the compact
window has worked out the copy, it lists each changed picture, before and
after, before anything is saved.**

## What it drives

Opens `fixtures/noisy-photo.pdf` (one 400×400 greyscale noise picture drawn at
300 ppi) with `PDFCER_DIAG_INVOKE=mode.edit,file.save_compacted`, through the
scripted pointer, so it runs under `--no-input`. After the first
`compact-recompressed` line it opens the Pictures fold and presses its switch,
which resizes the picture to the 150 ppi the switch opens on, waits for the
fresh `compact-recompressed` line, then presses the `compact.compare` fold.

## What it asserts

- `compared=1` on the recompression after the switch: the plan's one changed
  picture became one row.
- The `compact.compare` fold drew under the result, and pressing it drew
  `compact.compare.table`.

What each cell says is asserted by
`recompress::tests::the_comparison_lists_each_changed_picture` (sizes, JPEG,
quality asked, bytes) and `text::compact_compare`'s tests (the cell wording).

## The photograph

The check launches with `PDFCER_DIAG_EMBED=1`, which draws dialogs inside the
main window, so the main window's own screenshot shows the table. It parks the
pointer first, so no tooltip covers it, and writes
`compact_compare.table.png` as evidence for a reader, not an oracle.
