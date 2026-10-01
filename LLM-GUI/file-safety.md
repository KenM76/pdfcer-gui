# File safety: an open file is not live

## Facts

- Opening a PDF reads the whole file into memory. No lock is held, so the file
  on disk can be written while it is open.
- Nothing watches the file, and there is no Reload. Opening a path that is
  already open activates its tab.
- Save writes from the in-memory copy plus the user's edits, replacing the file.
  **A change written from outside is silently lost at the user's next Save.**
- Save appends the edits as an incremental update. Save a copy does the same to
  another path. Save a compacted copy rewrites the file fresh, dropping prior
  revisions and any digital signature.
- Each launch of `pdfcer-gui.exe` is a separate process and window. A second
  launch forwards nothing to the first.
- Undo history is in memory, per document, and covers only edits made in that
  window, including live-link edits. Changes made from outside are not in it.
- Passwords are never stored. An encrypted file asks every time it opens.

## Order of preference for changing an open document

1. **Live link** (live-link.md): the edit goes into the open document, undoable.
2. **Hand-off** (below), using the `pdfcer` CLI or a private hidden GUI.
3. Never write the file while the user's tab has unsaved edits: one set of
   changes is lost. `pdfcer-remote state` shows `unsaved=`.

## Hand-off procedure

1. Ask the user to Save (Ctrl+S) and close the tab (Ctrl+W). Close asks before
   discarding unsaved edits.
2. Make the change, preferably to a **new** file (`--output name-edited.pdf`).
   Overwrite the original only when asked. Overwriting is mechanically safe: the
   CLI writes a temporary file and renames it, so a failure leaves the original intact.
3. Verify by re-reading the result (`render-page`, `extract-text`,
   `list-annotations`, `object-list`). An exit code is not a result.
4. Ask the user to reopen it: File ▸ Open, the recent list, or
   `pdfcer-gui.exe "<path>"` (which opens a new window).

## Reading needs no hand-off

Inspecting, listing, extracting and rendering are safe at any time. They read
what is on disk, which excludes unsaved edits. If the user says "what I am
looking at", use the live link (`render`) or ask them to save first.

## Never

- Close or kill a pdfcer-gui window you did not start. It may hold unsaved work.
- End any process by image name (`taskkill /IM`): that closes every pdfcer-gui
  window the user has open. End only your own process, by PID.
- Start the user's own `pdfcer-gui.exe` with `PDFCER_DIAG*` variables set, or
  write anything beside it (`userdata\`, `models\`). See hidden-driving.md.
- Edit `userdata\recent.txt`. Reading it is fine (paths, one per line, newest first).
