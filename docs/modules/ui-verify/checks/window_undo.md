# `a_moved_document_keeps_its_undo`

**Defect it guards.** A document moved to another window with unsaved edits
arrives holding them, but without the steps that made them, so Ctrl+Z there
undoes nothing done before the move. The sender writes
`EditSession::export_history` beside the carried copy; the receiver opens the
original file pristine and replays it through `EditSession::import_history`
(`app::siblings::parcel`, `app::actions::historymove`).

**Steps.** Two pdfcer-gui processes, off the desktop, through `window_move`'s
launcher. A opens a copy of `fixtures/four-pages.pdf` and rotates the page from
the left rail, unsaved. B opens a copy of `fixtures/paragraph.pdf`. A presses
`ribbon.item.view.move_to_window`. B must trace `window-move-received`, then
`window-move-history replayed=true undo=1`. Ctrl+Z in B must trace a new
`undo-applied` line.

**Falsified** by not writing the history file in `PdfcerApp::parcel`: B traces
no `window-move-history` line, opens the copy, and the check fails there.

**What it does not prove.** The fallback to the copy when the replay is
refused (a file changed on disk since the sender opened it), which is worded
by `text::siblings::arrived_with_edits` and read from the code; and the
torn-off path, which packs through the same `parcel`.
