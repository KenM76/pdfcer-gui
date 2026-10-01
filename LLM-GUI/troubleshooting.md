# Troubleshooting

## Live link

| Symptom | Cause, and what to do |
|---|---|
| `no pdfcer-gui window is listening` (exit 2) | No window is open, or Remote control is set to Never. Ask the user. |
| `refused disabled` | The policy is `never`. Only the user can change it, in Settings ▸ Remote control. |
| The call hangs | The question bar is waiting in the user's window, for up to 120 s. Tell the user to look. |
| `refused cooling-off` | The user refused this client less than 60 s ago. Ask; do not retry. |
| `refused busy` | Another program's question is pending. Wait, then retry once. |
| Every call asks again | The user chose Allow once. Ask for "Allow for this session". |
| `err hidden` from `run` | The current mode lacks the command. `run mode.edit` (or `mode.review`) first. |
| `err disabled` | It needs a document, a selection or an undo step. Read `state`. |
| `unknown-command` | Absent from this build. Check `commands`. |
| The wrong window answered | Calls without `--pid` go to the newest window. Use `pdfcer-remote list`, then `--pid`. |

## In the window

- **A command is missing from the ribbon.** It is in another mode (Ctrl+1/2/3),
  on the Format tab (shown only while something is selected), folded into a
  narrow ribbon's overflow, or not in this build.
- **The ribbon and panels vanished.** Ctrl+H (Read mode view) hides the chrome;
  press it again. F11 is full screen. With Auto-hide ribbon on (`view.ribbon_auto_hide`), hovering where it was brings it back.
- **A click selects nothing, or the wrong thing.** Check the Select filter
  (bottom left). It decides which object classes a click may land on.
- **Text cannot be selected or found.** The page is a scan. Run File ▸ Recognise text
  or `pdfcer.exe ocr`.
- **Guides cannot be placed.** A guide is dragged out of a ruler. Turn rulers on
  (View ▸ Display).
- **Content beyond the sheet is invisible.** Off-page content is hidden in Read
  mode by default (View ▸ Display; preference `off_page.<mode>`).
- **The password is asked for every time.** By design: passwords are never stored.

## Files

- **The user's edit vanished.** The file was written from outside while it was
  open, and the user's next Save replaced it (file-safety.md).
- **The file keeps growing.** Every save is incremental. Save a compacted copy
  (or the CLI's `--mode full` once, with no signatures) rewrites it smaller.
- **A signature became invalid.** Any edit after signing breaks it, and a full
  save removes it. Run `list-signatures` before editing.

## CLI exit codes that need action

- 8: save refused (for example, a full save of a hybrid file). Use incremental.
- 9: edit refused; the printed reason names it (encrypted, certified, index out of range).
- 10: redaction residuals were disclosed. Read them, then pass `--acknowledge-residuals`.
- 11: the file opened only through recovery. The output is a full rewrite.

## Hidden driving

- **The trace is empty.** stderr was not redirected to a file (hidden-driving.md).
- **A real dialog appeared.** The picker's `PDFCER_DIAG_*` variable was unset
  (the trace says `source=native`). Set it before launch.
- **A click did nothing.** The coordinates came from a stale `ui-rect`. Re-read
  the trace, and check the result with `shot`.
- **The user's settings changed.** The user's own exe was driven. Always drive
  a copy in a scratch folder.
