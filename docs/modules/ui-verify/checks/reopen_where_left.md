# `a_document_reopens_where_it_was_left`

**Defect it catches:** with Settings > Display > *Reopen each document at the
page, position and zoom it was left at* on, a document closed and opened again
starts at its first page instead of where it was left; or, with it off, it
does not start where any document starts (O464).

## What it drives

Two launches off-screen (scripted pointer, `--no-input` safe), one with
`reopen_where_left = false` planted in the sandbox's preferences and one with
`true`; `last-view.txt` is deleted before each. Each opens
`fixtures/four-pages.pdf`, with `PDFCER_DIAG_OPEN_PATH` answering the Open
dialog with the same file.

1. Wheel 9 lines down at the canvas viewport's centre; the view must move.
2. Ctrl+W closes the tab; a `close slot=0` line must follow.
3. Ctrl+O opens the file again.

## Oracle

`canvas-pos at=` (screen pixels from the acting page's corner) after the
reopen, within 2 px: of the opening position with the setting off, and of the
position left with it on. Each launch is the other's control; the off run
proves a second open does not land on the scrolled place by accident.

SKIP when the wheel did not move the view or the close was not traced. The
preferences file and `last-view.txt` are removed afterwards.

## Not covered

The record made when pdfcer quits (the same step, reached through the exit
hook; quitting needs the OS close, so it is driven only on-screen), a
remembered zoom or fit other than the opening one, and a remembered page past
the end of a file that has since lost pages.
