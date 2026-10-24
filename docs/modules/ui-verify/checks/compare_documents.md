# `ui-verify/checks/compare_documents`

Three checks over `fixtures/compare-new.pdf` (opened) and
`fixtures/compare-old.pdf` (copied to scratch, answered to the pick by
`PDFCER_DIAG_COMPARE_PATH`), under `--no-input` in `mode.edit`. The pair is
built by `fixtures/compare-pair.PROVENANCE.py`: two pages each, page 1 the
same, page 2 with two replacements, one deletion, one insertion and a filled
square only the newer page has. `pdfcer compare` reports exactly those four
text changes.

Each launch presses Tools > Compare and waits (about ten seconds) for
`compare-listed`.

- `comparing_lists_every_change` — `kinds=` holds two `replaced@2`, one
  `inserted@2`, one `deleted@2|-`, at least one `area@2`, nothing `@1`, and
  `compare.row:<n>` is declared for every row. Area rows are counted at least
  one, not exactly, because a changed line also differs in appearance.
- `a_compare_row_shows_its_change` — clicks the first `replaced@2` row from
  page 1; requires `compare-jump … page=2` and then
  `min-reveal-solved page=1 why=compare-row`.
- `marking_a_comparison_writes_comments` — presses `compare.mark`; requires
  `compare-marked annotations>=4 folded=1`; saves a copy and requires page 1
  of it to list no annotation and page 2 to list as many as were written;
  one `Ctrl+Z`, saves again, and requires page 2 to list none.
