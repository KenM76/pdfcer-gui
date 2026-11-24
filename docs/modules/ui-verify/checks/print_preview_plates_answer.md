# `checks::print_preview_plates_answer` — the print preview's plates keep the window answering

Opens Print off the desktop on a copy of the licensed output suite's X4 test
pages (`D:\Dev\pdfTests\<suite folder>\*_PDF-Output-Test-V50_ALL_X4.pdf`),
chooses Colour → Separations and turns the plates switch on. It SKIPs when the
file is not on the machine.

**The script and its oracles:**

1. Plates on, then the next-sheet arrow at once. Waits (up to 180 s each) for
   sheet 2's colour picture and its plates. Fails when any `frame-long` since
   the switch lasted 1 s or more, or half the slowest `print-preview-sheet` or
   `print-preview-plates` render: a render ran on the window's thread. SKIPs
   when the slowest render took under 600 ms, too short to tell apart.
2. Previous sheet, next sheet. Fails when sheet 2's plates render again, or
   the preview draws a colour sheet instead of a plate.
3. The next-ink arrow twice. Fails when its declared rect changes.
4. Clicks the sheet-number box, Ctrl+A, types `3`, Enter. Fails when no
   `print-preview-render page=2` follows.

`PDFCER_DIAG_PRINT_DRY_RUN=1` is set, so nothing reaches a printer.
