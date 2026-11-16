# `ui-verify/checks/print_direct_compose`

`print_composes_sheets_for_the_pdf` — with *Send the PDF itself* on, a job of
four pages at two per sheet reaches a printer that reads PDF as two sheets the
engine composed (`pdfcer_print::compose::compose`), sent at actual size with no
orientation, and not through the driver. Off the desktop, scripted pointer,
`--no-input` compatible. Nothing reaches a real printer: the dialog is pointed
at `netprint_scripted`'s local IPP printer by `PDFCER_DIAG_PRINT_DIRECT_URI`,
under `PDFCER_DIAG_PRINT_DRY_RUN=1`.

## The oracle

From the fixture and RFC 8010, not from the program: `labelled-pages.pdf` has
four pages of one size and no `/Rotate`, so two per sheet is two sheets.

1. `print-direct-compose` carries `shape=nup` and `sheets=2`.
2. `print-direct route=direct` carries `composed=1` and `pages=2`. The
   document sent as it is traces `composed=0 pages=4`.
3. No `print-spool-done` line.
4. The Print-Job the local printer received carries `%PDF-`, the encoded
   attribute `print-scaling` = `none` (`print-scaling\x00\x04none`), and no
   `orientation-requested`.

`print-direct-note` lines (one per `Composed::notes` entry) are copied into
the report, not asserted: this fixture has no annotations, so the engine has
nothing to disclose.

## Steps

1. Copy `fixtures/labelled-pages.pdf` to the run folder and open Print with
   `PDFCER_DIAG_INVOKE=file.print`.
2. Click `print.impose.multiple` (two per sheet by default).
3. Wheel over `print.as-image` until `print.direct` is declared; press it and
   settle for the printer's answer.
4. Press `dialog.buttons.accept` and read the trace and the received jobs.

## Falsification

Not yet run. Predicted: making `compose::Layout::composed` false for n-up
sends the document as it is, which traces `composed=0 pages=4` and no
`print-direct-compose` line, failing oracle 1.
